use std::sync::Arc;

use mongodb::Client;

use crate::{
    dtos::cage_dtos::AddNewCageDto,
    models::cage::Cage,
    repository::{cage_repository::CageRepository, user_repository::UserRepository},
    utils::response::{ApiErrorResponse, ApiSuccessResponse},
};

pub struct CageService {
    client: Arc<Client>,
}

impl CageService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn add_new_cage(
        &self,
        user_id: String,
        add_new_cage: AddNewCageDto,
    ) -> Result<ApiSuccessResponse<Cage>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);
        let cage_repo = CageRepository::new(db);

        match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let cage = add_new_cage.to_model();

        let new_cage = cage_repo.create_new_cage(cage).await?;
        Ok(ApiSuccessResponse::new(
            String::from("New cage added succesfully"),
            new_cage,
            None,
        ))
    }
}
