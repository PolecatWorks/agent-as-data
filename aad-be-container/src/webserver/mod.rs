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
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};

use crate::config::WebServiceConfig;
use crate::state::AppState;

pub fn app_router(state: AppState) -> Router {
    let metric_layer = PrometheusMetricLayer::new();
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
        .layer(metric_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
}

pub async fn start_webserver(
    state: AppState,
    config: &WebServiceConfig,
    ct: tokio_util::sync::CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = app_router(state);
    let listener = tokio::net::TcpListener::bind(&config.address)
        .await
        .map_err(|e| format!("Listener bind error on {}: {}", config.address, e))?;

    info!("Axum REST Service listening on {}", config.address);
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            ct.cancelled().await;
            info!("Received cancellation token, shutting down web server");
        })
        .await
        .map_err(|e| format!("Axum serve error: {}", e))?;

    Ok(())
}
