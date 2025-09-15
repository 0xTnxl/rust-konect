use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, PgPool, Postgres};
use tracing::info;

pub async fn init_db(database_url: &str) -> Result<PgPool> {
    // Create database if it doesn't exist
    if !Postgres::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database...");
        Postgres::create_database(database_url).await?;
    }

    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Database initialized successfully");
    Ok(pool)
}