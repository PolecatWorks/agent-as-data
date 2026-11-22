//! HTTP webserver and API router definitions.

pub mod agents;
pub mod analytics;
pub mod benches;
pub mod execution;
pub mod fs;
pub mod knowledge;
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
    let mut router = Router::new()
        .nest("/agents", agents::router())
        .nest("/agent-context/search", search::router())
        .nest("/skills", skills::router())
        .nest("/traits", traits::router())
        .nest("/agents/tools", tools::router())
        .nest("/knowledge", knowledge::router())
        .merge(execution::router())
        .nest("/benches", benches::router())
        .nest("/benches", fs::router())
        .nest("/benches", memory::router())
        .nest("/threads", threads::router())
        .nest("/threads", fs::router())
        .nest("/analytics", analytics::router())
        .route("/health", get(|| async { "OK" }));

    if state.config.mcp.enabled {
        router = router.nest("/mcp", crate::mcp::rpc::router());
    }

    router.with_state(state)
}

pub fn app_router(state: AppState) -> Router {
    let metric_layer = PrometheusMetricLayer::new();
    let normalized_prefix = state.config.webservice.api_prefix.trim_end_matches('/');

    let mut base_router = Router::new().route("/health", get(|| async { "OK" }));
    if state.config.mcp.enabled {
        base_router = base_router.route("/mcp", post(crate::mcp::rpc::handle_http_rpc));
    }

    let mut router = base_router
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
    let app = app_router(state);
    let listener = tokio::net::TcpListener::bind(config.address)
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
                address: "127.0.0.1:8080".parse().unwrap(),
                api_prefix: api_prefix.into(),
            },
            llm: crate::config::LlmConfig {
                ollama_url: url::Url::parse("http://localhost:11434").unwrap(),
                model: "llama3".into(),
                timeout: std::time::Duration::from_secs(30),
                default_max_turns: 5,
            },
            mcp: crate::mcp::McpConfig::default(),
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
    async fn test_unified_mcp_routes_expose_knowledge_tools_and_retire_legacy_route() {
        use http_body_util::BodyExt;

        let state = build_test_state("/api/aad/v1");
        let app = app_router(state);

        let list_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        })
        .to_string();

        let knowledge_tools = [
            "search_knowledge",
            "read_knowledge",
            "ingest_knowledge",
            "update_knowledge",
            "delete_knowledge",
            "traverse_knowledge_graph",
        ];

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
            for kb_tool in knowledge_tools {
                assert!(
                    tools
                        .iter()
                        .any(|t| t["name"] == kb_tool && t["inputSchema"].is_object()),
                    "URI {} should expose {} with JSON inputSchema",
                    uri,
                    kb_tool
                );
            }
        }

        // Verify retired /knowledge/mcp POST endpoints are no longer mounted
        // (POST falls through to /knowledge/{id} which only accepts GET/PUT/DELETE -> 405)
        for retired_uri in ["/api/aad/v1/knowledge/mcp", "/api/v1/knowledge/mcp"] {
            let kb_res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(retired_uri)
                        .header("content-type", "application/json")
                        .body(Body::from(list_payload.clone()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(
                kb_res.status(),
                StatusCode::METHOD_NOT_ALLOWED,
                "Retired URI {} should not accept MCP POST requests",
                retired_uri
            );
        }
    }

    #[tokio::test]
    async fn test_mcp_disabled_returns_404() {
        let mut state = build_test_state("/api/aad/v1");
        state.config.mcp.enabled = false;
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
            assert_eq!(
                res.status(),
                StatusCode::NOT_FOUND,
                "URI {} should return 404 when mcp.enabled is false",
                uri
            );
        }
    }
}
