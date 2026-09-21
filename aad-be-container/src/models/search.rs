use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchResult {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub field_name: String,
    pub content: String,
    pub score: f64,
    pub match_reason: String,
}
