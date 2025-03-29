use std::collections::HashMap;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiSuccessResponse<T> {
    message: String,
    data: T,
    metadata: Option<HashMap<String, String>>,
}

impl<T> ApiSuccessResponse<T> {
    pub fn new(message: String, data: T, metadata: Option<HashMap<String, String>>) -> Self {
        Self {
            message,
            data,
            metadata,
        }
    }
}

impl<T> IntoResponse for ApiSuccessResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let json = Json(self);
        json.into_response()
    }
}

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    status: u16,
    message: String,
}

impl ApiErrorResponse {
    pub fn new(status: u16, message: String) -> Self {
        Self { status, message }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            Json(self),
        )
            .into_response()
    }
}
