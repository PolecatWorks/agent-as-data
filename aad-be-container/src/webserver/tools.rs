use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Router,
};
use chrono::Utc;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    models::{
        RegisterToolRequest, RegisterToolResponse, SyncToolResponse, TestToolRequest,
        TestToolResponse, Tool,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tools))
        .route("/register", post(register_tool))
        .route("/{id}", delete(delete_tool))
        .route("/{id}/sync", post(sync_tool))
        .route("/{id}/test", post(test_tool_execution))
}

pub async fn fetch_mcp_capabilities(url: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 1. Initialize
    let init_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "aad-be-container",
                "version": "0.1.0"
            }
        }
    });

    let init_res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&init_payload)
        .send()
        .await
        .map_err(|e| format!("MCP initialize connection failed: {}", e))?;

    if !init_res.status().is_success() {
        return Err(format!("MCP initialize returned HTTP {}", init_res.status()));
    }

    let init_json: serde_json::Value = init_res
        .json()
        .await
        .map_err(|e| format!("MCP initialize invalid JSON response: {}", e))?;

    if let Some(err) = init_json.get("error") {
        return Err(format!("MCP initialize error: {}", err));
    }

    // 2. notifications/initialized
    let notif_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let _ = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&notif_payload)
        .send()
        .await;

    // 3. tools/list
    let list_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let list_res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&list_payload)
        .send()
        .await
        .map_err(|e| format!("MCP tools/list connection failed: {}", e))?;

    if !list_res.status().is_success() {
        return Err(format!("MCP tools/list returned HTTP {}", list_res.status()));
    }

    let list_json: serde_json::Value = list_res
        .json()
        .await
        .map_err(|e| format!("MCP tools/list invalid JSON response: {}", e))?;

    if let Some(err) = list_json.get("error") {
        return Err(format!("MCP tools/list error: {}", err));
    }

    let tools_array = list_json
        .get("result")
        .and_then(|r| r.get("tools"))
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));

    Ok(serde_json::json!({
        "tools": tools_array,
        "resources": [],
        "prompts": []
    }))
}

pub async fn execute_remote_mcp_tool(
    url: &str,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let call_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": Uuid::new_v4().to_string(),
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&call_payload)
        .send()
        .await
        .map_err(|e| format!("MCP tools/call connection failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("MCP tools/call returned HTTP status {}", res.status()));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("MCP tools/call invalid JSON response: {}", e))?;

    if let Some(err) = json.get("error") {
        return Err(format!("MCP tools/call error response: {}", err));
    }

    let result = json
        .get("result")
        .ok_or_else(|| "MCP response missing 'result' field".to_string())?;

    if result.get("isError").and_then(|v| v.as_bool()).unwrap_or(false) {
        let err_text = result
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("Remote tool execution returned error");
        return Err(err_text.to_string());
    }

    let text_output = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("");

    if !text_output.is_empty() {
        Ok(serde_json::json!(text_output))
    } else {
        Ok(result.clone())
    }
}

pub async fn register_tool(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterToolRequest>,
) -> Result<(StatusCode, Json<RegisterToolResponse>), (StatusCode, String)> {
    let server_id = payload.id.unwrap_or_else(Uuid::new_v4);
    tracing::info!("Registering tool server '{}' (ID: {})", payload.server_name, server_id);

    let url = payload
        .endpoint_config
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing endpoint_config.url".to_string()))?;

    let cached_capabilities = if payload.transport_type.eq_ignore_ascii_case("http") {
        match fetch_mcp_capabilities(url).await {
            Ok(caps) => caps,
            Err(err) => {
                tracing::warn!(
                    "Failed to fetch capabilities from MCP server '{}' at '{}': {}",
                    payload.server_name,
                    url,
                    err
                );
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    format!("Failed to connect to MCP server: {}", err),
                ));
            }
        }
    } else {
        serde_json::json!({
            "tools": [],
            "resources": [],
            "prompts": []
        })
    };

    let cached_tools_count = cached_capabilities
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let row = sqlx::query(
        r#"
        INSERT INTO tools (id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at, last_sync_error)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NULL)
        ON CONFLICT (server_name) DO UPDATE 
        SET transport_type = EXCLUDED.transport_type, 
            endpoint_config = EXCLUDED.endpoint_config,
            cached_capabilities = EXCLUDED.cached_capabilities,
            owner_id = EXCLUDED.owner_id,
            sync_policy = EXCLUDED.sync_policy,
            sync_status = EXCLUDED.sync_status,
            last_synced_at = NOW(),
            last_sync_error = NULL
        RETURNING id
        "#,
    )
    .bind(server_id)
    .bind(&payload.server_name)
    .bind(&payload.transport_type)
    .bind(&payload.endpoint_config)
    .bind(&cached_capabilities)
    .bind(payload.owner_id)
    .bind(&payload.sync_policy)
    .bind("synced")
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("MCP Register Error: {}", e)))?;

    let final_id: Uuid = row.get("id");
    tracing::info!(
        "Tool server '{}' registered successfully with {} tools (ID: {})",
        payload.server_name,
        cached_tools_count,
        final_id
    );

    Ok((
        StatusCode::CREATED,
        Json(RegisterToolResponse {
            id: final_id,
            server_name: payload.server_name,
            transport_type: payload.transport_type,
            cached_tools_count,
            sync_status: "synced".to_string(),
            sync_policy: payload.sync_policy,
        }),
    ))
}

