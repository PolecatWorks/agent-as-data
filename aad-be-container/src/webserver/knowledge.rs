use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;

use crate::{
    models::{
        AnalyzeMarkdownRequest, AnalyzeMarkdownResponse, GraphTraverseRequest, GraphTraverseResult,
        ImportDocumentRequest, ImportDocumentResponse, KnowledgeNode, KnowledgeNodeProposal,
        KnowledgeSearchRequest, KnowledgeSearchResult, KnowledgeTuple, Triple,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_knowledge).post(ingest_knowledge))
        .route("/search", post(search_knowledge))
        .route("/graph/traverse", post(traverse_graph))
        .route("/analyze-markdown", post(analyze_markdown))
        .route("/import-document", post(import_document))
        .route(
            "/{id}",
            get(get_knowledge)
                .put(update_knowledge)
                .delete(delete_knowledge),
        )
        .route("/{id}/tuples", get(get_knowledge_tuples))
        .route("/{id}/derived-concepts", get(get_derived_concepts))
}

pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    text.chars()
        .collect::<Vec<char>>()
        .chunks(chunk_size)
        .map(|c| c.iter().collect::<String>())
        .collect()
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct LlmRawAnalysisOutput {
    #[serde(default)]
    document: Option<LlmDocumentMetadata>,
    #[serde(default)]
    proposals: Vec<KnowledgeNodeProposal>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct LlmDocumentMetadata {
    topic: Option<String>,
    title: Option<String>,
    description: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

pub fn extract_markdown_metadata(
    markdown: &str,
    suggested_topic: Option<&str>,
) -> KnowledgeNodeProposal {
    let mut title = String::new();
    let mut description = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && title.is_empty() {
            title = trimmed.trim_start_matches("# ").trim().to_string();
        } else if trimmed.starts_with("## ") && title.is_empty() {
            title = trimmed.trim_start_matches("## ").trim().to_string();
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') && description.is_empty() {
            description = trimmed.chars().take(300).collect();
        }
    }

    if title.is_empty() {
        title = "Imported Document".to_string();
    }
    if description.is_empty() {
        description = format!("Imported markdown document: {}", title);
    }

    let topic = suggested_topic.unwrap_or("document").to_string();
    let tags = vec!["document".to_string(), "imported".to_string()];

    KnowledgeNodeProposal {
        topic,
        title,
        description,
        tags,
        content: markdown.to_string(),
    }
}

pub fn parse_llm_markdown_analysis(
    content: &str,
    markdown: &str,
    suggested_topic: Option<&str>,
) -> Result<AnalyzeMarkdownResponse, String> {
    let fallback_doc = extract_markdown_metadata(markdown, suggested_topic);

    let parsed_val: serde_json::Value =
        serde_json::from_str(content).map_err(|e| format!("Invalid JSON: {}", e))?;

    if parsed_val.is_array() {
        let proposals: Vec<KnowledgeNodeProposal> = serde_json::from_value(parsed_val)
            .map_err(|e| format!("Failed to parse proposals array: {}", e))?;
        return Ok(AnalyzeMarkdownResponse {
            document: Some(fallback_doc),
            proposals,
        });
    }

    if parsed_val.is_object() {
        let structured: LlmRawAnalysisOutput = serde_json::from_value(parsed_val)
            .map_err(|e| format!("Failed to parse structured markdown analysis: {}", e))?;

        let doc = if let Some(meta) = structured.document {
            KnowledgeNodeProposal {
                topic: meta.topic.unwrap_or(fallback_doc.topic),
                title: meta.title.unwrap_or(fallback_doc.title),
                description: meta.description.unwrap_or(fallback_doc.description),
                tags: if let Some(t) = meta.tags {
                    if t.is_empty() { fallback_doc.tags } else { t }
                } else {
                    fallback_doc.tags
                },
                content: markdown.to_string(),
            }
        } else {
            fallback_doc
        };

        return Ok(AnalyzeMarkdownResponse {
            document: Some(doc),
            proposals: structured.proposals,
        });
    }

    Err("JSON was neither an object nor an array".to_string())
}

pub async fn analyze_markdown(
    State(state): State<AppState>,
    Json(payload): Json<AnalyzeMarkdownRequest>,
) -> Result<Json<AnalyzeMarkdownResponse>, crate::error::AppError> {
    let builder = rig_core::providers::ollama::Client::builder()
        .base_url(&state.config.llm.ollama_url)
        .api_key(rig_core::client::Nothing);

    let ollama_client = builder.build().map_err(|e| {
        crate::error::AppError::Message(format!("Failed to initialize Ollama client: {}", e))
    })?;

    let model = ollama_client.completion_model(&state.config.llm.model);

    let prompt = format!(
        "You are an expert knowledge extraction system. Your task is to analyze the following Markdown text and extract:\n\
         1. Document summary metadata:\n\
            - \"title\": A concise title for the overall document (e.g. from the primary heading).\n\
            - \"topic\": A broad category or domain for the entire document.\n\
            - \"description\": An executive summary of the whole document.\n\
            - \"tags\": An array of categorization strings.\n\
         2. Distinct, self-contained \"Concepts\" or \"Topics\" from the text.\n\
            Each concept proposal must have: \"topic\", \"title\", \"description\", \"tags\", \"content\".\n\n\
         Return a JSON object with keys \"document\" and \"proposals\".\n\n\
         Markdown Text:\n{}\n\n\
         Output ONLY a valid JSON object. Do not include markdown code blocks like ```json around the output.",
        payload.markdown
    );

    let request = model
        .completion_request(&prompt)
        .temperature(0.2)
        .max_tokens(4096);

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(state.config.llm.timeout_secs),
        request.send(),
    )
    .await
    .map_err(|e| crate::error::AppError::Message(format!("LLM request timed out: {}", e)))?
    .map_err(|e| crate::error::AppError::Message(format!("LLM error: {}", e)))?;

    if let Some(choice) = response.choice.first() {
        if let rig_core::completion::message::AssistantContent::Text(text) = choice {
            let content = text.text.trim();
            // Try to strip markdown code blocks if the LLM included them despite instructions
            let content = if content.starts_with("```json") {
                content
                    .trim_start_matches("```json")
                    .trim_end_matches("```")
                    .trim()
            } else if content.starts_with("```") {
                content
                    .trim_start_matches("```")
                    .trim_end_matches("```")
                    .trim()
            } else {
                content
            };

            match parse_llm_markdown_analysis(
                content,
                &payload.markdown,
                payload.suggested_topic.as_deref(),
            ) {
                Ok(resp) => return Ok(Json(resp)),
                Err(e) => {
                    tracing::error!("Failed to parse LLM response as JSON: {}", e);
                    tracing::error!("Raw LLM response: {}", content);
                    return Err(crate::error::AppError::Message(format!(
                        "Failed to parse LLM output: {}",
                        e
                    )));
                }
            }
        }
    }

    Err(crate::error::AppError::Message(
        "Failed to extract knowledge from LLM response".to_string(),
    ))
}

