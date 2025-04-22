use std::{fmt, str::FromStr};

use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::models::spm::{Cage, HealthSettings, ObjectRecognition};

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
            id: ObjectId::new(),
            cage_id: self.cage_id,
            livestock_no: self.livestock_no,
            assigned_monitor: self.assigned_monitor,
            co2: 0.0,
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
            timestamp: Utc::now(),
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
    pub co2: f32,
    pub object_recognition: ObjectRecognition,
    pub timestamp: DateTime<Utc>,
}

impl UpdateCageDto {
    pub fn to_model(self, cage_id: String, livestock_no: u32, assigned_monitor: String) -> Cage {
        Cage {
            id: ObjectId::new(),
            cage_id,
            livestock_no,
            assigned_monitor,
            co2: self.co2,
            ammonia: self.ammonia,
            humidity: self.humidity,
            pressure: self.pressure,
            temperature: self.temperature,
            object_recognition: self.object_recognition,
            timestamp: self.timestamp,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CageDto {
    pub id: String,
    pub cage_id: String,
    pub assigned_monitor: String,
    pub livestock_no: u32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: f32,
    pub object_recognition: ObjectRecognition,
    pub timestamp: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Cage> for CageDto {
    fn from(cage: Cage) -> Self {
        CageDto {
            id: cage.id.to_string(),
            cage_id: cage.cage_id,
            assigned_monitor: cage.assigned_monitor,
            livestock_no: cage.livestock_no,
            temperature: cage.temperature,
            humidity: cage.humidity,
            pressure: cage.pressure,
            ammonia: cage.ammonia,
            co2: cage.co2,
            object_recognition: cage.object_recognition,
            timestamp: cage.timestamp.to_rfc3339(),
            created_at: cage.created_at.to_rfc3339(),
            updated_at: cage.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CageCsvDto {
    #[serde(rename = "_id")]
    pub id: String,
    pub cage_id: String,
    pub assigned_monitor: String,
    pub livestock_no: u32,
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub ammonia: f32,
    pub co2: f32,
    pub coccidiosis: f32,
    pub newcastle: f32,
    pub salmonella: f32,
    pub healthy: f32,
    pub timestamp: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Cage> for CageCsvDto {
    fn from(cage: Cage) -> Self {
        CageCsvDto {
            id: cage.id.to_string(),
            cage_id: cage.cage_id,
            assigned_monitor: cage.assigned_monitor,
            livestock_no: cage.livestock_no,
            temperature: cage.temperature,
            humidity: cage.humidity,
            pressure: cage.pressure,
            ammonia: cage.ammonia,
            co2: cage.co2,
            coccidiosis: cage.object_recognition.coccidiosis,
            newcastle: cage.object_recognition.newcastle,
            salmonella: cage.object_recognition.salmonella,
            healthy: cage.object_recognition.healthy,
            timestamp: cage.timestamp.to_rfc3339(),
            created_at: cage.created_at.to_rfc3339(),
            updated_at: cage.updated_at.to_rfc3339(),
        }
    }
}

pub enum FileType {
    Csv,
    Pdf,
}

#[derive(Debug, Clone)]
pub struct ParseFileTypeError;

impl fmt::Display for ParseFileTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid file type")
    }
}

impl std::error::Error for ParseFileTypeError {}

impl FromStr for FileType {
    type Err = ParseFileTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "csv" | "Csv" | "CSV" => Ok(FileType::Csv),
            "pdf" | "Pdf" | "PDF" => Ok(FileType::Pdf),
            _ => Err(ParseFileTypeError),
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct DownloadCageReportDto {
    #[validate(length(min = 1, message = "cage id is required"))]
    pub cage_id: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    #[validate(length(min = 1, message = "file type is required"))]
    pub file_type: String,
}

#[derive(Validate, Deserialize)]
pub struct UpdateHealthSettingsDto {
    pub temperature: f32,
    pub pressure: f32,
    pub humidity: f32,
}

impl UpdateHealthSettingsDto {
    pub fn to_model(&self, cage_id: String) -> HealthSettings {
        HealthSettings {
            cage_id,
            temperature: self.temperature,
            pressure: self.pressure,
            humidity: self.humidity,
        }
    }
}
