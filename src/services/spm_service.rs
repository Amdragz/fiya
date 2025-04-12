use std::sync::Arc;

use mongodb::Client;

use crate::{
    dtos::spm_dtos::{AddNewCageDto, CageDto, UpdateCageDto},
    models::spm::Cage,
    repository::{spm_repository::SpmRepository, user_repository::UserRepository},
    utils::{
        helper::hash_id_with_secret,
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
    ) -> Result<ApiSuccessResponse<CageDto>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);
        let spm_repo = SpmRepository::new(db);

        match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cage = add_new_cage.to_model();

        let new_cage = spm_repo.create_new_cage(cage).await?;
        let cage_dto = CageDto::from(new_cage);

        Ok(ApiSuccessResponse::new(
            String::from("New cage added succesfully"),
            cage_dto,
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
    ) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let spm_repo = SpmRepository::new(db);

        let hashed_cage_id = hash_id_with_secret(&cage_id);
        let found_cage = match spm_repo.find_cage_by_id(&hashed_cage_id).await? {
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
