use bson::doc;
use futures::TryStreamExt;
use mongodb::{ClientSession, Collection, Database};

use crate::{
    models::spm::{Cage, SpmDeviceToken, UpdateCage},
    utils::{error_handler::internal_error, response::ApiErrorResponse},
};

pub struct SpmRepository {
    cages: Collection<Cage>,
    device_tokens: Collection<SpmDeviceToken>,
}

impl SpmRepository {
    pub fn new(db: Database) -> Self {
        let cages = db.collection("cage");
        let device_tokens = db.collection("device_token");

        Self {
            cages,
            device_tokens,
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
                    String::from("Cage already exist"),
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

    pub async fn find_cage_by_id(&self, id: &str) -> Result<Option<Cage>, ApiErrorResponse> {
        let filter = doc! { "_id": id };
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

    pub async fn update_cage_by_id(
        &self,
        cage_id: &str,
        update_cage: UpdateCage,
    ) -> Result<UpdateCage, ApiErrorResponse> {
        let filter = doc! { "_id": cage_id };
        let update = doc! {
            "$set":
            {
                "temperature": update_cage.temperature,
                "humidity": update_cage.humidity,
                "pressure": update_cage.pressure,
                "ammonia": update_cage.ammonia,
                "co2": update_cage.co2,
                "coccidiosis": update_cage.object_recognition.coccidiosis,
                "newcastle": update_cage.object_recognition.newcastle,
                "salmonella": update_cage.object_recognition.salmonella,
                "healthy": update_cage.object_recognition.healthy
            }
        };

        let result = self
            .cages
            .update_one(filter, update)
            .await
            .map_err(internal_error)?;

        match (result.matched_count, result.modified_count) {
            (0, _) => Err(ApiErrorResponse::new(404, String::from("user not found"))),
            (_, 0) => Err(ApiErrorResponse::new(
                200,
                String::from("No changes made to the data"),
            )),
            _ => Ok(update_cage),
        }
    }
}
