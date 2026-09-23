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
    Router::new().route("/", post(semantic_search))
}

pub async fn semantic_search(
    State(pool): State<PgPool>,
    Json(payload): Json<SemanticSearchRequest>,
) -> Result<Json<Vec<SemanticSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(5) as i64;
    let query_trimmed = payload.query.trim();
    if query_trimmed.is_empty() {
        return Ok(Json(vec![]));
    }
    let pattern = format!("%{}%", query_trimmed);

    let query = r#"
        WITH matches AS (
            SELECT
                e.entity_id,
                e.entity_type,
                e.field_name,
                e.content,
                COALESCE(a.name, s.name, t.server_name, tr.name) as name,
                COALESCE(a.description, s.description, tr.description) as description,
                (CASE
                    WHEN e.content ILIKE $1 THEN 0.98::float8
                    ELSE LEAST(0.96::float8, GREATEST(0.60::float8, (0.65 + (ts_rank_cd(to_tsvector('english', e.content), NULLIF(replace(plainto_tsquery('english', $2)::text, '&', '|'), '')::tsquery) * 1.5))::float8))
                END)::float8 as score
            FROM entity_embeddings e
            LEFT JOIN agents a ON e.entity_type IN ('agents', 'agent') AND e.entity_id = a.id
            LEFT JOIN skills s ON e.entity_type IN ('skills', 'skill') AND e.entity_id = s.id
            LEFT JOIN tools t ON e.entity_type IN ('tools', 'tool') AND e.entity_id = t.id
            LEFT JOIN trait_contracts tr ON e.entity_type IN ('traits', 'trait') AND e.entity_id = tr.id
            WHERE e.content ILIKE $1
               OR (
                   to_tsvector('english', e.content) @@ NULLIF(replace(plainto_tsquery('english', $2)::text, '&', '|'), '')::tsquery
               )
        ),
        deduped AS (
            SELECT DISTINCT ON (COALESCE(name, entity_id::text))
                entity_id,
                entity_type,
                field_name,
                content,
                name,
                description,
                score
            FROM matches
            ORDER BY COALESCE(name, entity_id::text), score DESC
        )
        SELECT * FROM deduped
        ORDER BY score DESC
        LIMIT $3
    "#;

    let rows = sqlx::query(query)
        .bind(pattern)
        .bind(query_trimmed)
        .bind(limit)
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Semantic Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| {
            let field_name: String = r.get("field_name");
            let match_reason = match field_name.as_str() {
                "name" => "Matched on entity name".to_string(),
                "description" => "Matched on entity description".to_string(),
                "prompt" => "Matched on agent prompt".to_string(),
                "definition" => "Matched on skill definition".to_string(),
                other => format!("Matched on {}", other),
            };

            SemanticSearchResult {
                entity_id: r.get("entity_id"),
                entity_type: r.get("entity_type"),
                name: r.try_get("name").ok(),
                description: r.try_get("description").ok(),
                field_name,
                content: r.get("content"),
                score: r.get("score"),
                match_reason,
                search_type: "hybrid".to_string(),
            }
        })
        .collect();

    Ok(Json(results))
}
