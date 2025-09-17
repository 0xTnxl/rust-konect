#[allow(dead_code)]

use crate::{error::AppError, models::*};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_room(
    pool: &PgPool,
    name: &str,
    description: Option<&str>,
) -> Result<Room, AppError> {
    let room_id = Uuid::new_v4();
    let now = Utc::now();

    let room = sqlx::query_as::<_, Room>(
        r#"
        INSERT INTO rooms (id, name, description, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#
    )
    .bind(room_id)
    .bind(name)
    .bind(description)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(room)
}

pub async fn get_rooms(pool: &PgPool) -> Result<Vec<Room>, AppError> {
    let rooms = sqlx::query_as::<_, Room>("SELECT * FROM rooms ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;

    Ok(rooms)
}

pub async fn send_message(
    pool: &PgPool,
    room_id: Uuid,
    user_id: Uuid,
    content: &str,
    message_type: &str,
) -> Result<Message, AppError> {
    let message_id = Uuid::new_v4();
    let now = Utc::now();

    let message = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (id, room_id, user_id, content, message_type, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#
    )
    .bind(message_id)
    .bind(room_id)
    .bind(user_id)
    .bind(content)
    .bind(message_type)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(message)
}

pub async fn get_messages(
    pool: &PgPool,
    room_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Message>, AppError> {
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT * FROM messages
        WHERE room_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(room_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(messages)
}

pub async fn get_room_by_id(pool: &PgPool, room_id: Uuid) -> Result<Option<Room>, AppError> {
    let room = sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE id = $1")
        .bind(room_id)
        .fetch_optional(pool)
        .await?;

    Ok(room)
}