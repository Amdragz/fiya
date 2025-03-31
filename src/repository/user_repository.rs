use mongodb::{
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
    Collection, Database, IndexModel,
};

use crate::{
    models::{
        refresh_token::RefreshToken,
        user::{NewUser, User},
    },
    utils::{error_handler::internal_error, response::ApiErrorResponse},
};

pub struct UserRepository {
    collection: Collection<User>,
    token_collection: Collection<RefreshToken>,
}

impl UserRepository {
    pub fn new(db: &Database) -> Self {
        let collection = db.collection::<User>("users");
        let token_collection = db.collection::<RefreshToken>("tokens");
        Self {
            collection,
            token_collection,
        }
    }

    pub async fn new_async(db: &Database) -> Result<Self, ApiErrorResponse> {
        let collection = db.collection::<User>("users");
        let token_collection = db.collection::<RefreshToken>("tokens");
        ensure_indexes(&collection).await?;
        Ok(Self {
            collection,
            token_collection,
        })
    }

    pub async fn create_user(&self, new_user: User) -> Result<NewUser, ApiErrorResponse> {
        let result = self.collection.insert_one(&new_user).await;

        match result {
            Ok(_) => Ok(NewUser {
                id: new_user.id,
                name: new_user.name,
                email: new_user.email,
                phone_number: new_user.phone_number,
                r#type: new_user.r#type,
                created_customers: new_user.created_customers,
                created_by: new_user.created_by,
                spm_id: new_user.spm_id,
            }),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => Err(
                ApiErrorResponse::new(400, "Email already exists".to_string()),
            ),
            Err(err) => Err(internal_error(err)),
        }
    }

    pub async fn find_admin_user_by_id(
        &self,
        admin_id: String,
    ) -> Result<Option<User>, ApiErrorResponse> {
        let obj_id = ObjectId::parse_str(&admin_id).map_err(internal_error)?;
        let admin_user = self
            .collection
            .find_one(doc! {
                "_id": obj_id,
                "type": "admin"
            })
            .await
            .map_err(internal_error)?;

        Ok(admin_user)
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, ApiErrorResponse> {
        let user = self
            .collection
            .find_one(doc! {
                "email": email
            })
            .await
            .map_err(internal_error)?;

        Ok(user)
    }

    pub async fn create_refresh_token(
        &self,
        refresh_token: RefreshToken,
    ) -> Result<RefreshToken, ApiErrorResponse> {
        self.token_collection
            .insert_one(&refresh_token)
            .await
            .map_err(internal_error)?;
        Ok(refresh_token)
    }
}

pub async fn ensure_indexes(collection: &Collection<User>) -> Result<(), ApiErrorResponse> {
    let index_options = IndexOptions::builder().unique(true).build();
    let index_model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(index_options)
        .build();

    collection
        .create_index(index_model)
        .await
        .map_err(internal_error)?;

    Ok(())
}
