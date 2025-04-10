use bson::doc;
use futures::TryStreamExt;
use mongodb::{Collection, Database};

use crate::{
    models::spm::{Cage, UpdateCage},
    utils::{error_handler::internal_error, response::ApiErrorResponse},
};

pub struct SpmRepository {
    collection: Collection<Cage>,
}

impl SpmRepository {
    pub fn new(db: Database) -> Self {
        let collection = db.collection("cage");

        Self { collection }
    }

    pub async fn create_new_cage(&self, cage: Cage) -> Result<Cage, ApiErrorResponse> {
        let result = self.collection.insert_one(&cage).await;

        match result {
            Ok(_) => Ok(cage),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => Err(
                ApiErrorResponse::new(400, String::from("Cage already exist")),
            ),
            Err(err) => Err(ApiErrorResponse::new(500, err.to_string())),
        }
    }

    pub async fn find_cage_by_id(&self, id: &str) -> Result<Option<Cage>, ApiErrorResponse> {
        let filter = doc! { "_id": id };
        let cage = self
            .collection
            .find_one(filter)
            .await
            .map_err(internal_error)?;

        Ok(cage)
    }

    pub async fn find_all_users_cages(
        &self,
        assigned_monitor: String,
    ) -> Result<Vec<Cage>, ApiErrorResponse> {
        let filter = doc! { "assigned_monitor": assigned_monitor };

        let cursor = self.collection.find(filter).await.map_err(internal_error)?;
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
            .collection
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
