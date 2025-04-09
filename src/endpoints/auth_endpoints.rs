use std::sync::Arc;

use axum::{extract::State, middleware, routing::post, Extension, Json, Router};
use axum_extra::{extract::CookieJar, headers::UserAgent, TypedHeader};

use crate::{
    dtos::auth_dto::{
        ChangePasswordDto, LoginDto, LoginSuccessDto, RefreshTokenRequestDto, UpdatePasswordDto,
    },
    middleware::auth_middleware,
    models::user::AuthUserDto,
    services::auth_service::AuthService,
    utils::{
        response::{ApiErrorResponse, ApiSuccessResponse, AuthLoginSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn auth_endpoints() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh-token", post(refresh_user_token))
        .route(
            "/update-password",
            post(update_user_one_time_password)
                .layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/change-password",
            post(change_user_password).layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
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

async fn update_user_one_time_password(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
    ValidatedJson(payload): ValidatedJson<UpdatePasswordDto>,
) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
    let auth_service = AuthService::new(app_state.mongo_client.clone());
    auth_service
        .update_user_password(auth_user.id, payload)
        .await
}

async fn change_user_password(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
    ValidatedJson(payload): ValidatedJson<ChangePasswordDto>,
) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
    let auth_service = AuthService::new(app_state.mongo_client.clone());
    auth_service
        .change_user_password(auth_user.id, payload)
        .await
}
