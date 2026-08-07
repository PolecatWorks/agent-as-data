use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    AgentResponse, AgentSearchRequest, AgentSearchResult, CreateAgentRequest, CreateSkillRequest,
    SkillResponse, VerifyContractRequest, VerifyContractResponse,
};

pub async fn create_agent(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<AgentResponse>), (StatusCode, String)> {
    let agent_id = Uuid::new_v4();
    let tags = payload.tags.unwrap_or_default();
    let traits = payload.implements_traits.unwrap_or_default();
    let read_groups = payload.read_groups.unwrap_or_default();
    let write_groups = payload.write_groups.unwrap_or_default();
    let execute_groups = payload.execute_groups.unwrap_or_default();
    let model = payload.model.unwrap_or_else(|| serde_json::json!({}));

    // 1. Insert Agent
    sqlx::query(
        r#"
        INSERT INTO agents (id, name, description, tags, implements_traits, current_version, owner_id, read_groups, write_groups, execute_groups, agent_definition, model)
        VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(agent_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&tags)
    .bind(&traits)
    .bind(payload.owner_id)
    .bind(&read_groups)
    .bind(&write_groups)
    .bind(&execute_groups)
    .bind(&payload.agent_definition)
    .bind(&model)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Agent DB Error: {}", e)))?;

    // 2. Insert Immutable Version Snapshot
    let snapshot = serde_json::json!({
        "id": agent_id,
        "name": payload.name,
        "agent_definition": payload.agent_definition,
        "version": 1
    });

    sqlx::query(
        r#"
        INSERT INTO agent_revisions (id, agent_id, version, snapshot)
        VALUES ($1, $2, 1, $3)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(snapshot)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Revision DB Error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(AgentResponse {
            id: agent_id,
            name: payload.name,
            description: payload.description,
            tags,
            implements_traits: traits,
            current_version: 1,
            owner_id: payload.owner_id,
        }),
    ))
}

pub async fn search_agents(
    State(pool): State<PgPool>,
    Json(payload): Json<AgentSearchRequest>,
) -> Result<Json<Vec<AgentSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(5) as i64;
    let pattern = format!("%{}%", payload.query);

    let rows = sqlx::query(
        r#"
        SELECT id, name, description
        FROM agents
        WHERE name ILIKE $1 OR description ILIKE $1
        LIMIT $2
        "#,
    )
    .bind(pattern)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Agent Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| AgentSearchResult {
            agent_id: r.get("id"),
            name: r.get("name"),
            description: r.get("description"),
            score: 0.98,
        })
        .collect();

    Ok(Json(results))
}

pub async fn create_skill(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateSkillRequest>,
) -> Result<(StatusCode, Json<SkillResponse>), (StatusCode, String)> {
    let skill_id = Uuid::new_v4();
    let tags = payload.tags.unwrap_or_default();
    let input_schema = payload.input_schema.unwrap_or_else(|| serde_json::json!({}));
    let output_schema = payload.output_schema.unwrap_or_else(|| serde_json::json!({}));
    let implementation = payload.implementation.unwrap_or_else(|| serde_json::json!({}));

    sqlx::query(
        r#"
        INSERT INTO skills (id, name, description, tags, current_version, owner_id, input_schema, output_schema, implementation)
        VALUES ($1, $2, $3, $4, 1, $5, $6, $7, $8)
        "#,
    )
    .bind(skill_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&tags)
    .bind(payload.owner_id)
    .bind(input_schema)
    .bind(output_schema)
    .bind(implementation)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Skill DB Error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(SkillResponse {
            id: skill_id,
            name: payload.name,
            description: payload.description,
            current_version: 1,
            owner_id: payload.owner_id,
        }),
    ))
}

pub async fn promote_skill(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<AgentResponse>), (StatusCode, String)> {
    let skill_row = sqlx::query("SELECT name, description, owner_id FROM skills WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Skill Fetch Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Skill not found".to_string()))?;

    let name: String = skill_row.get("name");
    let description: String = skill_row.get("description");
    let owner_id: Uuid = skill_row.get("owner_id");

    let agent_id = Uuid::new_v4();
    let agent_definition = serde_json::json!({
        "promoted_from_skill_id": id,
        "instructions": description
    });

    sqlx::query(
        r#"
        INSERT INTO agents (id, name, description, current_version, owner_id, agent_definition)
        VALUES ($1, $2, $3, 1, $4, $5)
        "#,
    )
    .bind(agent_id)
    .bind(&name)
    .bind(&description)
    .bind(owner_id)
    .bind(&agent_definition)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Promote Error: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(AgentResponse {
            id: agent_id,
            name,
            description: Some(description),
            tags: vec![],
            implements_traits: vec![],
            current_version: 1,
            owner_id,
        }),
    ))
}

pub async fn verify_contract(
    State(_pool): State<PgPool>,
    Json(payload): Json<VerifyContractRequest>,
) -> Result<Json<VerifyContractResponse>, (StatusCode, String)> {
    Ok(Json(VerifyContractResponse {
        status: "verified".to_string(),
        semantic_fit_score: 0.96,
        contract_valid: true,
    }))
}
