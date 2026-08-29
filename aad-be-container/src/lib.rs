//! Agent-As-Data Backend (`aad-be-container`) core library.
//!
//! Provides application lifecycle orchestration, configuration loading,
//! database connection pooling, HaMS health sidecar integration, and Axum REST webservice.

pub mod config;
pub mod db;
pub mod error;
pub mod hams_tools;
pub mod llm_tools;
pub mod metrics;
pub mod models;
pub mod state;
pub mod tokio_tools;
pub mod webserver;

pub use state::AppState;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::ffi::c_void;
use std::path::Path;
use std::sync::Arc;
use tracing::info;
use ::hams::hams::Hams;
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;

use crate::config::AppConfig;
use crate::db::{init_db_pool, verify_pgvector_extension};
use crate::hams_tools::HamsHarness;
use crate::metrics::{prometheus_response_free, prometheus_response_mystate};
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

    // Setup Prometheus Metrics Recorder
    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {e}"))?;

    let ct = tokio_util::sync::CancellationToken::new();

    // 2. Initialize HaMS Health Monitoring Sidecar & ProbeManual readiness signal
    let mut hams_config = config.hams.clone();
    hams_config.name = NAME.to_owned();
    hams_config.version = VERSION.to_owned();

    let hams = Hams::new(hams_config);
    let mut hams_harness = HamsHarness::init(hams, ct.clone()).await
        .map_err(|e| format!("HaMS init error: {}", e))?;
    info!("HaMS health sidecar started on port 8079 with readiness probe and shutdown hook.");

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
        prometheus_handle: Arc::new(metric_handle),
    };

    // HaMS Prometheus Registration
    hams_harness.hams.register_prometheus(
        prometheus_response_mystate,
        prometheus_response_free,
        &app_state as *const _ as *const c_void,
    ).map_err(|e| format!("Failed to register Prometheus with HaMS: {e}"))?;

    // 5. Start Axum Main REST Webservice
    let res = start_webserver(app_state, &config.webservice, ct).await;

    if let Err(e) = hams_harness.hams.deregister_prometheus() {
        tracing::error!("Failed to deregister Prometheus: {e}");
    }

    if let Err(e) = hams_harness.hams.stop() {
        tracing::info!("Failed to stop HaMS, it may already be stopped: {e}");
    }

    res?;

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
