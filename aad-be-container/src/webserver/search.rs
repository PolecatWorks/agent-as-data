use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::post,
    Router,
};
use sqlx::{PgPool, Row};

use crate::{
    models::{SemanticSearchRequest, SemanticSearchResult},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/semantic", post(semantic_search))
}

pub async fn semantic_search(
    State(pool): State<PgPool>,
    Json(payload): Json<SemanticSearchRequest>,
) -> Result<Json<Vec<SemanticSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(10) as i64;

    // In a real implementation, we would call an embedding model here.
    // Since sqlx requires pgvector type bindings to use vector types directly and we don't have it,
    // we'll cast a string literal to vector.
    let mock_embedding_str = format!("[{}]", vec!["0.1"; 1536].join(","));

    let query = r#"
        WITH vector_results AS (
            SELECT
                entity_id,
                entity_type,
                1.0 - (embedding <=> $1::vector) as similarity_score
            FROM entity_embeddings
            ORDER BY embedding <=> $1::vector
            LIMIT $2
        )
        SELECT
            v.entity_id as id,
            v.entity_type,
            v.similarity_score,
            COALESCE(a.name, s.name, t.name, tr.name) as name,
            COALESCE(a.description, s.description, t.description, tr.description) as description,
            COALESCE(a.tags, s.tags, t.tags, tr.tags, '{}'::text[]) as tags
        FROM vector_results v
        LEFT JOIN agents a ON v.entity_type = 'agents' AND v.entity_id = a.id
        LEFT JOIN skills s ON v.entity_type = 'skills' AND v.entity_id = s.id
        LEFT JOIN tools t ON v.entity_type = 'tools' AND v.entity_id = t.id
        LEFT JOIN traits tr ON v.entity_type = 'traits' AND v.entity_id = tr.id
        ORDER BY v.similarity_score DESC
    "#;

    let rows = sqlx::query(query)
        .bind(mock_embedding_str)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Semantic Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|row| SemanticSearchResult {
            id: row.get("id"),
            entity_type: row.get("entity_type"),
            name: row.get("name"),
            description: row.get("description"),
            tags: row.get("tags"),
            similarity_score: row.get("similarity_score"),
        })
        .collect();

    Ok(Json(results))
}
