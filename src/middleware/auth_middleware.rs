use axum::{extract::Request, http::header::AUTHORIZATION, middleware::Next, response::Response};

use crate::{
    models::user::AuthUserDto,
    utils::{
        error_handler::invalid_credentials_error,
        jwt::{self, Claims},
        response::ApiErrorResponse,
    },
};

pub async fn requires_auth(mut req: Request, next: Next) -> Result<Response, ApiErrorResponse> {
    let bearer_token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match bearer_token {
        Some(token) => token,
        None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
    };

    let claims: Claims =
        jwt::verify(token.to_string(), Some(true)).map_err(invalid_credentials_error)?;
    let current_user = AuthUserDto {
        id: claims.sub,
        user_type: claims.role,
    };
    req.extensions_mut().insert(current_user);
    let res = next.run(req).await;
    Ok(res)
}

pub async fn requires_spm_auth(mut req: Request, next: Next) -> Result<Response, ApiErrorResponse> {
    let bearer_token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match bearer_token {
        Some(token) => token,
        None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
    };

    let spm_device_auth = SpmDeviceAuth {
        token: token.to_string(),
    };

    req.extensions_mut().insert(spm_device_auth);
    let res = next.run(req).await;
    Ok(res)
}

#[derive(Clone)]
pub struct SpmDeviceAuth {
    pub token: String,
}
