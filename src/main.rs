use axum::{
    routing::{get, post, delete},
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
        // --- Zoom meeting management ---
        .route("/api/classroom/:classroom_id/zoom", post(handlers::create_zoom_meeting))
        .route("/api/classroom/:classroom_id/zoom", delete(handlers::delete_zoom_meeting))
        .route("/api/classroom/:classroom_id/zoom/join", get(handlers::get_zoom_join_url))
        // --- Meeting requests ---
        .route("/api/classroom/:classroom_id/meeting-requests", post(handlers::request_zoom_meeting))
        .route("/api/classroom/:classroom_id/meeting-requests", get(handlers::get_meeting_requests))
        .route("/api/meeting-request/:request_id", post(handlers::update_meeting_request))
        // Lesson and chat endpoints
        .route("/api/lesson", post(handlers::create_lesson))
        .route("/api/lesson", get(handlers::list_teacher_lessons))
        .route("/api/lesson/:id", get(handlers::get_lesson))
        .route("/api/lesson/:lesson_id/chat/:message_id/delete", post(handlers::delete_chat_message))
        .route("/api/lesson/:lesson_id/chat/close", post(handlers::close_chat))
        .route("/api/lesson/:lesson_id/participant/:user_id/mute", post(handlers::mute_participant))
        .route("/api/lesson/:lesson_id/participant/:user_id/unmute", post(handlers::unmute_participant))
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