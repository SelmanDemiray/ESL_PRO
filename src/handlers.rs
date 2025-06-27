use axum::{
    extract::{Path, State, Extension, Query},
    http::StatusCode,
    response::{Html, Json, Redirect},
};
use chrono::Utc;
use uuid::Uuid;
use tokio::fs;
use reqwest;

use crate::{
    auth::{create_token, hash_password, verify_password, Claims},
    models::{AuthResponse, LoginRequest, RegisterRequest, User, UserInfo, UserType, MeetingRequest, Classroom, Lesson, DigitalBook},
    AppState,
};

const ZOOM_CLIENT_ID: &str = "YOUR_ZOOM_CLIENT_ID";
const ZOOM_CLIENT_SECRET: &str = "YOUR_ZOOM_CLIENT_SECRET";
const ZOOM_REDIRECT_URI: &str = "http://localhost:8000/api/zoom/callback";

pub async fn home() -> Html<String> {
    let html = fs::read_to_string("static/index.html").await.unwrap_or_else(|_| "<h1>Not found</h1>".to_string());
    Html(html)
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if payload.email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Email is required.".to_string()));
    }
    if !payload.email.contains('@') {
        return Err((StatusCode::BAD_REQUEST, "Invalid email address.".to_string()));
    }
    if payload.password.len() < 8 {
        return Err((StatusCode::BAD_REQUEST, "Password must be at least 8 characters.".to_string()));
    }
    if payload.first_name.trim().is_empty() || payload.last_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "First and last name are required.".to_string()));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password.".to_string()))?;

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
        zoom_access_token: None,
        zoom_refresh_token: None,
        zoom_token_expiry: None,
    };

    // Check if user already exists
    if let Ok(Some(_)) = state.db.get_user_by_email(&user.email).await {
        return Err((StatusCode::CONFLICT, "Email already registered.".to_string()));
    }

    state.db.create_user(&user).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create user: {e}")))?;

    let token = create_token(user.id, user.user_type.clone())
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token.".to_string()))?;

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
    State(_state): State<AppState>,
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
    Path(_id): Path<String>,
    Extension(_claims): Extension<Claims>,
) -> Result<Html<String>, StatusCode> {
    let classroom_html = fs::read_to_string("static/classroom.html").await.unwrap_or_else(|_| "<h1>Not found</h1>".to_string());
    Ok(Html(classroom_html))
}

// Redirect user to Zoom OAuth
pub async fn zoom_connect() -> Redirect {
    let url = format!(
        "https://zoom.us/oauth/authorize?response_type=code&client_id={}&redirect_uri={}",
        ZOOM_CLIENT_ID, ZOOM_REDIRECT_URI
    );
    Redirect::temporary(&url)
}

// Handle Zoom OAuth callback
pub async fn zoom_callback(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Redirect, StatusCode> {
    let code = params.get("code").ok_or(StatusCode::BAD_REQUEST)?;
    // Exchange code for tokens
    let client = reqwest::Client::new();
    let res = client.post("https://zoom.us/oauth/token")
        .basic_auth(ZOOM_CLIENT_ID, Some(ZOOM_CLIENT_SECRET))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", ZOOM_REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if !res.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let json: serde_json::Value = res.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let access_token = json.get("access_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let refresh_token = json.get("refresh_token").and_then(|v| v.as_str()).map(|s| s.to_string());
    let expires_in = json.get("expires_in").and_then(|v| v.as_i64()).unwrap_or(0);

    // Store tokens in DB for this user
    let expiry = chrono::Utc::now() + chrono::Duration::seconds(expires_in);
    sqlx::query(
        "UPDATE users SET zoom_access_token = $1, zoom_refresh_token = $2, zoom_token_expiry = $3 WHERE id = $4"
    )
    .bind(&access_token)
    .bind(&refresh_token)
    .bind(expiry)
    .bind(claims.sub)
    .execute(state.db.get_pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Redirect to dashboard
    Ok(Redirect::to("/dashboard"))
}

// --- Zoom Meeting Management Endpoints ---

// Teacher: Create a Zoom meeting for a classroom
pub async fn create_zoom_meeting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(classroom_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    // TODO: Use teacher's Zoom access token to create meeting via Zoom API
    // For now, mock meeting_id and join_url
    let meeting_id = format!("mock-{}", classroom_id);
    let join_url = format!("https://zoom.us/j/{}", meeting_id);

    state.db.set_classroom_zoom_meeting(classroom_id, &meeting_id, &join_url)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "meeting_id": meeting_id, "join_url": join_url })))
}

