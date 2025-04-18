use std::{str::FromStr, sync::Arc};

use axum_extra::headers::UserAgent;
use bcrypt::hash;
use mongodb::Client;

use crate::{
    dtos::auth_dto::{
        ChangePasswordDto, LoginDto, LoginSuccessDto, RefreshTokenRequestDto, UpdatePasswordDto,
    },
    models::{
        refresh_token::RefreshToken,
        user::{NewUser, UserType},
    },
    repository::user_repository::UserRepository,
    utils::{
        error_handler::{bad_request_error, http_error, internal_error, invalid_credentials_error},
        helper::is_browser,
        jwt::{self, RefreshTokenClaims},
        response::{
            ApiErrorResponse, ApiSuccessResponse, AuthLoginSuccessResponse,
            AuthLogoutSuccessResponse,
        },
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

        let (refresh_token, refresh_token_expiry_date) =
            jwt::new_refresh_token(user_id.clone().to_string(), user_id.to_string())
                .map_err(invalid_credentials_error)?;

        user_repo
            .create_user_refresh_token(RefreshToken {
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
            let refresh_token_claims = jwt::verify::<RefreshTokenClaims>(refresh_token, None)
                .map_err(bad_request_error)?;
            let valid_refresh_token = match user_repo
                .find_valid_user_refresh_token_by_user_id(&refresh_token_claims.id)
                .await
                .map_err(bad_request_error)?
            {
                Some(refresh_token) => refresh_token,
                None => {
                    return Err(ApiErrorResponse::new(
                        400,
                        String::from("Bad request --> 01"),
                    ))
                }
            };

            let valid_user = match user_repo
                .find_user_by_id(&valid_refresh_token.user_id.to_string())
                .await?
            {
                Some(user) => user,
                None => {
                    return Err(ApiErrorResponse::new(
                        400,
                        String::from("Bad request ---> 02"),
                    ))
                }
            };
            let user_id = valid_user.id;
            let access_token = jwt::new(user_id.clone().to_string(), valid_user.r#type)
                .map_err(bad_request_error)?;

            let (refresh_token, refresh_token_expiry) =
                jwt::new_refresh_token(user_id.clone().to_string(), user_id.clone().to_string())
                    .map_err(bad_request_error)?;

            let _ = user_repo
                .create_user_refresh_token(RefreshToken {
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

    pub async fn logout(
        &self,
        user_id: String,
    ) -> Result<AuthLogoutSuccessResponse, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);

        let _ = user_repo
            .delete_user_refresh_token(user_id)
            .await
            .map_err(|_| {
                ApiErrorResponse::new(500, String::from("logout operation not successful"))
            });

        Ok(AuthLogoutSuccessResponse::new(String::from(
            "Logout successful",
        )))
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

    pub async fn update_user_password(
        &self,
        user_id: String,
        payload: UpdatePasswordDto,
    ) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);

        let found_user = match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };
        let new_password = hash(payload.password, 12).map_err(internal_error)?;
        user_repo
            .update_user_password_by_id(&found_user.id.to_string(), new_password)
            .await?;

        Ok(ApiSuccessResponse::new(
            String::from("Succesfully updated user password"),
            (),
            None,
        ))
    }

    pub async fn change_user_password(
        &self,
        user_id: String,
        payload: ChangePasswordDto,
    ) -> Result<ApiSuccessResponse<()>, ApiErrorResponse> {
        let db = self.client.database("fiyadb");
        let user_repo = UserRepository::new(&db);

        let found_user = match user_repo.find_user_by_id(&user_id).await? {
            Some(user) => user,
            None => return Err(ApiErrorResponse::new(401, String::from("Unauthorized"))),
        };

        let valid = bcrypt::verify(payload.old_password, &found_user.password)
            .map_err(invalid_credentials_error)?;
        let new_password = hash(payload.new_password, 12).map_err(internal_error)?;
        if valid {
            user_repo
                .update_user_password_by_id(&found_user.id.to_string(), new_password)
                .await?;
            Ok(ApiSuccessResponse::new(
                String::from("Succesfully changed password"),
                (),
                None,
            ))
        } else {
            Err(ApiErrorResponse::new(
                400,
                String::from("Unable to update password"),
            ))
        }
    }
}
