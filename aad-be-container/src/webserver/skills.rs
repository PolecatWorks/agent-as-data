use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    models::{Agent, Skill},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_skills).post(create_skill))
        .route("/{id}", get(get_skill).put(update_skill).delete(delete_skill))
        .route("/{id}/promote", post(promote_skill))
        .route("/{id}/demote", post(demote_skill))
        .route("/{id}/sync-embeddings", post(sync_skill_embeddings))
}

pub async fn list_skills(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Skill>>, (StatusCode, String)> {
    let skills = sqlx::query_as::<_, Skill>(
        "SELECT id, name, description, definition, tags, current_version, owner_id, attached_skills, attached_tools, input_schema, output_schema, implementation, implements_traits, uses_traits FROM skills ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Skills Error: {}", e)))?;
    Ok(Json(skills))
}

pub async fn get_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Skill>, (StatusCode, String)> {
    let skill = sqlx::query_as::<_, Skill>(
        "SELECT id, name, description, definition, tags, current_version, owner_id, attached_skills, attached_tools, input_schema, output_schema, implementation, implements_traits, uses_traits FROM skills WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Skill Error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Skill not found".to_string()))?;
    Ok(Json(skill))
}

pub async fn create_skill(
    State(pool): State<PgPool>,
    Json(payload): Json<Skill>,
) -> Result<(StatusCode, Json<Skill>), (StatusCode, String)> {
    let skill_id = payload.id.unwrap_or_else(Uuid::new_v4);
    let input_schema = payload.input_schema.clone().unwrap_or_else(|| serde_json::json!({}));
    let output_schema = payload.output_schema.clone().unwrap_or_else(|| serde_json::json!({}));
    let implementation = payload.implementation.clone().unwrap_or_else(|| serde_json::json!({}));
    let current_version = if payload.current_version.is_empty()
        || payload.current_version == "0"
        || payload.current_version == "1"
    {
        "1.0.0".to_string()
    } else {
        payload.current_version.clone()
    };

    tracing::info!("Creating skill '{}' (ID: {}, version: {})", payload.name, skill_id, current_version);

    let row = sqlx::query(
        r#"
        INSERT INTO skills (id, name, description, definition, tags, current_version, owner_id, attached_skills, attached_tools, input_schema, output_schema, implementation, implements_traits, uses_traits)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (name) DO UPDATE SET
            description = EXCLUDED.description,
            definition = EXCLUDED.definition,
            tags = EXCLUDED.tags,
            current_version = EXCLUDED.current_version,
            owner_id = EXCLUDED.owner_id,
            attached_skills = EXCLUDED.attached_skills,
            attached_tools = EXCLUDED.attached_tools,
            input_schema = EXCLUDED.input_schema,
            output_schema = EXCLUDED.output_schema,
            implementation = EXCLUDED.implementation,
            implements_traits = EXCLUDED.implements_traits,
            uses_traits = EXCLUDED.uses_traits,
            updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(skill_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.definition)
    .bind(&payload.tags)
    .bind(&current_version)
    .bind(payload.owner_id)
    .bind(&payload.attached_skills)
    .bind(&payload.attached_tools)
    .bind(input_schema)
    .bind(output_schema)
    .bind(implementation)
    .bind(&payload.implements_traits)
    .bind(&payload.uses_traits)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Skill DB Error: {}", e)))?;

    let final_id: Uuid = row.get("id");
    let mut response_skill = payload.clone();
    response_skill.id = Some(final_id);
    response_skill.current_version = current_version;

    tracing::info!("Skill '{}' saved successfully (ID: {})", payload.name, final_id);

    Ok((StatusCode::CREATED, Json(response_skill)))
}

pub async fn update_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Skill>,
) -> Result<Json<Skill>, (StatusCode, String)> {
    tracing::info!("Updating skill '{}' (ID: {})", payload.name, id);
    let input_schema = payload.input_schema.clone().unwrap_or_else(|| serde_json::json!({}));
    let output_schema = payload.output_schema.clone().unwrap_or_else(|| serde_json::json!({}));
    let implementation = payload.implementation.clone().unwrap_or_else(|| serde_json::json!({}));
    let current_version = crate::models::bump_minor_version(&payload.current_version);

    sqlx::query(
        r#"
        UPDATE skills
        SET name = $1, description = $2, definition = $3, tags = $4, current_version = $5, attached_skills = $6, attached_tools = $7, input_schema = $8, output_schema = $9, implementation = $10, implements_traits = $11, uses_traits = $12, updated_at = NOW()
        WHERE id = $13
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.definition)
    .bind(&payload.tags)
    .bind(&current_version)
    .bind(&payload.attached_skills)
    .bind(&payload.attached_tools)
    .bind(input_schema)
    .bind(output_schema)
    .bind(implementation)
    .bind(&payload.implements_traits)
    .bind(&payload.uses_traits)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Skill Error: {}", e)))?;

    let mut response_skill = payload.clone();
    response_skill.current_version = current_version;
    tracing::info!("Skill '{}' updated successfully (ID: {})", payload.name, id);
    Ok(Json(response_skill))
}

