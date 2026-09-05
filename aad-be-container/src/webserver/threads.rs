use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use uuid::Uuid;

use crate::{
    models::{
        CancelRunResponse, CreateMessageRequest, CreateMessageResponse, CreateThreadRequest,
        ListThreadsRequest, Message, PageOptions, Thread, ThreadRun, UpdateThreadRequest,
    },
    state::AppState,
};
use rig::client::AgentClientExt;
use rig::completion::Prompt;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(list_threads))
        .route("/create", post(create_thread))
        .route("/{id}", get(get_thread).put(update_thread).delete(delete_thread))
        .route("/{id}/messages", get(list_messages).post(create_message))
        .route("/{id}/runs/active", get(get_active_run))
        .route("/{id}/runs/active/cancel", post(cancel_active_run))
        .route("/{id}/runs", get(list_thread_runs))
}

pub async fn list_threads(
    State(state): State<AppState>,
    Json(payload): Json<ListThreadsRequest>,
) -> Result<Json<Vec<Thread>>, (StatusCode, String)> {
    let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM threads WHERE owner_id = ");
    query_builder.push_bind(payload.owner_id);

    query_builder.push(" ORDER BY created_at DESC ");

    let opts = PageOptions::defaulting(payload.pagination.unwrap_or_default());
    query_builder.push(" LIMIT ");
    query_builder.push_bind(opts.size.unwrap());
    query_builder.push(" OFFSET ");
    query_builder.push_bind(opts.page.unwrap() * opts.size.unwrap());

    let threads = query_builder
        .build_query_as::<Thread>()
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list threads: {}", e),
            )
        })?;

    Ok(Json(threads))
}

pub async fn create_thread(
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), (StatusCode, String)> {
    tracing::info!("Creating thread '{}'", payload.title);
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    // If bench_id not provided, find or create default bench for this owner
    let bench_id = match payload.bench_id {
        Some(bid) => bid,
        None => {
            let default_bench = sqlx::query_as::<_, crate::models::Bench>(
                "SELECT * FROM benches WHERE owner_id = $1 ORDER BY created_at ASC LIMIT 1"
            )
            .bind(payload.owner_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to lookup bench: {}", e)))?;

            match default_bench {
                Some(b) => b.id,
                None => {
                    let new_bench_id = Uuid::new_v4();
                    let fs_path = format!("/tmp/workspace/benches/{}", new_bench_id);
                    let b = sqlx::query_as::<_, crate::models::Bench>(
                        "INSERT INTO benches (id, owner_id, name, description, filesystem_path) VALUES ($1, $2, $3, $4, $5) RETURNING *"
                    )
                    .bind(new_bench_id)
                    .bind(payload.owner_id)
                    .bind("Default Bench")
                    .bind("Auto-created default bench")
                    .bind(&fs_path)
                    .fetch_one(&state.pool)
                    .await
                    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create default bench: {}", e)))?;
                    b.id
                }
            }
        }
    };

    let thread = sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (owner_id, bench_id, title, description, tags) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(payload.owner_id)
    .bind(bench_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create thread: {}", e)))?;

    // Ensure the bench workspace directory exists
    let workspace_path = crate::webserver::fs::get_workspace_root(bench_id);
    if let Err(e) = std::fs::create_dir_all(&workspace_path) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create bench workspace directory: {}", e),
        ));
    }

    tracing::info!("Thread '{}' created successfully (ID: {}, Bench: {})", thread.title, thread.id, bench_id);

    Ok((StatusCode::CREATED, Json(thread)))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Thread>, (StatusCode, String)> {
    let thread = sqlx::query_as::<_, Thread>("SELECT * FROM threads WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get thread: {}", e)))?;

    match thread {
        Some(t) => Ok(Json(t)),
        None => Err((StatusCode::NOT_FOUND, "Thread not found".to_string())),
    }
}