pub async fn sync_tool(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<SyncToolResponse>), (StatusCode, String)> {
    tracing::info!("Syncing tool server (ID: {})", id);

    let tool = sqlx::query_as::<_, Tool>(
        r#"
        SELECT id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at, last_sync_error
        FROM tools
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database query error: {}", e)))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Tool server '{}' not found", id)))?;

    let url = tool
        .endpoint_config
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing endpoint_config.url".to_string()))?;

    let now = Utc::now();

    match fetch_mcp_capabilities(url).await {
        Ok(new_caps) => {
            let count = new_caps
                .get("tools")
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            sqlx::query(
                r#"
                UPDATE tools
                SET cached_capabilities = $1,
                    sync_status = 'synced',
                    last_synced_at = $2,
                    last_sync_error = NULL
                WHERE id = $3
                "#
            )
            .bind(&new_caps)
            .bind(now)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update tool: {}", e)))?;

            Ok((
                StatusCode::OK,
                Json(SyncToolResponse {
                    id,
                    server_name: tool.server_name,
                    cached_tools_count: count,
                    sync_status: "synced".to_string(),
                    last_synced_at: now,
                    last_sync_error: None,
                }),
            ))
        }
        Err(err) => {
            tracing::warn!("Sync failed for tool server '{}' ({}): {}", tool.server_name, id, err);

            // Resilient degradation: preserve previous schema, mark degraded
            let existing_count = tool
                .cached_capabilities
                .get("tools")
                .and_then(|t| t.as_array())
                .map(|a| a.len())
                .unwrap_or(0);

            sqlx::query(
                r#"
                UPDATE tools
                SET sync_status = 'degraded',
                    last_synced_at = $1,
                    last_sync_error = $2
                WHERE id = $3
                "#
            )
            .bind(now)
            .bind(&err)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update tool: {}", e)))?;

            Ok((
                StatusCode::OK,
                Json(SyncToolResponse {
                    id,
                    server_name: tool.server_name,
                    cached_tools_count: existing_count,
                    sync_status: "degraded".to_string(),
                    last_synced_at: now,
                    last_sync_error: Some(err),
                }),
            ))
        }
    }
}

