use std::env;

use axum::http::StatusCode;
use chrono::{DateTime, Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::{error_handler::internal_error, response::ApiErrorResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub id: String,
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
}

pub fn new(user_id: String, user_role: String) -> Result<String, ApiErrorResponse> {
    dotenv().ok();
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = Utc::now();
    let expires_in = Duration::hours(1);
    let iat = now.timestamp() as usize;
    let expiry_date_time = now + expires_in;
    let exp = expiry_date_time.timestamp() as usize;

    let claims = Claims {
        exp,
        iat,
        sub: user_id,
        iss: String::from("Fiya webservice"),
        aud: String::from("Fiya webApp"),
        role: user_role,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(internal_error)?;
    Ok(token)
}

pub fn new_refresh_token(
    refresh_token_id: String,
    user_id: String,
) -> Result<(String, DateTime<Utc>), ApiErrorResponse> {
    dotenv().ok();
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let now = Utc::now();
    let expires_in = Duration::hours(1);
    let iat = now.timestamp() as usize;
    let expiry_date_time = now + expires_in;
    let exp = expiry_date_time.timestamp() as usize;

    let claims = RefreshTokenClaims {
        id: refresh_token_id,
        sub: user_id,
        iat,
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(internal_error)?;
    Ok((token, expiry_date_time))
}

pub fn verify<T: DeserializeOwned>(
    token: String,
    validate_aud: Option<bool>,
) -> Result<T, StatusCode> {
    dotenv().ok();
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let mut validation = Validation::default();
    let validate = validate_aud.unwrap_or(false);

    if validate {
        validation.set_audience(&["Fiya webApp"]);
    }
    let decoded_claims = decode::<T>(
        token.as_str(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    )
    .map_err(|err| {
        println!("{:?}", err.to_string());
        StatusCode::UNAUTHORIZED
    });
    decoded_claims.map(|dec| dec.claims)
}
