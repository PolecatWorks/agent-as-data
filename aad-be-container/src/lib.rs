//! Agent-As-Data Backend (`aad-be-container`) core library.
//!
//! Provides application lifecycle orchestration, configuration loading,
//! database connection pooling, HaMS health sidecar integration, and Axum REST webservice.

pub mod config;
pub mod db;
pub mod error;
pub mod hams_tools;
pub mod models;
pub mod state;
pub mod tokio_tools;
pub mod webserver;

pub use state::AppState;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::path::Path;
use tracing::info;
use ::hams::hams::Hams;

use crate::config::AppConfig;
use crate::db::{init_db_pool, verify_pgvector_extension};
use crate::hams_tools::HamsHarness;
use crate::webserver::start_webserver;

/// Main application service orchestrator.
///
/// 1. Loads fail-fast AppConfig from file and secrets.
/// 2. Validates configuration.
/// 3. Initializes HaMS sidecar on health port 8079.
/// 4. Connects to PostgreSQL pool and verifies pgvector extension.
/// 5. Executes automatic database schema migrations.
/// 6. Binds and serves the Axum REST webservice.
pub async fn service_main(
    config_path: &Path,
    secrets_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = AppConfig::load(config_path, secrets_dir).map_err(|e| {
        format!("Fail-Fast Error: Failed to load config: {}", e)
    })?;

    info!("Starting {} v{}", NAME, VERSION);

    // 1. Fail-Fast Config Validation
    config.validate().map_err(|e| {
        format!("Fail-Fast Configuration Error: {}", e)
    })?;

    // 2. Initialize HaMS Health Monitoring Sidecar & ProbeManual readiness signal
    let mut hams_config = config.hams.clone();
    hams_config.name = NAME.to_owned();
    hams_config.version = VERSION.to_owned();

    let hams = Hams::new(hams_config);
    let _hams_harness = HamsHarness::init(hams).await
        .map_err(|e| format!("HaMS init error: {}", e))?;
    info!("HaMS health sidecar started on port 8079 with readiness probe.");

    // 3. Connect DB Pool & Verify pgvector (Fail-Fast)
    let db_url: url::Url = config.database.url.clone().into();
    let pool = init_db_pool(db_url.as_str(), config.database.max_connections).await
        .map_err(|e| format!("Fail-Fast Error: Database connection failed: {}", e))?;

    verify_pgvector_extension(&pool).await
        .map_err(|e| format!("Fail-Fast pgvector check failed: {}", e))?;

    // 4. Run Automatic Schema Migrations
    info!("Executing database schema migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;
    info!("Database migrations applied successfully.");

    let app_state = AppState {
        pool,
        config: config.clone(),
    };

    // 5. Start Axum Main REST Webservice
    start_webserver(app_state, &config.webservice).await?;

    Ok(())
}

/// Runs the database schema migrations standalone.
pub async fn run_migrations(
    config_path: &Path,
    secrets_dir: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = AppConfig::load(config_path, secrets_dir).map_err(|e| {
        format!("Fail-Fast Error: Failed to load config: {}", e)
    })?;

    let db_url: url::Url = config.database.url.into();
    let pool = init_db_pool(db_url.as_str(), config.database.max_connections).await
        .map_err(|e| format!("DB connection error: {}", e))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;

    println!("Migrations completed successfully.");
    Ok(())
}
