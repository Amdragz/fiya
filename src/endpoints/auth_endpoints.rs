use std::sync::Arc;

use axum::{extract::State, routing::post, Router};
use axum_extra::{headers::UserAgent, TypedHeader};

use crate::{
    dtos::auth_dto::{LoginDto, LoginSuccessDto},
    services::auth_service::AuthService,
    utils::{
        response::{ApiErrorResponse, AuthLoginSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn auth_endpoints() -> Router<Arc<AppState>> {
    Router::new().route("/login", post(login))
}

async fn login(
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(app_state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<LoginDto>,
) -> Result<AuthLoginSuccessResponse<LoginSuccessDto>, ApiErrorResponse> {
    let auth_service = AuthService::new(app_state.mongo_client.clone());
    auth_service.login(user_agent, payload).await
}
