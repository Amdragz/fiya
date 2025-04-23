use std::{io::Cursor, sync::Arc};

use chrono::Utc;
use csv::WriterBuilder;
use mongodb::Client;

use crate::{
    dtos::spm_dtos::{
        AddNewCageDto, CageCsvDto, CageDto, DownloadCageReportDto, UpdateCageDto,
        UpdateHealthSettingsDto,
    },
    models::spm::{CageWithDeviceToken, HealthSettings, SpmDeviceToken},
    repository::{spm_repository::SpmRepository, user_repository::UserRepository},
    utils::{
        error_handler::{internal_error, internal_server_error},
        helper::{generate_pdf_for_cage_data, generate_secure_device_token, hash_id_with_secret},
        response::{
            ApiErrorResponse, ApiSuccessResponse, SpmDownloadCsvSuccessResponse,
            SpmDownloadPdfSuccessResponse,
        },
    },
};

pub struct SpmService {
    client: Arc<Client>,
}

impl SpmService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn add_new_cage(
        &self,
        user_id: String,
        add_new_cage: AddNewCageDto,
    ) -> Result<ApiSuccessResponse<CageWithDeviceToken>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);
        let spm_repo = SpmRepository::new(&db);

        match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cage = add_new_cage.to_model();
        let (device_token, hashed_device_token) = generate_secure_device_token();

        let spm_device_token = SpmDeviceToken {
            id: cage.cage_id.clone(),
            token: hashed_device_token,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let mut session = self.client.start_session().await.map_err(internal_error)?;
        session.start_transaction().await.map_err(internal_error)?;

        let new_cage_result = match spm_repo
            .create_new_cage(&mut session, cage, spm_device_token)
            .await
        {
            Ok(cage) => {
                session.commit_transaction().await.map_err(internal_error)?;
                Ok(cage)
            }
            Err(err) => {
                session.abort_transaction().await.map_err(internal_error)?;
                Err(err)
            }
        };

        let new_cage = new_cage_result?;
        let cage_with_device_token = CageWithDeviceToken {
            id: new_cage.id.to_string(),
            cage_id: new_cage.cage_id,
            device_token,
            assigned_monitor: new_cage.assigned_monitor,
            livestock_no: new_cage.livestock_no,
            temperature: new_cage.temperature,
            humidity: new_cage.humidity,
            pressure: new_cage.pressure,
            ammonia: new_cage.ammonia,
            co2: new_cage.co2,
            object_recognition: new_cage.object_recognition,
            timestamp: new_cage.timestamp.to_rfc3339(),
            created_at: new_cage.created_at.to_rfc3339(),
            updated_at: new_cage.updated_at.to_rfc3339(),
        };

        Ok(ApiSuccessResponse::new(
            String::from("New cage added succesfully"),
            cage_with_device_token,
            None,
        ))
    }

    pub async fn fetch_all_users_cages(
        &self,
        assigned_monitor: String,
    ) -> Result<ApiSuccessResponse<Vec<CageDto>>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);

        let cages = spm_repo.find_all_users_cages(assigned_monitor).await?;
        let cage_dtos = cages.into_iter().map(CageDto::from).collect();

        Ok(ApiSuccessResponse::new(
            String::from("Succesfully fetched all users cages"),
            cage_dtos,
            None,
        ))
    }

    pub async fn update_cage_info(
        &self,
        cage_id: String,
        update_cage_dto: UpdateCageDto,
        device_token: String,
    ) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);

        let found_spm_device_token = match spm_repo.find_device_token_by_id(&cage_id).await? {
            Some(spm_device_token) => spm_device_token,
            None => return Err(ApiErrorResponse::new(403, String::from("Unauthorized"))),
        };

        let hashed_device_token = hash_id_with_secret(&device_token);
        if hashed_device_token != found_spm_device_token.token {
            return Err(ApiErrorResponse::new(403, String::from("Unauthorized")));
        }

        let found_cage = match spm_repo.find_cage_by_cage_id(&cage_id).await? {
            Some(cage) => cage,
            None => {
                return Err(ApiErrorResponse::new(
                    401,
                    String::from("cage does not exits"),
                ))
            }
        };

        let update_cage = update_cage_dto.to_model(
            found_cage.cage_id,
            found_cage.livestock_no,
            found_cage.assigned_monitor,
        );
        let _ = spm_repo.add_cage_new_info(update_cage).await?;

        Ok(ApiSuccessResponse::new(
            String::from("Succesfully updated cage info"),
            (),
            None,
        ))
    }

    pub async fn generate_cage_report_in_csv_format(
        &self,
        id: String,
        payload: DownloadCageReportDto,
    ) -> Result<SpmDownloadCsvSuccessResponse, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);
        let spm_repo = SpmRepository::new(&db);

        match user_repo.find_user_by_id(&id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cages = spm_repo
            .find_cage_data_by_date_range(&payload.cage_id, payload.start_date, payload.end_date)
            .await?;

        let mut wrt = WriterBuilder::new().from_writer(Cursor::new(Vec::new()));

        cages
            .iter()
            .try_for_each(|cage| wrt.serialize(CageCsvDto::from(cage.clone())))
            .map_err(internal_error)?;

        let cage_csv = wrt
            .into_inner()
            .map(|cursor| cursor.into_inner())
            .map_err(|err| internal_server_error(err, "Error creating csv from recors"))?;

        Ok(SpmDownloadCsvSuccessResponse::new(cage_csv))
    }

    pub async fn generate_cage_report_in_pdf_format(
        &self,
        id: String,
        payload: DownloadCageReportDto,
    ) -> Result<SpmDownloadPdfSuccessResponse, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);
        let spm_repo = SpmRepository::new(&db);

        match user_repo.find_user_by_id(&id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cages = spm_repo
            .find_cage_data_by_date_range(&payload.cage_id, payload.start_date, payload.end_date)
            .await?;

        let pdf_data = generate_pdf_for_cage_data(cages).map_err(internal_error)?;
        Ok(SpmDownloadPdfSuccessResponse::new(pdf_data))
    }

    pub async fn fetch_all_cage_data_in_csv_format(
        &self,
        id: String,
    ) -> Result<SpmDownloadCsvSuccessResponse, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);
        let user_repo = UserRepository::new(&db);

        let found_user = match user_repo.find_user_by_id(&id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(403, String::from("Unauthorized"))),
        };

        let cages = spm_repo
            .find_all_users_cages(found_user.id.to_string())
            .await?;
        let mut wrt = WriterBuilder::new().from_writer(Cursor::new(Vec::new()));

        for cage in cages {
            let cage_csv = CageCsvDto::from(cage);
            wrt.serialize(cage_csv).map_err(internal_error)?;
        }

        let cage_csv = wrt
            .into_inner()
            .map(|cursor| cursor.into_inner())
            .map_err(|err| internal_server_error(err, "Error creating csv from recors"))?;

        Ok(SpmDownloadCsvSuccessResponse::new(cage_csv))
    }

    pub async fn fetch_all_cage_data_in_pdf_format(
        &self,
        id: String,
    ) -> Result<SpmDownloadPdfSuccessResponse, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);
        let user_repo = UserRepository::new(&db);

        let found_user = match user_repo.find_user_by_id(&id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(403, String::from("Unauthorized"))),
        };

        let cages = spm_repo
            .find_all_users_cages(found_user.id.to_string())
            .await?;
        let pdf_data = generate_pdf_for_cage_data(cages).map_err(internal_error)?;
        Ok(SpmDownloadPdfSuccessResponse::new(pdf_data))
    }

    pub async fn get_cage_health_settings_by_cage_id(
        &self,
        cage_id: String,
    ) -> Result<ApiSuccessResponse<HealthSettings>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);

        if (spm_repo.find_cage_by_cage_id(&cage_id).await?).is_none() {
            return Err(ApiErrorResponse::new(
                401,
                String::from("Cage does not exit"),
            ));
        }

        let health_settings = match spm_repo.find_health_settings_by_cage_id(&cage_id).await? {
            Some(health_settings) => health_settings,
            None => {
                return Err(ApiErrorResponse::new(
                    404,
                    String::from("cage health settings do not exist"),
                ))
            }
        };

        Ok(ApiSuccessResponse::new(
            String::from("Successfully got cage settings"),
            health_settings,
            None,
        ))
    }

    pub async fn update_cage_health_settings(
        &self,
        cage_id: String,
        update_health_settings_dto: UpdateHealthSettingsDto,
    ) -> Result<ApiSuccessResponse<HealthSettings>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(&db);

        if (spm_repo.find_cage_by_cage_id(&cage_id).await?).is_none() {
            return Err(ApiErrorResponse::new(
                403,
                String::from("Cage does not exist"),
            ));
        }

        let health_settings = update_health_settings_dto.to_model(cage_id);
        let updated_health_settings = spm_repo.update_health_settings(health_settings).await?;
        Ok(ApiSuccessResponse::new(
            String::from("Successfully updated health settings"),
            updated_health_settings,
            None,
        ))
    }
}
