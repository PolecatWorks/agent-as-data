use std::sync::Arc;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use crate::config::AppConfig;
use sqlx::PgPool;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: PgPool,
    pub config: AppConfig,
    pub prometheus_handle: Arc<PrometheusHandle>,
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
