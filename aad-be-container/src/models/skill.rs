use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::default_version;

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Skill {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub definition: String,
    pub tags: Vec<String>,
    pub owner_id: Uuid,
    #[serde(default = "default_version")]
    pub current_version: String,
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub implementation: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewSkillRequest {
    pub current_description: String,
    pub reviewer_skill_id: Option<Uuid>,
    pub skill_name: String,
    pub skill_tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewSkillResponse {
    pub feedback: String,
    pub suggested_rewrite: String,
    pub similar_skills: Vec<SimilarSkillInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SimilarSkillInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub overlap_reasoning: String,
}