pub async fn delete_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Deleting skill (ID: {})", id);
    sqlx::query("DELETE FROM skills WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Skill Error: {}", e)))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn promote_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, String)> {
    let skill_row = sqlx::query(
        "SELECT name, description, owner_id, implements_traits FROM skills WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Skill Fetch Error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, "Skill not found".to_string()))?;

    let name: String = skill_row.get("name");
    let description: String = skill_row.get("description");
    let owner_id: Uuid = skill_row.get("owner_id");
    let implements_traits: Vec<String> = skill_row.get("implements_traits");
    let uses_traits: Vec<String> = skill_row.try_get("uses_traits").unwrap_or_default();

    let agent_id = Uuid::new_v4();
    let agent_definition = serde_json::json!({
        "promoted_from_skill_id": id,
        "instructions": description
    });

    sqlx::query(
        r#"
        INSERT INTO agents (id, name, description, current_version, owner_id, agent_definition, implements_traits, uses_traits)
        VALUES ($1, $2, $3, '1.0.0', $4, $5, $6, $7)
        "#,
    )
    .bind(agent_id)
    .bind(&name)
    .bind(&description)
    .bind(owner_id)
    .bind(&agent_definition)
    .bind(&implements_traits)
    .bind(&uses_traits)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Promote Error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(Agent {
            id: Some(agent_id),
            name,
            description,
            tags: vec![],
            implements_traits,
            uses_traits,
            attached_tools: vec![],
            attached_agents: vec![],
            attached_skills: vec![],
            current_version: "1.0.0".to_string(),
            owner_id,
            judge_threshold: 0.8,
            input_guardrails: vec![],
            output_guardrails: vec![],
            guardrail_config: None,
            read_groups: vec![],
            write_groups: vec![],
            execute_groups: vec![],
            agent_definition,
            model: serde_json::json!({}),
            archived_at: None,
        }),
    ))
}

pub async fn demote_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let skill_row = sqlx::query("SELECT name, description, owner_id FROM skills WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Skill not found".to_string()))?;

    let name: String = skill_row.get("name");
    let description: String = skill_row.get("description");
    let owner_id: Uuid = skill_row.get("owner_id");

    let agent_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO agents (id, name, description, current_version, owner_id)
        VALUES ($1, $2, $3, '1.0.0', $4)
        "#,
    )
    .bind(agent_id)
    .bind(format!("Demoted_{}", name))
    .bind(&description)
    .bind(owner_id)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Agent DB Error: {}", e)))?;

    Ok(Json(serde_json::json!({
        "skill_id": id,
        "demoted_to_agent_id": agent_id,
        "status": "demoted"
    })))
}

pub async fn sync_skill_embeddings(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::SyncEmbeddingsResponse>, (StatusCode, String)> {
    let skill_row = sqlx::query("SELECT name, description, definition FROM skills WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    let skill_row = match skill_row {
        Some(row) => row,
        None => return Err((StatusCode::NOT_FOUND, "Skill not found".to_string())),
    };

    let name: String = skill_row.get("name");
    let description: String = skill_row.try_get("description").unwrap_or_default();
    let definition: String = skill_row.try_get("definition").unwrap_or_default();

    // Clean up old embeddings
    sqlx::query("DELETE FROM entity_embeddings WHERE entity_id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Old Error: {}", e)))?;

    let mut count = 0;

    // Insert Name
    sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'skills', 'name', $2)")
        .bind(id)
        .bind(&name)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Name Error: {}", e)))?;
    count += 1;

    // Insert Description
    if !description.is_empty() {
        sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'skills', 'description', $2)")
            .bind(id)
            .bind(&description)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Desc Error: {}", e)))?;
        count += 1;
    }

    // Insert Definition (Prompt)
    if !definition.is_empty() {
        sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'skills', 'prompt', $2)")
            .bind(id)
            .bind(&definition)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Def Error: {}", e)))?;
        count += 1;
    }

    Ok(Json(crate::models::SyncEmbeddingsResponse {
        status: "success".to_string(),
        entity_id: id,
        embeddings_created: count,
    }))
}
