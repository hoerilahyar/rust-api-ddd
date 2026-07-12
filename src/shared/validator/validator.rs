use axum::{
    async_trait,
    extract::{rejection::JsonRejection, FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::shared::errors::{AppError, FieldError};

/// Drop-in replacement for `axum::Json<T>` that also runs `validator`'s
/// `#[derive(Validate)]` checks and turns failures into `AppError::Validation`
/// with a Laravel-style `{field, message}` list.
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(map_json_rejection)?;

        value.validate().map_err(|errors| {
            let field_errors = errors
                .field_errors()
                .into_iter()
                .flat_map(|(field, errs)| {
                    errs.iter().map(move |e| FieldError {
                        field: field.to_string(),
                        message: e
                            .message
                            .clone()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| format!("{field} is invalid")),
                    })
                })
                .collect::<Vec<_>>();
            AppError::Validation(field_errors)
        })?;

        Ok(ValidatedJson(value))
    }
}

fn map_json_rejection(rejection: JsonRejection) -> AppError {
    AppError::BadRequest(format!("invalid request body: {rejection}"))
}
