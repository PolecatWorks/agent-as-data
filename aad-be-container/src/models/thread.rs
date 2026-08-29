use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::PageOptions;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListThreadsRequest {
    pub owner_id: Uuid,
    pub pagination: Option<PageOptions>,
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<sqlx::types::Json<Vec<String>>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateThreadRequest {
    pub title: String,
    pub owner_id: Uuid,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UpdateThreadRequest {
    pub title: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub role: String,
    pub content: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateMessageRequest {
    pub role: String,
    pub content: String,
}
