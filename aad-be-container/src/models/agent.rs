use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common::default_version;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputGuardrailType {
    PromptInjection,
    PiiRegex,
    MaxInputTokens,
    BlockedKeywords,
    VectorSimilarity,
    ClassifierModel,
    LlmJudge,
    DomainScoping,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OutputGuardrailType {
    SecretRedaction,
    PiiNerRedaction,
    InfraLeakageFilter,
    EnforceJsonSchema,
    MaxOutputTokens,
    BlockedOutputKeywords,
    ToxicityClassifier,
    BrandCompetitorProtection,
    RagGroundingHallucination,
    RefusalOfftopicDetector,
    StructuralFormattingRules,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Agent {
    pub id: Option<Uuid>,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
    #[serde(default)]
    pub attached_agents: Vec<Uuid>,
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
    #[serde(default = "default_version")]
    pub current_version: String,
    pub owner_id: Uuid,
    #[serde(default)]
    pub judge_threshold: f64,
    #[serde(default)]
    pub input_guardrails: Vec<InputGuardrailType>,
    #[serde(default)]
    pub output_guardrails: Vec<OutputGuardrailType>,
    pub guardrail_config: Option<serde_json::Value>,
    #[serde(default)]
    pub read_groups: Vec<String>,
    #[serde(default)]
    pub write_groups: Vec<String>,
    #[serde(default)]
    pub execute_groups: Vec<String>,
    #[serde(default)]
    pub agent_definition: serde_json::Value,
    #[serde(default)]
    pub model: serde_json::Value,
    #[serde(default)]
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestCase {
    pub input: serde_json::Value,
    pub expected_schema: Option<serde_json::Value>,
    pub rubric: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestAgentRequest {
    pub suite_id: Option<Uuid>,
    pub test_cases: Vec<TestCase>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestAgentResponse {
    pub test_run_id: Uuid,
    pub agent_id: Uuid,
    pub status: String,
    pub average_score: f64,
    pub version_bumped: bool,
    pub new_version: String,
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
    pub current_version: String,
    pub implements_traits: Vec<String>,
    #[serde(default)]
    pub uses_traits: Vec<String>,
    pub tags: Vec<String>,
    pub score: f64,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RefactorAnalyzeRequest {
    pub similarity_threshold: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RefactorAnalyzeResponse {
    pub clusters: Vec<serde_json::Value>,
    pub redundant_agents: Vec<Uuid>,
    pub deliberate_contradictions: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CompileAgentRequest {
    pub root_agent_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DiagnosticMessage {
    pub code: String,
    pub message: String,
    pub severity: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CompileAgentResponse {
    pub status: String,
    pub diagnostics: Vec<DiagnosticMessage>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentContextSearchRequest {
    pub query: String,
    pub depth: Option<usize>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentContextSearchResult {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub field_name: String,
    pub content: String,
    pub score: f64,
    pub match_reason: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SyncEmbeddingsResponse {
    pub status: String,
    pub entity_id: Uuid,
    pub embeddings_created: usize,
}
