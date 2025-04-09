use chrono::Utc;
use serde::Deserialize;
use validator::Validate;

use crate::models::spm::{Cage, ObjectRecognition, UpdateCage};

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
        Cage {
            id: self.cage_id,
            livestock_no: self.livestock_no,
            assigned_monitor: self.assigned_monitor,
            co2: 0,
            ammonia: 0.0,
            humidity: 0.0,
            pressure: 0.0,
            temperature: 38.6,
            object_recognition: ObjectRecognition {
                coccidiosis: 0,
                newcastle: 0,
                salmonella: 0,
                healthy: 0,
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
