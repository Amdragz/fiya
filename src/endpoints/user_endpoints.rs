use axum::{extract::State, routing::post, Router};
use std::sync::Arc;

use crate::{
    dtos::user::CreateAdminUserDto,
    models::user::NewUser,
    services::user_service::UserService,
    utils::{
        response::{ApiErrorResponse, ApiSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn user_endpoints() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_admin_user))
}

async fn create_admin_user(
    State(app_state): State<Arc<AppState>>,
    ValidatedJson(payload): ValidatedJson<CreateAdminUserDto>,
) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
    let user_service = UserService::new(app_state.mongo_client.clone());
    user_service.create_admin_user(payload).await
}
