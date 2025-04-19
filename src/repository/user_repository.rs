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
    users: Collection<User>,
    refresh_tokens: Collection<RefreshToken>,
}

impl UserRepository {
    pub fn new(db: &Database) -> Self {
        let users = db.collection::<User>("users");
        let refresh_tokens = db.collection::<RefreshToken>("tokens");
        Self {
            users,
            refresh_tokens,
        }
    }

    pub async fn new_async(db: &Database) -> Result<Self, ApiErrorResponse> {
        let users = db.collection::<User>("users");
        let refresh_tokens = db.collection::<RefreshToken>("tokens");
        ensure_indexes(&users).await?;
        Ok(Self {
            users,
            refresh_tokens,
        })
    }

    pub async fn create_user(&self, new_user: User) -> Result<NewUser, ApiErrorResponse> {
        let result = self.users.insert_one(&new_user).await;

        match result {
            Ok(_) => Ok(NewUser {
                id: new_user.id.to_string(),
                name: new_user.name,
                email: new_user.email,
                phone_number: new_user.phone_number,
                r#type: new_user.r#type,
                created_customers: new_user.created_customers,
                created_by: new_user.created_by,
                spm_id: new_user.spm_id,
                created_at: new_user.created_at,
                updated_at: new_user.updated_at,
            }),
            Err(err) if err.to_string().contains("E11000 duplicate key error") => Err(
                ApiErrorResponse::new(400, "Email already exists".to_string()),
            ),
            Err(err) => Err(internal_error(err)),
        }
    }

    pub async fn find_user_by_id(&self, id: &str) -> Result<Option<User>, ApiErrorResponse> {
        let obj_id = ObjectId::parse_str(id).map_err(internal_error)?;
        let user = self
            .users
            .find_one(doc! {
                "_id": obj_id,
            })
            .await
            .map_err(internal_error)?;

        Ok(user)
    }

    pub async fn find_admin_user_by_id(
        &self,
        admin_id: String,
    ) -> Result<Option<User>, ApiErrorResponse> {
        let obj_id = ObjectId::parse_str(&admin_id).map_err(internal_error)?;
        let admin_user = self
            .users
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
            .users
            .find_one(doc! {
                "email": email
            })
            .await
            .map_err(internal_error)?;

        Ok(user)
    }

    pub async fn create_user_refresh_token(
        &self,
        refresh_token: RefreshToken,
    ) -> Result<RefreshToken, ApiErrorResponse> {
        self.refresh_tokens
            .replace_one(doc! { "user_id": &refresh_token.user_id }, &refresh_token)
            .upsert(true)
            .await
            .map_err(internal_error)?;
        Ok(refresh_token)
    }

    pub async fn delete_user_refresh_token(&self, user_id: String) -> Result<(), ApiErrorResponse> {
        let user_obj_id = ObjectId::parse_str(user_id).map_err(internal_error)?;
        let filter = doc! { "user_id": user_obj_id };
        self.refresh_tokens
            .delete_one(filter)
            .await
            .map_err(internal_error)?;
        Ok(())
    }

    pub async fn find_valid_user_refresh_token_by_user_id(
        &self,
        id: &str,
    ) -> Result<Option<RefreshToken>, ApiErrorResponse> {
        let id = ObjectId::parse_str(id).map_err(internal_error)?;

        let refresh_token = self
            .refresh_tokens
            .find_one(doc! { "user_id": id  })
            .await
            .map_err(internal_error)?;
        Ok(refresh_token)
    }

    pub async fn update_user_password_by_id(
        &self,
        id: &str,
        new_password: String,
    ) -> Result<(), ApiErrorResponse> {
        let user_id = ObjectId::parse_str(id).map_err(internal_error)?;
        let filter = doc! { "_id": user_id };
        let update = doc! { "$set": { "password": new_password }  };

        let result = self
            .users
            .update_one(filter, update)
            .await
            .map_err(internal_error)?;

        match (result.matched_count, result.modified_count) {
            (0, _) => Err(ApiErrorResponse::new(404, String::from("user not found"))),
            (_, 0) => Err(ApiErrorResponse::new(
                200,
                String::from("No changes made to the data"),
            )),
            _ => Ok(()),
        }
    }
}

pub async fn ensure_indexes(users: &Collection<User>) -> Result<(), ApiErrorResponse> {
    let index_options = IndexOptions::builder().unique(true).build();
    let index_model = IndexModel::builder()
        .keys(doc! { "email": 1 })
        .options(index_options)
        .build();

    users
        .create_index(index_model)
        .await
        .map_err(internal_error)?;

    Ok(())
}
