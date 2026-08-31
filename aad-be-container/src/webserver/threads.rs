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
        CreateMessageRequest, CreateThreadRequest, ListThreadsRequest, Message, PageOptions,
        Thread, UpdateThreadRequest,
    },
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(list_threads))
        .route("/create", post(create_thread))
        .route("/{id}", get(get_thread).put(update_thread))
        .route("/{id}/messages", get(list_messages).post(create_message))
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

    let thread = sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (owner_id, title, description, tags) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(payload.owner_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create thread: {}", e)))?;

    // Create the isolated workspace directory for this thread
    let workspace_path = format!("/tmp/workspace/{}", thread.id);
    if let Err(e) = std::fs::create_dir_all(&workspace_path) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create workspace directory: {}", e),
        ));
    }

    tracing::info!("Thread '{}' created successfully (ID: {})", thread.title, thread.id);

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

async fn process_thread_message(state: &AppState, thread_id: Uuid, user_content: &str) -> String {
    let workspace_root = crate::webserver::fs::get_workspace_root(thread_id);
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

    let system_prompt = format!(
        "You are an AI assistant in an isolated developer workspace (thread {}). {}\nYou have filesystem tools available (list_files, read_file, write_file, replace_in_file, rename_file, delete_file).",
        thread_id, files_summary
    );

    let full_prompt = format!("System: {}\nUser: {}", system_prompt, user_content);
    tracing::info!(
        "LLM Prompt dispatched [Thread: {} | Model: {} | Endpoint: {}]:\n--- FULL PROMPT START ---\n{}\n--- FULL PROMPT END ---",
        thread_id, state.config.llm.model, state.config.llm.ollama_url, full_prompt
    );

    let builder = rig_core::providers::ollama::Client::builder()
        .base_url(&state.config.llm.ollama_url)
        .api_key(rig_core::client::Nothing);

    let llm_res = if let Ok(client) = builder.build() {
        use rig_core::client::CompletionClient;
        use rig_core::completion::CompletionModel;
        use rig_core::tool::portable_tool_definition;

        let model = client.completion_model(&state.config.llm.model);
        let req = model
            .completion_request(&full_prompt)
            .tool(portable_tool_definition(&crate::llm_tools::ReadFileTool { thread_id }))
            .tool(portable_tool_definition(&crate::llm_tools::WriteFileTool { thread_id }))
            .tool(portable_tool_definition(&crate::llm_tools::ReplaceInFileTool { thread_id }))
            .tool(portable_tool_definition(&crate::llm_tools::ListFilesTool { thread_id }))
            .tool(portable_tool_definition(&crate::llm_tools::DeleteFileTool { thread_id }))
            .tool(portable_tool_definition(&crate::llm_tools::RenameFileTool { thread_id }))
            .build();

        let timeout_secs = std::cmp::min(state.config.llm.timeout_secs, 5);
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        match tokio::time::timeout(timeout_duration, model.completion(req)).await {
            Ok(Ok(response)) => {
                if let rig_core::completion::message::AssistantContent::Text(text) = &response.choice[0] {
                    Some(text.text.clone())
                } else {
                    None
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("Ollama completion failed: {}", e);
                None
            }
            Err(_) => {
                tracing::warn!("Ollama completion timed out after {}s", timeout_secs);
                None
            }
        }
    } else {
        None
    };

    if let Some(text) = llm_res {
        text
    } else {
        let lower = user_content.to_lowercase();
        if lower.contains("file") && (lower.contains("what") || lower.contains("list") || lower.contains("show") || lower.contains("which") || lower.contains("are")) {
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
        }
    }
}

pub async fn create_message(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<Message>), (StatusCode, String)> {
    tracing::info!("Creating message in thread {} (role: {})", thread_id, payload.role);
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
        let assistant_reply = process_thread_message(&state, thread_id, &payload.content).await;
        tracing::info!("Agent response generated for thread {}: {}", thread_id, assistant_reply);

        let _ = sqlx::query(
            "INSERT INTO messages (thread_id, role, content) VALUES ($1, 'assistant', $2)"
        )
        .bind(thread_id)
        .bind(&assistant_reply)
        .execute(&state.pool)
        .await;
    }

    Ok((StatusCode::CREATED, Json(message)))
}
