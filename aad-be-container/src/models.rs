use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IngestKnowledgeRequest {
    pub topic: String,
    pub title: Option<String>,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
    pub tuples: Option<Vec<KnowledgeTupleInput>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KnowledgeTupleInput {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IngestKnowledgeResponse {
    pub id: Uuid,
    pub topic: String,
    pub chunks_created: usize,
    pub tuples_created: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KnowledgeSearchResult {
    pub node_id: Uuid,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub score: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphTraverseRequest {
    pub subject: String,
    pub max_depth: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphTraverseResult {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub confidence: f64,
    pub depth: usize,
}
