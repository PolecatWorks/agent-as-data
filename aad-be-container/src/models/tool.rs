use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolRequest {
    pub id: Option<Uuid>,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub owner_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub cached_tools_count: usize,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Tool {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub cached_capabilities: serde_json::Value,
    pub owner_id: Uuid,
}
