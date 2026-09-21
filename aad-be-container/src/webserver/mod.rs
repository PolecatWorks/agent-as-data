//! HTTP webserver and API router definitions.

pub mod agents;
pub mod execution;
pub mod fs;
pub mod knowledge;
pub mod skills;
pub mod threads;
pub mod tools;
pub mod traits;
pub mod benches;
pub mod memory;
pub mod analytics;

use axum::{routing::get, Router};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{info, Level};

use crate::config::WebServiceConfig;
use crate::state::AppState;

pub mod search;
pub fn app_router(state: AppState) -> Router {
    let metric_layer = PrometheusMetricLayer::new();
    let api_routes = Router::new()

        .nest("/v1/agents", agents::router())
        .nest("/v1/agent-context/search", search::router())
        .nest("/v1/skills", skills::router())
        .nest("/v1/traits", traits::router())
        .nest("/v1/agents/tools", tools::router())
        .nest("/v1/knowledge", knowledge::router())
        .nest("/v1", execution::router())
        .nest("/v1/benches", benches::router())
        .nest("/v1/benches", fs::router())
        .nest("/v1/benches", memory::router())
        .nest("/v1/threads", threads::router())
        .nest("/v1/threads", fs::router())
        .nest("/v1/analytics", analytics::router())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
    use sqlx::postgres::PgPoolOptions;
    use crate::config::AppConfig;

    #[tokio::test]
    async fn test_app_router_construction() {
        let handle = PrometheusBuilder::new().build_recorder().handle();
        let pool = PgPoolOptions::new().connect_lazy("postgres://user:pass@localhost:5432/test").unwrap();
        let config = AppConfig {
            debugging: crate::config::DebuggingConfig {
                environment: "test".into(),
                log_level: "info".into(),
                fail_debug_delay: std::time::Duration::from_secs(0),
            },
            webservice: crate::config::WebServiceConfig {
                address: "127.0.0.1:8080".into(),
                api_prefix: "/api".into(),
            },
            llm: crate::config::LlmConfig {
                ollama_url: "http://localhost:11434".into(),
                model: "llama3".into(),
                timeout_secs: 30,
            },
            runtime: crate::tokio_tools::ThreadRuntime::default(),
            database: crate::config::DatabaseConfig {
                url: crate::config::UrlWithUsernamePassword {
                    url: url::Url::parse("postgres://localhost:5432/test").unwrap(),
                    username: Some("user".into()),
                    password: Some("pass".into()),
                },
                max_connections: 1,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
        };

        let state = AppState {
            pool,
            config,
            prometheus_handle: Arc::new(handle),
            tokio_handle: tokio::runtime::Handle::current(),
        };

        // This verifies all nested routes and syntax (e.g. {id} vs :id) parse cleanly without panic
        let _router = app_router(state);
    }
}
