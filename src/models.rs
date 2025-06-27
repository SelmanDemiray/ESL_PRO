use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub user_type: UserType,
    pub first_name: String,
    pub last_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
    pub zoom_access_token: Option<String>,
    pub zoom_refresh_token: Option<String>,
    pub zoom_token_expiry: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
pub enum UserType {
    Student,
    Teacher,
    Admin,
}

impl fmt::Display for UserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            UserType::Student => "student",
            UserType::Teacher => "teacher",
            UserType::Admin => "admin",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    #[serde(deserialize_with = "user_type_from_str")]
    pub user_type: UserType,
    pub first_name: String,
    pub last_name: String,
}

// Helper for deserializing user_type from string
fn user_type_from_str<'de, D>(deserializer: D) -> Result<UserType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "student" => Ok(UserType::Student),
        "teacher" => Ok(UserType::Teacher),
        "admin" => Ok(UserType::Admin),
        _ => Err(serde::de::Error::custom("Invalid user_type")),
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub email: String,
    pub user_type: UserType,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Classroom {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub teacher_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // --- Zoom integration fields ---
    pub zoom_meeting_id: Option<String>,
    pub zoom_join_url: Option<String>,
}

// For student meeting requests
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MeetingRequest {
    pub id: Uuid,
    pub classroom_id: Uuid,
    pub student_id: Uuid,
    pub status: String, // "pending", "approved", "rejected"
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigitalBook {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub description: String,
    pub pdf_url: String,
    pub level: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Lesson {
    pub id: Uuid,
    pub classroom_id: Uuid,
    pub teacher_id: Uuid,
    pub title: String,
    pub description: String,
    pub scheduled_at: DateTime<Utc>,
    pub is_active: bool,
    pub chat_closed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LessonParticipant {
    pub lesson_id: Uuid,
    pub user_id: Uuid,
    pub is_muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LessonChatMessage {
    pub id: Uuid,
    pub lesson_id: Uuid,
    pub user_id: Uuid,
    pub username: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub deleted: bool,
}
