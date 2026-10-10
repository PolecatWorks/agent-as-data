use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncPolicy {
    Manual,
    Auto,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Synced,
    Degraded,
    Failed,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransportType {
    Sse,
    Stdio,
    Http,
}

fn default_sync_policy() -> SyncPolicy {
    SyncPolicy::Manual
}

fn default_sync_status() -> SyncStatus {
    SyncStatus::Synced
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolRequest {
    pub id: Option<Uuid>,
    pub server_name: String,
    pub transport_type: TransportType,
    pub endpoint_config: serde_json::Value,
    pub owner_id: Uuid,
    #[serde(default = "default_sync_policy")]
    pub sync_policy: SyncPolicy,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: TransportType,
    pub cached_tools_count: usize,
    pub sync_status: SyncStatus,
    pub sync_policy: SyncPolicy,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub cached_tools_count: usize,
    pub sync_status: SyncStatus,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_error: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Tool {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: TransportType,
    pub endpoint_config: serde_json::Value,
    pub cached_capabilities: serde_json::Value,
    pub owner_id: Uuid,
    #[serde(default = "default_sync_policy")]
    pub sync_policy: SyncPolicy,
    #[serde(default = "default_sync_status")]
    pub sync_status: SyncStatus,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_error: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestToolRequest {
    pub tool_name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestToolResponse {
    pub success: bool,
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub raw_result: serde_json::Value,
    pub latency_ms: u64,
}
