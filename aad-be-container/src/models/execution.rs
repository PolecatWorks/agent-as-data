use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecuteAgentRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub context: Option<serde_json::Value>,
    pub webhook_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecuteAgentResponse {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub status: ExecutionStatus,
    pub output: String,
    pub execution_version: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchAndExecuteRequest {
    pub task_query: String,
    pub prompt: String,
}
