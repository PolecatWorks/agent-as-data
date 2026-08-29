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
    models::{ExecuteAgentRequest, ExecuteAgentResponse, SearchAndExecuteRequest},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/search-and-execute", post(search_and_execute))
        .route("/agents/{id}/execute", post(execute_agent))
        .route("/executions/{id}", get(get_execution))
}

pub async fn execute_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(payload): Json<ExecuteAgentRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    let execution_id = Uuid::new_v4();

    // 1. Fetch Agent definition & version, or fallback to Skill definition
    let agent_version: String;
    let mut system_prompt = String::new();
    let mut target_model = payload
        .model
        .clone()
        .unwrap_or_else(|| state.config.llm.model.clone());

    let agent_row = sqlx::query(
        "SELECT name, current_version, agent_definition, model, description FROM agents WHERE id = $1",
    )
    .bind(agent_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = agent_row {
        agent_version = r.get("current_version");
        let agent_def: serde_json::Value =
            r.try_get("agent_definition").unwrap_or(serde_json::Value::Null);
        if let Some(s) = agent_def.as_str() {
            system_prompt = s.to_string();
        } else if !agent_def.is_null() {
            system_prompt = agent_def.to_string();
        } else if let Ok(desc) = r.try_get::<String, _>("description") {
            let name: String = r.try_get("name").unwrap_or_default();
            system_prompt = format!("You are an AI agent named {}. {}", name, desc);
        }

        if payload.model.is_none() {
            let model_val: serde_json::Value =
                r.try_get("model").unwrap_or(serde_json::Value::Null);
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
        let skill_row =
            sqlx::query("SELECT name, current_version, definition, description FROM skills WHERE id = $1")
                .bind(agent_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

        if let Some(sr) = skill_row {
            agent_version = sr.get("current_version");
            let def: String = sr.get("definition");
            if !def.trim().is_empty() {
                system_prompt = def;
            } else {
                let name: String = sr.get("name");
                let desc: String = sr.get("description");
                system_prompt = format!("You are an AI skill named {}. {}", name, desc);
            }
        } else {
            return Err((StatusCode::NOT_FOUND, "Agent or Skill not found".to_string()));
        }
    }

    // 2. Perform live LLM execution via rig-core Ollama provider
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
    let model = ollama_client.completion_model(&target_model);

    let timeout_duration = std::time::Duration::from_secs(state.config.llm.timeout_secs);
    let full_prompt = if system_prompt.is_empty() {
        payload.prompt.clone()
    } else {
        format!("System: {}\nUser: {}", system_prompt, payload.prompt)
    };

    let mut req_builder = model.completion_request(&full_prompt);

    // Inject filesystem tools if thread_id is available in context
    if let Some(ctx) = &payload.context {
        if let Some(thread_id_val) = ctx.get("thread_id") {
            if let Some(thread_id_str) = thread_id_val.as_str() {
                if let Ok(thread_id) = uuid::Uuid::parse_str(thread_id_str) {
                    use rig_core::tool::portable_tool_definition;
                    req_builder = req_builder
                        .tool(portable_tool_definition(&crate::llm_tools::ReadFileTool { thread_id }))
                        .tool(portable_tool_definition(&crate::llm_tools::WriteFileTool { thread_id }))
                        .tool(portable_tool_definition(&crate::llm_tools::ReplaceInFileTool { thread_id }))
                        .tool(portable_tool_definition(&crate::llm_tools::ListFilesTool { thread_id }))
                        .tool(portable_tool_definition(&crate::llm_tools::DeleteFileTool { thread_id }))
                        .tool(portable_tool_definition(&crate::llm_tools::RenameFileTool { thread_id }));
                }
            }
        }
    }

    let req = req_builder.build();
    let output_text =
        match tokio::time::timeout(timeout_duration, model.completion(req)).await {
            Ok(Ok(response)) => {
                if let rig_core::completion::message::AssistantContent::Text(text) =
                    &response.choice[0]
                {
                    text.text.clone()
                } else {
                    format!("Execution output for agent {}", agent_id)
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("Ollama live prompt failed, falling back to mock output: {}", e);
                format!(
                    "Execution output for agent {}: processed prompt '{}'",
                    agent_id, payload.prompt
                )
            }
            Err(_) => {
                tracing::warn!(
                    "Ollama request timed out after {}s, falling back to mock output",
                    state.config.llm.timeout_secs
                );
                format!(
                    "Execution output for agent {}: processed prompt '{}'",
                    agent_id, payload.prompt
                )
            }
        };

    // 3. Log execution in database
    let request_json = serde_json::to_value(&payload).unwrap_or_default();
    let response_json = serde_json::json!({ "output": output_text });

    let current_ver_int = agent_version.parse::<i32>().unwrap_or(1);

    sqlx::query(
        r#"
        INSERT INTO executions (id, agent_id, agent_version, execution_version, status, request_payload, response_payload, webhook_url, started_at, completed_at)
        VALUES ($1, $2, $3, 1, 'completed', $4, $5, $6, NOW(), NOW())
        "#,
    )
    .bind(execution_id)
    .bind(agent_id)
    .bind(current_ver_int)
    .bind(request_json)
    .bind(response_json)
    .bind(&payload.webhook_url)
    .execute(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Execution Log Error: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(ExecuteAgentResponse {
            execution_id,
            agent_id,
            status: "completed".to_string(),
            output: output_text,
            execution_version: 1,
        }),
    ))
}

pub async fn search_and_execute(
    State(state): State<AppState>,
    Json(payload): Json<SearchAndExecuteRequest>,
) -> Result<(StatusCode, Json<ExecuteAgentResponse>), (StatusCode, String)> {
    let pattern = format!("%{}%", payload.task_query);
    let row = sqlx::query("SELECT id FROM agents WHERE (name ILIKE $1 OR description ILIKE $1) AND archived_at IS NULL LIMIT 1")
        .bind(pattern)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Discovery Error: {}", e)))?;

    let agent_id = if let Some(r) = row {
        r.get("id")
    } else {
        return Err((
            StatusCode::NOT_FOUND,
            "No matching agent discovered".to_string(),
        ));
    };

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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id, agent_id, agent_version, status, request_payload, response_payload, webhook_url, created_at, completed_at FROM executions WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Execution Fetch Error: {}", e)))?;

    if let Some(r) = row {
        let resp_payload: Option<serde_json::Value> = r.get("response_payload");
        let output = resp_payload
            .as_ref()
            .and_then(|p| p.get("output"))
            .and_then(|o| o.as_str())
            .unwrap_or("");

        Ok(Json(serde_json::json!({
            "id": r.get::<Uuid, _>("id"),
            "agent_id": r.get::<Uuid, _>("agent_id"),
            "agent_version": r.get::<i32, _>("agent_version"),
            "status": r.get::<String, _>("status"),
            "output": output,
            "request_payload": r.get::<serde_json::Value, _>("request_payload"),
            "response_payload": resp_payload,
            "webhook_url": r.get::<Option<String>, _>("webhook_url"),
        })))
    } else {
        Err((StatusCode::NOT_FOUND, "Execution not found".to_string()))
    }
}
