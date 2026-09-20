use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SemanticSearchResult {
    pub id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub similarity_score: f64,
}
