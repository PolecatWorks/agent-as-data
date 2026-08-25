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

fn default_version() -> String {
    "1.0.0".to_string()
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

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Skill {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub definition: String,
    pub tags: Vec<String>,
    pub owner_id: Uuid,
    #[serde(default = "default_version")]
    pub current_version: String,
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
    pub input_schema: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub implementation: Option<serde_json::Value>,
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
pub struct RegisterToolRequest {
    pub id: Option<Uuid>,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub owner_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterToolResponse {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub cached_tools_count: usize,
}

#[derive(sqlx::FromRow, Deserialize, Serialize, Debug, Clone)]
pub struct Tool {
    pub id: Uuid,
    pub server_name: String,
    pub transport_type: String,
    pub endpoint_config: serde_json::Value,
    pub cached_capabilities: serde_json::Value,
    pub owner_id: Uuid,
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

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct TraitContract {
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub capability_requirements: Vec<String>,
    pub behavioral_invariants: Vec<String>,
    pub evaluation_criteria: Vec<String>,
    pub tags: Vec<String>,
    pub guardrails: serde_json::Value,
    pub owner_id: Uuid,
    #[sqlx(default)]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[sqlx(default)]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageOptions {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

impl Default for PageOptions {
    fn default() -> Self {
        Self {
            size: Some(10),
            page: Some(0),
        }
    }
}

impl PageOptions {
    pub fn defaulting(inval: PageOptions) -> PageOptions {
        PageOptions {
            size: Some(inval.size.unwrap_or(10)),
            page: Some(inval.page.unwrap_or(0)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPages {
    pub ids: Vec<Uuid>,
    pub pagination: PageOptions,
}

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

pub fn bump_minor_version(version: &str) -> String {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() >= 2
        && let Ok(major) = parts[0].parse::<i32>()
            && let Ok(minor) = parts[1].parse::<i32>() {
                return format!("{}.{}.0", major, minor + 1);
            }
    // Fallback if not valid SemVer
    let clean = version.trim_matches(|c: char| !c.is_numeric());
    if let Ok(num) = clean.parse::<i32>() {
        return format!("{}.0.0", num + 1);
    }
    "1.1.0".to_string()
}

