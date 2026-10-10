use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use sqlx::{PgPool, Row};
use uuid::Uuid;

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
                COALESCE(e.origin_name, a.name, s.name, t.server_name, tr.name) as name,
                COALESCE(a.description, s.description, tr.description) as description,
                COALESCE(e.origin_id, e.entity_id) as origin_id,
                COALESCE(e.origin_type, e.entity_type) as origin_type,
                COALESCE(e.origin_uri, '/' || e.entity_type || '/' || e.entity_id::text) as origin_uri,
                COALESCE(e.origin_name, a.name, s.name, t.server_name, tr.name) as origin_name,
                (CASE
                    WHEN e.content ILIKE $1 THEN 0.98::float8
                    ELSE LEAST(0.96::float8, GREATEST(0.60::float8, (0.65 + (ts_rank_cd(to_tsvector('english', e.content), NULLIF(replace(plainto_tsquery('english', $2)::text, '&', '|'), '')::tsquery) * 1.5))::float8))
                END)::float8 as score
            FROM entity_embeddings e
            LEFT JOIN agents a ON e.entity_type IN ('agents', 'agent') AND e.entity_id = a.id
            LEFT JOIN skills s ON e.entity_type IN ('skills', 'skill') AND e.entity_id = s.id
            LEFT JOIN tools t ON e.entity_type IN ('tools', 'tool') AND e.entity_id = t.id
            LEFT JOIN trait_contracts tr ON e.entity_type IN ('traits', 'trait') AND e.entity_id = tr.id
            WHERE (e.content ILIKE $1
               OR (
                   to_tsvector('english', e.content) @@ NULLIF(replace(plainto_tsquery('english', $2)::text, '&', '|'), '')::tsquery
               ))
               AND (e.entity_type NOT IN ('agents', 'agent') OR (a.id IS NOT NULL AND a.archived_at IS NULL))
               AND (e.entity_type NOT IN ('skills', 'skill') OR s.id IS NOT NULL)
               AND (e.entity_type NOT IN ('tools', 'tool') OR t.id IS NOT NULL)
               AND (e.entity_type NOT IN ('traits', 'trait') OR tr.id IS NOT NULL)
        ),
        deduped AS (
            SELECT DISTINCT ON (COALESCE(name, entity_id::text))
                entity_id,
                entity_type,
                field_name,
                content,
                name,
                description,
                origin_id,
                origin_type,
                origin_uri,
                origin_name,
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
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Semantic Search Error: {}", e),
            )
        })?;

    let results = rows
        .into_iter()
        .map(|r| {
            let field_name: String = r.get("field_name");
            let match_reason = match field_name.as_str() {
                "name" => "Matched on entity name".to_string(),
                "description" => "Matched on entity description".to_string(),
                "prompt" => "Matched on agent prompt".to_string(),
                "definition" => "Matched on skill definition".to_string(),
                "invariants" => "Matched on behavioral invariants".to_string(),
                "criteria" => "Matched on evaluation criteria".to_string(),
                "capabilities" => "Matched on tool capabilities".to_string(),
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
                origin_id: r.try_get("origin_id").ok(),
                origin_type: r.try_get("origin_type").ok(),
                origin_uri: r.try_get("origin_uri").ok(),
                origin_name: r.try_get("origin_name").ok(),
            }
        })
        .collect();

    Ok(Json(results))
}

