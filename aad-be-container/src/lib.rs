//! Agent-As-Data Backend (`aad-be-container`) core library.
//!
//! Provides application lifecycle orchestration, configuration loading,
//! database connection pooling, HaMS health sidecar integration, and Axum REST webservice.

pub mod cli;
pub mod config;
pub mod db;
pub mod error;
pub mod guardrails;
pub mod hams_tools;
pub mod kb_tools;
pub mod llm_tools;
pub mod mcp;
pub mod metrics;
pub mod models;
pub mod state;
pub mod tokio_tools;
pub mod webserver;

pub use state::AppState;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use ::hams::hams::Hams;
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
use std::sync::Arc;
use tracing::info;

use crate::config::AppConfig;
use crate::db::{init_db_pool, verify_pgvector_extension};
use crate::hams_tools::HamsHarness;
use crate::metrics::init_startup_metrics;
use crate::webserver::start_webserver;

/// Main application service orchestrator.
///
/// Expects a pre-loaded and validated [`AppConfig`].
///
/// Boot sequence (fail-fast):
/// 1. Connects to PostgreSQL pool and verifies `pgvector` extension.
/// 2. Executes automatic database schema migrations.
/// 3. Initializes Prometheus metrics, Tokio runtime metrics reporter, and `AppState`.
/// 4. Starts HaMS sidecar on health port 8079, registers Prometheus closure, and enables readiness.
/// 5. Binds and serves the Axum REST webservice.
pub async fn service_main(
    config: AppConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Starting {} v{}", NAME, VERSION);

    // 1. Connect DB Pool & Verify pgvector (Fail-Fast)
    let db_url: url::Url = config.database.url.clone().into();
    let pool = init_db_pool(db_url.as_str(), config.database.max_connections)
        .await
        .map_err(|e| format!("Fail-Fast Error: Database connection failed: {}", e))?;

    verify_pgvector_extension(&pool)
        .await
        .map_err(|e| format!("Fail-Fast pgvector check failed: {}", e))?;

    // 2. Run Automatic Schema Migrations
    info!("Executing database schema migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;
    info!("Database migrations applied successfully.");

    // 3. Setup Prometheus Metrics Recorder & Build AppState
    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {e}"))?;

    // Initialize baseline startup telemetry (guarantees non-empty /hams/metrics)
    init_startup_metrics(NAME, VERSION);

    // Spawn Tokio runtime metrics reporter
    tokio::task::spawn(
        tokio_metrics::RuntimeMetricsReporterBuilder::default()
            .with_interval(config.runtime.metrics_interval)
            .describe_and_run(),
    );

    let app_state = AppState {
        pool,
        config: config.clone(),
        prometheus_handle: Arc::new(metric_handle),
        tokio_handle: tokio::runtime::Handle::current(),
    };

    // 4. Initialize HaMS Health Monitoring Sidecar, register Prometheus, and enable readiness
    let ct = tokio_util::sync::CancellationToken::new();

    let mut hams_config = config.hams.clone();
    hams_config.name = NAME.to_owned();
    hams_config.version = VERSION.to_owned();

    let hams = Hams::new(hams_config);
    let mut hams_harness = HamsHarness::init(hams, ct.clone())
        .await
        .map_err(|e| format!("HaMS init error: {}", e))?;

    let handle_clone = Arc::clone(&app_state.prometheus_handle);
    hams_harness
        .hams
        .register_prometheus_closure(move || handle_clone.render())
        .map_err(|e| format!("Failed to register Prometheus closure with HaMS: {e}"))?;

    hams_harness.ready_signal.enable();
    info!("HaMS health sidecar started on port 8079 with readiness probe enabled and shutdown hook registered.");

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
///
/// Expects a pre-loaded and validated [`AppConfig`].
pub async fn run_migrations(
    config: AppConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let db_url: url::Url = config.database.url.into();
    let pool = init_db_pool(db_url.as_str(), config.database.max_connections)
        .await
        .map_err(|e| format!("DB connection error: {}", e))?;

    verify_pgvector_extension(&pool)
        .await
        .map_err(|e| format!("Fail-Fast pgvector check failed: {}", e))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;

    println!("Migrations completed successfully.");
    Ok(())
}
