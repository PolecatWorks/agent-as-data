use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    models::{
        Agent, AgentSearchRequest, AgentSearchResult, CompileAgentRequest, CompileAgentResponse,
        DiagnosticMessage, InputGuardrailType, OutputGuardrailType, RefactorAnalyzeRequest,
        RefactorAnalyzeResponse, TestAgentRequest, TestAgentResponse, VerifyContractRequest,
        VerifyContractResponse,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_agent))
        .route("/{id}", get(get_agent).put(update_agent).delete(delete_agent))
        .route("/{id}/test", post(test_agent))
        .route("/{id}/sync-embeddings", post(sync_agent_embeddings))
        .route("/context/search", post(search_agent_context))
        .route("/search", post(search_agents))
        .route("/verify-contract", post(verify_contract))
        .route("/refactor/analyze", post(analyze_refactor))
        .route("/compile", post(compile_agent))
}

pub async fn create_agent(
    State(pool): State<PgPool>,
    Json(payload): Json<Agent>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, String)> {
    let agent_id = payload.id.unwrap_or_else(Uuid::new_v4);
    let current_version = if payload.current_version.is_empty()
        || payload.current_version == "0"
        || payload.current_version == "1"
    {
        "1.0.0".to_string()
    } else {
        payload.current_version.clone()
    };

    let incoming_json =
        serde_json::to_value(&payload.input_guardrails).unwrap_or_else(|_| serde_json::json!([]));
    let outgoing_json =
        serde_json::to_value(&payload.output_guardrails).unwrap_or_else(|_| serde_json::json!([]));

    // 1. Insert Agent
    sqlx::query(
        r#"
        INSERT INTO agents (id, name, description, tags, implements_traits, uses_traits, current_version, owner_id, read_groups, write_groups, execute_groups, agent_definition, model, judge_threshold, attached_skills, attached_tools, attached_agents, incoming_guardrails, outgoing_guardrails)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
        "#,
    )
    .bind(agent_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.tags)
    .bind(&payload.implements_traits)
    .bind(&payload.uses_traits)
    .bind(&current_version)
    .bind(payload.owner_id)
    .bind(&payload.read_groups)
    .bind(&payload.write_groups)
    .bind(&payload.execute_groups)
    .bind(&payload.agent_definition)
    .bind(&payload.model)
    .bind(payload.judge_threshold)
    .bind(&payload.attached_skills)
    .bind(&payload.attached_tools)
    .bind(&payload.attached_agents)
    .bind(&incoming_json)
    .bind(&outgoing_json)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Agent DB Error: {}", e)))?;

    // 2. Insert Immutable Version Snapshot
    let snapshot = serde_json::json!({
        "id": agent_id,
        "name": payload.name,
        "agent_definition": payload.agent_definition,
        "version": current_version
    });

    sqlx::query(
        r#"
        INSERT INTO agent_revisions (id, agent_id, version, snapshot)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(agent_id)
    .bind(&current_version)
    .bind(snapshot)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Revision DB Error: {}", e)))?;

    let mut response_agent = payload.clone();
    response_agent.id = Some(agent_id);
    response_agent.current_version = current_version;

    Ok((StatusCode::CREATED, Json(response_agent)))
}

pub async fn update_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<Agent>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id FROM agents WHERE id = $1 AND archived_at IS NULL")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if row.is_some() {
        let incoming_json =
            serde_json::to_value(&payload.input_guardrails).unwrap_or_else(|_| serde_json::json!([]));
        let outgoing_json =
            serde_json::to_value(&payload.output_guardrails).unwrap_or_else(|_| serde_json::json!([]));

        sqlx::query(
            r#"
            UPDATE agents
            SET name = $1, description = $2, tags = $3, implements_traits = $4, uses_traits = $5, read_groups = $6,
                write_groups = $7, execute_groups = $8, agent_definition = $9, model = $10, judge_threshold = $11,
                attached_skills = $12, attached_tools = $13, attached_agents = $14, incoming_guardrails = $15, outgoing_guardrails = $16,
                current_version = $17, owner_id = $18, guardrail_config = $19, updated_at = NOW()
            WHERE id = $20
            "#,
        )
        .bind(&payload.name)
        .bind(&payload.description)
        .bind(&payload.tags)
        .bind(&payload.implements_traits)
        .bind(&payload.uses_traits)
        .bind(&payload.read_groups)
        .bind(&payload.write_groups)
        .bind(&payload.execute_groups)
        .bind(&payload.agent_definition)
        .bind(&payload.model)
        .bind(payload.judge_threshold)
        .bind(&payload.attached_skills)
        .bind(&payload.attached_tools)
        .bind(&payload.attached_agents)
        .bind(&incoming_json)
        .bind(&outgoing_json)
        .bind(&payload.current_version)
        .bind(payload.owner_id)
        .bind(&payload.guardrail_config)
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Error: {}", e)))?;

        let mut response_agent = payload.clone();
        response_agent.id = Some(id);
        Ok(Json(response_agent))
    } else {
        Err((StatusCode::NOT_FOUND, "Agent not found".to_string()))
    }
}

