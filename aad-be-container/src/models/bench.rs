use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::PageOptions;

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Bench {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub filesystem_path: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateBenchRequest {
    pub name: String,
    pub owner_id: Uuid,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateBenchRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListBenchesRequest {
    pub owner_id: Uuid,
    pub pagination: Option<PageOptions>,
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct BenchMemory {
    pub id: Uuid,
    pub bench_id: Uuid,
    pub memory_type: String,
    pub title: String,
    pub content: String,
    pub metadata: Option<sqlx::types::Json<serde_json::Value>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpsertBenchMemoryRequest {
    pub content: String,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AppendDecisionRequest {
    pub title: String,
    pub content: String,
    pub thread_id: Option<Uuid>,
}
