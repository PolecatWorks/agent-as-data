pub mod agents;
pub mod config;
pub mod db;
pub mod execution;
pub mod hams_tools;
pub mod knowledge;
pub mod tools;
pub mod fs_tools;
pub mod models;
pub mod traits;
pub mod tokio_tools;
pub mod threads;

#[derive(Clone, Debug)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: config::AppConfig,
}

impl axum::extract::FromRef<AppState> for sqlx::PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl axum::extract::FromRef<AppState> for config::AppConfig {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

pub const NAME: &str = "aad-be";
pub const VERSION: &str = "0.1.0";





