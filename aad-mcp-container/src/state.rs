use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use crate::config::AppConfig;
use crate::tools::ToolRegistry;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub tools: Arc<ToolRegistry>,
    pub prometheus_handle: Arc<PrometheusHandle>,
    pub sse_sessions: Arc<RwLock<HashMap<String, mpsc::Sender<String>>>>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        tools: Arc<ToolRegistry>,
        prometheus_handle: Arc<PrometheusHandle>,
    ) -> Self {
        Self {
            config,
            tools,
            prometheus_handle,
            sse_sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
