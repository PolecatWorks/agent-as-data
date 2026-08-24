use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{Thread, CreateThreadRequest, UpdateThreadRequest, Message, CreateMessageRequest, ListThreadsRequest, PageOptions};

pub async fn list_threads(
    State(pool): State<PgPool>,
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

    let threads = query_builder.build_query_as::<Thread>()
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list threads: {}", e)))?;

    Ok(Json(threads))
}

pub async fn create_thread(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), (StatusCode, String)> {
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    let thread = sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (owner_id, title, description, tags) VALUES ($1, $2, $3, $4) RETURNING *"
    )
    .bind(payload.owner_id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .fetch_one(&pool)
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

    Ok((StatusCode::CREATED, Json(thread)))
}

pub async fn get_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Thread>, (StatusCode, String)> {
    let thread = sqlx::query_as::<_, Thread>("SELECT * FROM threads WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get thread: {}", e)))?;

    if let Some(thread) = thread {
        Ok(Json(thread))
    } else {
        Err((StatusCode::NOT_FOUND, "Thread not found".to_string()))
    }
}

pub async fn update_thread(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateThreadRequest>,
) -> Result<Json<Thread>, (StatusCode, String)> {
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

    let thread = sqlx::query_as::<_, Thread>(
        "UPDATE threads SET title = $1, description = $2, tags = $3, updated_at = CURRENT_TIMESTAMP WHERE id = $4 RETURNING *"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(tags_json)
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update thread: {}", e)))?;

    if let Some(thread) = thread {
        Ok(Json(thread))
    } else {
        Err((StatusCode::NOT_FOUND, "Thread not found".to_string()))
    }
}

pub async fn list_messages(
    State(pool): State<PgPool>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE thread_id = $1 ORDER BY created_at ASC"
    )
    .bind(thread_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list messages: {}", e)))?;

    Ok(Json(messages))
}

pub async fn create_message(
    State(pool): State<PgPool>,
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<(StatusCode, Json<Message>), (StatusCode, String)> {
    let message = sqlx::query_as::<_, Message>(
        "INSERT INTO messages (thread_id, role, content) VALUES ($1, $2, $3) RETURNING *"
    )
    .bind(thread_id)
    .bind(&payload.role)
    .bind(&payload.content)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create message: {}", e)))?;

    // Update thread updated_at
    let _ = sqlx::query("UPDATE threads SET updated_at = CURRENT_TIMESTAMP WHERE id = $1")
        .bind(thread_id)
        .execute(&pool)
        .await;

    Ok((StatusCode::CREATED, Json(message)))
}
