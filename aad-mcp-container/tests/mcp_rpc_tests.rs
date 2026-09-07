use std::sync::Arc;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use serde_json::json;
use aad_mcp_container::config::{AppConfig, WebServiceConfig, DebuggingConfig};
use aad_mcp_container::state::AppState;
use aad_mcp_container::tools::ToolRegistry;
use aad_mcp_container::tools::hello::HelloTool;
use aad_mcp_container::webserver::create_app;
use http_body_util::BodyExt;
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
use tokio_stream::StreamExt;

fn setup_test_app() -> axum::Router {
    let mut registry = ToolRegistry::new();
    registry.register(Arc::new(HelloTool::new()));

    let config = AppConfig {
        webservice: WebServiceConfig {
            address: "127.0.0.1:8080".to_string(),
            api_prefix: "/api".to_string(),
        },
        hams: ::hams::hams::config::HamsConfig::default(),
        runtime: Default::default(),
        debugging: DebuggingConfig {
            environment: "test".to_string(),
            log_level: "info".to_string(),
            fail_debug_delay: std::time::Duration::from_secs(0),
        },
    };

    let recorder = PrometheusBuilder::new().install_recorder().unwrap_or_else(|_| {
        PrometheusBuilder::new().build_recorder().handle()
    });

    let state = AppState::new(config, Arc::new(registry), Arc::new(recorder));
    create_app(state)
}

#[tokio::test]
async fn test_rpc_initialize() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "1.0.0"
            }
        }
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["jsonrpc"], "2.0");
    assert_eq!(json_res["id"], 1);
    assert_eq!(json_res["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(json_res["result"]["serverInfo"]["name"], "aad-mcp");
    assert!(json_res["result"]["capabilities"]["tools"].is_object());
}

#[tokio::test]
async fn test_rpc_ping() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["id"], 2);
    assert_eq!(json_res["result"], json!({}));
}

#[tokio::test]
async fn test_rpc_tools_list() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list"
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["id"], 3);
    let tools = json_res["result"]["tools"].as_array().expect("tools should be array");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "hello");
    assert_eq!(tools[0]["inputSchema"]["required"], json!(["name"]));
}

#[tokio::test]
async fn test_rpc_tools_call_hello_success() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "hello",
            "arguments": {
                "name": "Antigravity"
            }
        }
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["id"], 4);
    assert_eq!(json_res["result"]["isError"], false);
    assert_eq!(json_res["result"]["content"][0]["text"], "Hello, Antigravity!");
}

#[tokio::test]
async fn test_rpc_tools_call_hello_validation_error() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "hello",
            "arguments": {
                "name": ""
            }
        }
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["id"], 5);
    assert_eq!(json_res["result"]["isError"], true);
    assert!(json_res["result"]["content"][0]["text"].as_str().unwrap().contains("non-empty string"));
}

#[tokio::test]
async fn test_rpc_unknown_method() {
    let app = setup_test_app();

    let request_payload = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "nonexistent_method"
    });

    let request = Request::builder()
        .uri("/api/v1/rpc")
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json_res: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json_res["id"], 6);
    assert_eq!(json_res["error"]["code"], -32601);
}

#[tokio::test]
async fn test_sse_endpoint_and_message_delivery() {
    let app = setup_test_app();

    // 1. Initiate SSE connection
    let sse_request = Request::builder()
        .uri("/api/v1/sse")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let sse_response = app.clone().oneshot(sse_request).await.unwrap();
    assert_eq!(sse_response.status(), StatusCode::OK);
    assert_eq!(
        sse_response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    // 2. Read initial endpoint event
    let mut body_stream = sse_response.into_body().into_data_stream();
    let first_chunk = body_stream.next().await.unwrap().unwrap();
    let first_chunk_str = String::from_utf8_lossy(&first_chunk);
    assert!(first_chunk_str.contains("event: endpoint"));
    assert!(first_chunk_str.contains("/api/v1/message?session_id="));

    // Extract session_id
    let session_id = first_chunk_str
        .lines()
        .find(|l| l.starts_with("data:"))
        .unwrap()
        .split("session_id=")
        .nth(1)
        .unwrap()
        .trim();

    // 3. Post a message to /api/v1/message with session_id
    let message_payload = json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "tools/call",
        "params": {
            "name": "hello",
            "arguments": {
                "name": "SSEClient"
            }
        }
    });

    let msg_request = Request::builder()
        .uri(format!("/api/v1/message?session_id={}", session_id))
        .method("POST")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&message_payload).unwrap()))
        .unwrap();

    let msg_response = app.oneshot(msg_request).await.unwrap();
    assert_eq!(msg_response.status(), StatusCode::ACCEPTED);

    // 4. Verify SSE stream receives the response
    let second_chunk = body_stream.next().await.unwrap().unwrap();
    let second_chunk_str = String::from_utf8_lossy(&second_chunk);
    assert!(second_chunk_str.contains("event: message"));
    assert!(second_chunk_str.contains("Hello, SSEClient!"));
}
