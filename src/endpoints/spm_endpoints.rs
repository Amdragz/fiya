use std::{str::FromStr, sync::Arc};

use crate::{
    dtos::spm_dtos::{
        AddNewCageDto, CageDto, DownloadCageReportDto, FileType, UpdateCageDto,
        UpdateHealthSettingsDto,
    },
    middleware::auth_middleware::{self, SpmDeviceAuth},
    models::{
        spm::{CageWithDeviceToken, HealthSettings},
        user::AuthUserDto,
    },
    services::spm_service::SpmService,
    utils::{
        error_handler::internal_error,
        response::{
            ApiErrorResponse, ApiSuccessResponse, SpmDownloadCsvSuccessResponse,
            SpmDownloadPdfSuccessResponse,
        },
        validators::ValidatedJson,
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};

pub fn spm_endpoints() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/cages",
            post(add_new_cage)
                .get(fetch_all_users_cage_data)
                .layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/:cage_id",
            post(update_cage_info).layer(middleware::from_fn(auth_middleware::requires_spm_auth)),
        )
        .route(
            "/report",
            post(export_cage_data).layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/export/csv",
            get(download_cage_report_in_csv_format)
                .layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/export/pdf",
            get(download_cage_report_in_pdf_format)
                .layer(middleware::from_fn(auth_middleware::requires_auth)),
        )
        .route(
            "/:cage_id/health-settings",
            post(update_users_cage_health_settings)
                .get(get_users_cage_health_settings)
                .layer(middleware::from_fn(auth_middleware::requires_auth)),
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

pub async fn fetch_all_users_cage_data(
    State(app_sate): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
) -> Result<ApiSuccessResponse<Vec<CageDto>>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service.fetch_all_users_cage_data(auth_user.id).await
}

pub async fn download_cage_report_in_csv_format(
    State(app_sate): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
) -> Result<SpmDownloadCsvSuccessResponse, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service
        .fetch_all_cage_data_in_csv_format(auth_user.id)
        .await
}

pub async fn download_cage_report_in_pdf_format(
    State(app_sate): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
) -> Result<SpmDownloadPdfSuccessResponse, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service
        .fetch_all_cage_data_in_pdf_format(auth_user.id)
        .await
}

pub async fn export_cage_data(
    State(app_sate): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUserDto>,
    ValidatedJson(payload): ValidatedJson<DownloadCageReportDto>,
) -> Result<impl IntoResponse, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    match FileType::from_str(&payload.file_type).map_err(internal_error)? {
        FileType::Pdf => {
            let pdf_response = spm_service
                .generate_cage_report_in_pdf_format(auth_user.id, payload)
                .await?;
            Ok(pdf_response.into_response())
        }
        FileType::Csv => {
            let csv_response = spm_service
                .generate_cage_report_in_csv_format(auth_user.id, payload)
                .await?;
            Ok(csv_response.into_response())
        }
    }
}

pub async fn update_users_cage_health_settings(
    State(app_sate): State<Arc<AppState>>,
    Extension(_): Extension<AuthUserDto>,
    Path(cage_id): Path<String>,
    ValidatedJson(payload): ValidatedJson<UpdateHealthSettingsDto>,
) -> Result<ApiSuccessResponse<HealthSettings>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service
        .update_cage_health_settings(cage_id, payload)
        .await
}

pub async fn get_users_cage_health_settings(
    State(app_sate): State<Arc<AppState>>,
    Extension(_): Extension<AuthUserDto>,
    Path(cage_id): Path<String>,
) -> Result<ApiSuccessResponse<HealthSettings>, ApiErrorResponse> {
    let spm_service = SpmService::new(app_sate.mongo_client.clone());
    spm_service
        .get_cage_health_settings_by_cage_id(cage_id)
        .await
}
