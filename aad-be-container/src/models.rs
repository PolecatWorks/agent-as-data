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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub implements_traits: Option<Vec<String>>,
    pub owner_id: Uuid,
    pub read_groups: Option<Vec<String>>,
    pub write_groups: Option<Vec<String>>,
    pub execute_groups: Option<Vec<String>>,
    pub agent_definition: serde_json::Value,
    pub model: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub implements_traits: Vec<String>,
    pub current_version: i32,
    pub owner_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentSearchResult {
    pub agent_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub score: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CreateSkillRequest {
    pub name: String,
    pub description: String,
    pub tags: Option<Vec<String>>,
    pub owner_id: Uuid,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub implementation: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SkillResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub current_version: i32,
    pub owner_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerifyContractRequest {
    pub referrer_agent_id: Uuid,
    pub target_agent_id: Uuid,
    pub trait_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VerifyContractResponse {
    pub status: String,
    pub semantic_fit_score: f64,
    pub contract_valid: bool,
}
