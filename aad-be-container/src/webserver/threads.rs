use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
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
        .route("/", get(list_threads).post(create_thread))
        .route(
            "/{id}",
            get(get_thread).put(update_thread).delete(delete_thread),
        )
        .route("/{id}/messages", get(list_messages).post(create_message))
        .route("/{id}/runs/active", get(get_active_run))
        .route("/{id}/runs/active/cancel", post(cancel_active_run))
        .route("/{id}/runs", get(list_thread_runs))
}

pub async fn list_threads(
    State(state): State<AppState>,
    Query(payload): Query<ListThreadsRequest>,
) -> Result<Json<Vec<Thread>>, crate::error::AppError> {
    let opts = PageOptions::defaulting(payload.pagination.unwrap_or_default());
    let limit = opts.size.unwrap();
    let offset = opts.page.unwrap() * limit;

    let threads = sqlx::query_as::<_, Thread>(
        "SELECT * FROM threads WHERE owner_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(payload.owner_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(threads))
}

pub async fn create_thread(
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), crate::error::AppError> {
    tracing::info!("Creating thread '{}'", payload.title);
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    // If bench_id not provided, find or create default bench for this owner
    let bench_id = match payload.bench_id {
        Some(bid) => bid,
        None => {
            let default_bench = sqlx::query_as::<_, crate::models::Bench>(
                "SELECT * FROM benches WHERE owner_id = $1 ORDER BY created_at ASC LIMIT 1",
            )
            .bind(payload.owner_id)
            .fetch_optional(&state.pool)
            .await?;

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
                    ?;
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
    ?;

    // Ensure the bench workspace directory exists
    let workspace_path = crate::webserver::fs::get_workspace_root(bench_id);
    if let Err(e) = std::fs::create_dir_all(&workspace_path) {
        return Err(crate::error::AppError::Message(format!(
            "Failed to create bench workspace directory: {}",
            e
        )));
    }

    tracing::info!(
        "Thread '{}' created successfully (ID: {}, Bench: {})",
        thread.title,
        thread.id,
        bench_id
    );

    Ok((StatusCode::CREATED, Json(thread)))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Thread>, crate::error::AppError> {
    let thread = sqlx::query_as::<_, Thread>("SELECT * FROM threads WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(thread))
}

pub async fn update_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Json<Thread>, crate::error::AppError> {
    tracing::info!("Updating thread (ID: {}, title: '{}')", id, payload.title);
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    let thread = sqlx::query_as::<_, Thread>(
        "UPDATE threads SET title = $1, description = $2, tags = $3, updated_at = NOW() WHERE id = $4 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .bind(id)
    .fetch_one(&state.pool)
    .await?;

    tracing::info!("Thread updated successfully (ID: {})", id);
    Ok(Json(thread))
}

pub async fn delete_thread(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, crate::error::AppError> {
    tracing::info!("Deleting thread ID: {}", id);

    let res = sqlx::query("DELETE FROM threads WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if res.rows_affected() == 0 {
        return Err(crate::error::AppError::NotFound(
            "Thread not found".to_string(),
        ));
    }

    tracing::info!("Thread deleted successfully (ID: {})", id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, crate::error::AppError> {
    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE thread_id = $1 ORDER BY created_at ASC",
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(messages))
}

async fn process_thread_message(
    state: &AppState,
    thread_id: Uuid,
    bench_id: Uuid,
    run_id: Option<Uuid>,
    user_content: &str,
    history: &[Message],
) -> Result<String, crate::error::AppError> {
    let workspace_root = crate::webserver::fs::get_workspace_root(bench_id);
    let mut files: Vec<String> = std::fs::read_dir(&workspace_root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    files.sort();

    let files_summary = if files.is_empty() {
        "No files currently exist in this workspace.".to_string()
    } else {
        format!("Current files in workspace: {}", files.join(", "))
    };

    // Fetch bench working memory to include in baseline prompt preamble
    let bench_memory = sqlx::query_scalar::<_, String>(
        "SELECT content FROM bench_memory WHERE bench_id = $1 AND memory_type = 'working' LIMIT 1",
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
        format!(
            "Shared Bench Working Memory:\n\"\"\"\n{}\n\"\"\"",
            bench_memory.trim()
        )
    };

    let system_prompt = format!(
        "You are an AI assistant collaborating with a developer in an isolated workspace (bench {}, thread {}).\n{}\n{}\nYou have filesystem tools (list_files, read_file, write_file, replace_in_file, rename_file, delete_file) and shared memory tools (read_bench_memory, update_bench_memory).\nPlease interpret questions and instructions in the context of the ongoing conversation, and respond helpfully.\n\nIMPORTANT: Files can be modified outside this chat by the user or other processes. ALWAYS use the `read_file` tool to read file contents when asked, rather than relying on your conversation history or memory, to ensure you see the current state of the file.",
        bench_id, thread_id, files_summary, memory_summary
    );

    let rig_history: Vec<rig::completion::Message> = history.iter().map(Into::into).collect();

    tracing::info!(
        "LLM Prompt dispatched [Bench: {} | Thread: {} | Model: {} | Endpoint: {} | Prior turns: {}]:\n--- PREAMBLE ---\n{}\n--- CURRENT PROMPT ---\n{}",
        bench_id,
        thread_id,
        state.config.llm.model,
        state.config.llm.ollama_url,
        rig_history.len(),
        system_prompt,
        user_content
    );

    let client_builder = rig::providers::ollama::Client::builder()
        .base_url(&state.config.llm.ollama_url)
        .api_key(rig_core::client::Nothing);

    let client = client_builder
        .build()
        .map_err(|_| crate::error::AppError::Message("Failed to build Rig client".to_string()))?;

    let agent = client
        .agent(&state.config.llm.model)
        .preamble(&system_prompt)
        .tool(crate::llm_tools::ReadFileTool { bench_id })
        .tool(crate::llm_tools::WriteFileTool { bench_id })
        .tool(crate::llm_tools::ReplaceInFileTool { bench_id })
        .tool(crate::llm_tools::ListFilesTool { bench_id })
        .tool(crate::llm_tools::DeleteFileTool { bench_id })
        .tool(crate::llm_tools::RenameFileTool { bench_id })
        .tool(crate::llm_tools::ReadBenchMemoryTool {
            bench_id,
            pool: state.pool.clone(),
        })
        .tool(crate::llm_tools::UpdateBenchMemoryTool {
            bench_id,
            pool: state.pool.clone(),
        })
        .tool(crate::llm_tools::ListSkillsTool {
            pool: state.pool.clone(),
        })
        .tool(crate::llm_tools::ViewSkillTool {
            pool: state.pool.clone(),
        })
        .default_max_turns(state.config.llm.default_max_turns)
        .build();

    let prompt_future = agent.prompt(user_content).history(rig_history.clone());

    let response = tokio::time::timeout(state.config.llm.timeout, prompt_future).await??;

    // Check if the response contains a raw tool call emitted as text (common with open-weight models like Qwen)
    let trimmed = response.trim();
    let tool_call_json: Option<serde_json::Value> =
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
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
                    tracing::info!(
                        "Run {} was cancelled before executing tool '{}'",
                        rid,
                        tool_name
                    );
                    record_cancellation_message(&state.pool, thread_id, rid).await;
                    return Err(crate::error::AppError::Message("Cancelled".to_string()));
                }
                set_run_phase(&state.pool, rid, "executing_tool", Some(tool_name)).await;
            }

            tracing::info!(
                "Detected raw tool call for '{}' in agent output, executing against bench workspace {}",
                tool_name,
                bench_id
            );
            let tool_result = crate::llm_tools::execute_workspace_tool(
                bench_id,
                tool_name,
                args,
                Some(&state.pool),
            )
            .await;

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

                    let second_prompt_future = agent
                        .prompt("Summarize the result for the user.")
                        .history(followup_history);
                    match tokio::time::timeout(state.config.llm.timeout, second_prompt_future).await
                    {
                        Ok(Ok(final_answer)) => Ok(final_answer),
                        Ok(Err(e)) => {
                            tracing::warn!("Agent follow-up after tool execution failed: {}", e);
                            Ok(format_tool_execution_result(tool_name, &output))
                        }
                        Err(_) => {
                            tracing::warn!("Agent follow-up after tool execution timed out");
                            Ok(format_tool_execution_result(tool_name, &output))
                        }
                    }
                }
                Err(err_msg) => {
                    tracing::warn!("Tool '{}' execution failed: {}", tool_name, err_msg);
                    Ok(format!(
                        "Attempted to execute tool `{}` but encountered an error: {}",
                        tool_name, err_msg
                    ))
                }
            }
        } else {
            Ok(response)
        }
    } else {
        Ok(response)
    }
}

pub async fn create_message(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<CreateMessageResponse>), crate::error::AppError> {
    tracing::info!(
        "Creating message in thread {} (role: {})",
        thread_id,
        payload.role
    );

    // Retrieve previous conversation history before storing new message
    let prior_messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE thread_id = $1 ORDER BY created_at ASC",
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    let message = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (thread_id, role, content) VALUES ($1, $2, $3) RETURNING *",
    )
    .bind(thread_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .fetch_one(&state.pool)
    .await?;

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
        ?;

        let run_id = run.id;
        let state_clone = state.clone();
        let content_clone = payload.content.clone();

        tokio::spawn(async move {
            let assistant_reply_result = process_thread_message(
                &state_clone,
                thread_id,
                bench_id,
                Some(run_id),
                &content_clone,
                &prior_messages,
            )
            .await;

            let parsed_result = match assistant_reply_result {
                Ok(reply) => Ok(reply),
                Err(e) => {
                    let err_str = match e {
                        crate::error::AppError::Message(msg) => msg,
                        other => other.to_string(),
                    };
                    Err(err_str)
                }
            };

            match parsed_result {
                Ok(reply) => {
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
                }
                Err(err_str) => {
                    if is_run_cancelled(&state_clone.pool, run_id).await {
                        record_cancellation_message(&state_clone.pool, thread_id, run_id).await;
                    } else {
                        let _ = sqlx::query(
                            "UPDATE thread_runs SET status = 'failed', current_phase = 'failed', error = $1, updated_at = NOW() WHERE id = $2"
                        )
                        .bind(err_str)
                        .bind(run_id)
                        .execute(&state_clone.pool)
                        .await;
                    }
                }
            }
        });

        Ok((
            StatusCode::ACCEPTED,
            Json(CreateMessageResponse {
                message,
                run_id: Some(run_id),
            }),
        ))
    } else {
        Ok((
            StatusCode::CREATED,
            Json(CreateMessageResponse {
                message,
                run_id: None,
            }),
        ))
    }
}

pub async fn get_active_run(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<(StatusCode, Json<Option<ThreadRun>>), crate::error::AppError> {
    let run = sqlx::query_as::<_, ThreadRun>(
        "SELECT * FROM thread_runs WHERE thread_id = $1 AND status IN ('pending', 'running') ORDER BY created_at DESC LIMIT 1"
    )
    .bind(thread_id)
    .fetch_optional(&state.pool)
    .await
    ?;

    match run {
        Some(r) => Ok((StatusCode::OK, Json(Some(r)))),
        None => Ok((StatusCode::NO_CONTENT, Json(None))),
    }
}

pub async fn cancel_active_run(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<CancelRunResponse>, crate::error::AppError> {
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
) -> Result<Json<Vec<ThreadRun>>, crate::error::AppError> {
    let runs = sqlx::query_as::<_, ThreadRun>(
        "SELECT * FROM thread_runs WHERE thread_id = $1 ORDER BY created_at DESC LIMIT 20",
    )
    .bind(thread_id)
    .fetch_all(&state.pool)
    .await?;

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

pub fn format_tool_execution_result(tool_name: &str, output: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(output) {
        if let (Some(success), Some(message)) = (
            val.get("success").and_then(|v| v.as_bool()),
            val.get("message").and_then(|v| v.as_str()),
        ) {
            let status = if success { "success" } else { "failed" };
            return format!("Executed `{}` ({}): {}", tool_name, status, message);
        }
    }
    format!("Executed `{}`: {}", tool_name, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_tool_execution_result_success() {
        let json = r#"{"success":true,"message":"Successfully wrote to ben.md"}"#;
        let formatted = format_tool_execution_result("write_file", json);
        assert_eq!(
            formatted,
            "Executed `write_file` (success): Successfully wrote to ben.md"
        );
    }

    #[test]
    fn test_format_tool_execution_result_failure() {
        let json = r#"{"success":false,"message":"File not found"}"#;
        let formatted = format_tool_execution_result("delete_file", json);
        assert_eq!(formatted, "Executed `delete_file` (failed): File not found");
    }

    #[test]
    fn test_format_tool_execution_result_unstructured() {
        let formatted = format_tool_execution_result("custom_tool", "plain text output");
        assert_eq!(formatted, "Executed `custom_tool`: plain text output");
    }
}