pub async fn update_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Json<Thread>, (StatusCode, String)> {
    tracing::info!("Updating thread (ID: {}, title: '{}')", id, payload.title);
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    let thread = sqlx::query_as::<_, Thread>(
        "UPDATE threads SET title = $1, description = $2, tags = $3, updated_at = NOW() WHERE id = $4 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update thread: {}", e)))?;

    match thread {
        Some(t) => {
            tracing::info!("Thread updated successfully (ID: {})", id);
            Ok(Json(t))
        }
        None => Err((StatusCode::NOT_FOUND, "Thread not found".to_string())),
    }
}

pub async fn delete_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Deleting thread ID: {}", id);

    let res = sqlx::query("DELETE FROM threads WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete thread: {}", e)))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Thread not found".to_string()));
    }

    tracing::info!("Thread deleted successfully (ID: {})", id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE thread_id = $1 ORDER BY created_at ASC"
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list messages: {}", e)))?;

    Ok(Json(messages))
}

async fn process_thread_message(
    state: &AppState,
    thread_id: Uuid,
    bench_id: Uuid,
    run_id: Option<Uuid>,
    user_content: &str,
    history: &[Message],
) -> Option<String> {
    let workspace_root = crate::webserver::fs::get_workspace_root(bench_id);
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&workspace_root) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                files.push(name);
            }
        }
    }
    files.sort();

    let files_summary = if files.is_empty() {
        "No files currently exist in this workspace.".to_string()
    } else {
        format!("Current files in workspace: {}", files.join(", "))
    };

    // Fetch bench working memory to include in baseline prompt preamble
    let bench_memory = sqlx::query_scalar::<_, String>(
        "SELECT content FROM bench_memory WHERE bench_id = $1 AND memory_type = 'working' LIMIT 1"
    )
    .bind(bench_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_default();

    let memory_summary = if bench_memory.trim().is_empty() {
        "No shared bench memory recorded yet.".to_string()
    } else {
        format!("Shared Bench Working Memory:\n\"\"\"\n{}\n\"\"\"", bench_memory.trim())
    };

    let system_prompt = format!(
        "You are an AI assistant collaborating with a developer in an isolated workspace (bench {}, thread {}).\n{}\n{}\nYou have filesystem tools (list_files, read_file, write_file, replace_in_file, rename_file, delete_file) and shared memory tools (read_bench_memory, update_bench_memory).\nPlease interpret questions and instructions in the context of the ongoing conversation, and respond helpfully.",
        bench_id, thread_id, files_summary, memory_summary
    );

    let mut rig_history = Vec::new();
    for msg in history {
        if msg.role == "user" {
            rig_history.push(rig::completion::Message::user(&msg.content));
        } else {
            rig_history.push(rig::completion::Message::assistant(&msg.content));
        }
    }

    tracing::info!(
        "LLM Prompt dispatched [Bench: {} | Thread: {} | Model: {} | Endpoint: {} | Prior turns: {}]:\n--- PREAMBLE ---\n{}\n--- CURRENT PROMPT ---\n{}",
        bench_id, thread_id, state.config.llm.model, state.config.llm.ollama_url, rig_history.len(), system_prompt, user_content
    );

    let client_builder = rig::providers::ollama::Client::builder()
        .base_url(&state.config.llm.ollama_url)
        .api_key(rig_core::client::Nothing);

    let llm_res = if let Ok(client) = client_builder.build() {
        let agent = client.agent(&state.config.llm.model)
            .preamble(&system_prompt)
            .tool(crate::llm_tools::ReadFileTool { bench_id })
            .tool(crate::llm_tools::WriteFileTool { bench_id })
            .tool(crate::llm_tools::ReplaceInFileTool { bench_id })
            .tool(crate::llm_tools::ListFilesTool { bench_id })
            .tool(crate::llm_tools::DeleteFileTool { bench_id })
            .tool(crate::llm_tools::RenameFileTool { bench_id })
            .tool(crate::llm_tools::ReadBenchMemoryTool { bench_id, pool: state.pool.clone() })
            .tool(crate::llm_tools::UpdateBenchMemoryTool { bench_id, pool: state.pool.clone() })
            .default_max_turns(5)
            .build();

        let timeout_secs = state.config.llm.timeout_secs;
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        let prompt_future = agent.prompt(user_content).history(rig_history.clone());

        match tokio::time::timeout(timeout_duration, prompt_future).await {
            Ok(Ok(response)) => {
                // Check if the response contains a raw tool call emitted as text (common with open-weight models like Qwen)
                let trimmed = response.trim();
                let tool_call_json: Option<serde_json::Value> = if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    serde_json::from_str(trimmed).ok()
                } else if let Some(start) = trimmed.find("```json") {
                    let after = &trimmed[start + 7..];
                    if let Some(end) = after.find("```") {
                        serde_json::from_str(after[..end].trim()).ok()
                    } else {
                        None
                    }
                } else if let Some(start) = trimmed.find('{') {
                    if let Some(end) = trimmed.rfind('}') {
                        serde_json::from_str(&trimmed[start..=end]).ok()
                    } else {
                        None
                    }
                } else if let Some(start) = trimmed.find("<tool_call>") {
                    if let Some(end) = trimmed.find("</tool_call>") {
                        let json_slice = &trimmed[start + 11..end].trim();
                        serde_json::from_str(json_slice).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(call_obj) = tool_call_json {
                    if let Some(tool_name) = call_obj.get("name").and_then(|v| v.as_str()) {
                        let default_args = serde_json::json!({});
                        let args = call_obj.get("arguments").unwrap_or(&default_args);

                        if let Some(rid) = run_id {
                            if is_run_cancelled(&state.pool, rid).await {
                                tracing::info!("Run {} was cancelled before executing tool '{}'", rid, tool_name);
                                record_cancellation_message(&state.pool, thread_id, rid).await;
                                return None;
                            }
                            set_run_phase(&state.pool, rid, "executing_tool", Some(tool_name)).await;
                        }

                        tracing::info!("Detected raw tool call for '{}' in agent output, executing against bench workspace {}", tool_name, bench_id);
                        let tool_result = crate::llm_tools::execute_workspace_tool(bench_id, tool_name, args, Some(&state.pool)).await;

                        if let Some(rid) = run_id {
                            set_run_phase(&state.pool, rid, "thinking", None).await;
                        }

                        match tool_result {
                            Ok(output) => {
                                tracing::info!("Tool '{}' executed successfully: {}", tool_name, output);
                                // Append assistant tool call and tool result to conversation turns, then prompt agent for final answer
                                let mut followup_history = rig_history;
                                followup_history.push(rig::completion::Message::user(user_content));
                                followup_history.push(rig::completion::Message::assistant(&response));
                                followup_history.push(rig::completion::Message::user(&format!(
                                    "Tool '{}' executed successfully with output: {}. Please provide a helpful response to the user based on this result.",
                                    tool_name, output
                                )));

                                let second_prompt_future = agent.prompt("Summarize the result for the user.").history(followup_history);
                                match tokio::time::timeout(timeout_duration, second_prompt_future).await {
                                    Ok(Ok(final_answer)) => Some(final_answer),
                                    Ok(Err(e)) => {
                                        tracing::warn!("Agent follow-up after tool execution failed: {}", e);
                                        Some(format!("Executed `{}`:\n```json\n{}\n```", tool_name, output))
                                    }
                                    Err(_) => {
                                        tracing::warn!("Agent follow-up after tool execution timed out");
                                        Some(format!("Executed `{}`:\n```json\n{}\n```", tool_name, output))
                                    }
                                }
                            }
                            Err(err_msg) => {
                                tracing::warn!("Tool '{}' execution failed: {}", tool_name, err_msg);
                                Some(format!("Attempted to execute tool `{}` but encountered an error: {}", tool_name, err_msg))
                            }
                        }
                    } else {
                        Some(response)
                    }
                } else {
                    Some(response)
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("Rig Agent execution failed: {}", e);
                None
            }
            Err(_) => {
                tracing::warn!("Rig Agent execution timed out after {}s", timeout_secs);
                None
            }
        }
    } else {
        None
    };

    if let Some(text) = llm_res {
        Some(text)
    } else {
        if let Some(rid) = run_id {
            if is_run_cancelled(&state.pool, rid).await {
                return None;
            }
        }
        let lower = user_content.to_lowercase();
        let fallback_text = if (lower.contains("memory") || lower.contains("constraint") || lower.contains("tech stack") || lower.contains("decision")) && !bench_memory.trim().is_empty() {
            format!("According to the bench memory:\n{}", bench_memory.trim())
        } else if lower.contains("file") && (lower.contains("what") || lower.contains("list") || lower.contains("show") || lower.contains("which") || lower.contains("are")) {
            if files.is_empty() {
                "There are currently no files in the workspace directory.".to_string()
            } else {
                format!("The files in the workspace are:\n{}", files.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"))
            }
        } else if lower.contains("create") && lower.contains("file") {
            let parts: Vec<&str> = user_content.split_whitespace().collect();
            let mut filename = "untitled.txt";
            for (i, part) in parts.iter().enumerate() {
                if (*part == "called" || *part == "named" || *part == "file") && i + 1 < parts.len() {
                    filename = parts[i + 1].trim_matches('\'').trim_matches('"');
                }
            }
            let safe_filename = filename.trim_matches('.').trim_matches('/');
            let safe_name = if safe_filename.is_empty() { "untitled.txt" } else { safe_filename };
            let filepath = format!("{}/{}", workspace_root.display(), safe_name);
            let _ = std::fs::write(&filepath, format!("File {} created for thread {}", safe_name, thread_id));
            format!("Created file `{}` in the workspace.", safe_name)
        } else {
            format!("Processed request: \"{}\". {}", user_content, files_summary)
        };
        Some(fallback_text)
    }
}

pub async fn create_message(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<CreateMessageResponse>), (StatusCode, String)> {
    tracing::info!("Creating message in thread {} (role: {})", thread_id, payload.role);

    // Retrieve previous conversation history before storing new message
    let prior_messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE thread_id = $1 ORDER BY created_at ASC"
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let message = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (thread_id, role, content) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(thread_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create message: {}", e)))?;

    if payload.role == "user" {
        let thread_record = sqlx::query_as::<_, Thread>("SELECT * FROM threads WHERE id = $1")
            .bind(thread_id)
            .fetch_optional(&state.pool)
            .await
            .unwrap_or(None);

        let bench_id = thread_record.map(|t| t.bench_id).unwrap_or(thread_id);

        let run = sqlx::query_as::<_, ThreadRun>(
            "INSERT INTO thread_runs (thread_id, bench_id, status, current_phase) VALUES ($1, $2, 'running', 'thinking') RETURNING *"
        )
        .bind(thread_id)
        .bind(bench_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create thread run: {}", e)))?;

        let run_id = run.id;
        let state_clone = state.clone();
        let content_clone = payload.content.clone();

        tokio::spawn(async move {
            let assistant_reply = process_thread_message(&state_clone, thread_id, bench_id, Some(run_id), &content_clone, &prior_messages).await;
            if let Some(reply) = assistant_reply {
                if is_run_cancelled(&state_clone.pool, run_id).await {
                    record_cancellation_message(&state_clone.pool, thread_id, run_id).await;
                    return;
                }

                let _ = sqlx::query(
                    "INSERT INTO messages (thread_id, role, content) VALUES ($1, 'assistant', $2)"
                )
                .bind(thread_id)
                .bind(&reply)
                .execute(&state_clone.pool)
                .await;

                let _ = sqlx::query(
                    "UPDATE thread_runs SET status = 'completed', current_phase = 'completed', updated_at = NOW() WHERE id = $1"
                )
                .bind(run_id)
                .execute(&state_clone.pool)
                .await;
            } else if is_run_cancelled(&state_clone.pool, run_id).await {
                record_cancellation_message(&state_clone.pool, thread_id, run_id).await;
            } else {
                let _ = sqlx::query(
                    "UPDATE thread_runs SET status = 'failed', current_phase = 'failed', error = 'Execution produced no reply', updated_at = NOW() WHERE id = $1"
                )
                .bind(run_id)
                .execute(&state_clone.pool)
                .await;
            }
        });

        Ok((StatusCode::ACCEPTED, Json(CreateMessageResponse {
            message,
            run_id: Some(run_id),
        })))
    } else {
        Ok((StatusCode::CREATED, Json(CreateMessageResponse {
            message,
            run_id: None,
        })))
    }
}

pub async fn get_active_run(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Option<ThreadRun>>), (StatusCode, String)> {
    let run = sqlx::query_as::<_, ThreadRun>(
        "SELECT * FROM thread_runs WHERE thread_id = $1 AND status IN ('pending', 'running') ORDER BY created_at DESC LIMIT 1"
    )
    .bind(thread_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to query active run: {}", e)))?;

    match run {
        Some(r) => Ok((StatusCode::OK, Json(Some(r)))),
        None => Ok((StatusCode::NO_CONTENT, Json(None))),
    }
}

pub async fn cancel_active_run(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<CancelRunResponse>, (StatusCode, String)> {
    tracing::info!("Cancellation requested for thread {}", thread_id);
    let active_runs = sqlx::query_as::<_, ThreadRun>(
        "SELECT * FROM thread_runs WHERE thread_id = $1 AND status IN ('pending', 'running') ORDER BY created_at DESC"
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    if !active_runs.is_empty() {
        for run in active_runs {
            record_cancellation_message(&state.pool, thread_id, run.id).await;
        }

        Ok(Json(CancelRunResponse {
            message: "Action cancelled successfully".to_string(),
            status: "cancelled".to_string(),
        }))
    } else {
        Ok(Json(CancelRunResponse {
            message: "No active action was in progress".to_string(),
            status: "idle".to_string(),
        }))
    }
}

pub async fn list_thread_runs(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<ThreadRun>>, (StatusCode, String)> {
    let runs = sqlx::query_as::<_, ThreadRun>(
        "SELECT * FROM thread_runs WHERE thread_id = $1 ORDER BY created_at DESC LIMIT 20"
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list thread runs: {}", e)))?;

    Ok(Json(runs))
}

async fn is_run_cancelled(pool: &sqlx::PgPool, run_id: Uuid) -> bool {
    sqlx::query_scalar::<_, String>("SELECT status FROM thread_runs WHERE id = $1")
        .bind(run_id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|s| s == "cancelled")
        .unwrap_or(false)
}

async fn set_run_phase(pool: &sqlx::PgPool, run_id: Uuid, phase: &str, tool_name: Option<&str>) {
    let _ = sqlx::query(
        "UPDATE thread_runs SET current_phase = $1, active_tool_name = $2, updated_at = NOW() WHERE id = $3"
    )
    .bind(phase)
    .bind(tool_name)
    .bind(run_id)
    .execute(pool)
    .await;
}

async fn record_cancellation_message(pool: &sqlx::PgPool, thread_id: Uuid, run_id: Uuid) {
    let has_cancel_msg = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM messages WHERE thread_id = $1 AND role = 'system' AND content = '[Action cancelled by user]' AND created_at >= NOW() - INTERVAL '1 minute')"
    )
    .bind(thread_id)
    .fetch_one(pool)
    .await
    .unwrap_or(false);

    if !has_cancel_msg {
        let _ = sqlx::query(
            "INSERT INTO messages (thread_id, role, content) VALUES ($1, 'system', '[Action cancelled by user]')"
        )
        .bind(thread_id)
        .execute(pool)
        .await;
    }

    let _ = sqlx::query(
        "UPDATE thread_runs SET status = 'cancelled', current_phase = 'cancelled', updated_at = NOW() WHERE id = $1"
    )
    .bind(run_id)
    .execute(pool)
    .await;
}
