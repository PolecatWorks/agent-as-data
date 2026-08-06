use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub input_guardrails: serde_json::Value,
    pub output_guardrails: serde_json::Value,
    pub agent_definition: serde_json::Value,
    pub model: serde_json::Value,
    pub tools: serde_json::Value,
    pub available_skills: serde_json::Value,
    pub available_agents: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentDto {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub input_guardrails: serde_json::Value,
    #[serde(default)]
    pub output_guardrails: serde_json::Value,
    #[serde(default)]
    pub agent_definition: serde_json::Value,
    #[serde(default)]
    pub model: serde_json::Value,
    #[serde(default)]
    pub tools: serde_json::Value,
    #[serde(default)]
    pub available_skills: serde_json::Value,
    #[serde(default)]
    pub available_agents: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub input_guardrails: Option<serde_json::Value>,
    pub output_guardrails: Option<serde_json::Value>,
    pub agent_definition: Option<serde_json::Value>,
    pub model: Option<serde_json::Value>,
    pub tools: Option<serde_json::Value>,
    pub available_skills: Option<serde_json::Value>,
    pub available_agents: Option<serde_json::Value>,
}
