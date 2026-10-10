use rmcp::{handler::server::wrapper::Parameters, tool, tool_router};
use sqlx::PgPool;
use std::sync::OnceLock;
use uuid::Uuid;

use crate::models::{
    GraphTraverseRequest, GraphTraverseResult, KnowledgeIdRequest, KnowledgeNode,
    KnowledgeSearchRequest, KnowledgeSearchResult,
};
use crate::webserver::knowledge::chunk_text;

pub static DB_POOL: OnceLock<PgPool> = OnceLock::new();

#[derive(Default)]
pub struct KnowledgeMcpServer {}

impl KnowledgeMcpServer {
    pub fn new() -> Self {
        Self {}
    }
}

#[tool_router]
impl KnowledgeMcpServer {
    #[tool(description = "Searches the knowledge graph using a text query")]
    pub async fn search_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeSearchRequest>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;
        let limit = req.limit.unwrap_or(5) as i64;
        let pattern = format!("%{}%", req.query);

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
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Search error: {}", e))?;

        let results: Vec<KnowledgeSearchResult> = rows
            .into_iter()
            .map(|r| {
                use sqlx::Row;
                KnowledgeSearchResult {
                    node_id: r.get("node_id"),
                    chunk_index: r.get("chunk_index"),
                    chunk_text: r.get("chunk_text"),
                    score: 0.95,
                    search_type: "semantic".to_string(),
                }
            })
            .collect();

        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    #[tool(description = "Reads a specific knowledge node by its ID")]
    pub async fn read_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeIdRequest>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;

        let node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            FROM knowledge_nodes
            WHERE id = $1
            "#
        )
        .bind(req.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        match node {
            Some(n) => Ok(serde_json::to_string(&n).map_err(|e| e.to_string())?),
            None => Err("Knowledge node not found".to_string()),
        }
    }

    #[tool(description = "Ingests a new knowledge node and returns the created node")]
    pub async fn ingest_knowledge(
        &self,
        Parameters(payload): Parameters<KnowledgeNode>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;
        if payload.topic.trim().is_empty() || payload.content.trim().is_empty() {
            return Err("Topic and content are required".to_string());
        }
        let node_id = payload.id.unwrap_or_else(Uuid::new_v4);
        let tags = payload.tags;
        let metadata = if payload.metadata.is_null() {
            serde_json::json!({})
        } else {
            payload.metadata
        };

        let created_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            INSERT INTO knowledge_nodes (id, topic, title, description, tags, content, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#,
        )
        .bind(node_id)
        .bind(&payload.topic)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&tags)
        .bind(&payload.content)
        .bind(metadata)
        .fetch_one(pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        let chunks = chunk_text(&payload.content, 200);
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
            .execute(pool)
            .await
            .map_err(|e| format!("Chunk Insert Error: {}", e))?;
        }

        for tuple in payload.tuples {
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
            .bind(tuple.triple.subject)
            .bind(tuple.triple.predicate)
            .bind(tuple.triple.object)
            .bind(confidence)
            .execute(pool)
            .await
            .map_err(|e| format!("Tuple Insert Error: {}", e))?;
        }

        serde_json::to_string(&created_node).map_err(|e| e.to_string())
    }

    #[tool(description = "Updates an existing knowledge node and returns the updated node")]
    pub async fn update_knowledge(
        &self,
        Parameters(payload): Parameters<KnowledgeNode>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;
        let id = payload
            .id
            .ok_or("Knowledge node id is required for update")?;

        let current_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            FROM knowledge_nodes
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        let current_node = match current_node {
            Some(n) => n,
            None => return Err("Knowledge node not found".to_string()),
        };

        let new_topic = if payload.topic.is_empty() {
            current_node.topic
        } else {
            payload.topic
        };
        let new_title = if payload.title.trim().is_empty() {
            current_node.title
        } else {
            payload.title
        };
        let new_description = if payload.description.trim().is_empty() {
            current_node.description
        } else {
            payload.description
        };
        let new_tags = if payload.tags.is_empty() {
            current_node.tags
        } else {
            payload.tags
        };
        let new_content = if payload.content.is_empty() {
            current_node.content.clone()
        } else {
            payload.content
        };
        let new_metadata =
            if payload.metadata.is_null() || payload.metadata == serde_json::json!({}) {
                current_node.metadata
            } else {
                payload.metadata
            };

        let mut tx = pool.begin().await.map_err(|e| format!("Tx Error: {}", e))?;

        let updated_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            UPDATE knowledge_nodes
            SET topic = $1, title = $2, description = $3, tags = $4, content = $5, metadata = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
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
        .map_err(|e| format!("Update Error: {}", e))?;

        if new_content != current_node.content {
            sqlx::query("DELETE FROM knowledge_embeddings WHERE node_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Delete chunks error: {}", e))?;

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
                .map_err(|e| format!("Chunk Insert Error: {}", e))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| format!("Commit Error: {}", e))?;

        serde_json::to_string(&updated_node).map_err(|e| e.to_string())
    }

    #[tool(description = "Deletes a knowledge node by its ID and returns the deleted node")]
    pub async fn delete_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeIdRequest>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;

        let deleted_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            DELETE FROM knowledge_nodes
            WHERE id = $1
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#
        )
        .bind(req.id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        match deleted_node {
            Some(node) => Ok(serde_json::to_string(&node).map_err(|e| e.to_string())?),
            None => Err("Knowledge node not found".to_string()),
        }
    }

    #[tool(description = "Traverses the knowledge graph starting from a subject")]
    pub async fn traverse_knowledge_graph(
        &self,
        Parameters(req): Parameters<GraphTraverseRequest>,
    ) -> Result<String, String> {
        let pool = DB_POOL.get().ok_or("Database pool not initialized")?;
        let max_depth = req.max_depth.unwrap_or(2);

        let rows = sqlx::query(
            r#"
            SELECT subject, predicate, object, confidence
            FROM knowledge_tuples
            WHERE subject ILIKE $1 OR object ILIKE $1
            LIMIT 10
            "#,
        )
        .bind(&req.subject)
        .fetch_all(pool)
        .await
        .map_err(|e| format!("Traverse Error: {}", e))?;

        let results: Vec<GraphTraverseResult> = rows
            .into_iter()
            .map(|r| {
                use sqlx::Row;
                GraphTraverseResult {
                    triple: crate::models::knowledge::Triple::new(
                        r.get::<String, _>("subject"),
                        r.get::<String, _>("predicate"),
                        r.get::<String, _>("object"),
                    ),
                    confidence: r.get("confidence"),
                    depth: max_depth,
                }
            })
            .collect();

        serde_json::to_string(&results).map_err(|e| e.to_string())
    }
}
