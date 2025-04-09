use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cage {
    #[serde(rename = "_id")]
    pub id: String,
    pub assigned_monitor: String,
    pub livestock_no: u32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: u32,
    pub object_recognition: ObjectRecognition,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRecognition {
    pub coccidiosis: u32,
    pub newcastle: u32,
    pub salmonella: u32,
    pub healthy: u32,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateCage {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: u32,
    pub object_recognition: ObjectRecognition,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
