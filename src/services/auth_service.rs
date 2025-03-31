use std::{str::FromStr, sync::Arc};

use axum_extra::headers::UserAgent;
use mongodb::{bson::oid::ObjectId, Client};

use crate::{
    dtos::auth_dto::{LoginDto, LoginSuccessDto},
    models::{refresh_token::RefreshToken, user::UserType},
    repository::user_repository::UserRepository,
    utils::{
        error_handler::{http_error, invalid_credentials_error},
        helper::is_browser,
        jwt,
        response::{ApiErrorResponse, AuthLoginSuccessResponse},
    },
};

pub struct AuthService {
    client: Arc<Client>,
}

impl AuthService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    pub async fn login(
        &self,
        user_agent: UserAgent,
        payload: LoginDto,
    ) -> Result<AuthLoginSuccessResponse<LoginSuccessDto>, ApiErrorResponse> {
        let database = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&database);

        let user_type = payload.user_type.unwrap_or(UserType::Admin.to_string());
        let _ =
            UserType::from_str(&user_type).map_err(|e| http_error(e, 400, "Invalid user type"))?;

        let found_user = user_repo.find_user_by_email(&payload.email).await?;

        let found_user = match found_user {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("User not found"))),
        };

        if user_type != found_user.r#type {
            return Err(ApiErrorResponse::new(
                401,
                String::from("Invalid credentials -> 02"),
            ));
        }

        let valid = bcrypt::verify(payload.password, &found_user.password)
            .map_err(invalid_credentials_error)?;

        if !valid {
            return Err(ApiErrorResponse::new(
                401,
                String::from("Invalid credentials -> 03"),
            ));
        }

        let user_id = found_user.id;
        let access_token = jwt::new(user_id.clone().to_string(), found_user.r#type)
            .map_err(invalid_credentials_error)?;

        let refresh_token_id = ObjectId::new();
        let (refresh_token, refresh_token_expiry_date) =
            jwt::new_refresh_token(refresh_token_id.clone().to_string(), user_id.to_string())
                .map_err(invalid_credentials_error)?;

        user_repo
            .create_refresh_token(RefreshToken {
                id: refresh_token_id,
                user_id,
                refresh_token: refresh_token.clone(),
                expires_at: refresh_token_expiry_date,
                revoked: None,
            })
            .await?;

        let token_type = "Bearer".to_string();
        let http_only_refresh_token =
            is_browser(user_agent).then(|| (refresh_token.clone(), refresh_token_expiry_date));

        Ok(AuthLoginSuccessResponse::new(
            String::from("Login successful"),
            LoginSuccessDto {
                access_token,
                refresh_token,
                token_type,
            },
            None,
            http_only_refresh_token,
        ))
    }
}
