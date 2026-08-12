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
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub implements_traits: Option<Vec<String>>,
    pub attached_tools: Option<Vec<String>>,
    pub attached_agents: Option<Vec<Uuid>>,
    pub attached_skills: Option<Vec<String>>,
    pub owner_id: Uuid,
    pub read_groups: Option<Vec<String>>,
    pub write_groups: Option<Vec<String>>,
    pub execute_groups: Option<Vec<String>>,
    pub agent_definition: serde_json::Value,
    pub model: Option<serde_json::Value>,
    pub judge_threshold: Option<f64>,
    pub input_guardrails: Option<Vec<InputGuardrailType>>,
    pub output_guardrails: Option<Vec<OutputGuardrailType>>,
    pub guardrail_config: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AgentResponse {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub implements_traits: Vec<String>,
    pub attached_tools: Vec<String>,
    pub attached_agents: Vec<Uuid>,
    pub attached_skills: Vec<String>,
    pub current_version: i32,
    pub owner_id: Uuid,
    pub judge_threshold: f64,
    pub input_guardrails: Vec<InputGuardrailType>,
    pub output_guardrails: Vec<OutputGuardrailType>,
    pub guardrail_config: Option<serde_json::Value>,
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
    pub new_version: i32,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterMcpServerRequest {
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterMcpServerResponse {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub cached_tools_count: usize,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecuteAgentRequest {
    pub prompt: String,
    pub context: Option<serde_json::Value>,
    pub webhook_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExecuteAgentResponse {
    pub execution_id: Uuid,
    pub agent_id: Uuid,
    pub status: String,
    pub output: String,
    pub execution_version: i32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SearchAndExecuteRequest {
    pub task_query: String,
    pub prompt: String,
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

