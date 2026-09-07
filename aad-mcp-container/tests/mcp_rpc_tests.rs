use std::sync::Arc;
use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
use rmcp::{
    model::{CallToolRequestParams, ClientInfo},
    ServiceExt,
    transport::{
        StreamableHttpClientTransport,
        streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use tokio_util::sync::CancellationToken;

use aad_mcp_container::config::{AppConfig, DebuggingConfig, WebServiceConfig};
use aad_mcp_container::state::AppState;
use aad_mcp_container::tools::AadMcpServer;
use aad_mcp_container::webserver::create_app;

#[tokio::test]
async fn test_rmcp_server_end_to_end() -> anyhow::Result<()> {
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

    let transport = StreamableHttpClientTransport::from_config(
        StreamableHttpClientTransportConfig::with_uri(format!("http://{addr}/api/v1/mcp")),
    );
    let client = ClientInfo::default().serve(transport).await?;

    // 1. Tool discovery
    let tool_list = client.list_tools(Default::default()).await?;
    assert_eq!(tool_list.tools.len(), 1);
    assert_eq!(tool_list.tools[0].name, "hello");
    assert!(!tool_list.tools[0].description.as_deref().unwrap_or_default().is_empty());

    // 2. Tool invocation success
    let args: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({ "name": "Antigravity" }))?;
    let call_res = client
        .call_tool(CallToolRequestParams::new("hello").with_arguments(args))
        .await?;
    assert_ne!(call_res.is_error, Some(true));
    let content_json = serde_json::to_string(&call_res.content[0])?;
    assert!(content_json.contains("Hello, Antigravity!"));

    // 3. Tool invocation whitespace trimming
    let args_ws: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({ "name": "   Alice   " }))?;
    let call_res_ws = client
        .call_tool(CallToolRequestParams::new("hello").with_arguments(args_ws))
        .await?;
    assert_ne!(call_res_ws.is_error, Some(true));
    let content_ws = serde_json::to_string(&call_res_ws.content[0])?;
    assert!(content_ws.contains("Hello, Alice!"));

    // 4. Tool validation error for empty name
    let args_empty: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({ "name": "   " }))?;
    let call_res_empty = client
        .call_tool(CallToolRequestParams::new("hello").with_arguments(args_empty))
        .await?;
    assert_eq!(call_res_empty.is_error, Some(true));

    // 5. Tool call for nonexistent tool
    let args_dummy: serde_json::Map<String, serde_json::Value> =
        serde_json::from_value(serde_json::json!({}))?;
    let call_res_missing = client
        .call_tool(CallToolRequestParams::new("nonexistent").with_arguments(args_dummy))
        .await;
    assert!(call_res_missing.is_err() || call_res_missing.unwrap().is_error == Some(true));

    let _ = client.cancel().await;
    ct.cancel();
    let _ = server_handle.await;

    Ok(())
}
