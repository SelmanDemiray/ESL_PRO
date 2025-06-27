use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{AppState, auth::verify_token};
use crate::auth::Claims;
use axum::http::StatusCode;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: String,
    room: String, // now: "lesson-{lesson_id}"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: String,
}

static LESSON_CHAT_TX: Lazy<std::sync::Mutex<std::collections::HashMap<String, broadcast::Sender<ChatMessage>>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(state): State<AppState>,
) -> Response {
    // Validate JWT token before upgrading
    let claims = match verify_token(&params.token) {
        Ok(claims) => claims,
        Err(_) => {
            // If invalid, reject upgrade with 401
            return Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(axum::body::Body::from("Invalid or missing token"))
                .unwrap();
        }
    };
    let lesson_id = params.room.strip_prefix("lesson-").map(|s| s.to_string());
    ws.on_upgrade(move |socket| handle_socket(socket, params, claims, state, lesson_id))
}

async fn handle_socket(
    socket: WebSocket,
    params: WsQuery,
    claims: Claims,
    state: AppState,
    lesson_id: Option<String>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Per-lesson chat channel
    let tx = {
        let mut map = LESSON_CHAT_TX.lock().unwrap();
        map.entry(params.room.clone())
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(200);
                tx
            })
            .clone()
    };
    let mut rx = tx.subscribe();

    // Spawn a task to handle incoming messages
    let tx_clone = tx.clone();
    let state_clone = state.clone();
    let user_id = claims.sub.clone();
    let _username = claims.user_type.to_string();
    let lesson_id_uuid = lesson_id.as_ref().and_then(|id| uuid::Uuid::parse_str(id).ok());

    // Listen for incoming messages
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        // Parse as ChatMessage
                        if let Ok(mut chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                            chat_msg.user_id = user_id.clone();
                            // Permission check: if student, check muted and chat_closed
                            if let Some(lesson_id) = lesson_id_uuid {
                                let is_teacher = claims.user_type == crate::models::UserType::Teacher;
                                let is_muted = if !is_teacher {
                                    state_clone.db.is_participant_muted(lesson_id, uuid::Uuid::parse_str(&user_id).unwrap()).await.unwrap_or(false)
                                } else { false };
                                let lesson = state_clone.db.get_lesson(lesson_id).await.unwrap_or(None);
                                let chat_closed = lesson.map(|l| l.chat_closed).unwrap_or(false);
                                if chat_closed && !is_teacher {
                                    continue;
                                }
                                if is_muted && !is_teacher {
                                    continue;
                                }
                                // Save to DB
                                let db_msg = crate::models::LessonChatMessage {
                                    id: uuid::Uuid::new_v4(),
                                    lesson_id,
                                    user_id: uuid::Uuid::parse_str(&user_id).unwrap(),
                                    username: chat_msg.username.clone(),
                                    message: chat_msg.message.clone(),
                                    timestamp: chrono::Utc::now(),
                                    deleted: false,
                                };
                                let _ = state_clone.db.add_chat_message(&db_msg).await;
                            }
                            let _ = tx_clone.send(chat_msg);
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    });

    // Outgoing messages
    while let Ok(msg) = rx.recv().await {
        let text = serde_json::to_string(&msg).unwrap();
        if sender.send(Message::Text(text)).await.is_err() {
            break;
        }
    }
}
