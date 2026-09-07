use serde_json::{json, Value};
use crate::state::AppState;

pub async fn handle_json_rpc(state: &AppState, raw_request: Value) -> Option<Value> {
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
                    "name": "aad-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })
        }
        "notifications/initialized" => {
            if id.is_none() {
                return None;
            }
            json!({})
        }
        "ping" => {
            json!({})
        }
        "tools/list" => {
            json!({
                "tools": state.tools.list()
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

            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match state.tools.get(tool_name) {
                Some(tool) => {
                    let tool_result = tool.execute(arguments).await;
                    json!(tool_result)
                }
                None => {
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error: Tool '{}' not found", tool_name)
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
