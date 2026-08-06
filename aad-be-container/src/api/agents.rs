use crate::{
    error::AppError,
    models::agent::{Agent, CreateAgentDto, UpdateAgentDto},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn list_agents(State(pool): State<PgPool>) -> Result<Json<Vec<Agent>>, AppError> {
    let agents = sqlx::query_as::<_, Agent>(
        "SELECT id, name, description, tags, input_guardrails, output_guardrails, agent_definition, model, tools, available_skills, available_agents, created_at, updated_at FROM agents ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(agents))
}

pub async fn get_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, AppError> {
    let agent = sqlx::query_as::<_, Agent>(
        "SELECT id, name, description, tags, input_guardrails, output_guardrails, agent_definition, model, tools, available_skills, available_agents, created_at, updated_at FROM agents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?;

    match agent {
        Some(a) => Ok(Json(a)),
        None => Err(AppError::NotFound(format!("Agent {} not found", id))),
    }
}

pub async fn create_agent(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateAgentDto>,
) -> Result<(StatusCode, Json<Agent>), AppError> {
    let agent = sqlx::query_as::<_, Agent>(
        r#"
        INSERT INTO agents (name, description, tags, input_guardrails, output_guardrails, agent_definition, model, tools, available_skills, available_agents)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, name, description, tags, input_guardrails, output_guardrails, agent_definition, model, tools, available_skills, available_agents, created_at, updated_at
        "#,
    )
    .bind(payload.name)
    .bind(payload.description)
    .bind(payload.tags)
    .bind(payload.input_guardrails)
    .bind(payload.output_guardrails)
    .bind(payload.agent_definition)
    .bind(payload.model)
    .bind(payload.tools)
    .bind(payload.available_skills)
    .bind(payload.available_agents)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(agent)))
}

pub async fn update_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateAgentDto>,
) -> Result<Json<Agent>, AppError> {
    let existing = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?;

    let existing = match existing {
        Some(a) => a,
        None => return Err(AppError::NotFound(format!("Agent {} not found", id))),
    };

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.or(existing.description);
    let tags = payload.tags.unwrap_or(existing.tags);
    let input_guardrails = payload.input_guardrails.unwrap_or(existing.input_guardrails);
    let output_guardrails = payload.output_guardrails.unwrap_or(existing.output_guardrails);
    let agent_definition = payload.agent_definition.unwrap_or(existing.agent_definition);
    let model = payload.model.unwrap_or(existing.model);
    let tools = payload.tools.unwrap_or(existing.tools);
    let available_skills = payload.available_skills.unwrap_or(existing.available_skills);
    let available_agents = payload.available_agents.unwrap_or(existing.available_agents);

    let updated = sqlx::query_as::<_, Agent>(
        r#"
        UPDATE agents
        SET name = $1, description = $2, tags = $3, input_guardrails = $4, output_guardrails = $5,
            agent_definition = $6, model = $7, tools = $8, available_skills = $9, available_agents = $10,
            updated_at = NOW()
        WHERE id = $11
        RETURNING id, name, description, tags, input_guardrails, output_guardrails, agent_definition, model, tools, available_skills, available_agents, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(description)
    .bind(tags)
    .bind(input_guardrails)
    .bind(output_guardrails)
    .bind(agent_definition)
    .bind(model)
    .bind(tools)
    .bind(available_skills)
    .bind(available_agents)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(updated))
}

pub async fn delete_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM agents WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("Agent {} not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
