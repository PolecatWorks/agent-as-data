use sqlx::PgPool;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(#[source] sqlx::Error),
    #[error("Failed to query PostgreSQL available extensions: {0}")]
    AvailableExtensionQueryFailed(#[source] sqlx::Error),
    #[error("Fail-Fast Error: pgvector extension is NOT installed/active in PostgreSQL (extension missing from server)")]
    PgVectorMissingFromServer,
    #[error("Failed to enable pgvector extension: {0}")]
    EnablePgVectorFailed(#[source] sqlx::Error),
    #[error("Failed to query PostgreSQL extensions: {0}")]
    ExtensionQueryFailed(#[source] sqlx::Error),
    #[error("Fail-Fast Error: pgvector extension is NOT installed/active in PostgreSQL")]
    PgVectorNotEnabled,
}

pub async fn init_db_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    info!("Database pool connected successfully.");
    Ok(pool)
}

/// Pre-flight Fail-Fast check verifying `pgvector` extension is active.
pub async fn verify_pgvector_extension(pool: &PgPool) -> Result<(), DatabaseError> {
    let avail: (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'vector')",
    )
    .fetch_one(pool)
    .await
    .map_err(DatabaseError::AvailableExtensionQueryFailed)?;

    if !avail.0 {
        return Err(DatabaseError::PgVectorMissingFromServer);
    }

    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(pool)
        .await
        .map_err(DatabaseError::EnablePgVectorFailed)?;

    let row: (bool,) =
        sqlx::query_as("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector')")
            .fetch_one(pool)
            .await
            .map_err(DatabaseError::ExtensionQueryFailed)?;

    if !row.0 {
        return Err(DatabaseError::PgVectorNotEnabled);
    }

    info!("Fail-Fast Verification: pgvector extension is ACTIVE.");
    Ok(())
}
