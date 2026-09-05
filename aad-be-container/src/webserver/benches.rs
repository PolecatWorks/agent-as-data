use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use uuid::Uuid;

use crate::{
    models::{Bench, CreateBenchRequest, CreateThreadRequest, ListBenchesRequest, PageOptions, Thread, UpdateBenchRequest},
    state::AppState,
    webserver::fs::get_workspace_root,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_benches_get).post(list_benches))
        .route("/create", post(create_bench))
        .route("/{id}", get(get_bench).put(update_bench).delete(delete_bench))
        .route("/{id}/threads", get(list_bench_threads).post(create_bench_thread))
}

pub async fn list_benches_get(
    State(state): State<AppState>,
) -> Result<Json<Vec<Bench>>, (StatusCode, String)> {
    let benches = sqlx::query_as::<_, Bench>("SELECT * FROM benches ORDER BY updated_at DESC")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list benches: {}", e)))?;

    Ok(Json(benches))
}

pub async fn list_benches(
    State(state): State<AppState>,
    Json(payload): Json<ListBenchesRequest>,
) -> Result<Json<Vec<Bench>>, (StatusCode, String)> {
    let mut query_builder = sqlx::QueryBuilder::new("SELECT * FROM benches WHERE owner_id = ");
    query_builder.push_bind(payload.owner_id);
    query_builder.push(" ORDER BY updated_at DESC ");

    let opts = PageOptions::defaulting(payload.pagination.unwrap_or_default());
    query_builder.push(" LIMIT ");
    query_builder.push_bind(opts.size.unwrap());
    query_builder.push(" OFFSET ");
    query_builder.push_bind(opts.page.unwrap() * opts.size.unwrap());

    let benches = query_builder
        .build_query_as::<Bench>()
        .fetch_all(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list benches: {}", e)))?;

    Ok(Json(benches))
}

pub async fn create_bench(
    State(state): State<AppState>,
    Json(payload): Json<CreateBenchRequest>,
) -> Result<(StatusCode, Json<Bench>), (StatusCode, String)> {
    tracing::info!("Creating bench '{}'", payload.name);

    let bench_id = Uuid::new_v4();
    let fs_path = format!("/tmp/workspace/benches/{}", bench_id);

    let bench = sqlx::query_as::<_, Bench>(
        "INSERT INTO benches (id, owner_id, name, description, filesystem_path) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(bench_id)
    .bind(payload.owner_id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&fs_path)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create bench: {}", e)))?;

    // Create the isolated workspace directory for this bench
    let workspace_path = get_workspace_root(bench.id);
    if let Err(e) = std::fs::create_dir_all(&workspace_path) {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create bench workspace directory: {}", e),
        ));
    }

    // Automatically scaffold an initial "General" thread for the bench
    let _ = sqlx::query_as::<_, Thread>(
        "INSERT INTO threads (owner_id, bench_id, title, description, tags) VALUES ($1, $2, $3, $4, $5) RETURNING *"
    )
    .bind(payload.owner_id)
    .bind(bench.id)
    .bind("General")
    .bind("Initial default thread")
    .bind(sqlx::types::Json(vec!["general".to_string()]))
    .fetch_one(&state.pool)
    .await;

    tracing::info!("Bench '{}' created successfully (ID: {})", bench.name, bench.id);

    Ok((StatusCode::CREATED, Json(bench)))
}

pub async fn get_bench(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Bench>, (StatusCode, String)> {
    let bench = sqlx::query_as::<_, Bench>("SELECT * FROM benches WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to get bench: {}", e)))?;

    match bench {
        Some(b) => Ok(Json(b)),
        None => Err((StatusCode::NOT_FOUND, "Bench not found".to_string())),
    }
}

pub async fn update_bench(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBenchRequest>,
) -> Result<Json<Bench>, (StatusCode, String)> {
    tracing::info!("Updating bench ID: {}", id);

    let bench = sqlx::query_as::<_, Bench>(
        "UPDATE benches SET 
            name = COALESCE($1, name), 
            description = COALESCE($2, description), 
            updated_at = NOW() 
         WHERE id = $3 RETURNING *"
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update bench: {}", e)))?;

    match bench {
        Some(b) => {
            tracing::info!("Bench updated successfully (ID: {})", id);
            Ok(Json(b))
        }
        None => Err((StatusCode::NOT_FOUND, "Bench not found".to_string())),
    }
}

pub async fn delete_bench(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Deleting bench ID: {}", id);

    let res = sqlx::query("DELETE FROM benches WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete bench: {}", e)))?;

    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Bench not found".to_string()));
    }

    // Clean up bench workspace directory
    let workspace_path = get_workspace_root(id);
    if workspace_path.exists() {
        let _ = std::fs::remove_dir_all(&workspace_path);
    }

    tracing::info!("Bench deleted successfully (ID: {})", id);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_bench_threads(
    State(state): State<AppState>,
    Path(bench_id): Path<Uuid>,
) -> Result<Json<Vec<Thread>>, (StatusCode, String)> {
    let threads = sqlx::query_as::<_, Thread>(
        "SELECT * FROM threads WHERE bench_id = $1 ORDER BY updated_at DESC, created_at DESC"
    )
    .bind(bench_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to list bench threads: {}", e)))?;

    Ok(Json(threads))
}

pub async fn create_bench_thread(
    State(state): State<AppState>,
    Path(bench_id): Path<Uuid>,
    Json(payload): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), (StatusCode, String)> {
    tracing::info!("Creating thread '{}' under bench {}", payload.title, bench_id);
    let tags_json = payload.tags.map(|t| sqlx::types::Json(t));

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

    Ok((StatusCode::CREATED, Json(thread)))
}
