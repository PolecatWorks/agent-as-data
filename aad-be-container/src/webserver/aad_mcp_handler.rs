//! Axum HTTP JSON-RPC 2.0 handler for the unified in-process AAD MCP server.

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde_json::{Value, json};

use super::aad_mcp::AadMcpServer;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(handle_http_rpc))
}

pub async fn handle_http_rpc(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let server = AadMcpServer::new(state);
    match handle_json_rpc(&server, payload).await {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

pub async fn handle_json_rpc(server: &AadMcpServer, raw_request: Value) -> Option<Value> {
    let id = raw_request.get("id").cloned();
    let Some(method) = raw_request.get("method").and_then(|m| m.as_str()) else {
        return id.map(|id_val| {
            json!({
                "jsonrpc": "2.0",
                "id": id_val,
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: missing method"
                }
            })
        });
    };

    let params = raw_request.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => {
            let info = server.info();
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "aad-be-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": info.instructions.unwrap_or_default()
            })
        }
        "notifications/initialized" => {
            id.as_ref()?;
            json!({})
        }
        "ping" => json!({}),
        "tools/list" => {
            let tools = server.list_tools();
            json!({
                "tools": tools
            })
        }
        "tools/call" => {
            let Some(tool_name) = params.get("name").and_then(|n| n.as_str()) else {
                return id.map(|id_val| {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id_val,
                        "error": {
                            "code": -32602,
                            "message": "Invalid params: 'name' is required"
                        }
                    })
                });
            };

            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            match server.call_tool(tool_name, arguments).await {
                Ok(text) => json!({
                    "content": [
                        {
                            "type": "text",
                            "text": text
                        }
                    ],
                    "isError": false
                }),
                Err(err) => json!({
                    "content": [
                        {
                            "type": "text",
                            "text": err
                        }
                    ],
                    "isError": true
                }),
            }
        }
        _ => {
            return id.map(|id_val| {
                json!({
                    "jsonrpc": "2.0",
                    "id": id_val,
                    "error": {
                        "code": -32601,
                        "message": format!("Method '{}' not found", method)
                    }
                })
            });
        }
    };

    id.map(|id_val| {
        json!({
            "jsonrpc": "2.0",
            "id": id_val,
            "result": result
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;

    fn test_server() -> AadMcpServer {
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
                api_prefix: "/api/aad/v1".into(),
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
        let state = AppState {
            pool,
            config,
            prometheus_handle: Arc::new(handle),
            tokio_handle: tokio::runtime::Handle::current(),
        };
        AadMcpServer::new(state)
    }

    #[tokio::test]
    async fn test_initialize_and_ping_and_notification() {
        let server = test_server();

        let init = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }),
        )
        .await
        .expect("initialize must return a response");
        assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(init["result"]["serverInfo"]["name"], "aad-be-mcp");

        let notif = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        )
        .await;
        assert!(notif.is_none());

        let ping = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "id": 2, "method": "ping" }),
        )
        .await
        .expect("ping must return a response");
        assert_eq!(ping["result"], json!({}));
    }

    #[tokio::test]
    async fn test_tools_list_exposes_all_aad_capabilities_with_input_schemas() {
        let server = test_server();
        let resp = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/list" }),
        )
        .await
        .expect("tools/list must return a response");

        let tools = resp["result"]["tools"]
            .as_array()
            .expect("tools must be an array");

        let expected_tools = [
            "list_agents",
            "get_agent",
            "search_agents",
            "create_agent",
            "update_agent",
            "delete_agent",
            "list_skills",
            "get_skill",
            "create_skill",
            "update_skill",
            "delete_skill",
            "list_traits",
            "get_trait",
            "create_trait",
            "update_trait",
            "delete_trait",
            "list_mcp_servers",
            "get_mcp_server",
            "search_agent_context",
            "search_knowledge",
            "read_knowledge",
            "ingest_knowledge",
            "update_knowledge",
            "delete_knowledge",
            "traverse_knowledge_graph",
            "list_benches",
            "get_bench",
            "list_threads",
            "read_bench_memory",
            "update_bench_memory",
            "execute_agent",
            "get_execution",
        ];

        for expected in expected_tools {
            let found = tools
                .iter()
                .find(|t| t["name"].as_str() == Some(expected))
                .unwrap_or_else(|| panic!("Missing expected MCP tool '{}'", expected));
            assert!(
                found["inputSchema"].is_object(),
                "Tool '{}' must include a JSON Schema inputSchema",
                expected
            );
            assert!(
                found["description"]
                    .as_str()
                    .map(|d| !d.trim().is_empty())
                    .unwrap_or(false),
                "Tool '{}' must include a non-empty description",
                expected
            );
        }
    }

    #[tokio::test]
    async fn test_tools_call_validation_and_error_handling() {
        let server = test_server();

        // Missing tool name -> JSON-RPC -32602
        let missing_name = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "id": 10, "method": "tools/call", "params": {} }),
        )
        .await
        .unwrap();
        assert_eq!(missing_name["error"]["code"], -32602);

        // Unknown tool -> isError: true
        let unknown = handle_json_rpc(
            &server,
            json!({
                "jsonrpc": "2.0",
                "id": 11,
                "method": "tools/call",
                "params": { "name": "nonexistent_tool", "arguments": {} }
            }),
        )
        .await
        .unwrap();
        assert_eq!(unknown["result"]["isError"], true);

        // Empty query validation -> isError: true
        let empty_query = handle_json_rpc(
            &server,
            json!({
                "jsonrpc": "2.0",
                "id": 12,
                "method": "tools/call",
                "params": { "name": "search_agents", "arguments": { "query": "   " } }
            }),
        )
        .await
        .unwrap();
        assert_eq!(empty_query["result"]["isError"], true);

        // Missing both id and name on get_agent -> isError: true
        let missing_id_name = handle_json_rpc(
            &server,
            json!({
                "jsonrpc": "2.0",
                "id": 13,
                "method": "tools/call",
                "params": { "name": "get_agent", "arguments": {} }
            }),
        )
        .await
        .unwrap();
        assert_eq!(missing_id_name["result"]["isError"], true);

        // Unknown JSON-RPC method -> -32601
        let bad_method = handle_json_rpc(
            &server,
            json!({ "jsonrpc": "2.0", "id": 14, "method": "foo/bar" }),
        )
        .await
        .unwrap();
        assert_eq!(bad_method["error"]["code"], -32601);
    }
}
