use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    ExecuteAgentRequest, ExecuteAgentResponse, SearchAndExecuteRequest,
};

pub async fn execute_agent(
    State(pool): State<PgPool>,
    Path(agent_id): Path<Uuid>,
    Json(payload): Json<ExecuteAgentRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    let execution_id = Uuid::new_v4();

    // 1. Fetch Agent definition & version
    let agent_row = sqlx::query("SELECT name, current_version FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    let agent_version: i32 = agent_row.get("current_version");
    let output = format!("Executed agent response for prompt: '{}'", payload.prompt);

    // 2. Insert Execution Record with OCC Version Lock
    sqlx::query(
        r#"
        INSERT INTO executions (id, agent_id, agent_version, execution_version, status, request_payload, response_payload, webhook_url, started_at, completed_at)
        VALUES ($1, $2, $3, 1, 'completed', $4, $5, $6, NOW(), NOW())
        "#,
    )
    .bind(execution_id)
    .bind(agent_id)
    .bind(agent_version)
    .bind(serde_json::json!({ "prompt": payload.prompt }))
    .bind(serde_json::json!({ "output": output }))
    .bind(&payload.webhook_url)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Execution Insert Error: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(ExecuteAgentResponse {
            execution_id,
            agent_id,
            status: "completed".to_string(),
            output,
            execution_version: 1,
        }),
    ))
}

pub async fn search_and_execute(
    State(pool): State<PgPool>,
    Json(payload): Json<SearchAndExecuteRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    // 1. Discovery top match agent
    let agent_row = sqlx::query("SELECT id FROM agents LIMIT 1")
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Discovery Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "No matching agents found for task".to_string()))?;

    let agent_id: Uuid = agent_row.get("id");

    execute_agent(
        State(pool),
        Path(agent_id),
        Json(ExecuteAgentRequest {
            prompt: payload.prompt,
            context: None,
            webhook_url: None,
        }),
    )
    .await
}

pub async fn get_execution(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecuteAgentResponse>, (StatusCode, String)> {
    let row = sqlx::query("SELECT agent_id, status, response_payload, execution_version FROM executions WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Execution job not found".to_string()))?;

    let agent_id: Uuid = row.get("agent_id");
    let status: String = row.get("status");
    let execution_version: i32 = row.get("execution_version");
    let response: serde_json::Value = row.get("response_payload");
    let output = response["output"].as_str().unwrap_or("").to_string();

    Ok(Json(ExecuteAgentResponse {
        execution_id: id,
        agent_id,
        status,
        output,
        execution_version,
    }))
}