pub async fn test_tool_execution(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TestToolRequest>,
) -> Result<(StatusCode, Json<TestToolResponse>), (StatusCode, String)> {
    tracing::info!("Testing tool '{}' on server ID: {}", payload.tool_name, id);

    let tool = sqlx::query_as::<_, Tool>(
        r#"
        SELECT id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at, last_sync_error
        FROM tools
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database query error: {}", e)))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Tool server '{}' not found", id)))?;

    // Verify tool_name exists in cached_capabilities.tools
    let tools_list = tool
        .cached_capabilities
        .get("tools")
        .and_then(|t| t.as_array());

    let tool_found = tools_list
        .map(|arr| {
            arr.iter()
                .any(|t| t.get("name").and_then(|n| n.as_str()) == Some(&payload.tool_name))
        })
        .unwrap_or(false);

    if !tool_found {
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            format!(
                "Tool '{}' not found in server capabilities",
                payload.tool_name
            ),
        ));
    }

    let url = tool
        .endpoint_config
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing endpoint_config.url".to_string()))?;

    let start = std::time::Instant::now();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to build HTTP client: {}", e),
            )
        })?;

    let call_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": Uuid::new_v4().to_string(),
        "method": "tools/call",
        "params": {
            "name": payload.tool_name,
            "arguments": payload.arguments
        }
    });

    let res = client
        .post(url)
        .header("Content-Type", "application/json")
        .json(&call_payload)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Remote MCP server communication failed: {}", e),
            )
        })?;

    let latency_ms = start.elapsed().as_millis() as u64;

    if !res.status().is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            format!("Remote MCP server returned HTTP status {}", res.status()),
        ));
    }

    let json: serde_json::Value = res
        .json()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                format!("Invalid JSON response from MCP server: {}", e),
            )
        })?;

    if let Some(err) = json.get("error") {
        return Ok((
            StatusCode::OK,
            Json(TestToolResponse {
                success: false,
                tool_name: payload.tool_name,
                output: None,
                error: Some(err.to_string()),
                raw_result: json,
                latency_ms,
            }),
        ));
    }

    let result = json
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let is_error = result
        .get("isError")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let text_output = result
        .get("content")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|item| item.get("text"))
        .and_then(|t| t.as_str())
        .map(|s| s.to_string());

    if is_error {
        let err_msg = text_output
            .clone()
            .unwrap_or_else(|| "Remote tool execution failed".to_string());
        Ok((
            StatusCode::OK,
            Json(TestToolResponse {
                success: false,
                tool_name: payload.tool_name,
                output: text_output,
                error: Some(err_msg),
                raw_result: result,
                latency_ms,
            }),
        ))
    } else {
        Ok((
            StatusCode::OK,
            Json(TestToolResponse {
                success: true,
                tool_name: payload.tool_name,
                output: text_output,
                error: None,
                raw_result: result,
                latency_ms,
            }),
        ))
    }
}

