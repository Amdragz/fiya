use std::sync::Arc;

use axum::{extract::State, middleware, routing::post, Extension, Router};

use crate::{
    dtos::cage_dtos::AddNewCageDto,
    middleware::auth_middleware,
    models::{cage::Cage, user::AuthUserDto},
    services::cage_service::CageService,
    utils::{
        response::{ApiErrorResponse, ApiSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};

pub fn cage_endpoints() -> Router<Arc<AppState>> {
    Router::new().route(
        "/",
        post(add_new_cage).layer(middleware::from_fn(auth_middleware::requires_auth)),
    )
}

pub async fn add_new_cage(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
    ValidatedJson(payload): ValidatedJson<AddNewCageDto>,
) -> Result<ApiSuccessResponse<Cage>, ApiErrorResponse> {
    let cage_service = CageService::new(app_state.mongo_client.clone());
    cage_service.add_new_cage(auth_user.id, payload).await
}
