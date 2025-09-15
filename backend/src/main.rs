use axum::{
    extract::ws::{WebSocketUpgrade},
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Multipart;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing::{error, info, warn};
use uuid::Uuid;

mod auth;
mod chat;
mod database;
mod error;
mod models;
mod websocket;
mod xmpp_bridge;

use auth::{create_user, login_user, verify_token, AuthClaims};
use chat::{create_room, get_messages, get_rooms, send_message};
use database::init_db;
use error::AppError;
use models::*;
use websocket::handle_socket;

type SharedState = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub rooms: Arc<RwLock<HashMap<Uuid, broadcast::Sender<String>>>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/rust_konect".to_string());

    let db = init_db(&database_url).await?;
    
    let state = AppState {
        db,
        rooms: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = create_router(Arc::new(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(serve_frontend))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/rooms", get(get_rooms_handler))
        .route("/api/rooms", post(create_room_handler))
        .route("/api/rooms/:room_id/messages", get(get_messages_handler))
        .route("/api/rooms/:room_id/messages", post(send_message_handler))
        .route("/api/upload", post(upload_file))
        .route("/ws/:room_id", get(websocket_handler))
        .nest_service("/static", ServeDir::new("frontend/static"))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

async fn serve_frontend() -> Html<&'static str> {
    Html(include_str!("../../frontend/index.html"))
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

async fn register(
    State(state): State<SharedState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = create_user(&state.db, &req.username, &req.email, &req.password).await?;
    Ok(Json(AuthResponse {
        token: user.token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
        },
    }))
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

async fn login(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = login_user(&state.db, &req.email, &req.password).await?;
    Ok(Json(AuthResponse {
        token: user.token,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
        },
    }))
}

async fn get_rooms_handler(
    State(state): State<SharedState>,
) -> Result<Json<Vec<Room>>, AppError> {
    let rooms = get_rooms(&state.db).await?;
    Ok(Json(rooms))
}

#[derive(Deserialize)]
struct CreateRoomRequest {
    name: String,
    description: Option<String>,
}

async fn create_room_handler(
    State(state): State<SharedState>,
    Json(req): Json<CreateRoomRequest>,
) -> Result<Json<Room>, AppError> {
    let room = create_room(&state.db, &req.name, req.description.as_deref()).await?;
    
    // Create broadcast channel for the new room
    let (tx, _) = broadcast::channel(1000);
    state.rooms.write().await.insert(room.id, tx);
    
    Ok(Json(room))
}

#[derive(Deserialize)]
struct MessagesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn get_messages_handler(
    State(state): State<SharedState>,
    Path(room_id): Path<Uuid>,
    Query(query): Query<MessagesQuery>,
) -> Result<Json<Vec<Message>>, AppError> {
    let messages = get_messages(
        &state.db,
        room_id,
        query.limit.unwrap_or(50),
        query.offset.unwrap_or(0),
    ).await?;
    Ok(Json(messages))
}

#[derive(Deserialize)]
struct SendMessageRequest {
    content: String,
    message_type: Option<String>,
}

async fn send_message_handler(
    State(state): State<SharedState>,
    Path(room_id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<Message>, AppError> {
    // For now, use a dummy user ID - in production this would come from JWT
    let user_id = Uuid::new_v4();
    
    let message = send_message(
        &state.db,
        room_id,
        user_id,
        &req.content,
        req.message_type.as_deref().unwrap_or("text"),
    ).await?;
    
    // Broadcast message to WebSocket clients
    if let Some(tx) = state.rooms.read().await.get(&room_id) {
        let message_json = serde_json::to_string(&message).unwrap();
        let _ = tx.send(message_json);
    }
    
    Ok(Json(message))
}

async fn upload_file(
    State(_state): State<SharedState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|_| AppError::BadRequest("Invalid multipart data".to_string()))? {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let data = field.bytes().await.map_err(|_| AppError::BadRequest("Failed to read file".to_string()))?;
            
            // Save file to storage directory
            let file_id = Uuid::new_v4();
            let file_path = format!("uploads/{}", file_id);
            
            tokio::fs::create_dir_all("uploads").await.map_err(|_| AppError::InternalError("Failed to create uploads directory".to_string()))?;
            tokio::fs::write(&file_path, &data).await.map_err(|_| AppError::InternalError("Failed to save file".to_string()))?;
            
            // Store file metadata in database
            let file_url = format!("/api/files/{}", file_id);
            
            return Ok(Json(UploadResponse {
                id: file_id,
                filename,
                url: file_url,
                size: data.len() as i64,
            }));
        }
    }
    
    Err(AppError::BadRequest("No file provided".to_string()))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    Path(room_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, room_id, state))
}