use std::net::SocketAddr;

use axum::{
    body::{to_bytes, Body},
    extract::{ConnectInfo, MatchedPath, Request, State},
    http::{header, Method},
    middleware::Next,
    response::Response,
};
use serde_json::Value;
use uuid::Uuid;

use crate::bootstrap::state::AppState;
use crate::modules::auth::domain::value_object::Claims;
use crate::shared::contracts::{Activity, MethodRequest, Module, RecordActivity};
use crate::shared::errors::AppError;
use crate::shared::middleware::rate_limiter::resolve_client_ip;

/// Records one row in `activity_logs` for every request that reaches it, via
/// `shared::contracts::ActivityRecorder` -- the generic counterpart to
/// `AuditRecorder` (which only covers login attempts).
///
/// **Must be layered *inside* `require_auth`** -- i.e. added to a router
/// *before* the `require_auth` `route_layer` call, so it ends up as the
/// inner layer and `Claims` are already sitting in the request's extensions
/// by the time this runs (axum treats the layer added *last* as outermost,
/// so `require_auth` needs to be last). See any module's `presentation::routes`
/// for the pattern:
///
/// ```ignore
/// Router::new()
///     .route(...)
///     .route_layer(from_fn_with_state(state.clone(), activity_log_middleware))
///     .route_layer(from_fn_with_state(state, require_auth))
/// ```
///
/// Recording happens on a spawned task after the response is ready, so a
/// logging failure (or a slow write) never delays or breaks the actual
/// response.
pub async fn activity_log_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let method = req.method().clone();
    let actual_path = req.uri().path().to_string();
    let matched_path = req
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string());
    let user_id = req.extensions().get::<Claims>().map(|c| c.sub);
    let ip_address = Some(resolve_client_ip(
        req.headers(),
        addr.ip(),
        state.config.rate_limit.trust_proxy,
    ));
    let user_agent = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());

    let template = matched_path.as_deref().unwrap_or(&actual_path);
    let module = infer_module(template);
    let activity = infer_activity(&method, template);
    let resource_id = last_path_param(template, &actual_path);

    let response = next.run(req).await;

    let status = response.status();
    let status_code = Some(status.as_u16() as i16);

    // Only buffer the body when it's realistically a small JSON error
    // payload -- never for a success response, and never when we can't
    // already tell from headers that it's small JSON. This matters
    // because responses like a file download stream gigabytes through
    // this same middleware; buffering those into memory here would defeat
    // the whole point of streaming them.
    const MAX_DESCRIPTION_BODY_BYTES: usize = 64 * 1024;
    let is_small_json_error = (status.is_client_error() || status.is_server_error())
        && response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("application/json"))
        && response
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .is_some_and(|len| len <= MAX_DESCRIPTION_BODY_BYTES);

    let (response, description) = if is_small_json_error {
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, MAX_DESCRIPTION_BODY_BYTES)
            .await
            .unwrap_or_default();
        let description = serde_json::from_slice::<Value>(&bytes)
            .ok()
            .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from));
        (Response::from_parts(parts, Body::from(bytes)), description)
    } else {
        (response, None)
    };

    let recorder = state.activity_recorder.clone();
    let record = RecordActivity {
        user_id,
        activity,
        module,
        resource_type: Some(module_resource_type(&module).to_string()),
        resource_id,
        method: to_method_request(&method),
        path: actual_path,
        description,
        ip_address,
        user_agent,
        status_code,
        trace_id: Some(Uuid::new_v4()),
    };

    // Fire-and-forget: writing the activity trail must never slow down or
    // fail the actual request.
    tokio::spawn(async move {
        if let Err(err) = recorder.record_activity(record).await {
            tracing::error!(error = ?err, "failed to record activity log");
        }
    });

    Ok(response)
}

fn to_method_request(method: &Method) -> MethodRequest {
    match method.as_str() {
        "POST" => MethodRequest::Post,
        "PUT" => MethodRequest::Put,
        "PATCH" => MethodRequest::Patch,
        "DELETE" => MethodRequest::Delete,
        _ => MethodRequest::Get,
    }
}

/// First real path segment (after the `/api/v1` nest prefix, if present)
/// decides the module. `/me...` routes belong to `user`/`user_setting`/`menu`
/// depending on the second segment, since they're all "my own resource"
/// shortcuts rather than a module of their own.
fn infer_module(template: &str) -> Module {
    let mut segments = template
        .split('/')
        .filter(|s| !s.is_empty() && *s != "api" && *s != "v1");

    match segments.next() {
        Some("users") => Module::User,
        Some("roles") => Module::Role,
        Some("permissions") => Module::Permission,
        Some("menus") => Module::Menu,
        Some("settings") => Module::Setting,
        Some("audit") => Module::Audit,
        Some("activity-logs") => Module::ActivityLog,
        Some("files") => Module::File,
        Some("me") => match segments.next() {
            Some("menu") => Module::Menu,
            Some("settings") => Module::UserSetting,
            _ => Module::User, // /me, /me/password
        },
        _ => Module::ActivityLog,
    }
}

fn module_resource_type(module: &Module) -> &'static str {
    match module {
        Module::Auth => "auth",
        Module::User => "user",
        Module::Role => "role",
        Module::Permission => "permission",
        Module::Menu => "menu",
        Module::Setting => "setting",
        Module::UserSetting => "user_setting",
        Module::Audit => "login_log",
        Module::ActivityLog => "activity_log",
        Module::File => "file",
    }
}

/// Maps method + templated path to a semantic `Activity`. Special-cases the
/// handful of assign/unassign-shaped endpoints (`POST .../roles`,
/// `DELETE .../roles/:role`, `POST .../permission`,
/// `DELETE .../permission/:permission`) so those read as `Assign`/`Unassign`
/// rather than the generic `Create`/`Delete`, and the file module's
/// `POST /files` / `GET .../download` as `Upload`/`Download`.
fn infer_activity(method: &Method, template: &str) -> Activity {
    let last_segment_is_param = template
        .rsplit('/')
        .find(|s| !s.is_empty())
        .map(|s| s.starts_with(':'))
        .unwrap_or(false);

    if method == Method::GET && template.ends_with("/download") {
        return Activity::Download;
    }

    match *method {
        Method::POST => {
            if template.ends_with("/roles") || template.ends_with("/permission") {
                Activity::Assign
            } else if template.ends_with("/files") {
                Activity::Upload
            } else {
                Activity::Create
            }
        }
        Method::PUT | Method::PATCH => Activity::Update,
        Method::DELETE => {
            if template.contains("/roles/") || template.contains("/permission/") {
                Activity::Unassign
            } else {
                Activity::Delete
            }
        }
        _ => {
            if last_segment_is_param {
                Activity::View
            } else {
                Activity::List
            }
        }
    }
}

/// Best-effort resource id: the actual-path segment lined up against the
/// last `:param` segment in the matched template (e.g. template
/// `/users/:id/roles/:role` + actual `/users/42/roles/admin` -> `"admin"`,
/// the specific thing this request acted on).
fn last_path_param(template: &str, actual_path: &str) -> Option<String> {
    let template_segments: Vec<&str> = template.split('/').filter(|s| !s.is_empty()).collect();
    let actual_segments: Vec<&str> = actual_path.split('/').filter(|s| !s.is_empty()).collect();

    template_segments
        .iter()
        .zip(actual_segments.iter())
        .rev()
        .find(|(t, _)| t.starts_with(':'))
        .map(|(_, actual)| actual.to_string())
}
