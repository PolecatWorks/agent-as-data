use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde_json::{json, Value};
use super::knowledge_mcp::KnowledgeMcpServer;

pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/", post(handle_http_rpc))
}

async fn handle_http_rpc(
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let server = KnowledgeMcpServer::new();
    let response = handle_json_rpc(&server, payload).await;
    match response {
        Some(resp) => (StatusCode::OK, Json(resp)).into_response(),
        None => StatusCode::NO_CONTENT.into_response(),
    }
}

pub async fn handle_json_rpc(server: &KnowledgeMcpServer, raw_request: Value) -> Option<Value> {
    let id = raw_request.get("id").cloned();
    let method = match raw_request.get("method").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => {
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
        }
    };

    let params = raw_request.get("params").cloned().unwrap_or(Value::Null);

    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "aad-be-mcp",
                    "version": "1.0.0"
                },
                "instructions": "Knowledge Management MCP Server"
            })
        }
        "notifications/initialized" => {
            id.as_ref()?;
            json!({})
        }
        "ping" => {
            json!({})
        }
        "tools/list" => {
            let tools = serde_json::json!([
                { "name": "search_knowledge", "description": "Searches the knowledge graph using a text query" },
                { "name": "read_knowledge", "description": "Reads a specific knowledge node by its ID" },
                { "name": "ingest_knowledge", "description": "Ingests a new knowledge node and returns the created node" },
                { "name": "update_knowledge", "description": "Updates an existing knowledge node and returns the updated node" },
                { "name": "delete_knowledge", "description": "Deletes a knowledge node by its ID and returns the deleted node" },
                { "name": "traverse_knowledge_graph", "description": "Traverses the knowledge graph starting from a subject" }
            ]);
            json!({
                "tools": tools
            })
        }
        "tools/call" => {
            let tool_name = match params.get("name").and_then(|n| n.as_str()) {
                Some(n) => n,
                None => {
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
                }
            };

            let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

            let result_str = if tool_name == "search_knowledge" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.search_knowledge(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for search_knowledge: {e}")),
                }
            } else if tool_name == "read_knowledge" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.read_knowledge(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for read_knowledge: {e}")),
                }
            } else if tool_name == "ingest_knowledge" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.ingest_knowledge(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for ingest_knowledge: {e}")),
                }
            } else if tool_name == "update_knowledge" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.update_knowledge(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for update_knowledge: {e}")),
                }
            } else if tool_name == "delete_knowledge" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.delete_knowledge(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for delete_knowledge: {e}")),
                }
            } else if tool_name == "traverse_knowledge_graph" {
                match serde_json::from_value(arguments) {
                    Ok(args) => server.traverse_knowledge_graph(rmcp::handler::server::wrapper::Parameters(args)).await,
                    Err(e) => Err(format!("Invalid arguments for traverse_knowledge_graph: {e}")),
                }
            } else {
                Err(format!("Method {} not supported", tool_name))
            };

            match result_str {
                Ok(val) => json!({"content": [{"type": "text", "text": val}]}),
                Err(e) => {
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": e
                            }
                        ],
                        "isError": true
                    })
                }
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
