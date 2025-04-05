use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Deserialize, Validate, Serialize)]
pub struct LoginDto {
    #[validate(length(min = 1, message = "email can not be empty"))]
    pub email: String,
    #[validate(length(min = 1, message = "password can not be empty"))]
    pub password: String,
    #[validate(length(min = 1, message = "user_type can not be empty"))]
    pub user_type: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct LoginSuccessDto {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Deserialize, Validate)]
pub struct RefreshTokenRequestDto {
    #[validate(length(min = 1, message = "refresh_token can not be empty"))]
    pub refresh_token: Option<String>,
}

#[derive(Deserialize, Validate, Serialize, Debug)]
pub struct ChangePasswordDto {
    #[validate(length(min = 1, message = "new_password can not be empty"))]
    pub old_password: String,
    #[validate(length(min = 1, message = "old_password can not be empty"))]
    pub new_password: String,
}

#[derive(Deserialize, Validate)]
pub struct UpdatePasswordDto {
    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}