pub async fn list_tools(
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<Tool>>), (StatusCode, String)> {
    let servers = sqlx::query_as::<_, Tool>(
        r#"
        SELECT id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at, last_sync_error
        FROM tools
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list MCP servers: {}", e)))?;

    Ok((StatusCode::OK, Json(servers)))
}

pub async fn delete_tool(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Deleting tool server (ID: {})", id);
    sqlx::query("DELETE FROM tools WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete MCP server: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn start_mock_mcp_server() -> (String, tokio::sync::oneshot::Sender<()>) {
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();

        let app = axum::Router::new().route(
            "/mcp",
            axum::routing::post(|axum::Json(payload): axum::Json<serde_json::Value>| async move {
                let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
                let id = payload.get("id").cloned().unwrap_or(json!(1));

                match method {
                    "initialize" => axum::Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": { "tools": {} },
                            "serverInfo": { "name": "mock-mcp", "version": "1.0.0" }
                        }
                    })),
                    "notifications/initialized" => axum::Json(json!({})),
                    "tools/list" => axum::Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "tools": [
                                {
                                    "name": "mock_greeting",
                                    "description": "Mock greeting description",
                                    "inputSchema": {
                                        "type": "object",
                                        "properties": {
                                            "name": { "type": "string" }
                                        },
                                        "required": ["name"]
                                    }
                                }
                            ]
                        }
                    })),
                    "tools/call" => {
                        let tool_name = payload
                            .get("params")
                            .and_then(|p| p.get("name"))
                            .and_then(|n| n.as_str())
                            .unwrap_or("");
                        let name_arg = payload
                            .get("params")
                            .and_then(|p| p.get("arguments"))
                            .and_then(|a| a.get("name"))
                            .and_then(|n| n.as_str());

                        if tool_name == "mock_greeting" && name_arg.is_some() {
                            axum::Json(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [
                                        { "type": "text", "text": format!("Mock hello, {}!", name_arg.unwrap()) }
                                    ],
                                    "isError": false
                                }
                            }))
                        } else {
                            axum::Json(json!({
                                "jsonrpc": "2.0",
                                "id": id,
                                "result": {
                                    "content": [
                                        { "type": "text", "text": "Missing name parameter" }
                                    ],
                                    "isError": true
                                }
                            }))
                        }
                    }
                    _ => axum::Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32601, "message": "Method not found" }
                    })),
                }
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}/mcp", addr);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = rx.await;
                })
                .await
                .unwrap();
        });

        (url, tx)
    }

    #[tokio::test]
    async fn test_fetch_mcp_capabilities_success() {
        let (url, _shutdown) = start_mock_mcp_server().await;
        let res = fetch_mcp_capabilities(&url).await;
        assert!(res.is_ok(), "Expected Ok capabilities, got: {:?}", res);
        let caps = res.unwrap();
        let tools = caps.get("tools").and_then(|t| t.as_array()).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "mock_greeting");
    }

    #[tokio::test]
    async fn test_fetch_mcp_capabilities_unreachable() {
        let res = fetch_mcp_capabilities("http://127.0.0.1:54321/mcp").await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("MCP initialize connection failed"), "Got: {}", err);
    }

    #[tokio::test]
    async fn test_execute_remote_mcp_tool_success() {
        let (url, _shutdown) = start_mock_mcp_server().await;
        let res = execute_remote_mcp_tool(&url, "mock_greeting", json!({"name": "Tester"})).await;
        assert!(res.is_ok(), "Expected Ok tool output, got: {:?}", res);
        assert_eq!(res.unwrap(), json!("Mock hello, Tester!"));
    }

    #[tokio::test]
    async fn test_execute_remote_mcp_tool_error() {
        let (url, _shutdown) = start_mock_mcp_server().await;
        let res = execute_remote_mcp_tool(&url, "mock_greeting", json!({})).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Missing name parameter");
    }

    #[tokio::test]
    async fn test_live_sample_mcp_container_hello_tool() {
        // Verify against the running sample container if available on 8082
        let sample_url = "http://127.0.0.1:8082/mcp";
        if let Ok(caps) = fetch_mcp_capabilities(sample_url).await {
            let tools = caps.get("tools").and_then(|t| t.as_array()).unwrap();
            assert!(tools.iter().any(|t| t["name"] == "hello"));

            let exec_res = execute_remote_mcp_tool(sample_url, "hello", json!({"name": "Antigravity"})).await;
            assert!(exec_res.is_ok());
            assert_eq!(exec_res.unwrap(), json!("Hello, Antigravity!"));
        } else {
            // Live container not running; skip live assertion gracefully
            println!("Note: Live sample MCP container not running on 8082, skipping live test");
        }
    }

    #[tokio::test]
    async fn test_tool_verification_handler() {
        let db_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:mysecretpassword@localhost:5432/aaddb".to_string());
        if let Ok(pool) = sqlx::postgres::PgPoolOptions::new().connect(&db_url).await {
            let (mock_url, _shutdown) = start_mock_mcp_server().await;
            let server_name = format!("test-mcp-verify-{}", Uuid::new_v4());
            let server_id = Uuid::new_v4();

            // Insert tool
            let insert_res = sqlx::query(
                r#"
                INSERT INTO tools (id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at)
                VALUES ($1, $2, 'http', $3, $4, '00000000-0000-0000-0000-000000000001', 'manual', 'synced', NOW())
                "#
            )
            .bind(server_id)
            .bind(&server_name)
            .bind(json!({"url": mock_url}))
            .bind(json!({"tools": [{"name": "mock_greeting"}]}))
            .execute(&pool)
            .await;

            if insert_res.is_err() {
                println!("Note: Database not migrated or tools schema missing, skipping test_tool_verification_handler");
                return;
            }

            // 1. Successful verification
            let req = TestToolRequest {
                tool_name: "mock_greeting".to_string(),
                arguments: json!({"name": "Agent"}),
            };
            let res = test_tool_execution(axum::extract::State(pool.clone()), axum::extract::Path(server_id), axum::Json(req.clone())).await;
            assert!(res.is_ok());
            let (status, axum::Json(test_res)) = res.unwrap();
            assert_eq!(status, StatusCode::OK);
            assert!(test_res.success);
            assert_eq!(test_res.output, Some("Mock hello, Agent!".to_string()));

            // 2. Unknown tool fails fast with 422
            let unknown_req = TestToolRequest {
                tool_name: "non_existent".to_string(),
                arguments: json!({}),
            };
            let unk_res = test_tool_execution(axum::extract::State(pool.clone()), axum::extract::Path(server_id), axum::Json(unknown_req)).await;
            assert!(unk_res.is_err());
            let (unk_status, unk_msg) = unk_res.unwrap_err();
            assert_eq!(unk_status, StatusCode::UNPROCESSABLE_ENTITY);
            assert!(unk_msg.contains("not found in server capabilities"));

            // 3. Unknown server fails with 404
            let missing_server_id = Uuid::new_v4();
            let missing_res = test_tool_execution(axum::extract::State(pool.clone()), axum::extract::Path(missing_server_id), axum::Json(req)).await;
            assert!(missing_res.is_err());
            let (missing_status, _) = missing_res.unwrap_err();
            assert_eq!(missing_status, StatusCode::NOT_FOUND);

            // Cleanup
            let _ = sqlx::query("DELETE FROM tools WHERE id = $1").bind(server_id).execute(&pool).await;
        }
    }
}


