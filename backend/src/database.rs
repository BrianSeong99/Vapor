use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::info;

pub async fn init_db(database_url: &str) -> anyhow::Result<SqlitePool> {
    info!("Connecting to database: {}", database_url);
    
    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    info!("Running database migrations...");
    
    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            id TEXT PRIMARY KEY,
            order_type INTEGER NOT NULL,
            from_address TEXT,
            to_address TEXT, 
            token_id INTEGER NOT NULL,
            amount TEXT NOT NULL,
            banking_hash TEXT,
            status INTEGER NOT NULL DEFAULT 0,
            batch_id INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS batches (
            id INTEGER PRIMARY KEY,
            prev_state_root TEXT NOT NULL,
            prev_orders_root TEXT NOT NULL,
            new_state_root TEXT NOT NULL,
            new_orders_root TEXT NOT NULL,
            proof_data TEXT,
            status INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            submitted_at DATETIME
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS account_states (
            address TEXT PRIMARY KEY,
            token_id INTEGER NOT NULL,
            balance TEXT NOT NULL DEFAULT '0',
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("Database migrations completed");
    Ok(())
}
