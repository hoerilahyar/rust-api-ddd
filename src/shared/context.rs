use tokio::task_local;

/// Per-request data that application-layer code (e.g. audit trail writers)
/// needs but shouldn't have to receive as an explicit parameter on every
/// service trait method. Populated once in `activity_log_middleware` and
/// read back via [`current_request_context`].
#[derive(Debug, Clone, Default)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

task_local! {
    pub static REQUEST_CONTEXT: RequestContext;
}

/// Best-effort accessor for the current request's IP/user-agent. Must be
/// called synchronously (before any `tokio::spawn` boundary) from within the
/// future scoped by `REQUEST_CONTEXT.scope(...)` in `activity_log_middleware`
/// -- a spawned task does not inherit the parent task's task-locals. Returns
/// a default (all `None`) outside of a request scope, e.g. in tests.
pub fn current_request_context() -> RequestContext {
    REQUEST_CONTEXT.try_with(|ctx| ctx.clone()).unwrap_or_default()
}
