use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        user_profile::application::{UpsertUserProfileRequest, UserProfileResponse},
    },
    shared::{
        errors::AppError, middleware::ensure_permission, response::ApiResponse,
        validator::ValidatedJson,
    },
};

pub async fn get_my_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state.user_profile_service.get(claims.sub).await?;
    Ok(ApiResponse::new("ok", UserProfileResponse::from(profile)))
}

/// `PUT /me/profile` -- create-or-replace, always scoped to the caller's
/// own `claims.sub`. Fields omitted from the body keep their current
/// stored value (see `UserProfileRepository::upsert`).
pub async fn upsert_my_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<UpsertUserProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state
        .user_profile_service
        .upsert(claims.sub, payload)
        .await?;

    Ok(ApiResponse::new(
        "profile saved",
        UserProfileResponse::from(profile),
    ))
}

pub async fn delete_my_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    state.user_profile_service.delete(claims.sub).await?;
    Ok(ApiResponse::message("profile deleted"))
}

/// `GET /users/:id/profile` -- admin read of another user's profile,
/// gated behind `user.manage` (same permission as the rest of the admin
/// `/users` surface in `modules::user`).
pub async fn get_user_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let profile = state.user_profile_service.get(id).await?;
    Ok(ApiResponse::new("ok", UserProfileResponse::from(profile)))
}
