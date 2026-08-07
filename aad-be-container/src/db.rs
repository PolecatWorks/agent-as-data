use sqlx::PgPool;
use tracing::info;


pub async fn init_db_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    info!("Database pool connected successfully.");
    Ok(pool)
}

/// Pre-flight Fail-Fast check verifying `pgvector` extension is active.
pub async fn verify_pgvector_extension(pool: &PgPool) -> Result<(), String> {
    let row: (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')"
    )
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to query PostgreSQL extensions: {}", e))?;

    if !row.0 {
        return Err("Fail-Fast Error: pgvector extension is NOT installed/active in PostgreSQL".to_string());
    }

    info!("Fail-Fast Verification: pgvector extension is ACTIVE.");
    Ok(())
}
