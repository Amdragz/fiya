use std::sync::Arc;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        Method, StatusCode,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use endpoints::{auth_endpoints::auth_endpoints, user_endpoints::user_endpoints};
use mongodb::Client;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod dtos;
mod endpoints;
mod middleware;
mod models;
mod repository;
mod services;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pub mongo_client: Arc<Client>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fiya=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let mongo_client = config::database::extablish_mongodb_connection().await;

    let app_state = Arc::new(AppState {
        mongo_client: Arc::new(mongo_client),
    });

    let _web_cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5172".parse().unwrap(),
            "https://fiya-wep-app.vercel.app".parse().unwrap(),
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE, ACCEPT])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ]);

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/users", user_endpoints())
        .nest("/auth", auth_endpoints())
        .with_state(app_state)
        .layer(_web_cors)
        .layer(TraceLayer::new_for_http());

    pub async fn health_check() -> impl IntoResponse {
        (StatusCode::OK, "healthy").into_response()
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
