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

    use sqlx::Row;
    let row = sqlx::query(
        r#"
        INSERT INTO mcp_servers (id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (server_name) DO UPDATE 
        SET transport_type = EXCLUDED.transport_type, 
            endpoint_config = EXCLUDED.endpoint_config,
            owner_id = EXCLUDED.owner_id,
            last_synced_at = NOW()
        RETURNING id
        "#,
    )
    .bind(server_id)
    .bind(&payload.server_name)
    .bind(&payload.transport_type)
    .bind(&payload.endpoint_config)
    .bind(&cached_capabilities)
    .bind(&payload.owner_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("MCP Register Error: {}", e)))?;

    let final_id: Uuid = row.get("id");

    Ok((
        StatusCode::CREATED,
        Json(RegisterMcpServerResponse {
            id: final_id,
            server_name: payload.server_name,
            transport_type: payload.transport_type,
            cached_tools_count: 2,
        }),
    ))
}

pub async fn list_mcp_servers(
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<crate::models::McpServer>>), (StatusCode, String)> {
    let servers = sqlx::query_as::<_, crate::models::McpServer>(
        "SELECT id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id FROM mcp_servers"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list MCP servers: {}", e)))?;

    Ok((StatusCode::OK, Json(servers)))
}

use axum::extract::Path;

pub async fn delete_mcp_server(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query("DELETE FROM mcp_servers WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete MCP server: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
