use std::sync::Arc;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use crate::config::AppConfig;
use crate::server::AadMcpServer;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub server: AadMcpServer,
    pub prometheus_handle: Arc<PrometheusHandle>,
    pub tokio_handle: Handle,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        server: AadMcpServer,
        prometheus_handle: Arc<PrometheusHandle>,
        tokio_handle: Handle,
    ) -> Self {
        Self {
            config,
            server,
            prometheus_handle,
            tokio_handle,
        }
    }
}
