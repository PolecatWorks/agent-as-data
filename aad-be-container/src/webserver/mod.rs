//! HTTP webserver and API router definitions.

pub mod aad_mcp;
pub mod aad_mcp_handler;
pub mod agents;
pub mod analytics;
pub mod benches;
pub mod execution;
pub mod fs;
pub mod knowledge;
pub mod knowledge_mcp;
pub mod knowledge_mcp_handler;
pub mod memory;
pub mod skills;
pub mod threads;
pub mod tools;
pub mod traits;

use axum::{Router, routing::{get, post}};
use axum_prometheus::PrometheusMetricLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};

use crate::config::WebServiceConfig;
use crate::state::AppState;

pub mod search;

fn v1_routes(state: AppState) -> Router {
    Router::new()
        .nest("/agents", agents::router())
        .nest("/agent-context/search", search::router())
        .nest("/skills", skills::router())
        .nest("/traits", traits::router())
        .nest("/agents/tools", tools::router())
        .nest("/knowledge", knowledge::router())
        .nest("/knowledge/mcp", knowledge_mcp_handler::router())
        .nest("/mcp", aad_mcp_handler::router())
        .merge(execution::router())
        .nest("/benches", benches::router())
        .nest("/benches", fs::router())
        .nest("/benches", memory::router())
        .nest("/threads", threads::router())
        .nest("/threads", fs::router())
        .nest("/analytics", analytics::router())
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
}

pub fn app_router(state: AppState) -> Router {
    let metric_layer = PrometheusMetricLayer::new();
    let prefix = state.config.webservice.api_prefix.trim_end_matches('/');
    let normalized_prefix = if prefix.is_empty() {
        "/api/aad/v1"
    } else {
        prefix
    };

    let mut router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/mcp", post(aad_mcp_handler::handle_http_rpc))
        .with_state(state.clone())
        .nest(normalized_prefix, v1_routes(state.clone()));

    let prefixed_v1 = format!("{}/v1", normalized_prefix);
    if !normalized_prefix.ends_with("/v1") {
        router = router.nest(&prefixed_v1, v1_routes(state.clone()));
    }

    if normalized_prefix != "/api/v1" && prefixed_v1 != "/api/v1" {
        router = router.nest("/api/v1", v1_routes(state.clone()));
    }

    router
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
    let _ = knowledge_mcp::DB_POOL.set(state.pool.clone());

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
    use crate::config::AppConfig;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn build_test_state(api_prefix: &str) -> AppState {
        let handle = PrometheusBuilder::new().build_recorder().handle();
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost:5432/test")
            .unwrap();
        let config = AppConfig {
            debugging: crate::config::DebuggingConfig {
                environment: "test".into(),
                log_level: "info".into(),
                fail_debug_delay: std::time::Duration::from_secs(0),
            },
            webservice: crate::config::WebServiceConfig {
                address: "127.0.0.1:8080".into(),
                api_prefix: api_prefix.into(),
            },
            llm: crate::config::LlmConfig {
                ollama_url: "http://localhost:11434".into(),
                model: "llama3".into(),
                timeout: std::time::Duration::from_secs(30),
                default_max_turns: 5,
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

        AppState {
            pool,
            config,
            prometheus_handle: Arc::new(handle),
            tokio_handle: tokio::runtime::Handle::current(),
        }
    }

    #[tokio::test]
    async fn test_app_router_construction() {
        let state = build_test_state("/api");
        // This verifies all nested routes and syntax parse cleanly without panic
        let _router = app_router(state);
    }

    #[tokio::test]
    async fn test_configurable_api_prefix_and_backward_compatibility() {
        let state = build_test_state("/api/aad/v1");
        let app = app_router(state);

        // 1. Root /health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 2. Prefixed /api/aad/v1/health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/aad/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 3. Backward-compatible alias /api/v1/health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // 4. Verify /api/aad/v1/agents does not 404 (method not allowed / 405 for GET without query or body)
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/aad/v1/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(res.status(), StatusCode::NOT_FOUND);

        // 5. Verify backward compatible /api/v1/agents does not 404
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_ne!(res.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_custom_api_prefix_without_v1() {
        let state = build_test_state("/custom");
        let app = app_router(state);

        // Should route to /custom/health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/custom/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // Should also route to /custom/v1/health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/custom/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        // And backward-compatible /api/v1/health
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unified_mcp_and_knowledge_mcp_routes() {
        use http_body_util::BodyExt;

        let state = build_test_state("/api/aad/v1");
        let app = app_router(state);

        let list_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        })
        .to_string();

        for uri in ["/mcp", "/api/aad/v1/mcp", "/api/v1/mcp"] {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(uri)
                        .header("content-type", "application/json")
                        .body(Body::from(list_payload.clone()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK, "URI {} should return 200", uri);
            let bytes = res.into_body().collect().await.unwrap().to_bytes();
            let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            let tools = body["result"]["tools"].as_array().unwrap();
            assert!(
                tools.iter().any(|t| t["name"] == "list_agents"),
                "URI {} should expose list_agents",
                uri
            );
            assert!(
                tools.iter().any(|t| t["name"] == "search_knowledge"),
                "URI {} should expose search_knowledge",
                uri
            );
        }

        // Verify backward-compatible /api/aad/v1/knowledge/mcp still returns Knowledge tools with schemas
        let kb_res = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/aad/v1/knowledge/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(list_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(kb_res.status(), StatusCode::OK);
        let kb_bytes = kb_res.into_body().collect().await.unwrap().to_bytes();
        let kb_body: serde_json::Value = serde_json::from_slice(&kb_bytes).unwrap();
        let kb_tools = kb_body["result"]["tools"].as_array().unwrap();
        assert!(kb_tools.iter().any(|t| t["name"] == "search_knowledge" && t["inputSchema"].is_object()));
    }
}
