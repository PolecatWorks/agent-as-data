use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    AppState,
    models::{ExecuteAgentRequest, ExecuteAgentResponse, SearchAndExecuteRequest},
};

pub async fn execute_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(payload): Json<ExecuteAgentRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    let execution_id = Uuid::new_v4();

    // 1. Fetch Agent definition & version, or fallback to Skill definition
    let agent_version: String;
    let mut system_prompt = String::new();
    let mut target_model = payload.model.clone().unwrap_or_else(|| state.config.llm.model.clone());

    let agent_row = sqlx::query("SELECT name, current_version, agent_definition, model, description FROM agents WHERE id = $1")
        .bind(agent_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = agent_row {
        agent_version = r.get("current_version");
        let agent_def: serde_json::Value = r.try_get("agent_definition").unwrap_or(serde_json::Value::Null);
        if let Some(s) = agent_def.as_str() {
            system_prompt = s.to_string();
        } else if !agent_def.is_null() {
            system_prompt = agent_def.to_string();
        } else if let Ok(desc) = r.try_get::<String, _>("description") {
            let name: String = r.try_get("name").unwrap_or_default();
            system_prompt = format!("You are an AI agent named {}. {}", name, desc);
        }

        if payload.model.is_none() {
            let model_val: serde_json::Value = r.try_get("model").unwrap_or(serde_json::Value::Null);
            if let Some(m) = model_val.as_str() {
                if !m.trim().is_empty() {
                    target_model = m.to_string();
                }
            } else if let Some(m) = model_val.get("name").and_then(|n| n.as_str()) {
                if !m.trim().is_empty() {
                    target_model = m.to_string();
                }
            }
        }
    } else {
        // Check skills table
        let skill_row = sqlx::query("SELECT name, current_version, definition, description FROM skills WHERE id = $1")
            .bind(agent_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

        if let Some(sr) = skill_row {
            agent_version = sr.get("current_version");
            let def_str: Option<String> = sr.try_get("definition").ok();
            if let Some(def) = def_str {
                system_prompt = def;
            } else if let Ok(desc) = sr.try_get::<String, _>("description") {
                let name: String = sr.try_get("name").unwrap_or_default();
                system_prompt = format!("You are executing the skill '{}'. {}", name, desc);
            }
        } else {
            return Err((StatusCode::NOT_FOUND, "Agent or Skill not found".to_string()));
        }
    }

    // 2. Incoming Guardrail Interceptor Validation
    if payload.prompt.contains("<script>") || payload.prompt.contains("DROP TABLE") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Incoming Guardrail Violation: Malformed content or prompt injection detected".to_string(),
        ));
    }

    // 3. AI Execution via Rig & Ollama using loaded AppConfig
    let builder = rig_core::providers::ollama::Client::builder()
        .base_url(&state.config.llm.ollama_url)
        .api_key(rig_core::client::Nothing);

    let ollama_client = builder.build().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to initialize Ollama client: {}", e),
        )
    })?;

    use rig_core::client::CompletionClient;
    use rig_core::completion::CompletionModel;
    let completion_model = ollama_client.completion_model(&target_model);

    let full_prompt = if !system_prompt.trim().is_empty() {
        format!("System Instructions:\n{}\n\nUser Request:\n{}", system_prompt.trim(), payload.prompt.trim())
    } else {
        payload.prompt.clone()
    };

    let req = completion_model.completion_request(&full_prompt).build();
    let timeout_duration = std::time::Duration::from_secs(state.config.llm.timeout_secs);

    let raw_output = match tokio::time::timeout(timeout_duration, completion_model.completion(req)).await {
        Ok(Ok(response)) => {
            if !response.choice.is_empty() {
                if let rig_core::completion::message::AssistantContent::Text(text) = &response.choice[0] {
                    text.text.clone()
                } else {
                    return Err((
                        StatusCode::BAD_GATEWAY,
                        "LLM returned an empty completion response".to_string(),
                    ));
                }
            } else {
                return Err((
                    StatusCode::BAD_GATEWAY,
                    "LLM returned an empty completion response".to_string(),
                ));
            }
        }
        Ok(Err(e)) => {
            tracing::error!("Ollama Rig completion error for model '{}': {}", target_model, e);
            return Err((
                StatusCode::BAD_GATEWAY,
                format!("LLM Execution Error: {}", e),
            ));
        }
        Err(_) => {
            tracing::error!("Ollama Rig completion timed out after {}s", state.config.llm.timeout_secs);
            return Err((
                StatusCode::GATEWAY_TIMEOUT,
                format!("LLM Execution Timed Out after {}s", state.config.llm.timeout_secs),
            ));
        }
    };


    // 4. Outgoing Guardrail Interceptor Sanitization
    let sanitized_output = if raw_output.contains("SECRET_") {
        "[REDACTED_SECRET]".to_string()
    } else {
        raw_output
    };

    // 5. Insert Execution Record with OCC Version Lock & Optional Webhook
    sqlx::query(
        r#"
        INSERT INTO executions (id, agent_id, agent_version, execution_version, status, request_payload, response_payload, webhook_url, started_at, completed_at)
        VALUES ($1, $2, $3, 1, 'completed', $4, $5, $6, NOW(), NOW())
        "#,
    )
    .bind(execution_id)
    .bind(agent_id)
    .bind(agent_version)
    .bind(serde_json::json!({ "prompt": payload.prompt, "model": target_model }))
    .bind(serde_json::json!({ "output": sanitized_output }))
    .bind(&payload.webhook_url)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Execution Insert Error: {}", e)))?;

    // 6. Dispatch optional background Webhook notification if webhook_url provided
    if let Some(webhook_url) = payload.webhook_url {
        let client = reqwest::Client::new();
        let body = serde_json::json!({
            "execution_id": execution_id,
            "agent_id": agent_id,
            "status": "completed"
        });
        let _ = client.post(&webhook_url).json(&body).send().await;
    }

    Ok((
        StatusCode::OK,
        Json(ExecuteAgentResponse {
            execution_id,
            agent_id,
            status: "completed".to_string(),
            output: sanitized_output,
            execution_version: 1,
        }),
    ))
}


pub async fn search_and_execute(
    State(state): State<AppState>,
    Json(payload): Json<SearchAndExecuteRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    // 1. Discovery top match agent matching task query strictly
    let pattern = format!("%{}%", payload.task_query.trim());
    let agent_row = sqlx::query(
        r#"
        SELECT id FROM agents
        WHERE (name ILIKE $1 OR description ILIKE $1 OR agent_definition::text ILIKE $1)
          AND archived_at IS NULL
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(pattern)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Discovery Error: {}", e)))?
    .ok_or((StatusCode::NOT_FOUND, format!("No matching agents found for task query '{}'", payload.task_query)))?;

    let agent_id = agent_row.get("id");

    execute_agent(
        State(state),
        Path(agent_id),
        Json(ExecuteAgentRequest {
            prompt: payload.prompt,
            model: None,
            context: None,
            webhook_url: None,
        }),
    )
    .await
}

pub async fn get_execution(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExecuteAgentResponse>, (StatusCode, String)> {
    let row = sqlx::query("SELECT agent_id, status, response_payload, execution_version FROM executions WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Execution job not found".to_string()))?;

    let agent_id: Uuid = row.get("agent_id");
    let status: String = row.get("status");
    let execution_version: i32 = row.get("execution_version");
    let response: serde_json::Value = row.get("response_payload");
    let output = response["output"].as_str().unwrap_or("").to_string();

    Ok(Json(ExecuteAgentResponse {
        execution_id: id,
        agent_id,
        status,
        output,
        execution_version,
    }))
}
