use crate::{error::AppError, models::*};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

const JWT_SECRET: &str = "your-secret-key"; // In production, use environment variable

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthClaims {
    pub sub: String, // user id
    pub username: String,
    pub exp: usize,
}

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    password: &str,
) -> Result<AuthUser, AppError> {
    // Check if user already exists
    let existing_user = sqlx::query("SELECT id FROM users WHERE email = $1 OR username = $2")
        .bind(email)
        .bind(username)
        .fetch_optional(pool)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::Validation("User already exists".to_string()));
    }

    // Hash password
    let password_hash = hash(password, DEFAULT_COST)?;
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    // Create user
    sqlx::query(
        r#"
        INSERT INTO users (id, username, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#
    )
    .bind(user_id)
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    // Generate JWT token
    let token = generate_token(&user_id.to_string(), username)?;

    Ok(AuthUser {
        id: user_id,
        username: username.to_string(),
        email: email.to_string(),
        token,
    })
}

pub async fn login_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> Result<AuthUser, AppError> {
    // Get user by email
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid credentials".to_string()))?;

    // Verify password
    if !verify(password, &user.password_hash)? {
        return Err(AppError::Auth("Invalid credentials".to_string()));
    }

    // Generate JWT token
    let token = generate_token(&user.id.to_string(), &user.username)?;

    Ok(AuthUser {
        id: user.id,
        username: user.username,
        email: user.email,
        token,
    })
}

pub fn generate_token(user_id: &str, username: &str) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("Valid timestamp")
        .timestamp() as usize;

    let claims = AuthClaims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )?;

    Ok(token)
}

pub fn verify_token(token: &str) -> Result<AuthClaims, AppError> {
    let token_data = decode::<AuthClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}