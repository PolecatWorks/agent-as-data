use std::sync::Arc;
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use aad_mcp_container::config::{AppConfig, DebuggingConfig, WebServiceConfig};
use aad_mcp_container::state::AppState;
use aad_mcp_container::server::AadMcpServer;
use aad_mcp_container::webserver::create_app;

#[tokio::test]
async fn test_http_json_rpc_end_to_end() -> anyhow::Result<()> {
    let ct = CancellationToken::new();

    let config = AppConfig {
        webservice: WebServiceConfig {
            address: "127.0.0.1:0".to_string(),
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

    let state = AppState::new(config, AadMcpServer::new(), Arc::new(recorder));
    let app = create_app(state, ct.clone());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server_handle = tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    let client = reqwest::Client::new();
    let url = format!("http://{addr}/api/v1/mcp");

    // 0. Health check
    let health_resp = client.get(format!("http://{addr}/healthz")).send().await?;
    assert_eq!(health_resp.status(), reqwest::StatusCode::OK);
    assert_eq!(health_resp.text().await?, "ok");

    // 1. Initialize
    let init_payload = json!({
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
    let resp = client.post(&url).json(&init_payload).send().await?;
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = resp.json().await?;
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(body["result"]["serverInfo"]["name"], "aad-mcp");

    // 2. Initialized notification
    let initialized_payload = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let notif_resp = client.post(&url).json(&initialized_payload).send().await?;
    assert_eq!(notif_resp.status(), reqwest::StatusCode::NO_CONTENT);

    // 3. Ping
    let ping_payload = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "ping"
    });
    let ping_resp = client.post(&url).json(&ping_payload).send().await?;
    assert_eq!(ping_resp.status(), reqwest::StatusCode::OK);
    let ping_body: serde_json::Value = ping_resp.json().await?;
    assert_eq!(ping_body["id"], 2);
    assert!(ping_body["result"].is_object());

    // 4. Tools list
    let list_payload = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/list"
    });
    let list_resp = client.post(&url).json(&list_payload).send().await?;
    assert_eq!(list_resp.status(), reqwest::StatusCode::OK);
    let list_body: serde_json::Value = list_resp.json().await?;
    assert_eq!(list_body["id"], 3);
    let tools = list_body["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools.len(), 6);
    let hello_tool = tools.iter().find(|t| t["name"] == "hello").unwrap();
    assert!(hello_tool["description"].as_str().unwrap().contains("greeting"));
    assert!(hello_tool["inputSchema"]["properties"]["name"].is_object());

    // 5. Tools call success
    let call_payload = json!({
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
    let call_resp = client.post(&url).json(&call_payload).send().await?;
    assert_eq!(call_resp.status(), reqwest::StatusCode::OK);
    let call_body: serde_json::Value = call_resp.json().await?;
    assert_eq!(call_body["id"], 4);
    assert_eq!(call_body["result"]["isError"], false);
    assert_eq!(call_body["result"]["content"][0]["text"], "Hello, Antigravity!");

    // 6. Tools call whitespace trimming
    let call_ws = json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "hello",
            "arguments": {
                "name": "   Alice   "
            }
        }
    });
    let call_ws_resp = client.post(&url).json(&call_ws).send().await?;
    let call_ws_body: serde_json::Value = call_ws_resp.json().await?;
    assert_eq!(call_ws_body["result"]["isError"], false);
    assert_eq!(call_ws_body["result"]["content"][0]["text"], "Hello, Alice!");

    // 7. Tools call empty name error
    let call_err = json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "hello",
            "arguments": {
                "name": "   "
            }
        }
    });
    let call_err_resp = client.post(&url).json(&call_err).send().await?;
    let call_err_body: serde_json::Value = call_err_resp.json().await?;
    assert_eq!(call_err_body["result"]["isError"], true);
    assert!(call_err_body["result"]["content"][0]["text"].as_str().unwrap().contains("non-empty string"));

    // 8. Nonexistent tool call
    let call_missing = json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "nonexistent",
            "arguments": {}
        }
    });
    let call_missing_resp = client.post(&url).json(&call_missing).send().await?;
    let call_missing_body: serde_json::Value = call_missing_resp.json().await?;
    assert_eq!(call_missing_body["result"]["isError"], true);
    assert!(call_missing_body["result"]["content"][0]["text"].as_str().unwrap().contains("not found"));

    // 9. Root route alias
    let root_url = format!("http://{addr}/");
    let root_resp = client.post(&root_url).json(&ping_payload).send().await?;
    assert_eq!(root_resp.status(), reqwest::StatusCode::OK);

    // 10. /mcp route alias
    let mcp_alias_url = format!("http://{addr}/mcp");
    let mcp_alias_resp = client.post(&mcp_alias_url).json(&ping_payload).send().await?;
    assert_eq!(mcp_alias_resp.status(), reqwest::StatusCode::OK);

    ct.cancel();
    let _ = server_handle.await;

    Ok(())
}
