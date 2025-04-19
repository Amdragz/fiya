use std::collections::HashMap;

use super::helper::datetime_to_offset_datetime;
use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, Expiration};
use chrono::{DateTime, Utc};
use serde::Serialize;
use time::Duration;

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

#[derive(Serialize)]
pub struct AuthLoginSuccessResponse<T> {
    message: String,
    data: T,
    metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing)]
    http_only_refresh_token: Option<(String, DateTime<Utc>)>,
}

impl<T> AuthLoginSuccessResponse<T> {
    pub fn new(
        message: String,
        data: T,
        metadata: Option<HashMap<String, String>>,
        http_only_refresh_token: Option<(String, DateTime<Utc>)>,
    ) -> Self {
        Self {
            message,
            data,
            metadata,
            http_only_refresh_token,
        }
    }
}

impl<T> IntoResponse for AuthLoginSuccessResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        let optional_refresh_token = self.http_only_refresh_token.clone();
        let json = Json(self);
        match optional_refresh_token {
            Some(refresh_token) => {
                let cookie = Cookie::build(("refresh_token", refresh_token.0))
                    .http_only(true)
                    .expires(Expiration::from(datetime_to_offset_datetime(
                        refresh_token.1,
                    )))
                    .path("/");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::SET_COOKIE, cookie.to_string())
                    .body(json.into_response().into_body())
                    .unwrap()
            }
            None => json.into_response(),
        }
    }
}

#[derive(Serialize)]
pub struct AuthLogoutSuccessResponse {
    message: String,
}

impl AuthLogoutSuccessResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl IntoResponse for AuthLogoutSuccessResponse {
    fn into_response(self) -> axum::response::Response {
        let json = Json(self);
        let cookie = Cookie::build(("refresh_token", ""))
            .http_only(true)
            .max_age(Duration::ZERO)
            .path("/");
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::SET_COOKIE, cookie.to_string())
            .body(json.into_response().into_body())
            .unwrap()
    }
}

#[derive(Serialize)]
pub struct SpmDownloadCsvSuccessResponse {
    pub data: Vec<u8>,
}

impl SpmDownloadCsvSuccessResponse {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl IntoResponse for SpmDownloadCsvSuccessResponse {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"))
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_static("attachment; filename=\"cage_data.csv\""),
            )
            .body(Body::from(self.data))
            .unwrap()
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
