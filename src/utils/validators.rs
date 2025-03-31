use async_trait::async_trait;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{FromRequest, Query, Request};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use thiserror::Error;
use validator::Validate;

use super::response::ApiErrorResponse;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = InvalidRequestError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = InvalidRequestError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}

#[derive(Debug, Error)]
pub enum InvalidRequestError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),

    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),

    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
}

impl IntoResponse for InvalidRequestError {
    fn into_response(self) -> Response {
        match self {
            InvalidRequestError::ValidationError(_) => {
                let message = format!("Input validation error: [{self}]").replace('\n', ", ");
                ApiErrorResponse::new(400, message)
            }

            InvalidRequestError::AxumJsonRejection(_) => {
                ApiErrorResponse::new(400, self.to_string())
            }

            InvalidRequestError::AxumQueryRejection(_) => {
                ApiErrorResponse::new(400, "Invalid query parameters".to_string())
            }
        }
        .into_response()
    }
}
