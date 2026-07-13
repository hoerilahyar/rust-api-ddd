use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        user_setting::application::{UpsertUserSettingRequest, UserSettingResponse},
    },
    shared::{errors::AppError, response::ApiResponse, validator::ValidatedJson},
};

pub async fn list_my_settings(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let settings = state.user_setting_service.list(claims.sub).await?;
    let data: Vec<UserSettingResponse> = settings.into_iter().map(UserSettingResponse::from).collect();
    Ok(ApiResponse::new("ok", data))
}

pub async fn get_my_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let setting = state.user_setting_service.get(claims.sub, &key).await?;
    Ok(ApiResponse::new("ok", UserSettingResponse::from(setting)))
}

/// `PUT /me/settings/:key` -- create-or-replace, always scoped to the
/// caller's own `claims.sub`.
pub async fn upsert_my_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
    ValidatedJson(payload): ValidatedJson<UpsertUserSettingRequest>,
) -> Result<impl IntoResponse, AppError> {
    let setting = state
        .user_setting_service
        .upsert(claims.sub, &key, payload)
        .await?;
    Ok(ApiResponse::new("setting saved", UserSettingResponse::from(setting)))
}

pub async fn delete_my_setting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    state.user_setting_service.delete(claims.sub, &key).await?;
    Ok(ApiResponse::message("setting deleted"))
}
