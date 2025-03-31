use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantNames};

#[derive(Serialize, Deserialize, Debug, EnumString, VariantNames, Display)]
#[strum(serialize_all = "snake_case")]
pub enum UserType {
    Admin,
    Customer,
}

#[derive(Clone)]
pub struct AuthUserDto {
    pub id: String,
    pub user_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_customers: Option<Vec<ObjectId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spm_id: Option<String>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct NewUser {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
    pub email: String,
    pub phone_number: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_customers: Option<Vec<ObjectId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spm_id: Option<String>,
}
