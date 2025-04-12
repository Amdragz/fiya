use chrono::Utc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    models::spm::{Cage, ObjectRecognition, UpdateCage},
    utils::helper::hash_id_with_secret,
};

#[derive(Deserialize, Validate)]
pub struct AddNewCageDto {
    #[validate(length(min = 1, message = "cageID is required"))]
    pub cage_id: String,
    #[validate(range(min = 0, message = "You must can not have negative livestock no"))]
    pub livestock_no: u32,
    #[validate(length(min = 1, message = "cageID is required"))]
    pub assigned_monitor: String,
}

impl AddNewCageDto {
    pub fn to_model(self) -> Cage {
        let id = hash_id_with_secret(&self.cage_id);
        Cage {
            id,
            livestock_no: self.livestock_no,
            assigned_monitor: self.assigned_monitor,
            co2: 0,
            ammonia: 0.0,
            humidity: 0.0,
            pressure: 0.0,
            temperature: 38.6,
            object_recognition: ObjectRecognition {
                coccidiosis: 0.0,
                newcastle: 0.0,
                salmonella: 0.0,
                healthy: 0.0,
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct UpdateCageDto {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: u32,
    pub object_recognition: ObjectRecognition,
}

impl UpdateCageDto {
    pub fn to_model(self) -> UpdateCage {
        UpdateCage {
            temperature: self.temperature,
            humidity: self.humidity,
            pressure: self.pressure,
            ammonia: self.ammonia,
            co2: self.co2,
            object_recognition: self.object_recognition,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CageDto {
    pub id: String,
    pub assigned_monitor: String,
    pub livestock_no: u32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: u32,
    pub object_recognition: ObjectRecognition,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Cage> for CageDto {
    fn from(cage: Cage) -> Self {
        CageDto {
            id: cage.id,
            assigned_monitor: cage.assigned_monitor,
            livestock_no: cage.livestock_no,
            temperature: cage.temperature,
            humidity: cage.humidity,
            pressure: cage.pressure,
            ammonia: cage.ammonia,
            co2: cage.co2,
            object_recognition: cage.object_recognition,
            created_at: cage.created_at.to_rfc3339(),
            updated_at: cage.updated_at.to_rfc3339(),
        }
    }
}
