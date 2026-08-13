use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{TraitContract, CreateTraitRequest, UpdateTraitRequest};

pub async fn list_traits(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<TraitContract>>, (StatusCode, String)> {
    let traits = sqlx::query_as::<_, TraitContract>(
        "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at FROM trait_contracts ORDER BY created_at DESC"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Fetch Error: {}", e)))?;

    Ok(Json(traits))
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
    Json(payload): Json<CreateTraitRequest>,
) -> Result<(StatusCode, Json<TraitContract>), (StatusCode, String)> {
    let id = Uuid::new_v4();
    let capability_requirements = payload.capability_requirements.unwrap_or_default();
    let behavioral_invariants = payload.behavioral_invariants.unwrap_or_default();
    let evaluation_criteria = payload.evaluation_criteria.unwrap_or_default();
    let tags = payload.tags.unwrap_or_default();
    let guardrails = payload.guardrails.unwrap_or_else(|| serde_json::json!({}));

    let new_trait = sqlx::query_as::<_, TraitContract>(
        r#"
        INSERT INTO trait_contracts (id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails)
        VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8)
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&capability_requirements)
    .bind(&behavioral_invariants)
    .bind(&evaluation_criteria)
    .bind(&tags)
    .bind(&guardrails)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Creation Error: {}", e)))?;

    Ok((StatusCode::CREATED, Json(new_trait)))
}

pub async fn update_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTraitRequest>,
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

    let name = payload.name.unwrap_or(existing.name);
    let description = payload.description.unwrap_or(existing.description);
    let capability_requirements = payload.capability_requirements.unwrap_or(existing.capability_requirements);
    let behavioral_invariants = payload.behavioral_invariants.unwrap_or(existing.behavioral_invariants);
    let evaluation_criteria = payload.evaluation_criteria.unwrap_or(existing.evaluation_criteria);
    let tags = payload.tags.unwrap_or(existing.tags);
    let guardrails = payload.guardrails.unwrap_or(existing.guardrails);
    let new_version = existing.version + 1;

    let updated = sqlx::query_as::<_, TraitContract>(
        r#"
        UPDATE trait_contracts
        SET name = $1, description = $2, version = $3, capability_requirements = $4, behavioral_invariants = $5, evaluation_criteria = $6, tags = $7, guardrails = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, created_at, updated_at
        "#
    )
    .bind(&name)
    .bind(&description)
    .bind(new_version)
    .bind(&capability_requirements)
    .bind(&behavioral_invariants)
    .bind(&evaluation_criteria)
    .bind(&tags)
    .bind(&guardrails)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Update Error: {}", e)))?;

    Ok(Json(updated))
}

pub async fn delete_trait(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM trait_contracts WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Delete Error: {}", e)))?;

    if result.rows_affected() == 0 {
        Err((StatusCode::NOT_FOUND, "Trait contract not found".to_string()))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