pub async fn import_document(
    State(pool): State<PgPool>,
    Json(payload): Json<ImportDocumentRequest>,
) -> Result<(StatusCode, Json<ImportDocumentResponse>), crate::error::AppError> {
    if payload.document.title.trim().is_empty() || payload.document.content.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest(
            "Document title and content are required".to_string(),
        ));
    }

    let mut tx = pool.begin().await.map_err(|e| {
        crate::error::AppError::Message(format!("Failed to start transaction: {}", e))
    })?;

    let doc_id = Uuid::new_v4();
    let concept_ids: Vec<Uuid> = (0..payload.concepts.len())
        .map(|_| Uuid::new_v4())
        .collect();

    let doc_meta = serde_json::json!({
        "is_source_document": true,
        "source_format": "markdown",
        "content_length": payload.document.content.len(),
        "child_concept_ids": concept_ids,
    });

    let created_doc = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        INSERT INTO knowledge_nodes (id, topic, title, description, tags, content, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        "#,
    )
    .bind(doc_id)
    .bind(&payload.document.topic)
    .bind(&payload.document.title)
    .bind(&payload.document.description)
    .bind(&payload.document.tags)
    .bind(&payload.document.content)
    .bind(doc_meta)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("Failed to insert document: {}", e)))?;

    // Embeddings for parent document
    let doc_chunks = chunk_text(&payload.document.content, 200);
    for (idx, chunk) in doc_chunks.iter().enumerate() {
        let chunk_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(chunk_id)
        .bind(doc_id)
        .bind(idx as i32)
        .bind(chunk)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            crate::error::AppError::Message(format!("Chunk Insert Error for Document: {}", e))
        })?;
    }

    let mut created_concepts = Vec::new();
    let mut tuples_created = 0;

    for (i, concept) in payload.concepts.iter().enumerate() {
        let concept_id = concept_ids[i];
        let child_meta = serde_json::json!({
            "is_extracted_concept": true,
            "source_document_id": doc_id,
            "source_document_title": payload.document.title,
        });

        let created_concept = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            INSERT INTO knowledge_nodes (id, topic, title, description, tags, content, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#,
        )
        .bind(concept_id)
        .bind(&concept.topic)
        .bind(&concept.title)
        .bind(&concept.description)
        .bind(&concept.tags)
        .bind(&concept.content)
        .bind(child_meta)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| crate::error::AppError::Message( format!("Failed to insert concept node: {}", e)))?;

        // Concept embeddings
        let concept_chunks = chunk_text(&concept.content, 200);
        for (idx, chunk) in concept_chunks.iter().enumerate() {
            let chunk_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(chunk_id)
            .bind(concept_id)
            .bind(idx as i32)
            .bind(chunk)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                crate::error::AppError::Message(format!("Chunk Insert Error for Concept: {}", e))
            })?;
        }

        if payload.create_tuples {
            let tuple_id = Uuid::new_v4();
            let tuple_meta = serde_json::json!({
                "provenance_type": "markdown_extraction",
                "source_document_id": doc_id,
                "child_concept_id": concept_id,
            });

            sqlx::query(
                r#"
                INSERT INTO knowledge_tuples (id, source_node_id, subject, predicate, object, confidence, metadata)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(tuple_id)
            .bind(concept_id)
            .bind(&concept.title)
            .bind("derived_from")
            .bind(&payload.document.title)
            .bind(1.0f64)
            .bind(tuple_meta)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Message( format!("Tuple Insert Error: {}", e)))?;

            tuples_created += 1;
        }

        created_concepts.push(created_concept);
    }

    tx.commit().await.map_err(|e| {
        crate::error::AppError::Message(format!("Failed to commit transaction: {}", e))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(ImportDocumentResponse {
            document: created_doc,
            concepts: created_concepts,
            tuples_created,
        }),
    ))
}

