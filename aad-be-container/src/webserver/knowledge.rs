use axum::{
    Json,
    extract::State,
    http::StatusCode,
    routing::post,
    Router,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    models::{
        GraphTraverseRequest, GraphTraverseResult, IngestKnowledgeRequest, IngestKnowledgeResponse,
        KnowledgeSearchRequest, KnowledgeSearchResult,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(ingest_knowledge))
        .route("/search", post(search_knowledge))
        .route("/graph/traverse", post(traverse_graph))
}

pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect::<String>())
        .collect()
}

pub async fn ingest_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<IngestKnowledgeRequest>,
) -> Result<(StatusCode, Json<IngestKnowledgeResponse>), (StatusCode, String)> {
    let node_id = Uuid::new_v4();
    tracing::info!("Ingesting/Saving knowledge node '{:?}' (topic: '{}', ID: {})", payload.title, payload.topic, node_id);
    let metadata = payload.metadata.unwrap_or_else(|| serde_json::json!({}));

    // 1. Insert Node
    sqlx::query(
        r#"
        INSERT INTO knowledge_nodes (id, topic, title, content, metadata)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(node_id)
    .bind(&payload.topic)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(metadata)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    // 2. Chunk text and store mock vector embeddings
    let chunks = chunk_text(&payload.content, 200);
    let chunks_created = chunks.len();

    for (idx, chunk) in chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(chunk_id)
        .bind(node_id)
        .bind(idx as i32)
        .bind(chunk)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chunk Insert Error: {}", e)))?;
    }

    // 3. Insert Tuples if provided
    let mut tuples_created = 0;
    if let Some(tuples) = payload.tuples {
        tuples_created = tuples.len();
        for tuple in tuples {
            let tuple_id = Uuid::new_v4();
            let confidence = tuple.confidence.unwrap_or(1.0);
            sqlx::query(
                r#"
                INSERT INTO knowledge_tuples (id, source_node_id, subject, predicate, object, confidence)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(tuple_id)
            .bind(node_id)
            .bind(tuple.subject)
            .bind(tuple.predicate)
            .bind(tuple.object)
            .bind(confidence)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tuple Insert Error: {}", e)))?;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(IngestKnowledgeResponse {
            id: node_id,
            topic: payload.topic,
            chunks_created,
            tuples_created,
        }),
    ))
}

pub async fn search_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<KnowledgeSearchRequest>,
) -> Result<Json<Vec<KnowledgeSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(5) as i64;
    let pattern = format!("%{}%", payload.query);

    let rows = sqlx::query(
        r#"
        SELECT node_id, chunk_index, chunk_text
        FROM knowledge_embeddings
        WHERE chunk_text ILIKE $1
        LIMIT $2
        "#,
    )
    .bind(pattern)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| KnowledgeSearchResult {
            node_id: r.get("node_id"),
            chunk_index: r.get("chunk_index"),
            chunk_text: r.get("chunk_text"),
            score: 0.95,
        })
        .collect();

    Ok(Json(results))
}

pub async fn traverse_graph(
    State(pool): State<PgPool>,
    Json(payload): Json<GraphTraverseRequest>,
) -> Result<Json<Vec<GraphTraverseResult>>, (StatusCode, String)> {
    let max_depth = payload.max_depth.unwrap_or(2);

    let rows = sqlx::query(
        r#"
        SELECT subject, predicate, object, confidence
        FROM knowledge_tuples
        WHERE subject ILIKE $1 OR object ILIKE $1
        LIMIT 10
        "#,
    )
    .bind(&payload.subject)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Traverse Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| GraphTraverseResult {
            subject: r.get("subject"),
            predicate: r.get("predicate"),
            object: r.get("object"),
            confidence: r.get("confidence"),
            depth: max_depth,
        })
        .collect();

    Ok(Json(results))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = "abcdefghij";
        let chunks = chunk_text(text, 3);
        assert_eq!(chunks, vec!["abc", "def", "ghi", "j"]);
    }
}
