use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        setting::application::{SettingResponse, UpsertSettingRequest},
    },
    shared::{
        errors::AppError, middleware::ensure_permission, response::ApiResponse,
        validator::ValidatedJson,
    },
};

pub async fn list_settings(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "settings.manage")?;

    let settings = state.setting_service.list().await?;
    let data: Vec<SettingResponse> = settings.into_iter().map(SettingResponse::from).collect();
    Ok(ApiResponse::new("ok", data))
}

pub async fn get_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "settings.manage")?;
    let setting = state.setting_service.get_by_key(&key).await?;
    Ok(ApiResponse::new("ok", SettingResponse::from(setting)))
}

/// `PUT /settings/:key` -- create-or-replace, so a client can introduce a
/// brand-new setting key just by PUTting it, same as any other keyed REST
/// resource.
pub async fn upsert_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
    ValidatedJson(payload): ValidatedJson<UpsertSettingRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "settings.manage")?;
    let setting = state.setting_service.upsert(&key, payload, claims.sub).await?;
    Ok(ApiResponse::new("setting saved", SettingResponse::from(setting)))
}

pub async fn delete_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "settings.manage")?;
    state.setting_service.delete(&key, claims.sub).await?;
    Ok(ApiResponse::message("setting deleted"))
}
