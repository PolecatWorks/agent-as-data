use axum::{Json, extract::State, http::StatusCode};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    GraphTraverseRequest, GraphTraverseResult, IngestKnowledgeRequest, IngestKnowledgeResponse,
    KnowledgeSearchRequest, KnowledgeSearchResult,
};

pub async fn ingest_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<IngestKnowledgeRequest>,
) -> Result<(StatusCode, Json<IngestKnowledgeResponse>), (StatusCode, String)> {
    let node_id = Uuid::new_v4();
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
                INSERT INTO knowledge_tuples (id, subject, subject_canonical, predicate, object, object_canonical, confidence, source_node_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(tuple_id)
            .bind(&tuple.subject)
            .bind(tuple.subject.to_lowercase())
            .bind(&tuple.predicate)
            .bind(&tuple.object)
            .bind(tuple.object.to_lowercase())
            .bind(confidence)
            .bind(node_id)
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
    let subject_canonical = payload.subject.to_lowercase();

    let rows = sqlx::query(
        r#"
        SELECT subject, predicate, object, confidence
        FROM knowledge_tuples
        WHERE LOWER(subject) = $1 OR LOWER(subject_canonical) = $1
        "#,
    )
    .bind(subject_canonical)
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
            depth: 1,
        })
        .collect();

    Ok(Json(results))
}

fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }
    text.chars()
        .collect::<Vec<char>>()
        .chunks(max_len)
        .map(|c| c.iter().collect())
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text() {
        let text = "Hello world from Agent-As-Data Knowledge Engine";
        let chunks = chunk_text(text, 10);
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0], "Hello worl");
    }
}
