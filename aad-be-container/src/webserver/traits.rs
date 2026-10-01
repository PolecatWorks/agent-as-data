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
    models::{ListPages, PageOptions, TraitContract},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_traits).post(create_trait))
        .route("/{id}", get(get_trait).put(update_trait).delete(delete_trait))
        .route("/{id}/sync-embeddings", post(sync_trait_embeddings))
}

pub async fn list_traits(
    State(pool): State<PgPool>,
    Query(options): Query<PageOptions>,
) -> Result<Json<ListPages>, (StatusCode, String)> {
    let options = PageOptions::defaulting(options);

    let rows = sqlx::query("SELECT id FROM trait_contracts ORDER BY created_at DESC LIMIT $1 OFFSET $2")
        .bind(options.size)
        .bind(options.page.unwrap_or(0) * options.size.unwrap_or(10))
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    let ids: Vec<Uuid> = rows.iter().map(|r| r.get("id")).collect();

    Ok(Json(ListPages {
        ids,
        pagination: options,
    }))
}

pub async fn get_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TraitContract>, (StatusCode, String)> {
    let trait_opt = sqlx::query_as::<_, TraitContract>(
        "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at FROM trait_contracts WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    match trait_opt {
        Some(t) => Ok(Json(t)),
        None => Err((StatusCode::NOT_FOUND, "Trait contract not found".to_string())),
    }
}

pub async fn create_trait(
    State(pool): State<PgPool>,
    Json(payload): Json<TraitContract>,
) -> Result<(StatusCode, Json<TraitContract>), (StatusCode, String)> {
    let id = payload.id.unwrap_or_else(Uuid::new_v4);
    let version = if payload.version.is_empty() || payload.version == "0" || payload.version == "1" {
        "1.0.0".to_string()
    } else {
        payload.version
    };

    tracing::info!("Creating/Saving trait contract '{}' (ID: {}, version: {})", payload.name, id, version);

    let new_trait = sqlx::query_as::<_, TraitContract>(
        r#"
        INSERT INTO trait_contracts (id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (name) DO UPDATE
            SET description = EXCLUDED.description,
                tags = EXCLUDED.tags,
                guardrails = EXCLUDED.guardrails,
                owner_id = EXCLUDED.owner_id,
                updated_at = NOW()
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&version)
    .bind(&payload.capability_requirements)
    .bind(&payload.behavioral_invariants)
    .bind(&payload.evaluation_criteria)
    .bind(&payload.tags)
    .bind(&payload.guardrails)
    .bind(payload.owner_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Insert Trait Error: {}", e)))?;

    tracing::info!("Trait contract '{}' saved successfully (ID: {})", payload.name, id);

    // Automatically sync embeddings with reverse references
    let inv_str = new_trait.behavioral_invariants.join("; ");
    let crit_str = new_trait.evaluation_criteria.join("; ");
    let cap_str = new_trait.capability_requirements.join("; ");
    let _ = crate::webserver::search::sync_entity_embeddings(
        &pool,
        id,
        "traits",
        &new_trait.name,
        Some(new_trait.description.as_str()),
        &[
            ("invariants", &inv_str),
            ("criteria", &crit_str),
            ("capabilities", &cap_str),
        ],
    )
    .await;

    Ok((StatusCode::CREATED, Json(new_trait)))
}

pub async fn update_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TraitContract>,
) -> Result<Json<TraitContract>, (StatusCode, String)> {
    tracing::info!("Updating trait contract '{}' (ID: {})", payload.name, id);
    let current_version = crate::models::bump_minor_version(&payload.version);

    let updated_trait = sqlx::query_as::<_, TraitContract>(
        r#"
        UPDATE trait_contracts
        SET name = $1, description = $2, version = $3, capability_requirements = $4, behavioral_invariants = $5,
            evaluation_criteria = $6, tags = $7, guardrails = $8, owner_id = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at
        "#
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&current_version)
    .bind(&payload.capability_requirements)
    .bind(&payload.behavioral_invariants)
    .bind(&payload.evaluation_criteria)
    .bind(&payload.tags)
    .bind(&payload.guardrails)
    .bind(payload.owner_id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Trait Error: {}", e)))?;

    tracing::info!("Trait contract '{}' updated successfully (ID: {})", payload.name, id);

    // Automatically sync embeddings with reverse references
    let inv_str = updated_trait.behavioral_invariants.join("; ");
    let crit_str = updated_trait.evaluation_criteria.join("; ");
    let cap_str = updated_trait.capability_requirements.join("; ");
    let _ = crate::webserver::search::sync_entity_embeddings(
        &pool,
        id,
        "traits",
        &updated_trait.name,
        Some(updated_trait.description.as_str()),
        &[
            ("invariants", &inv_str),
            ("criteria", &crit_str),
            ("capabilities", &cap_str),
        ],
    )
    .await;

    Ok(Json(updated_trait))
}

pub async fn delete_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TraitContract>, (StatusCode, String)> {
    tracing::info!("Deleting trait contract (ID: {})", id);
    let deleted = sqlx::query_as::<_, TraitContract>(
        r#"
        DELETE FROM trait_contracts
        WHERE id = $1
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Trait Error: {}", e)))?;

    // Purge embeddings for the deleted trait
    let _ = crate::webserver::search::purge_entity_embeddings(&pool, id).await;

    match deleted {
        Some(t) => Ok(Json(t)),
        None => Err((StatusCode::NOT_FOUND, "Trait not found".to_string())),
    }
}

pub async fn sync_trait_embeddings(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<crate::models::SyncEmbeddingsResponse>, (StatusCode, String)> {
    let t = get_trait(State(pool.clone()), Path(id)).await?.0;
    let inv_str = t.behavioral_invariants.join("; ");
    let crit_str = t.evaluation_criteria.join("; ");
    let cap_str = t.capability_requirements.join("; ");

    let count = crate::webserver::search::sync_entity_embeddings(
        &pool,
        id,
        "traits",
        &t.name,
        Some(t.description.as_str()),
        &[
            ("invariants", &inv_str),
            ("criteria", &crit_str),
            ("capabilities", &cap_str),
        ],
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Sync Error: {}", e)))?;

    Ok(Json(crate::models::SyncEmbeddingsResponse {
        status: "success".to_string(),
        entity_id: id,
        embeddings_created: count,
    }))
}
