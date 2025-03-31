use bcrypt::hash;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use validator::Validate;

use crate::{
    models::user::{User, UserType},
    utils::{error_handler::internal_error, helper::generate_password, response::ApiErrorResponse},
};

#[derive(Deserialize, Validate)]
pub struct CreateAdminUserDto {
    #[validate(length(min = 1, message = "Name is required"))]
    name: String,
    #[validate(email(message = "Email is invalid"))]
    email: String,
    #[validate(length(min = 1, message = "Phone number is required"))]
    phone_number: String,
    #[validate(length(min = 1, message = "Password is required"))]
    password: String,
}

impl CreateAdminUserDto {
    pub fn to_model(self) -> Result<User, ApiErrorResponse> {
        let hashed_password = hash(self.password, 12).map_err(internal_error)?;
        Ok(User {
            id: ObjectId::new(),
            name: self.name,
            email: self.email,
            phone_number: self.phone_number,
            password: hashed_password,
            spm_id: None,
            r#type: UserType::Admin.to_string(),
            created_by: None,
            created_customers: Some(vec![]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}

#[derive(Deserialize, Validate)]
pub struct CreateCustomerDto {
    #[validate(length(min = 1, message = "Name is required"))]
    name: String,
    #[validate(email(message = "Email is invalid"))]
    email: String,
    #[validate(length(min = 1, message = "Phone number is required"))]
    phone_number: String,
    #[validate(length(min = 1, message = "Spm_id is required"))]
    spm_id: String,
}

impl CreateCustomerDto {
    pub fn to_model(self, admin_id: ObjectId) -> Result<User, ApiErrorResponse> {
        let password = generate_password(12);
        let hashed_password = hash(password, 12).map_err(internal_error)?;

        Ok(User {
            id: ObjectId::new(),
            name: self.name,
            email: self.email,
            phone_number: self.phone_number,
            password: hashed_password,
            spm_id: Some(self.spm_id),
            r#type: UserType::Customer.to_string(),
            created_by: Some(admin_id),
            created_customers: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
}
