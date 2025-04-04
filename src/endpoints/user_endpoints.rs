use axum::{
    extract::{Path, State},
    middleware,
    routing::{get, post},
    Extension, Router,
};
use std::sync::Arc;

use crate::{
    dtos::user::{CreateAdminUserDto, CreateCustomerDto},
    middleware::auth_middleware,
    models::user::{AuthUserDto, NewUser},
    services::user_service::UserService,
    utils::{
        response::{ApiErrorResponse, ApiSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn user_endpoints() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_admin_user))
        .route("/:id/customer", post(create_customer_user))
        .route(
            "/:id",
            get(get_authenticated_user).layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
}

async fn create_admin_user(
    State(app_state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateAdminUserDto>,
) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
    let user_service = UserService::new(app_state.mongo_client.clone());
    user_service.create_admin_user(payload).await
}

async fn create_customer_user(
    State(app_state): State<Arc<AppState>>,
    Path(admin_id): Path<String>,
    ValidatedJson(payload): ValidatedJson<CreateCustomerDto>,
) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
    let user_service = UserService::new(app_state.mongo_client.clone());
    user_service.create_customer_user(admin_id, payload).await
}

async fn get_authenticated_user(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
    let user_service = UserService::new(app_state.mongo_client.clone());
    user_service.get_authenticated_user(auth_user.id).await
}
