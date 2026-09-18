use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::get,
    Router,
};
use sqlx::{PgPool, Row};

use crate::{
    models::{AgentUsageLog, AnalyticsUsageResponse},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/usage", get(get_usage))
}

pub async fn get_usage(
    State(pool): State<PgPool>,
) -> Result<Json<AnalyticsUsageResponse>, (StatusCode, String)> {
    let rows = sqlx::query(
        r#"
        SELECT
            id,
            agent_id,
            agent_version,
            caller_identity,
            tool_calls,
            token_metrics,
            guardrail_events,
            created_at
        FROM agent_usage_logs
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    let usage_logs: Vec<AgentUsageLog> = rows
        .into_iter()
        .map(|r| AgentUsageLog {
            id: r.get("id"),
            agent_id: r.get("agent_id"),
            agent_version: r.get("agent_version"),
            caller_identity: r.get("caller_identity"),
            tool_calls: r.get("tool_calls"),
            token_metrics: r.get("token_metrics"),
            guardrail_events: r.get("guardrail_events"),
            created_at: r.get("created_at"),
        })
        .collect();

    Ok(Json(AnalyticsUsageResponse { usage_logs }))
}
