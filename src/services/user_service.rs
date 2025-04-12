use std::sync::Arc;

use mongodb::Client;

use crate::{
    dtos::user::{CreateAdminUserDto, CreateCustomerDto},
    models::user::NewUser,
    repository::user_repository::UserRepository,
    utils::response::{ApiErrorResponse, ApiSuccessResponse},
};

pub struct UserService {
    client: Arc<Client>,
}

impl UserService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn create_admin_user(
        &self,
        payload: CreateAdminUserDto,
    ) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
        let database = self.client.database("fiyadb");
        let user_repository = UserRepository::new_async(&database).await?;

        let new_user = payload.to_model()?;
        let user = user_repository.create_user(new_user).await?;

        Ok(ApiSuccessResponse::new(
            String::from("Succesfully created a user"),
            user,
            None,
        ))
    }

    pub async fn create_customer_user(
        &self,
        admin_id: String,
        payload: CreateCustomerDto,
    ) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
        let database = self.client.database("fiyadb");
        let user_repository = UserRepository::new_async(&database).await?;
        let admin_user = user_repository.find_admin_user_by_id(admin_id).await?;

        match admin_user {
            Some(admin_user) => {
                if let Some(created_customers) = admin_user.created_customers {
                    if created_customers.len() > 4 {
                        return Err(ApiErrorResponse::new(
                            401,
                            String::from("Maximum number of customers has been created"),
                        ));
                    }
                }

                let new_user = payload.to_model(admin_user.id)?;
                let user = user_repository.create_user(new_user).await?;

                Ok(ApiSuccessResponse::new(
                    String::from("Succesfully created a user"),
                    user,
                    None,
                ))
            }
            None => Err(ApiErrorResponse::new(
                401,
                String::from("Unauthorized user doesn't exist"),
            )),
        }
    }

    pub async fn get_authenticated_user(
        &self,
        id: String,
    ) -> Result<ApiSuccessResponse<NewUser>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);

        let user = user_repo.find_user_by_id(&id).await?;

        match user {
            Some(found_user) => {
                let user = NewUser {
                    id: found_user.id.to_string(),
                    name: found_user.name,
                    r#type: found_user.r#type,
                    email: found_user.email,
                    spm_id: found_user.spm_id,
                    created_by: found_user.created_by,
                    phone_number: found_user.phone_number,
                    created_customers: found_user.created_customers,
                    created_at: found_user.created_at,
                    updated_at: found_user.updated_at,
                };

                Ok(ApiSuccessResponse::new(
                    String::from("Succesfully fetched authenticated user"),
                    user,
                    None,
                ))
            }
            None => Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        }
    }
}
