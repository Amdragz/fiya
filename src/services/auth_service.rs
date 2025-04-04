use std::{str::FromStr, sync::Arc};

use axum_extra::headers::UserAgent;
use mongodb::{bson::oid::ObjectId, Client};

use crate::{
    dtos::auth_dto::{LoginDto, LoginSuccessDto, RefreshTokenRequestDto},
    models::{refresh_token::RefreshToken, user::UserType},
    repository::user_repository::UserRepository,
    utils::{
        error_handler::{bad_request_error, http_error, invalid_credentials_error},
        helper::is_browser,
        jwt::{self, RefreshTokenClaims},
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

    pub async fn refresh_user_token(
        &self,
        user_agent: UserAgent,
        refresh_token_from_cookie: Option<String>,
        payload: RefreshTokenRequestDto,
    ) -> Result<AuthLoginSuccessResponse<LoginSuccessDto>, ApiErrorResponse> {
        let database = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&database);

        let refresh_token = match (refresh_token_from_cookie, payload.refresh_token) {
            (Some(_), Some(payload_refresh_token)) => Some(payload_refresh_token),
            (Some(cookie_refresh_token), None) => Some(cookie_refresh_token),
            (None, Some(payload_refresh_token)) => Some(payload_refresh_token),
            _ => None,
        };

        if let Some(refresh_token) = refresh_token {
            let refresh_token_claims =
                jwt::verify::<RefreshTokenClaims>(refresh_token).map_err(bad_request_error)?;
            let valid_refresh_token = match user_repo
                .find_valid_user_refresh_token_by_id(&refresh_token_claims.id)
                .await
                .map_err(bad_request_error)?
            {
                Some(refresh_token) => refresh_token,
                None => return Err(ApiErrorResponse::new(400, String::from("Bad request"))),
            };

            let valid_user = match user_repo
                .find_user_by_id(&valid_refresh_token.user_id.to_string())
                .await?
            {
                Some(user) => user,
                None => return Err(ApiErrorResponse::new(400, String::from("Bad request"))),
            };
            let user_id = valid_user.id;
            let access_token = jwt::new(user_id.clone().to_string(), valid_user.r#type)
                .map_err(bad_request_error)?;

            let refresh_token_id = ObjectId::new();
            let (refresh_token, refresh_token_expiry) =
                jwt::new_refresh_token(refresh_token_id.to_string(), user_id.clone().to_string())
                    .map_err(bad_request_error)?;

            let _ = user_repo
                .create_refresh_token(RefreshToken {
                    id: ObjectId::new(),
                    user_id,
                    refresh_token: refresh_token.clone(),
                    expires_at: refresh_token_expiry,
                    revoked: None,
                })
                .await?;
            let token_type = String::from("Bearer");

            let http_only_refresh_token =
                is_browser(user_agent).then(|| (refresh_token.clone(), refresh_token_expiry));
            Ok(AuthLoginSuccessResponse::new(
                "token refreshed successfull".to_string(),
                LoginSuccessDto {
                    access_token,
                    refresh_token,
                    token_type,
                },
                None,
                http_only_refresh_token,
            ))
        } else {
            Err(ApiErrorResponse::new(
                401,
                "Invalid refresh token request".to_string(),
            ))
        }
    }
}
