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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailEnforcementMode {
    Soft,
    Strict,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StatisticalRigorStatus {
    Passed,
    Warned,
    Blocked,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StatisticalRigorGuardrailConfig {
    pub enabled: bool,
    pub mode: GuardrailEnforcementMode,
    pub require_assumption_acknowledgement: bool,
    pub require_multiple_testing_correction_above: usize,
    pub require_effect_size_and_ci: bool,
    pub flag_unverified_causal_language: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct StatisticalWarning {
    pub gate_code: String,
    pub severity: String,
    pub message: String,
    pub suggested_fix: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecuteAgentResponse {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub status: ExecutionStatus,
    pub output: String,
    pub execution_version: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistical_rigor_status: Option<StatisticalRigorStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statistical_warnings: Option<Vec<StatisticalWarning>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_run_ids: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchAndExecuteRequest {
    pub task_query: String,
    pub prompt: String,
}