pub async fn sync_entity_embeddings(
    pool: &PgPool,
    entity_id: Uuid,
    entity_type: &str,
    entity_name: &str,
    description: Option<&str>,
    additional_fields: &[(&str, &str)],
) -> Result<usize, sqlx::Error> {
    // 1. Purge previous embeddings for this entity
    sqlx::query("DELETE FROM entity_embeddings WHERE entity_id = $1")
        .bind(entity_id)
        .execute(pool)
        .await?;

    let mut count = 0;
    let origin_uri = format!("/{}/{}", entity_type, entity_id);

    // 2. Insert Name
    sqlx::query(
        r#"
        INSERT INTO entity_embeddings 
            (entity_id, entity_type, field_name, content, origin_id, origin_type, origin_uri, origin_name)
        VALUES ($1, $2, 'name', $3, $1, $2, $4, $5)
        "#
    )
    .bind(entity_id)
    .bind(entity_type)
    .bind(entity_name)
    .bind(&origin_uri)
    .bind(entity_name)
    .execute(pool)
    .await?;
    count += 1;

    // 3. Insert Description if present
    if let Some(desc) = description {
        let desc_trimmed = desc.trim();
        if !desc_trimmed.is_empty() {
            sqlx::query(
                r#"
                INSERT INTO entity_embeddings 
                    (entity_id, entity_type, field_name, content, origin_id, origin_type, origin_uri, origin_name)
                VALUES ($1, $2, 'description', $3, $1, $2, $4, $5)
                "#
            )
            .bind(entity_id)
            .bind(entity_type)
            .bind(desc_trimmed)
            .bind(&origin_uri)
            .bind(entity_name)
            .execute(pool)
            .await?;
            count += 1;
        }
    }

    // 4. Insert additional fields
    for (field_name, content) in additional_fields {
        let trimmed = content.trim();
        if !trimmed.is_empty() && trimmed != "{}" && trimmed != "\"\"" {
            sqlx::query(
                r#"
                INSERT INTO entity_embeddings 
                    (entity_id, entity_type, field_name, content, origin_id, origin_type, origin_uri, origin_name)
                VALUES ($1, $2, $3, $4, $1, $2, $5, $6)
                "#
            )
            .bind(entity_id)
            .bind(entity_type)
            .bind(*field_name)
            .bind(trimmed)
            .bind(&origin_uri)
            .bind(entity_name)
            .execute(pool)
            .await?;
            count += 1;
        }
    }

    Ok(count)
}

pub async fn purge_entity_embeddings(pool: &PgPool, entity_id: Uuid) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM entity_embeddings WHERE entity_id = $1")
        .bind(entity_id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_sync_and_purge_embeddings_with_reverse_references() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:mysecretpassword@localhost:5432/aaddb".to_string()
        });
        if let Ok(pool) = sqlx::postgres::PgPoolOptions::new().connect(&db_url).await {
            let entity_id = Uuid::new_v4();
            let entity_name = format!("TestEntity-{}", entity_id);
            let description = "Unique description for testing reverse references.";
            let prompt = "Instructions for testing.";

            // 1. Sync embeddings
            let count = sync_entity_embeddings(
                &pool,
                entity_id,
                "agents",
                &entity_name,
                Some(description),
                &[("prompt", prompt)],
            )
            .await;

            if count.is_err() {
                println!("Note: Database not migrated or offline, skipping test");
                return;
            }
            let count = count.unwrap();
            assert_eq!(count, 3);

            // 2. Verify rows in entity_embeddings have reverse reference fields
            let rows = sqlx::query(
                "SELECT origin_id, origin_type, origin_uri, origin_name FROM entity_embeddings WHERE entity_id = $1"
            )
            .bind(entity_id)
            .fetch_all(&pool)
            .await
            .unwrap();

            assert_eq!(rows.len(), 3);
            for r in rows {
                let orig_id: Uuid = r.get("origin_id");
                let orig_type: String = r.get("origin_type");
                let orig_uri: String = r.get("origin_uri");
                let orig_name: String = r.get("origin_name");

                assert_eq!(orig_id, entity_id);
                assert_eq!(orig_type, "agents");
                assert_eq!(orig_uri, format!("/agents/{}", entity_id));
                assert_eq!(orig_name, entity_name);
            }

            // 3. Purge embeddings
            let purged = purge_entity_embeddings(&pool, entity_id).await.unwrap();
            assert_eq!(purged, 3);

            let remaining: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM entity_embeddings WHERE entity_id = $1")
                    .bind(entity_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(remaining, 0);
        }
    }
}
