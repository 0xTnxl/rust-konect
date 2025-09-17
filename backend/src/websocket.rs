use crate::{models::*, SharedState};
use axum::extract::ws::{Message, WebSocket};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

pub async fn handle_socket(socket: WebSocket, room_id: Uuid, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();

    // Get or create broadcast channel for this room
    let mut rx = {
        let mut rooms = state.rooms.write().await;
        if let Some(tx) = rooms.get(&room_id) {
            tx.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(1000);
            rooms.insert(room_id, tx);
            rx
        }
    };

    // Spawn task to handle incoming WebSocket messages
    let _state_clone = Arc::clone(&state);
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn task to handle outgoing WebSocket messages
    let state_clone2 = Arc::clone(&state);
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                        match ws_msg.message_type.as_str() {
                            "chat_message" => {
                                if let Ok(chat_msg) = serde_json::from_value::<ChatMessage>(ws_msg.data) {
                                    // TODO: Extract user ID from WebSocket connection authentication
                                    let user_id = Uuid::new_v4(); // Placeholder
                                    
                                    match crate::chat::send_message(
                                        &state_clone2.db,
                                        room_id,
                                        user_id,
                                        &chat_msg.content,
                                        chat_msg.message_type.as_deref().unwrap_or("text"),
                                    ).await {
                                        Ok(message) => {
                                            // Broadcast to all clients in the room
                                            if let Some(tx) = state_clone2.rooms.read().await.get(&room_id) {
                                                let message_json = serde_json::to_string(&message).unwrap();
                                                let _ = tx.send(message_json);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to send message: {}", e);
                                        }
                                    }
                                }
                            }
                            "join_room" => {
                                info!("User joined room: {}", room_id);
                                // Send room history or welcome message
                            }
                            "leave_room" => {
                                info!("User left room: {}", room_id);
                                break;
                            }
                            _ => {
                                warn!("Unknown WebSocket message type: {}", ws_msg.message_type);
                            }
                        }
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Received binary message, ignoring");
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => {
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed for room: {}", room_id);
}