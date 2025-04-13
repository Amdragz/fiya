use std::sync::Arc;

use chrono::Utc;
use futures::FutureExt;
use mongodb::Client;

use crate::{
    dtos::spm_dtos::{AddNewCageDto, CageDto, UpdateCageDto},
    models::spm::{CageWithDeviceToken, SpmDeviceToken},
    repository::{spm_repository::SpmRepository, user_repository::UserRepository},
    utils::{
        error_handler::internal_error,
        helper::{generate_secure_device_token, hash_id_with_secret},
        response::{ApiErrorResponse, ApiSuccessResponse},
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
        let spm_repo = SpmRepository::new(db);

        match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cage = add_new_cage.to_model();
        let (device_token, hashed_device_token) = generate_secure_device_token();

        let spm_device_token = SpmDeviceToken {
            id: cage.id.clone(),
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
            id: new_cage.id,
            device_token,
            assigned_monitor: new_cage.assigned_monitor,
            livestock_no: new_cage.livestock_no,
            temperature: new_cage.temperature,
            humidity: new_cage.humidity,
            pressure: new_cage.pressure,
            ammonia: new_cage.ammonia,
            co2: new_cage.co2,
            object_recognition: new_cage.object_recognition,
            created_at: new_cage.created_at,
            updated_at: new_cage.updated_at,
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
        let spm_repo = SpmRepository::new(db);

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
        let spm_repo = SpmRepository::new(db);

        let found_spm_device_token = match spm_repo.find_device_token_by_id(&cage_id).await? {
            Some(spm_device_token) => spm_device_token,
            None => return Err(ApiErrorResponse::new(403, String::from("Unauthorized"))),
        };

        let hashed_device_token = hash_id_with_secret(&device_token);
        if hashed_device_token != found_spm_device_token.token {
            return Err(ApiErrorResponse::new(403, String::from("Unauthorized")));
        }

        let found_cage = match spm_repo.find_cage_by_id(&cage_id).await? {
            Some(cage) => cage,
            None => {
                return Err(ApiErrorResponse::new(
                    401,
                    String::from("cage does not exits"),
                ))
            }
        };

        let update_cage = update_cage_dto.to_model();
        let _ = spm_repo
            .update_cage_by_id(&found_cage.id, update_cage)
            .await?;

        Ok(ApiSuccessResponse::new(
            String::from("Succesfully updated cage info"),
            (),
            None,
        ))
    }
}
