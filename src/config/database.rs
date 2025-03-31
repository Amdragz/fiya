use std::env;

use dotenvy::dotenv;
use mongodb::{options::ClientOptions, Client};

pub async fn extablish_mongodb_connection() -> Client {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE URL must be set");

    let options = ClientOptions::parse(&database_url)
        .await
        .expect("Failed to pass database url");
    Client::with_options(options).expect("Failed to create Mongodb client")
}
