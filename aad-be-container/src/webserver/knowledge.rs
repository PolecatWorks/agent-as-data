use axum::{
    Json,
    extract::{State, Path},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    models::{
        GraphTraverseRequest, GraphTraverseResult, IngestKnowledgeRequest, IngestKnowledgeResponse,
        KnowledgeNode, KnowledgeSearchRequest, KnowledgeSearchResult, UpdateKnowledgeRequest,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_knowledge).post(ingest_knowledge))
        .route("/search", post(search_knowledge))
        .route("/graph/traverse", post(traverse_graph))
        .route("/{id}", get(get_knowledge).put(update_knowledge).delete(delete_knowledge))
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
    metrics::counter!("knowledge_ingestion_total").increment(1);

    tracing::info!("Ingesting/Saving knowledge node '{:?}' (topic: '{}', ID: {})", payload.title, payload.topic, node_id);
    let metadata = payload.metadata.unwrap_or_else(|| serde_json::json!({}));

    let tags = payload.tags.unwrap_or_default();

    // 1. Insert Node
    sqlx::query(
        r#"
        INSERT INTO knowledge_nodes (id, topic, title, description, tags, content, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(node_id)
    .bind(&payload.topic)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&tags)
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

pub async fn list_knowledge(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<KnowledgeNode>>, (StatusCode, String)> {
    let nodes = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    Ok(Json(nodes))
}

pub async fn get_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<KnowledgeNode>, (StatusCode, String)> {
    let node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    match node {
        Some(n) => Ok(Json(n)),
        None => Err((StatusCode::NOT_FOUND, "Knowledge node not found".to_string())),
    }
}

pub async fn update_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateKnowledgeRequest>,
) -> Result<Json<KnowledgeNode>, (StatusCode, String)> {
    let current_node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    let current_node = match current_node {
        Some(n) => n,
        None => return Err((StatusCode::NOT_FOUND, "Knowledge node not found".to_string())),
    };

    let new_topic = payload.topic.unwrap_or(current_node.topic);
    let new_title = payload.title.or(current_node.title);
    let new_description = payload.description.or(current_node.description);
    let new_tags = payload.tags.unwrap_or(current_node.tags);
    let new_content = payload.content.unwrap_or(current_node.content.clone());
    let new_metadata = payload.metadata.unwrap_or(current_node.metadata);

    let mut tx = pool.begin().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Tx Error: {}", e)))?;

    let updated_node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        UPDATE knowledge_nodes
        SET topic = $1, title = $2, description = $3, tags = $4, content = $5, metadata = $6, updated_at = NOW()
        WHERE id = $7
        RETURNING id, topic, title, description, tags, content, metadata, created_at, updated_at
        "#
    )
    .bind(new_topic)
    .bind(new_title)
    .bind(new_description)
    .bind(&new_tags)
    .bind(new_content.clone())
    .bind(new_metadata)
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Error: {}", e)))?;

    // If content changed, we should ideally re-chunk and update embeddings.
    // Here we'll just delete old and insert new chunks as a simple strategy.
    if new_content != current_node.content {
        // Cascade delete tuples associated with the content
        sqlx::query("DELETE FROM knowledge_tuples WHERE source_node_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete tuples error: {}", e)))?;

        sqlx::query("DELETE FROM knowledge_embeddings WHERE node_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete chunks error: {}", e)))?;

        let chunks = chunk_text(&new_content, 200);
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(chunk_id)
            .bind(id)
            .bind(idx as i32)
            .bind(chunk)
            .execute(&mut *tx)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chunk Insert Error: {}", e)))?;
        }
    }

    tx.commit().await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Commit Error: {}", e)))?;

    Ok(Json(updated_node))
}


pub async fn search_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<KnowledgeSearchRequest>,
) -> Result<Json<Vec<KnowledgeSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(5) as i64;

    // Mock incoming embedding string
    let mock_embedding_str = format!("[{}]", vec!["0.1"; 1536].join(","));

    let rows = sqlx::query(
        r#"
        SELECT node_id, chunk_index, chunk_text,
               1.0 - (embedding <=> $1::vector) as similarity_score
        FROM knowledge_embeddings
        ORDER BY embedding <=> $1::vector
        LIMIT $2
        "#,
    )
    .bind(mock_embedding_str)
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
            score: r.get("similarity_score"),
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


pub async fn delete_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Deleting knowledge node: {}", id);

    let result = sqlx::query("DELETE FROM knowledge_nodes WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Knowledge node not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
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

    #[test]
    fn test_knowledge_router_construction() {
        let _r: Router<AppState> = router();
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_chunk_text_again() {
        let text = "1234567890";
        let chunks = chunk_text(text, 4);
        assert_eq!(chunks, vec!["1234", "5678", "90"]);
    }
}
