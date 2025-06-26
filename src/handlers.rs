use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::{Html, Json},
};
use chrono::Utc;
use uuid::Uuid;
use tokio::fs;

use crate::{
    auth::{create_token, hash_password, verify_password, Claims},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserInfo, UserType},
    AppState,
};

pub async fn home() -> Html<String> {
    let html = fs::read_to_string("static/index.html").await.unwrap_or_else(|_| "<h1>Not found</h1>".to_string());
    Html(html)
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if payload.email.is_empty() || payload.password.len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email.clone(),
        password_hash,
        user_type: payload.user_type.clone(),
        first_name: payload.first_name.clone(),
        last_name: payload.last_name.clone(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        is_active: true,
    };

    state.db.create_user(&user).await
        .map_err(|_| StatusCode::CONFLICT)?;

    let token = create_token(user.id, user.user_type.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            user_type: user.user_type,
            first_name: user.first_name,
            last_name: user.last_name,
        },
    };

    Ok(Json(response))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = state.db.get_user_by_email(&payload.email).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !verify_password(&payload.password, &user.password_hash) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = create_token(user.id, user.user_type.clone())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = AuthResponse {
        token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            user_type: user.user_type,
            first_name: user.first_name,
            last_name: user.last_name,
        },
    };

    Ok(Json(response))
}

pub async fn dashboard(
    Extension(claims): Extension<Claims>,
    State(state): State<AppState>,
) -> Result<Html<String>, StatusCode> {
    let dashboard_path = match claims.user_type {
        UserType::Teacher => "static/teacher-dashboard.html",
        UserType::Student => "static/student-dashboard.html",
        UserType::Admin => "static/student-dashboard.html",
    };
    let dashboard_html = fs::read_to_string(dashboard_path).await.unwrap_or_else(|_| "<h1>Not found</h1>".to_string());
    Ok(Html(dashboard_html))
}

pub async fn classroom(
    Path(id): Path<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<Html<String>, StatusCode> {
    let classroom_html = fs::read_to_string("static/classroom.html").await.unwrap_or_else(|_| "<h1>Not found</h1>".to_string());
    Ok(Html(classroom_html))
}
