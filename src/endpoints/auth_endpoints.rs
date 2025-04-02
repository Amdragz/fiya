use std::sync::Arc;

use axum::{extract::State, routing::post, Json, Router};
use axum_extra::{extract::CookieJar, headers::UserAgent, TypedHeader};

use crate::{
    dtos::auth_dto::{LoginDto, LoginSuccessDto, RefreshTokenRequestDto},
    services::auth_service::AuthService,
    utils::{
        response::{ApiErrorResponse, AuthLoginSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn auth_endpoints() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh-token", post(refresh_user_token))
}

async fn login(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(app_state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> Result<AuthLoginSuccessResponse<LoginSuccessDto>, ApiErrorResponse> {
    let auth_service = AuthService::new(app_state.mongo_client.clone());
    auth_service.login(user_agent, payload).await
}

async fn refresh_user_token(
    jar: CookieJar,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<RefreshTokenRequestDto>,
) -> Result<AuthLoginSuccessResponse<LoginSuccessDto>, ApiErrorResponse> {
    let refresh_token_from_cookie = jar.get("refresh_token").map(|c| c.value().to_owned());
    let auth_service = AuthService::new(app_state.mongo_client.clone());
    auth_service
        .refresh_user_token(user_agent, refresh_token_from_cookie, payload)
        .await
}
