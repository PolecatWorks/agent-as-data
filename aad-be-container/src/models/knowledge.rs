use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema, sqlx::FromRow)]
pub struct KnowledgeNode {
    #[serde(default)]
    pub id: Option<Uuid>,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default = "chrono::Utc::now")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    #[sqlx(skip)]
    pub tuples: Vec<KnowledgeTupleInput>,
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct KnowledgeIdRequest {
    #[schemars(description = "The UUID of the knowledge node")]
    pub id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema, sqlx::FromRow, PartialEq)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl Triple {
    pub fn new(subject: impl Into<String>, predicate: impl Into<String>, object: impl Into<String>) -> Self {
        Self {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct KnowledgeTupleInput {
    #[serde(flatten)]
    pub triple: Triple,
    #[serde(default)]
    pub confidence: Option<f64>,
}

impl std::ops::Deref for KnowledgeTupleInput {
    type Target = Triple;
    fn deref(&self) -> &Self::Target {
        &self.triple
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct KnowledgeTuple {
    pub id: Uuid,
    pub source_node_id: Option<Uuid>,
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub triple: Triple,
    pub confidence: f64,
    pub traversal_count: i32,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl std::ops::Deref for KnowledgeTuple {
    type Target = Triple;
    fn deref(&self) -> &Self::Target {
        &self.triple
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct KnowledgeSearchResult {
    pub node_id: Uuid,
    pub chunk_index: i32,
    pub chunk_text: String,
    pub score: f64,
    #[serde(default)]
    pub search_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct GraphTraverseRequest {
    pub subject: String,
    pub max_depth: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone, schemars::JsonSchema)]
pub struct GraphTraverseResult {
    #[serde(flatten)]
    pub triple: Triple,
    pub confidence: f64,
    pub depth: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnalyzeMarkdownRequest {
    pub markdown: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KnowledgeNodeProposal {
    pub topic: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub content: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AnalyzeMarkdownResponse {
    pub proposals: Vec<KnowledgeNodeProposal>,
}

// Deprecated type aliases kept for backwards compatibility
pub type IngestKnowledgeRequest = KnowledgeNode;
pub type UpdateKnowledgeRequest = KnowledgeNode;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct IngestKnowledgeResponse {
    pub id: Uuid,
    pub topic: String,
    pub chunks_created: usize,
    pub tuples_created: usize,
}
