use std::sync::Arc;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use crate::config::AppConfig;
use crate::server::AadMcpServer;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub server: AadMcpServer,
    pub prometheus_handle: Arc<PrometheusHandle>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        server: AadMcpServer,
        prometheus_handle: Arc<PrometheusHandle>,
    ) -> Self {
        Self {
            config,
            server,
            prometheus_handle,
        }
    }
}
