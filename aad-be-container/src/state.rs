use crate::config::AppConfig;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub prometheus_handle: Arc<PrometheusHandle>,
    pub tokio_handle: Handle,
}

impl axum::extract::FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}
