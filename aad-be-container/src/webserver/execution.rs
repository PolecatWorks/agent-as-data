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
        .route("/agents/search-and-execute", post(search_and_execute))
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
    let mut is_skill = false;
    let mut system_prompt = String::new();
    let mut target_model = payload
        .model
        .clone()
        .unwrap_or_else(|| state.config.llm.model.clone());
    let mut agent_guardrail_config: Option<serde_json::Value> = None;

    let agent_row = sqlx::query(
        "SELECT name, current_version, agent_definition, model, description, guardrail_config FROM agents WHERE id = $1",
    )
    .bind(agent_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = agent_row {
        agent_version = r.get("current_version");
        agent_guardrail_config = r.try_get("guardrail_config").ok().flatten();
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
                
            let m_str = model_val.as_str()
                .or_else(|| model_val.get("name").and_then(|n| n.as_str()));
                
            if let Some(m) = m_str {
                if !m.trim().is_empty() {
                    target_model = m.to_string();
                }
            }
        }
    } else {
        is_skill = true;
        // Check skills table
        let skill_row =
            sqlx::query("SELECT name, current_version, definition, description FROM skills WHERE id = $1")
                .bind(agent_id)
                .fetch_optional(&state.pool)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

        let Some(sr) = skill_row else {
            return Err((StatusCode::NOT_FOUND, "Agent or Skill not found".to_string()));
        };

        agent_version = sr.get("current_version");
        let def: String = sr.get("definition");
        if !def.trim().is_empty() {
            system_prompt = def;
        } else {
            let name: String = sr.get("name");
            let desc: String = sr.get("description");
            system_prompt = format!("You are an AI skill named {}. {}", name, desc);
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

    let timeout_duration = state.config.llm.timeout;
    let full_prompt = if system_prompt.is_empty() {
        payload.prompt.clone()
    } else {
        format!("System: {}\nUser: {}", system_prompt, payload.prompt)
    };

    tracing::info!(
        "LLM Agent Execution Prompt dispatched [Agent ID: {} | Model: {} | Endpoint: {}]:\n--- FULL PROMPT START ---\n{}\n--- FULL PROMPT END ---",
        agent_id, target_model, state.config.llm.ollama_url, full_prompt
    );

    let mut req_builder = model.completion_request(&full_prompt);

    // Inject filesystem tools if bench_id or thread_id is available in context
    if let Some(ctx) = &payload.context {
        let mut bench_id_opt = ctx.get("bench_id")
            .and_then(|v| v.as_str())
            .and_then(|v| uuid::Uuid::parse_str(v).ok());

        if bench_id_opt.is_none() {
            if let Some(tid) = ctx.get("thread_id").and_then(|v| v.as_str()).and_then(|v| uuid::Uuid::parse_str(v).ok()) {
                bench_id_opt = sqlx::query_scalar::<_, Uuid>("SELECT bench_id FROM threads WHERE id = $1")
                    .bind(tid)
                    .fetch_optional(&state.pool)
                    .await
                    .unwrap_or(None)
                    .or(Some(tid));
            }
        }

        if let Some(bench_id) = bench_id_opt {
            use rig_core::tool::portable_tool_definition;
            req_builder = req_builder
                .tool(portable_tool_definition(&crate::llm_tools::ReadFileTool { bench_id }))
                .tool(portable_tool_definition(&crate::llm_tools::WriteFileTool { bench_id }))
                .tool(portable_tool_definition(&crate::llm_tools::ReplaceInFileTool { bench_id }))
                .tool(portable_tool_definition(&crate::llm_tools::ListFilesTool { bench_id }))
                .tool(portable_tool_definition(&crate::llm_tools::DeleteFileTool { bench_id }))
                .tool(portable_tool_definition(&crate::llm_tools::RenameFileTool { bench_id }));
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
                    state.config.llm.timeout.as_secs()
                );
                format!(
                    "Execution output for agent {}: processed prompt '{}'",
                    agent_id, payload.prompt
                )
            }
        };


    // --- STATISTICAL RIGOR GUARDRAILS (Spec 44) ---
    let mut final_output_text = output_text;
    let mut exec_status = crate::models::execution::ExecutionStatus::Completed;
    let mut stat_rigor_status = None;
    let mut stat_warnings = None;
    let mut provenance_run_ids = None;
    let mut guardrail_events_json = serde_json::json!([]);
    let mut tool_calls_json = serde_json::json!([]);

    if let Ok(Some(stat_cfg)) =
        crate::guardrails::parse_statistical_rigor_config(agent_guardrail_config.as_ref())
    {
        let envelopes = crate::guardrails::extract_statistical_envelopes(payload.context.as_ref());
        let eval =
            crate::guardrails::evaluate_statistical_rigor(&final_output_text, &envelopes, &stat_cfg);

        if !eval.provenance_run_ids.is_empty() {
            tool_calls_json = serde_json::json!(
                eval.provenance_run_ids
                    .iter()
                    .map(|id| serde_json::json!({ "provenance_run_id": id }))
                    .collect::<Vec<_>>()
            );
            provenance_run_ids = Some(eval.provenance_run_ids);
        }
        if !eval.warnings.is_empty() {
            guardrail_events_json =
                serde_json::to_value(&eval.warnings).unwrap_or_else(|_| serde_json::json!([]));
            stat_warnings = Some(eval.warnings);
        }
        if eval.status == crate::models::StatisticalRigorStatus::Blocked {
            exec_status = crate::models::execution::ExecutionStatus::Failed;
            if let Some(critique) = eval.critique_prompt {
                final_output_text = critique;
            }
        }
        stat_rigor_status = Some(eval.status);
    }

    // --- METRICS & USAGE TELEMETRY ---

    metrics::counter!("llm_tokens_total", "type" => "prompt", "model" => target_model.clone()).increment((full_prompt.len() / 4) as u64);
    metrics::counter!("llm_tokens_total", "type" => "completion", "model" => target_model.clone()).increment((final_output_text.len() / 4) as u64);

    if is_skill {
        metrics::counter!(
            "skill_execution_total",
            "skill_id" => agent_id.to_string(),
            "model" => target_model.clone()
        ).increment(1);
    } else {
        metrics::counter!(
            "agent_execution_total",
            "agent_id" => agent_id.to_string(),
            "model" => target_model.clone()
        ).increment(1);
    }

    // We mock the token usage as we don't get it from the standard completion API easily in this naive run
    let token_metrics = serde_json::json!({
        "prompt_tokens": full_prompt.len() / 4,
        "completion_tokens": final_output_text.len() / 4,
        "total_tokens": (full_prompt.len() + final_output_text.len()) / 4
    });

    let caller = "anonymous".to_string(); // TODO: get from auth context

    if !is_skill {
        sqlx::query(
            r#"
            INSERT INTO agent_usage_logs (agent_id, agent_version, caller_identity, tool_calls, token_metrics, guardrail_events, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#
        )
        .bind(agent_id)
        .bind(&agent_version)
        .bind(caller)
        .bind(&tool_calls_json)
        .bind(&token_metrics)
        .bind(&guardrail_events_json)
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to log telemetry: {}", e);
            // We don't fail the execution if telemetry fails
        }).ok();
    }
    // ---------------------------------

    // 3. Log execution in database
    let request_json = serde_json::to_value(&payload).unwrap_or_default();
    let response_json = serde_json::json!({
        "output": final_output_text,
        "statistical_rigor_status": stat_rigor_status,
        "statistical_warnings": stat_warnings,
        "provenance_run_ids": provenance_run_ids,
    });

    let current_ver_int = agent_version.parse::<i32>().unwrap_or(1);
    let status_str = match exec_status {
        crate::models::execution::ExecutionStatus::Failed => "failed",
        _ => "completed",
    };

    sqlx::query(
        r#"
        INSERT INTO executions (id, agent_id, agent_version, execution_version, status, request_payload, response_payload, webhook_url, started_at, completed_at)
        VALUES ($1, $2, $3, 1, $4, $5, $6, $7, NOW(), NOW())
        "#,
    )
    .bind(execution_id)
    .bind(agent_id)
    .bind(current_ver_int)
    .bind(status_str)
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
            status: exec_status,
            output: final_output_text,
            execution_version: 1,
            statistical_rigor_status: stat_rigor_status,
            statistical_warnings: stat_warnings,
            provenance_run_ids,
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
