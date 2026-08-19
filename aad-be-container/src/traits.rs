use axum::{
    Json,
    extract::{Path, State, Query},
    http::StatusCode,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{TraitContract, PageOptions, ListPages, bump_minor_version};

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
        "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at FROM trait_contracts WHERE id = $1"
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
    let version = if payload.version.is_empty() || payload.version == "0" || payload.version == "1" { "1.0.0".to_string() } else { payload.version };

    let new_trait = sqlx::query_as::<_, TraitContract>(
        r#"
        INSERT INTO trait_contracts (id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(version)
    .bind(&payload.capability_requirements)
    .bind(&payload.behavioral_invariants)
    .bind(&payload.evaluation_criteria)
    .bind(&payload.tags)
    .bind(&payload.guardrails)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Creation Error: {}", e)))?;

    Ok((StatusCode::CREATED, Json(new_trait)))
}

pub async fn update_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TraitContract>,
) -> Result<Json<TraitContract>, (StatusCode, String)> {
    let existing = sqlx::query_as::<_, TraitContract>(
        "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at FROM trait_contracts WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    let existing = match existing {
        Some(t) => t,
        None => return Err((StatusCode::NOT_FOUND, "Trait contract not found".to_string())),
    };

    let new_version = bump_minor_version(&existing.version);

    let updated = sqlx::query_as::<_, TraitContract>(
        r#"
        UPDATE trait_contracts
        SET name = $1, description = $2, version = $3, capability_requirements = $4, behavioral_invariants = $5, evaluation_criteria = $6, tags = $7, guardrails = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at
        "#
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(new_version)
    .bind(&payload.capability_requirements)
    .bind(&payload.behavioral_invariants)
    .bind(&payload.evaluation_criteria)
    .bind(&payload.tags)
    .bind(&payload.guardrails)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Error: {}", e)))?;

    Ok(Json(updated))
}

pub async fn delete_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TraitContract>, (StatusCode, String)> {
    let trait_res = get_trait(State(pool.clone()), Path(id)).await?;

    sqlx::query("DELETE FROM trait_contracts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Error: {}", e)))?;

    Ok(trait_res)
}