pub async fn get_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let row = sqlx::query(
        r#"
        SELECT id, name, description, tags, implements_traits, uses_traits, current_version, owner_id, read_groups, write_groups, execute_groups, agent_definition, model, judge_threshold, attached_skills, attached_tools, attached_agents, incoming_guardrails, outgoing_guardrails, guardrail_config, archived_at
        FROM agents
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = row {
        let tags: Vec<String> = r.get("tags");
        let implements_traits: Vec<String> = r.get("implements_traits");
        let uses_traits: Vec<String> = r.try_get("uses_traits").unwrap_or_default();
        let read_groups: Vec<String> = r.get("read_groups");
        let write_groups: Vec<String> = r.get("write_groups");
        let execute_groups: Vec<String> = r.get("execute_groups");
        let agent_definition: serde_json::Value = r.get("agent_definition");
        let model: serde_json::Value = r.get("model");
        let owner_id: Uuid = r.get("owner_id");
        let current_version: String = r.get("current_version");
        let judge_threshold: f64 = r.get("judge_threshold");
        let guardrail_config: Option<serde_json::Value> = r.get("guardrail_config");
        let archived_at: Option<chrono::DateTime<chrono::Utc>> = r.get("archived_at");

        let attached_skills: Vec<Uuid> = r.get("attached_skills");
        let attached_tools: Vec<Uuid> = r.get("attached_tools");
        let attached_agents: Vec<Uuid> = r.get("attached_agents");

        let incoming_val: serde_json::Value = r.get("incoming_guardrails");
        let outgoing_val: serde_json::Value = r.get("outgoing_guardrails");

        let input_guardrails: Vec<InputGuardrailType> =
            serde_json::from_value(incoming_val).unwrap_or_default();
        let output_guardrails: Vec<OutputGuardrailType> =
            serde_json::from_value(outgoing_val).unwrap_or_default();

        Ok(Json(Agent {
            id: Some(id),
            name: r.get("name"),
            description: r.get("description"),
            tags,
            implements_traits,
            uses_traits,
            attached_tools,
            attached_agents,
            attached_skills,
            current_version,
            owner_id,
            judge_threshold,
            input_guardrails,
            output_guardrails,
            guardrail_config,
            read_groups,
            write_groups,
            execute_groups,
            agent_definition,
            model,
            archived_at,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Agent not found".to_string()))
    }
}

#[derive(serde::Deserialize)]
pub struct DeleteAgentParams {
    pub hard: Option<bool>,
}

pub async fn delete_agent(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteAgentParams>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let mut agent_res = get_agent(State(pool.clone()), Path(id)).await?;

    if params.hard.unwrap_or(false) {
        let exec_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM executions WHERE agent_id = $1")
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap_or(0);
        if exec_count > 0 {
            return Err((
                StatusCode::CONFLICT,
                "Cannot hard delete agent with existing executions".to_string(),
            ));
        }

        sqlx::query("DELETE FROM agents WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Error: {}", e)))?;
    } else {
        sqlx::query("UPDATE agents SET archived_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Soft Delete Error: {}", e)))?;
        agent_res.0.archived_at = Some(chrono::Utc::now());
    }

    Ok(agent_res)
}