pub async fn get_derived_concepts(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<KnowledgeNode>>, crate::error::AppError> {
    let concepts = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        WHERE metadata->>'source_document_id' = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(id.to_string())
    .fetch_all(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    Ok(Json(concepts))
}

pub async fn ingest_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<KnowledgeNode>,
) -> Result<(StatusCode, Json<KnowledgeNode>), crate::error::AppError> {
    if payload.topic.trim().is_empty() || payload.content.trim().is_empty() {
        return Err(crate::error::AppError::BadRequest(
            "Topic and content are required".to_string(),
        ));
    }

    let node_id = payload.id.unwrap_or_else(Uuid::new_v4);
    metrics::counter!("knowledge_ingestion_total").increment(1);

    tracing::info!(
        "Ingesting/Saving knowledge node '{:?}' (topic: '{}', ID: {})",
        payload.title,
        payload.topic,
        node_id
    );
    let metadata = if payload.metadata.is_null() {
        serde_json::json!({})
    } else {
        payload.metadata
    };

    let tags = payload.tags;

    // 1. Insert Node
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
    .fetch_one(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    // 2. Chunk text and store mock vector embeddings
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
        .execute(&pool)
        .await
        .map_err(|e| crate::error::AppError::Message(format!("Chunk Insert Error: {}", e)))?;
    }

    // 3. Insert Tuples if provided
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
        .execute(&pool)
        .await
        .map_err(|e| crate::error::AppError::Message( format!("Tuple Insert Error: {}", e)))?;
    }

    Ok((StatusCode::CREATED, Json(created_node)))
}

