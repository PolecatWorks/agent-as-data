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
