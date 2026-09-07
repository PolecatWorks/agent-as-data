use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_sync_policy() -> String {
    "manual".to_string()
}

fn default_sync_status() -> String {
    "synced".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolRequest {
    pub id: Option<Uuid>,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub owner_id: Uuid,
    #[serde(default = "default_sync_policy")]
    pub sync_policy: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub cached_tools_count: usize,
    pub sync_status: String,
    pub sync_policy: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub cached_tools_count: usize,
    pub sync_status: String,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_error: Option<String>,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Tool {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub cached_capabilities: serde_json::Value,
    pub owner_id: Uuid,
    #[serde(default = "default_sync_policy")]
    pub sync_policy: String,
    #[serde(default = "default_sync_status")]
    pub sync_status: String,
    pub last_synced_at: chrono::DateTime<chrono::Utc>,
    pub last_sync_error: Option<String>,
}