pub async fn list_knowledge(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<KnowledgeNode>>, crate::error::AppError> {
    let nodes = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    Ok(Json(nodes))
}

pub async fn get_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<KnowledgeNode>, crate::error::AppError> {
    let node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    match node {
        Some(n) => Ok(Json(n)),
        None => Err(crate::error::AppError::NotFound(
            "Knowledge node not found".to_string(),
        )),
    }
}

pub async fn get_knowledge_tuples(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<KnowledgeTuple>>, crate::error::AppError> {
    let tuples = sqlx::query_as::<_, KnowledgeTuple>(
        "SELECT * FROM knowledge_tuples WHERE source_node_id = $1",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message(format!("DB Error: {}", e)))?;

    Ok(Json(tuples))
}

pub async fn update_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<KnowledgeNode>,
) -> Result<Json<KnowledgeNode>, crate::error::AppError> {
    let current_node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        FROM knowledge_nodes
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    let current_node = match current_node {
        Some(n) => n,
        None => {
            return Err(crate::error::AppError::NotFound(
                "Knowledge node not found".to_string(),
            ));
        }
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
    let new_metadata = if payload.metadata.is_null() || payload.metadata == serde_json::json!({}) {
        current_node.metadata
    } else {
        payload.metadata
    };

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| crate::error::AppError::Message(format!("Tx Error: {}", e)))?;

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
    .map_err(|e| crate::error::AppError::Message( format!("Update Error: {}", e)))?;

    // If content changed, we should ideally re-chunk and update embeddings.
    // Here we'll just delete old and insert new chunks as a simple strategy.
    if new_content != current_node.content {
        // Cascade delete tuples associated with the content
        sqlx::query("DELETE FROM knowledge_tuples WHERE source_node_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Message(format!("Delete tuples error: {}", e)))?;

        sqlx::query("DELETE FROM knowledge_embeddings WHERE node_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Message(format!("Delete chunks error: {}", e)))?;

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
            .map_err(|e| crate::error::AppError::Message(format!("Chunk Insert Error: {}", e)))?;
        }
    }

    if !payload.tuples.is_empty() {
        sqlx::query("DELETE FROM knowledge_tuples WHERE source_node_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Message(format!("Delete tuples error: {}", e)))?;

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
            .bind(id)
            .bind(tuple.triple.subject)
            .bind(tuple.triple.predicate)
            .bind(tuple.triple.object)
            .bind(confidence)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::AppError::Message( format!("Tuple Insert Error: {}", e)))?;
        }
    }
    tx.commit()
        .await
        .map_err(|e| crate::error::AppError::Message(format!("Commit Error: {}", e)))?;

    Ok(Json(updated_node))
}

pub async fn search_knowledge(
    State(pool): State<PgPool>,
    Json(payload): Json<KnowledgeSearchRequest>,
) -> Result<Json<Vec<KnowledgeSearchResult>>, crate::error::AppError> {
    let limit = payload.limit.unwrap_or(5) as i64;
    let pattern = format!("%{}%", payload.query.trim());

    // Mock incoming embedding string
    let mock_embedding_str = format!("[{}]", vec!["0.1"; 1536].join(","));

    let rows = sqlx::query(
        r#"
        SELECT node_id, chunk_index, chunk_text,
               (CASE
                   WHEN chunk_text ILIKE $1 THEN 0.95::float8
                   ELSE COALESCE(1.0 - (embedding <=> $2::vector), 0.5)::float8
               END) as similarity_score,
               (CASE
                   WHEN chunk_text ILIKE $1 THEN 'fulltext'
                   ELSE 'semantic'
               END) as search_type
        FROM knowledge_embeddings
        WHERE chunk_text ILIKE $1
           OR embedding IS NOT NULL
        ORDER BY (CASE WHEN chunk_text ILIKE $1 THEN 0 ELSE 1 END),
                 embedding <=> $2::vector NULLS LAST
        LIMIT $3
        "#,
    )
    .bind(pattern)
    .bind(mock_embedding_str)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message(format!("Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| KnowledgeSearchResult {
            node_id: r.get("node_id"),
            chunk_index: r.get("chunk_index"),
            chunk_text: r.get("chunk_text"),
            score: r.try_get("similarity_score").unwrap_or(0.5),
            search_type: r
                .try_get("search_type")
                .unwrap_or_else(|_| "semantic".to_string()),
        })
        .collect();

    Ok(Json(results))
}

pub async fn traverse_graph(
    State(pool): State<PgPool>,
    Json(payload): Json<GraphTraverseRequest>,
) -> Result<Json<Vec<GraphTraverseResult>>, crate::error::AppError> {
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
    .map_err(|e| crate::error::AppError::Message(format!("Traverse Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| GraphTraverseResult {
            triple: Triple::new(
                r.get::<String, _>("subject"),
                r.get::<String, _>("predicate"),
                r.get::<String, _>("object"),
            ),
            confidence: r.get("confidence"),
            depth: max_depth,
        })
        .collect();

    Ok(Json(results))
}

pub async fn delete_knowledge(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<KnowledgeNode>, crate::error::AppError> {
    tracing::info!("Deleting knowledge node: {}", id);

    let deleted_node = sqlx::query_as::<_, KnowledgeNode>(
        r#"
        DELETE FROM knowledge_nodes
        WHERE id = $1
        RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| crate::error::AppError::Message( format!("DB Error: {}", e)))?;

    match deleted_node {
        Some(node) => Ok(Json(node)),
        None => Err(crate::error::AppError::NotFound(
            "Knowledge node not found".to_string(),
        )),
    }
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

    #[test]
    fn test_extract_markdown_metadata_with_h1() {
        let md = "# System Architecture\n\nThis is an executive summary of the system architecture.\n\n## Section 1\nDetails here.";
        let doc = extract_markdown_metadata(md, Some("engineering"));
        assert_eq!(doc.title, "System Architecture");
        assert_eq!(doc.topic, "engineering");
        assert_eq!(
            doc.description,
            "This is an executive summary of the system architecture."
        );
        assert_eq!(
            doc.tags,
            vec!["document".to_string(), "imported".to_string()]
        );
        assert_eq!(doc.content, md);
    }

    #[test]
    fn test_extract_markdown_metadata_fallback() {
        let md = "No headers here at all, just plain text.";
        let doc = extract_markdown_metadata(md, None);
        assert_eq!(doc.title, "Imported Document");
        assert_eq!(doc.topic, "document");
        assert_eq!(doc.description, "No headers here at all, just plain text.");
    }

    #[test]
    fn test_parse_analyze_markdown_llm_json_wrapped() {
        let raw_json = r#"{
            "document": {
                "topic": "architecture",
                "title": "Platform Blueprint",
                "description": "Blueprint of the core platform.",
                "tags": ["blueprint", "platform"]
            },
            "proposals": [
                {
                    "topic": "storage",
                    "title": "Dual Store",
                    "description": "Dual storage approach.",
                    "tags": ["db"],
                    "content": "Content here"
                }
            ]
        }"#;

        let res = parse_llm_markdown_analysis(raw_json, "# Platform Blueprint\nBody", None)
            .expect("Should parse");
        assert!(res.document.is_some());
        let doc = res.document.unwrap();
        assert_eq!(doc.title, "Platform Blueprint");
        assert_eq!(doc.topic, "architecture");
        assert_eq!(doc.content, "# Platform Blueprint\nBody");
        assert_eq!(res.proposals.len(), 1);
        assert_eq!(res.proposals[0].title, "Dual Store");
    }

    #[test]
    fn test_parse_analyze_markdown_llm_array_fallback() {
        let raw_json = r#"[
            {
                "topic": "storage",
                "title": "Dual Store",
                "description": "Dual storage approach.",
                "tags": ["db"],
                "content": "Content here"
            }
        ]"#;

        let res =
            parse_llm_markdown_analysis(raw_json, "# Inferred Blueprint\nBody text.", Some("tech"))
                .expect("Should parse");
        assert!(res.document.is_some());
        let doc = res.document.unwrap();
        assert_eq!(doc.title, "Inferred Blueprint");
        assert_eq!(doc.topic, "tech");
        assert_eq!(res.proposals.len(), 1);
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
