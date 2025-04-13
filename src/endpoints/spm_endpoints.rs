use std::sync::Arc;

use crate::{
    dtos::spm_dtos::{AddNewCageDto, CageDto, UpdateCageDto},
    middleware::auth_middleware::{self, SpmDeviceAuth},
    models::{spm::CageWithDeviceToken, user::AuthUserDto},
    services::spm_service::SpmService,
    utils::{
        response::{ApiErrorResponse, ApiSuccessResponse},
        validators::ValidatedJson,
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    middleware,
    routing::{get, post},
    Extension, Router,
};

pub fn spm_endpoints() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/cages",
            post(add_new_cage).layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/:cage_id",
            post(update_cage_info).layer(middleware::from_fn(auth_middleware::requires_spm_auth)),
        )
        .route(
            "/cages/:assined_monitor",
            get(fetch_all_users_cages).layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
}

pub async fn add_new_cage(
    State(app_state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
    ValidatedJson(payload): ValidatedJson<AddNewCageDto>,
) -> Result<ApiSuccessResponse<CageWithDeviceToken>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_state.mongo_client.clone());
    spm_service.add_new_cage(auth_user.id, payload).await
}

pub async fn update_cage_info(
    State(app_sate): State<Arc<AppState>>,
    Extension(spm_device_auth): Extension<SpmDeviceAuth>,
    Path(cage_id): Path<String>,
    ValidatedJson(payload): ValidatedJson<UpdateCageDto>,
) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service
        .update_cage_info(cage_id, payload, spm_device_auth.token)
        .await
}

pub async fn fetch_all_users_cages(
    State(app_sate): State<Arc<AppState>>,
    Extension(_): Extension<AuthUserDto>,
    Path(assigned_monitor): Path<String>,
) -> Result<ApiSuccessResponse<Vec<CageDto>>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service.fetch_all_users_cages(assigned_monitor).await
}
