use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::default_version;

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct TraitContract {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub capability_requirements: Vec<String>,
    pub behavioral_invariants: Vec<String>,
    pub evaluation_criteria: Vec<String>,
    pub tags: Vec<String>,
    pub guardrails: serde_json::Value,
    pub owner_id: Uuid,
    #[sqlx(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
