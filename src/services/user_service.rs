use mongodb::Client;
use std::sync::Arc;

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
}
