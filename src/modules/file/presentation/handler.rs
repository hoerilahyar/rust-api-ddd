use axum::body::Body;
use axum::extract::{Extension, Multipart, Path, Query, State};
use axum::http::header;
use axum::response::IntoResponse;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::bootstrap::state::AppState;
use crate::modules::auth::domain::value_object::Claims;
use crate::modules::file::application::dto::FileResponse;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;
use crate::shared::middleware::ensure_permission;
use crate::shared::response::{ApiResponse, PaginatedResponse};

/// `multipart/form-data` upload. Expects a single file field (any field
/// name); the first part that has a filename wins. `DefaultBodyLimit` for
/// this route is set from `config.storage.max_upload_bytes` in `routes()`,
/// so an oversized body is rejected by the framework before this handler
/// even runs -- the size check inside `FileService::upload` is the second,
/// belt-and-suspenders line of defense.
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "file.upload")?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let Some(original_name) = field.file_name().map(|s| s.to_string()) else {
            // Not a file part (e.g. a plain text field) -- skip it.
            continue;
        };
        let mime_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();
        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let file = state
            .file_service
            .upload(Some(claims.sub), original_name, mime_type, bytes)
            .await?;

        return Ok(ApiResponse::new("file uploaded", FileResponse::from(file)).created());
    }

    Err(AppError::BadRequest(
        "no file part found in the request".to_string(),
    ))
}

pub async fn list_files(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "file.read")?;

    let (files, total) = state.file_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<FileResponse> = files.into_iter().map(FileResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn get_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "file.read")?;

    let file = state.file_service.get_by_uuid(uuid).await?;
    Ok(ApiResponse::new("ok", FileResponse::from(file)))
}

/// Streams the file straight from disk into the response body -- the bytes
/// are never fully buffered in memory, so this scales to large files.
pub async fn download_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "file.read")?;

    let (file, handle) = state.file_service.open_for_download(uuid).await?;
    let stream = ReaderStream::new(handle);
    let body = Body::from_stream(stream);

    let disposition = format!(
        "attachment; filename=\"{}\"",
        file.original_name.replace('"', "")
    );

    Ok((
        [
            (header::CONTENT_TYPE, file.mime_type),
            (header::CONTENT_DISPOSITION, disposition),
            (header::CONTENT_LENGTH, file.size_bytes.to_string()),
        ],
        body,
    ))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "file.delete")?;

    state.file_service.delete(uuid).await?;
    Ok(ApiResponse::<()>::message("file deleted"))
}
