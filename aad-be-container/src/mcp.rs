use axum::{Json, extract::State, http::StatusCode};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{RegisterMcpServerRequest, RegisterMcpServerResponse};

pub async fn register_mcp_server(
    State(pool): State<PgPool>,
    Json(payload): Json<RegisterMcpServerRequest>,
) -> Result<(StatusCode, Json<RegisterMcpServerResponse>), (StatusCode, String)> {
    let server_id = Uuid::new_v4();

    // Mock cached tools capability discovery
    let cached_capabilities = serde_json::json!({
        "tools": [
            { "name": "search_agents", "description": "RAG search for matching agents" },
            { "name": "execute_agent", "description": "Run agent with payload" }
        ],
        "resources": [],
        "prompts": []
    });

    sqlx::query(
        r#"
        INSERT INTO mcp_servers (id, server_name, transport_type, endpoint_config, cached_capabilities)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(server_id)
    .bind(&payload.server_name)
    .bind(&payload.transport_type)
    .bind(&payload.endpoint_config)
    .bind(&cached_capabilities)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("MCP Register Error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterMcpServerResponse {
            id: server_id,
            server_name: payload.server_name,
            transport_type: payload.transport_type,
            cached_tools_count: 2,
        }),
    ))
}
