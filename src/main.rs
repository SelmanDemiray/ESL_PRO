use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use std::sync::Arc;
use tower_http::{services::ServeDir, cors::CorsLayer};
use tracing_subscriber;

mod auth;
mod database;
mod handlers;
mod models;
mod websocket;

use database::Database;

#[derive(Clone)]
pub struct AppState {
    db: Arc<Database>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://esluser:eslpass@postgres:5432/esldb".to_string());

    let db = Database::new(&database_url).await?;
    db.migrate().await?;

    let state = AppState { db: Arc::new(db) };

    // Protected routes that require authentication
    let protected_routes = Router::new()
        .route("/api/dashboard", get(handlers::dashboard))
        .route("/api/classroom/:id", get(handlers::classroom))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware));

    let app = Router::new()
        .route("/", get(handlers::home))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .merge(protected_routes)
        .route("/ws", get(websocket::websocket_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    
    println!("🚀 ESL Learning Platform running on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}