// Teacher: Delete Zoom meeting for a classroom
pub async fn delete_zoom_meeting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(classroom_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    // TODO: Call Zoom API to delete meeting if needed
    state.db.clear_classroom_zoom_meeting(classroom_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

// Student: Request a Zoom meeting for a classroom
pub async fn request_zoom_meeting(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(classroom_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Student {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.create_meeting_request(classroom_id, claims.sub.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

// Teacher: View pending meeting requests for a classroom
pub async fn get_meeting_requests(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(classroom_id): Path<Uuid>,
) -> Result<Json<Vec<MeetingRequest>>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let requests = state.db.get_pending_meeting_requests(classroom_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(requests))
}

// Teacher: Approve/reject a meeting request
pub async fn update_meeting_request(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((request_id,)): Path<(Uuid,)>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("pending");
    state.db.update_meeting_request_status(request_id, status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

// Student: Get Zoom join URL for a classroom
pub async fn get_zoom_join_url(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(classroom_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let classroom = sqlx::query_as::<_, Classroom>(
        "SELECT * FROM classrooms WHERE id = $1"
    )
    .bind(classroom_id)
    .fetch_optional(state.db.get_pool())
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(join_url) = classroom.zoom_join_url {
        Ok(Json(serde_json::json!({ "join_url": join_url })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Lesson management endpoints
use axum::extract::Json as AxumJson;

pub async fn create_lesson(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    AxumJson(payload): AxumJson<Lesson>,
) -> Result<AxumJson<Lesson>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.create_lesson(&payload).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(payload))
}

// List all lessons for the teacher
pub async fn list_teacher_lessons(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<AxumJson<Vec<Lesson>>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let teacher_id = claims.sub.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let lessons = state.db.get_lessons_by_teacher(teacher_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(lessons))
}

// Chat moderation endpoints
pub async fn mute_participant(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((lesson_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.set_participant_muted(lesson_id, user_id, true).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn unmute_participant(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((lesson_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.set_participant_muted(lesson_id, user_id, false).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn delete_chat_message(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((lesson_id, message_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.delete_chat_message(message_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

pub async fn close_chat(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(lesson_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    state.db.close_lesson_chat(lesson_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::OK)
}

// --- Classroom CRUD for Teacher ---

pub async fn create_classroom(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    AxumJson(payload): AxumJson<serde_json::Value>,
) -> Result<AxumJson<Classroom>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("").trim();
    if name.is_empty() || description.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let classroom = Classroom {
        id: uuid::Uuid::new_v4(),
        name: name.to_string(),
        description: description.to_string(),
        teacher_id: claims.sub.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        is_active: true,
        created_at: chrono::Utc::now(),
        zoom_meeting_id: None,
        zoom_join_url: None,
    };
    state.db.create_classroom(&classroom).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(classroom))
}

pub async fn list_teacher_classrooms(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<AxumJson<Vec<Classroom>>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let teacher_id = claims.sub.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let classes = state.db.get_classrooms_by_teacher(teacher_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(classes))
}

// --- Material upload/list ---

pub async fn upload_material(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    AxumJson(payload): AxumJson<serde_json::Value>,
) -> Result<AxumJson<DigitalBook>, StatusCode> {
    if claims.user_type != UserType::Teacher {
        return Err(StatusCode::FORBIDDEN);
    }
    let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("").trim();
    let author = payload.get("author").and_then(|v| v.as_str()).unwrap_or("").trim();
    let description = payload.get("description").and_then(|v| v.as_str()).unwrap_or("").trim();
    let level = payload.get("level").and_then(|v| v.as_str()).unwrap_or("").trim();
    let pdf_url = payload.get("pdf_url").and_then(|v| v.as_str()).unwrap_or("").trim();
    if title.is_empty() || author.is_empty() || description.is_empty() || level.is_empty() || pdf_url.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let material = DigitalBook {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        author: author.to_string(),
        description: description.to_string(),
        pdf_url: pdf_url.to_string(),
        level: level.to_string(),
        created_at: chrono::Utc::now(),
    };
    state.db.create_material(&material).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(material))
}

pub async fn list_materials(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<AxumJson<Vec<DigitalBook>>, StatusCode> {
    // Teachers see all materials, or could be filtered by teacher in future
    let books = state.db.get_all_books().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(AxumJson(books))
}

// --- Add missing get_lesson handler ---
pub async fn get_lesson(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<AxumJson<Lesson>, StatusCode> {
    let lesson = state.db.get_lesson(id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    match lesson {
        Some(lesson) => Ok(AxumJson(lesson)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
