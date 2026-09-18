use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentUsageLog {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_version: String,
    pub caller_identity: Option<String>,
    pub tool_calls: serde_json::Value,
    pub token_metrics: serde_json::Value,
    pub guardrail_events: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnalyticsUsageResponse {
    pub usage_logs: Vec<AgentUsageLog>,
}
