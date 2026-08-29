//! HTTP webserver and API router definitions.

pub mod agents;
pub mod execution;
pub mod fs;
pub mod knowledge;
pub mod skills;
pub mod threads;
pub mod tools;
pub mod traits;

use axum::{routing::get, Router};
use tracing::info;

use crate::config::WebServiceConfig;
use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .nest("/v1/agents", agents::router())
        .nest("/v1/skills", skills::router())
        .nest("/v1/traits", traits::router())
        .nest("/v1/agents/tools", tools::router())
        .nest("/v1/knowledge", knowledge::router())
        .nest("/v1", execution::router())
        .nest("/v1/threads", threads::router())
        .nest("/v1/threads", fs::router())
        .with_state(state.clone());

    Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest(&state.config.webservice.api_prefix, api_routes)
}

pub async fn start_webserver(
    state: AppState,
    config: &WebServiceConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = app_router(state);
    let listener = tokio::net::TcpListener::bind(&config.address)
        .await
        .map_err(|e| format!("Listener bind error on {}: {}", config.address, e))?;

    info!("Axum REST Service listening on {}", config.address);
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("Axum serve error: {}", e))?;

    Ok(())
}
