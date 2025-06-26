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

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: String,
    room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: String,
}

static GLOBAL_TX: Lazy<broadcast::Sender<ChatMessage>> = Lazy::new(|| {
    let (tx, _rx) = broadcast::channel(100);
    tx
});

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WsQuery>,
    State(_state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, params))
}

async fn handle_socket(socket: WebSocket, _params: WsQuery) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = GLOBAL_TX.subscribe();

    // Spawn a task to handle incoming messages
    let tx_clone = GLOBAL_TX.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        if let Ok(chat_msg) = serde_json::from_str::<ChatMessage>(&text) {
                            let _ = tx_clone.send(chat_msg);
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    });

    // Handle outgoing messages
    while let Ok(msg) = rx.recv().await {
        let text = serde_json::to_string(&msg).unwrap();
        if sender.send(Message::Text(text)).await.is_err() {
            break;
        }
    }
}
