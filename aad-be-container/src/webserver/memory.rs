use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    models::{AppendDecisionRequest, BenchMemory, UpsertBenchMemoryRequest},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{id}/memory", get(get_bench_memory).put(upsert_working_memory))
        .route("/{id}/memory/decision", post(append_decision))
}

pub async fn get_bench_memory(
    State(state): State<AppState>,
    Path(bench_id): Path<Uuid>,
) -> Result<Json<Vec<BenchMemory>>, (StatusCode, String)> {
    let records = sqlx::query_as::<_, BenchMemory>(
        "SELECT * FROM bench_memory WHERE bench_id = $1 ORDER BY memory_type ASC, updated_at DESC"
    )
    .bind(bench_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get bench memory: {}", e)))?;

    Ok(Json(records))
}

pub async fn upsert_working_memory(
    State(state): State<AppState>,
    Path(bench_id): Path<Uuid>,
    Json(payload): Json<UpsertBenchMemoryRequest>,
) -> Result<Json<BenchMemory>, (StatusCode, String)> {
    tracing::info!("Upserting working memory for bench {}", bench_id);

    let title = payload.title.unwrap_or_else(|| "Active Working Memory".to_string());
    let metadata_json = payload.metadata.map(sqlx::types::Json);

    let memory = sqlx::query_as::<_, BenchMemory>(
        "INSERT INTO bench_memory (bench_id, memory_type, title, content, metadata) 
         VALUES ($1, 'working', $2, $3, $4)
         ON CONFLICT (bench_id) WHERE memory_type = 'working'
         DO UPDATE SET 
             content = EXCLUDED.content, 
             title = EXCLUDED.title,
             metadata = COALESCE(EXCLUDED.metadata, bench_memory.metadata),
             updated_at = NOW()
         RETURNING *"
    )
    .bind(bench_id)
    .bind(title)
    .bind(&payload.content)
    .bind(metadata_json)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to upsert working memory: {}", e)))?;

    Ok(Json(memory))
}

pub async fn append_decision(
    State(state): State<AppState>,
    Path(bench_id): Path<Uuid>,
    Json(payload): Json<AppendDecisionRequest>,
) -> Result<(StatusCode, Json<BenchMemory>), (StatusCode, String)> {
    tracing::info!("Appending decision for bench {}: {}", bench_id, payload.title);

    let metadata_val = serde_json::json!({
        "thread_id": payload.thread_id
    });

    let decision = sqlx::query_as::<_, BenchMemory>(
        "INSERT INTO bench_memory (bench_id, memory_type, title, content, metadata) 
         VALUES ($1, 'episodic', $2, $3, $4) 
         RETURNING *"
    )
    .bind(bench_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(sqlx::types::Json(metadata_val))
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to append decision: {}", e)))?;

    Ok((StatusCode::CREATED, Json(decision)))
}
