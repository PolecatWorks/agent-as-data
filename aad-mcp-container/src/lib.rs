//! Agent-As-Data MCP Server (`aad-mcp-container`) core library.
//!
//! Provides application lifecycle orchestration, configuration loading,
//! Model Context Protocol (MCP) server endpoints, HaMS health sidecar integration, and Axum webserver.

pub mod config;
pub mod hams_tools;
pub mod metrics;
pub mod state;
pub mod tokio_tools;
pub mod tools;
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
use crate::hams_tools::HamsHarness;
use crate::metrics::{prometheus_response_free, prometheus_response_mystate};
use crate::tools::hello::HelloTool;
use crate::tools::ToolRegistry;
use crate::webserver::start_webserver;

/// Main application service orchestrator.
///
/// 1. Loads fail-fast AppConfig from file and secrets.
/// 2. Validates configuration.
/// 3. Initializes HaMS sidecar on health port (default 8079).
/// 4. Initializes tool registry with standard baseline tools (e.g. HelloTool).
/// 5. Binds and serves the Axum MCP webservice.
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

    // 2. Setup Prometheus Metrics Recorder
    let metric_handle = PrometheusBuilder::new()
        .install_recorder()
        .map_err(|e| format!("Failed to install Prometheus recorder: {e}"))?;

    let ct = tokio_util::sync::CancellationToken::new();

    // 3. Initialize HaMS Health Monitoring Sidecar & ProbeManual readiness signal
    let mut hams_config = config.hams.clone();
    hams_config.name = NAME.to_owned();
    hams_config.version = VERSION.to_owned();

    let hams = Hams::new(hams_config);
    let mut hams_harness = HamsHarness::init(hams, ct.clone()).await
        .map_err(|e| format!("HaMS init error: {}", e))?;
    info!("HaMS health sidecar started on port {} with readiness probe and shutdown hook.", config.hams.address.port());

    // 4. Initialize Tool Registry
    let mut tool_registry = ToolRegistry::new();
    tool_registry.register(Arc::new(HelloTool::new()));
    info!("Registered default MCP tools: hello");

    let app_state = AppState::new(
        config.clone(),
        Arc::new(tool_registry),
        Arc::new(metric_handle),
    );

    // HaMS Prometheus Registration
    hams_harness.hams.register_prometheus(
        prometheus_response_mystate,
        prometheus_response_free,
        &app_state as *const _ as *const c_void,
    ).map_err(|e| format!("Failed to register Prometheus with HaMS: {e}"))?;

    // 5. Start Axum MCP Webservice
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
