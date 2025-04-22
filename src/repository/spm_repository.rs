use bson::{doc, DateTime as BsonDateTime};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::{ClientSession, Collection, Database};

use crate::{
    models::spm::{Cage, HealthSettings, SpmDeviceToken},
    utils::{error_handler::internal_error, response::ApiErrorResponse},
};

pub struct SpmRepository {
    cages: Collection<Cage>,
    device_tokens: Collection<SpmDeviceToken>,
    health_settings: Collection<HealthSettings>,
}

impl SpmRepository {
    pub fn new(db: &Database) -> Self {
        let cages = db.collection("cage");
        let device_tokens = db.collection("device_token");
        let health_settings = db.collection("health_settings");

        Self {
            cages,
            device_tokens,
            health_settings,
        }
    }

    pub async fn create_new_cage(
        &self,
        session: &mut ClientSession,
        cage: Cage,
        spm_device_token: SpmDeviceToken,
    ) -> Result<Cage, ApiErrorResponse> {
        match self
            .device_tokens
            .insert_one(&spm_device_token)
            .session(&mut *session)
            .await
        {
            Ok(_) => println!("Whatever"),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => {
                return Err(ApiErrorResponse::new(
                    400,
                    String::from("Device token already exist"),
                ))
            }
            Err(err) => return Err(ApiErrorResponse::new(500, err.to_string())),
        };

        match self.cages.insert_one(&cage).session(&mut *session).await {
            Ok(_) => Ok(cage),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => Err(
                ApiErrorResponse::new(400, String::from("Cage already exist")),
            ),
            Err(err) => Err(ApiErrorResponse::new(500, err.to_string())),
        }
    }

    pub async fn find_cage_by_cage_id(&self, id: &str) -> Result<Option<Cage>, ApiErrorResponse> {
        let filter = doc! { "cage_id": id };
        let cage = self.cages.find_one(filter).await.map_err(internal_error)?;

        Ok(cage)
    }

    pub async fn find_device_token_by_id(
        &self,
        id: &str,
    ) -> Result<Option<SpmDeviceToken>, ApiErrorResponse> {
        let filter = doc! { "_id": id };
        let device_token = self
            .device_tokens
            .find_one(filter)
            .await
            .map_err(internal_error)?;

        Ok(device_token)
    }

    pub async fn find_all_users_cages(
        &self,
        assigned_monitor: String,
    ) -> Result<Vec<Cage>, ApiErrorResponse> {
        let filter = doc! { "assigned_monitor": assigned_monitor };

        let cursor = self.cages.find(filter).await.map_err(internal_error)?;
        let cages: Vec<Cage> = cursor.try_collect().await.map_err(internal_error)?;

        Ok(cages)
    }

    pub async fn find_cage_data_by_date_range(
        &self,
        cage_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<Vec<Cage>, ApiErrorResponse> {
        let filter = doc! {
            "cage_id": cage_id,
            "created_at": {
                "$gte": BsonDateTime::from_chrono(start_date),
                "$lte": BsonDateTime::from_chrono(end_date),
            }
        };

        let cage_cursor = self.cages.find(filter).await.map_err(internal_error)?;
        let cages: Vec<Cage> = cage_cursor.try_collect().await.map_err(internal_error)?;
        Ok(cages)
    }

    pub async fn add_cage_new_info(&self, new_cage_info: Cage) -> Result<Cage, ApiErrorResponse> {
        let result = self.cages.insert_one(&new_cage_info).await;

        match result {
            Ok(_) => Ok(new_cage_info),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => Err(
                ApiErrorResponse::new(400, "Cage already exists".to_string()),
            ),
            Err(err) => Err(internal_error(err)),
        }
    }

    pub async fn update_health_settings(
        &self,
        health_settings: HealthSettings,
    ) -> Result<HealthSettings, ApiErrorResponse> {
        self.health_settings
            .replace_one(
                doc! { "cage_id": &health_settings.cage_id },
                &health_settings,
            )
            .upsert(true)
            .await
            .map_err(internal_error)?;
        Ok(health_settings)
    }
}