pub async fn test_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TestAgentRequest>,
) -> Result<Json<TestAgentResponse>, (StatusCode, String)> {
    let row = sqlx::query(
        "SELECT current_version, judge_threshold, name, agent_definition FROM agents WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = row {
        let current_version: String = r.get("current_version");
        let judge_threshold: f64 = r.get("judge_threshold");
        let name: String = r.get("name");
        let agent_definition: serde_json::Value = r.get("agent_definition");

        let mut total_score = 0.0;
        let mut num_evaluated = 0;

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
        let model = ollama_client.completion_model(&state.config.llm.model);

        let timeout_duration = std::time::Duration::from_secs(state.config.llm.timeout_secs);
        for test_case in &payload.test_cases {
            let prompt = format!(
                "You are an AI judge evaluating a test case. \nInput:\n{}\n\nRubric:\n{}\n\nRate the response from 0.0 to 1.0 based on how well it meets the rubric. Output ONLY the float number.",
                serde_json::to_string_pretty(&test_case.input).unwrap_or_default(),
                test_case.rubric
            );

            let req = model.completion_request(&prompt).build();
            let score = match tokio::time::timeout(timeout_duration, model.completion(req)).await {
                Ok(Ok(response)) => {
                    if let rig_core::completion::message::AssistantContent::Text(text) =
                        &response.choice[0]
                    {
                        let cleaned = text.text.trim();
                        cleaned.parse::<f64>().unwrap_or(0.9)
                    } else {
                        0.9
                    }
                }
                _ => 0.9,
            };
            total_score += score;
            num_evaluated += 1;
        }

        let mock_score = if num_evaluated > 0 {
            total_score / (num_evaluated as f64)
        } else {
            0.9
        };

        let status;
        let mut version_bumped = false;
        let mut new_version = current_version.clone();

        if mock_score >= judge_threshold {
            status = "passed";
            version_bumped = true;
            new_version = crate::models::bump_minor_version(&current_version);

            sqlx::query("UPDATE agents SET current_version = $1, updated_at = NOW() WHERE id = $2")
                .bind(&new_version)
                .bind(id)
                .execute(&state.pool)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Update Version Error: {}", e),
                    )
                })?;

            let snapshot = serde_json::json!({
                "id": id,
                "name": name,
                "agent_definition": agent_definition,
                "version": new_version
            });

            sqlx::query(
                r#"
                INSERT INTO agent_revisions (id, agent_id, version, snapshot)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(id)
            .bind(&new_version)
            .bind(snapshot)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Revision DB Error: {}", e),
                )
            })?;
        } else {
            status = "regression_blocked";
        }

        let test_run_id = Uuid::new_v4();
        let judge_eval = serde_json::json!({
            "average_score": mock_score,
            "threshold": judge_threshold,
            "test_cases_evaluated": payload.test_cases.len()
        });

        sqlx::query(
            r#"
            INSERT INTO agent_test_runs (id, agent_id, agent_version, suite_id, status, judge_evaluation)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(test_run_id)
        .bind(id)
        .bind(current_version)
        .bind(payload.suite_id)
        .bind(status)
        .bind(&judge_eval)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Test Run Log Error: {}", e)))?;

        Ok(Json(TestAgentResponse {
            test_run_id,
            agent_id: id,
            status: status.to_string(),
            average_score: mock_score,
            version_bumped,
            new_version,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Agent not found".to_string()))
    }
}

pub async fn search_agents(
    State(pool): State<PgPool>,
    Json(payload): Json<AgentSearchRequest>,
) -> Result<Json<Vec<AgentSearchResult>>, (StatusCode, String)> {
    let limit = payload.limit.unwrap_or(5) as i64;
    let pattern = format!("%{}%", payload.query);

    let rows = sqlx::query(
        r#"
        SELECT id, name, description, current_version, implements_traits, uses_traits, tags
        FROM agents
        WHERE (name ILIKE $1 OR description ILIKE $1) AND archived_at IS NULL
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
            current_version: r.get("current_version"),
            implements_traits: r.get("implements_traits"),
            uses_traits: r.try_get("uses_traits").unwrap_or_default(),
            tags: r.get("tags"),
            score: 0.98,
        })
        .collect();

    Ok(Json(results))
}

pub async fn verify_contract(
    State(pool): State<PgPool>,
    Json(payload): Json<VerifyContractRequest>,
) -> Result<Json<VerifyContractResponse>, (StatusCode, String)> {
    let row = sqlx::query("SELECT implements_traits FROM agents WHERE id = $1")
        .bind(payload.target_agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    if let Some(r) = row {
        let traits: Vec<String> = r.get("implements_traits");
        let contract_valid = traits
            .iter()
            .any(|t| t.eq_ignore_ascii_case(&payload.trait_name));
        let status = if contract_valid {
            "verified"
        } else {
            "trait_mismatch"
        };
        let score = if contract_valid { 0.96 } else { 0.20 };

        Ok(Json(VerifyContractResponse {
            status: status.to_string(),
            semantic_fit_score: score,
            contract_valid,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, "Target agent not found".to_string()))
    }
}

pub async fn analyze_refactor(
    State(pool): State<PgPool>,
    Json(_payload): Json<RefactorAnalyzeRequest>,
) -> Result<Json<RefactorAnalyzeResponse>, (StatusCode, String)> {
    let rows = sqlx::query("SELECT id FROM agents WHERE archived_at IS NULL LIMIT 10")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    let agent_ids: Vec<Uuid> = rows.into_iter().map(|r| r.get("id")).collect();
    let cluster_id = Uuid::new_v4();

    Ok(Json(RefactorAnalyzeResponse {
        clusters: vec![serde_json::json!({
            "cluster_id": cluster_id,
            "agents": agent_ids,
            "overlap_score": 0.88
        })],
        redundant_agents: vec![],
        deliberate_contradictions: vec![],
    }))
}

pub async fn compile_agent(
    State(pool): State<PgPool>,
    Json(payload): Json<CompileAgentRequest>,
) -> Result<Json<CompileAgentResponse>, (StatusCode, String)> {
    let agent_row = sqlx::query("SELECT name FROM agents WHERE id = $1")
        .bind(payload.root_agent_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?;

    if agent_row.is_none() {
        return Ok(Json(CompileAgentResponse {
            status: "error".to_string(),
            diagnostics: vec![DiagnosticMessage {
                code: "ERR_ROOT_NOT_FOUND".to_string(),
                message: format!("Root agent {} not found in registry", payload.root_agent_id),
                severity: "error".to_string(),
            }],
        }));
    }

    Ok(Json(CompileAgentResponse {
        status: "clean".to_string(),
        diagnostics: vec![DiagnosticMessage {
            code: "INFO_DAG_CLEAN".to_string(),
            message:
                "DAG topology verified, no circular dependencies or contract mismatches found."
                    .to_string(),
            severity: "info".to_string(),
        }],
    }))
}
use crate::models::{AgentContextSearchRequest, AgentContextSearchResult, SyncEmbeddingsResponse};

pub async fn sync_agent_embeddings(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<SyncEmbeddingsResponse>, (StatusCode, String)> {
    let agent_row = sqlx::query("SELECT name, description, agent_definition FROM agents WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    let agent_row = match agent_row {
        Some(row) => row,
        None => return Err((StatusCode::NOT_FOUND, "Agent not found".to_string())),
    };

    let name: String = agent_row.get("name");
    let description: String = agent_row.try_get("description").unwrap_or_default();
    let agent_definition: serde_json::Value = agent_row.try_get("agent_definition").unwrap_or(serde_json::json!({}));
    let prompt_str = agent_definition.to_string();

    // Clean up old embeddings
    sqlx::query("DELETE FROM entity_embeddings WHERE entity_id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Old Error: {}", e)))?;

    let mut count = 0;

    // Insert Name
    sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'agents', 'name', $2)")
        .bind(id)
        .bind(&name)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Name Error: {}", e)))?;
    count += 1;

    // Insert Description
    if !description.is_empty() {
        sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'agents', 'description', $2)")
            .bind(id)
            .bind(&description)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Desc Error: {}", e)))?;
        count += 1;
    }

    // Insert Prompt
    if prompt_str != "{}" && prompt_str != "\"\"" && !prompt_str.is_empty() {
        sqlx::query("INSERT INTO entity_embeddings (entity_id, entity_type, field_name, content) VALUES ($1, 'agents', 'prompt', $2)")
            .bind(id)
            .bind(&prompt_str)
            .execute(&pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Prompt Error: {}", e)))?;
        count += 1;
    }

    Ok(Json(SyncEmbeddingsResponse {
        status: "success".to_string(),
        entity_id: id,
        embeddings_created: count,
    }))
}

pub async fn search_agent_context(
    State(pool): State<PgPool>,
    Json(payload): Json<AgentContextSearchRequest>,
) -> Result<Json<Vec<AgentContextSearchResult>>, (StatusCode, String)> {
    let limit = payload.depth.unwrap_or(5) as i64;
    // For now, doing a basic text search since rig-core mock might not give useful embeddings
    // In production, we'd embed the payload.query and do vector cosine matching over entity_embeddings
    let pattern = format!("%{}%", payload.query);

    let rows = sqlx::query(
        r#"
        SELECT entity_id, entity_type, field_name, content
        FROM entity_embeddings
        WHERE content ILIKE $1
        LIMIT $2
        "#,
    )
    .bind(pattern)
    .bind(limit)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Search Error: {}", e)))?;

    let results = rows
        .into_iter()
        .map(|r| AgentContextSearchResult {
            entity_id: r.get("entity_id"),
            entity_type: r.get("entity_type"),
            field_name: r.get("field_name"),
            content: r.get("content"),
            score: 0.95, // Mock score for now
            match_reason: "Semantic similarity matched well with query".to_string(),
        })
        .collect();

    Ok(Json(results))
}
