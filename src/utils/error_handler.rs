use super::response::ApiErrorResponse;

pub fn internal_error<E>(err: E) -> ApiErrorResponse
where
    E: std::error::Error,
{
    ApiErrorResponse::new(500, err.to_string())
}

pub fn internal_server_error<E>(_: E, message: &str) -> ApiErrorResponse {
    ApiErrorResponse::new(500, message.to_string())
}

pub fn invalid_credentials_error<E>(_: E) -> ApiErrorResponse {
    ApiErrorResponse::new(401, "Invalid credentials".to_string())
}

pub fn access_denied_error<E>(_: E) -> ApiErrorResponse {
    ApiErrorResponse::new(403, "access denied".to_string())
}

pub fn bad_request_error<E>(_: E) -> ApiErrorResponse {
    ApiErrorResponse::new(400, "bad request".to_string())
}

pub fn not_found_error<E>(_: E, message: &str) -> ApiErrorResponse {
    ApiErrorResponse::new(404, message.to_string())
}

pub fn http_error<E>(_: E, status: u16, message: &str) -> ApiErrorResponse {
    ApiErrorResponse::new(status, message.to_string())
}
