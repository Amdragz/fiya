use chrono::Utc;
use serde::Deserialize;
use validator::Validate;

use crate::models::cage::Cage;

#[derive(Deserialize, Validate)]
pub struct AddNewCageDto {
    #[validate(length(min = 1, message = "cageID is required"))]
    pub cage_id: String,
    pub livestock_no: i32,
    #[validate(length(min = 1, message = "cageID is required"))]
    pub assigned_monitor: String,
}

impl AddNewCageDto {
    pub fn to_model(self) -> Cage {
        Cage {
            cage_id: self.cage_id,
            livestock_no: self.livestock_no,
            assigned_monitor: self.assigned_monitor,
            date_added: Utc::now(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
