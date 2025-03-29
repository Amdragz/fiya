use std::env;

use mongodb::{options::ClientOptions, Client};

pub async fn extablish_mongodb_connection() -> Client {
    let database_url = env::var("MONGODB_URL").expect("MONGODB_URL must be set");

    let options = ClientOptions::parse(&database_url)
        .await
        .expect("Failed to pass mongodb url");
    Client::with_options(options).expect("Failed to create Mongodb client")
}
