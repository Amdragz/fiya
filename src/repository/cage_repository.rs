use mongodb::{Collection, Database};

use crate::{models::cage::Cage, utils::response::ApiErrorResponse};

pub struct CageRepository {
    collection: Collection<Cage>,
}

impl CageRepository {
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
}
