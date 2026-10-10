use super::common::PageOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ListThreadsRequest {
    pub owner_id: Uuid,
    #[serde(flatten)]
    pub pagination: Option<PageOptions>,
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub bench_id: Uuid,
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
    pub bench_id: Option<Uuid>,
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
    pub role: MessageRole,
    pub content: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateMessageRequest {
    pub role: MessageRole,
    pub content: String,
}

impl From<&Message> for rig_core::completion::Message {
    fn from(msg: &Message) -> Self {
        match msg.role {
            MessageRole::User => rig_core::completion::Message::user(&msg.content),
            MessageRole::System => rig_core::completion::Message::system(&msg.content),
            MessageRole::Assistant => rig_core::completion::Message::assistant(&msg.content),
        }
    }
}
impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}
