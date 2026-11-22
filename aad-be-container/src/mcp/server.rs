//! Unified in-process Agent-As-Data (AAD) Model Context Protocol (MCP) server.
//!
//! Exposes all core AAD capabilities—Agents, Skills, Traits, Registered MCP Tool Servers,
//! Semantic Context Search, Knowledge Base & Graph, Workbenches/Threads/Memory, and
//! Agent Execution—through a single `rmcp` `ToolRouter` backed directly by PostgreSQL
//! and `AppState`. Used both by the HTTP JSON-RPC 2.0 endpoints (`/mcp`, `{api_prefix}/mcp`,
//! `/api/v1/mcp`) and in-process by internal workbench `rig` tools.

use axum::{
    Json,
    extract::{Path, Query, State},
};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerConfig},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    mcp::{McpConfig, McpError},
    models::{
        Agent, AgentSearchRequest, Bench, ExecuteAgentRequest, GraphTraverseRequest,
        GraphTraverseResult, KnowledgeIdRequest, KnowledgeNode, KnowledgeSearchRequest,
        KnowledgeSearchResult, SemanticSearchRequest, Skill, Thread, Tool, TraitContract,
        knowledge::Triple, search::SearchType,
    },
    state::AppState,
    webserver::{
        agents::{DeleteAgentParams, ListAgentsParams},
        knowledge::chunk_text,
    },
};

// ============================================================================
// MCP Tool Request Schemas
// ============================================================================

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListAgentsMcpRequest {
    #[schemars(description = "Optional exact agent name filter.")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetEntityByIdOrNameRequest {
    #[schemars(description = "UUID of the entity to retrieve (provide either id or name).")]
    pub id: Option<Uuid>,
    #[schemars(description = "Exact name of the entity to retrieve (provide either id or name).")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchAgentsMcpRequest {
    #[schemars(description = "Natural-language or keyword search query for agents.")]
    pub query: String,
    #[schemars(description = "Maximum number of matching agents to return (default: 5).")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CreateAgentMcpRequest {
    #[schemars(description = "Unique display name of the agent.")]
    pub name: String,
    #[schemars(description = "Human-readable description of the agent's purpose.")]
    #[serde(default)]
    pub description: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Optional semantic version string (defaults to 1.0.0).")]
    #[serde(default)]
    pub current_version: String,
    #[schemars(description = "Discovery tags associated with the agent.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Trait contract names implemented by this agent.")]
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[schemars(description = "Trait contract names consumed/delegated by this agent.")]
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[schemars(description = "Declarative agent prompt/definition JSON payload.")]
    #[serde(default)]
    pub agent_definition: Value,
    #[schemars(description = "Model configuration JSON payload (e.g. {\"name\": \"llama3.1:8b\"}).")]
    #[serde(default)]
    pub model: Value,
    #[schemars(description = "Minimum LLM-as-a-Judge pass threshold (0.0 - 1.0).")]
    #[serde(default)]
    pub judge_threshold: f64,
    #[schemars(description = "UUIDs of attached MCP tool servers.")]
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached sub-agents.")]
    #[serde(default)]
    pub attached_agents: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached managed skills.")]
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct UpdateAgentMcpRequest {
    #[schemars(description = "UUID of the existing agent to update.")]
    pub id: Uuid,
    #[schemars(description = "Updated agent name.")]
    pub name: String,
    #[schemars(description = "Updated agent description.")]
    #[serde(default)]
    pub description: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Semantic version string.")]
    #[serde(default)]
    pub current_version: String,
    #[schemars(description = "Discovery tags.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Trait contracts implemented by this agent.")]
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[schemars(description = "Trait contracts used by this agent.")]
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[schemars(description = "Declarative agent definition JSON.")]
    #[serde(default)]
    pub agent_definition: Value,
    #[schemars(description = "Model configuration JSON.")]
    #[serde(default)]
    pub model: Value,
    #[schemars(description = "Minimum LLM-as-a-Judge pass threshold.")]
    #[serde(default)]
    pub judge_threshold: f64,
    #[schemars(description = "UUIDs of attached MCP tool servers.")]
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached sub-agents.")]
    #[serde(default)]
    pub attached_agents: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached managed skills.")]
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DeleteAgentMcpRequest {
    #[schemars(description = "UUID of the agent to archive or delete.")]
    pub id: Uuid,
    #[schemars(description = "If true, permanently delete instead of soft-archiving (default: false).")]
    pub hard: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListSkillsMcpRequest {
    #[schemars(description = "Optional substring filter on skill name.")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CreateSkillMcpRequest {
    #[schemars(description = "Unique name of the skill.")]
    pub name: String,
    #[schemars(description = "Summary description of what the skill does.")]
    pub description: String,
    #[schemars(description = "Full markdown/instructional definition of the skill.")]
    #[serde(default)]
    pub definition: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Optional semantic version string.")]
    #[serde(default)]
    pub current_version: String,
    #[schemars(description = "Tags categorizing this skill.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Trait contracts implemented by this skill.")]
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[schemars(description = "Trait contracts consumed by this skill.")]
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[schemars(description = "UUIDs of nested skills.")]
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached MCP tool servers.")]
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct UpdateSkillMcpRequest {
    #[schemars(description = "UUID of the existing skill to update.")]
    pub id: Uuid,
    #[schemars(description = "Updated skill name.")]
    pub name: String,
    #[schemars(description = "Updated skill description.")]
    pub description: String,
    #[schemars(description = "Updated instructional definition.")]
    #[serde(default)]
    pub definition: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Semantic version string.")]
    #[serde(default)]
    pub current_version: String,
    #[schemars(description = "Tags categorizing this skill.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Trait contracts implemented by this skill.")]
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[schemars(description = "Trait contracts consumed by this skill.")]
    #[serde(default)]
    pub uses_traits: Vec<String>,
    #[schemars(description = "UUIDs of nested skills.")]
    #[serde(default)]
    pub attached_skills: Vec<Uuid>,
    #[schemars(description = "UUIDs of attached MCP tool servers.")]
    #[serde(default)]
    pub attached_tools: Vec<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct DeleteByIdRequest {
    #[schemars(description = "UUID of the entity to delete.")]
    pub id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListTraitsMcpRequest {
    #[schemars(description = "Optional zero-based page number (default: 0).")]
    pub page: Option<i64>,
    #[schemars(description = "Optional page size limit (default: 50).")]
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct CreateTraitMcpRequest {
    #[schemars(description = "Unique name of the trait contract.")]
    pub name: String,
    #[schemars(description = "Description of the trait contract.")]
    pub description: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Optional semantic version (defaults to 1.0.0).")]
    #[serde(default)]
    pub version: String,
    #[schemars(description = "Required capabilities for implementing agents/skills.")]
    #[serde(default)]
    pub capability_requirements: Vec<String>,
    #[schemars(description = "Behavioral invariants enforced by this trait.")]
    #[serde(default)]
    pub behavioral_invariants: Vec<String>,
    #[schemars(description = "Evaluation criteria used by LLM judges.")]
    #[serde(default)]
    pub evaluation_criteria: Vec<String>,
    #[schemars(description = "Discovery tags.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Optional guardrail policy JSON configuration.")]
    #[serde(default)]
    pub guardrails: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct UpdateTraitMcpRequest {
    #[schemars(description = "UUID of the trait contract to update.")]
    pub id: Uuid,
    #[schemars(description = "Updated trait name.")]
    pub name: String,
    #[schemars(description = "Updated trait description.")]
    pub description: String,
    #[schemars(description = "UUID of the owning user or service principal.")]
    pub owner_id: Uuid,
    #[schemars(description = "Semantic version string.")]
    #[serde(default)]
    pub version: String,
    #[schemars(description = "Required capabilities.")]
    #[serde(default)]
    pub capability_requirements: Vec<String>,
    #[schemars(description = "Behavioral invariants.")]
    #[serde(default)]
    pub behavioral_invariants: Vec<String>,
    #[schemars(description = "Evaluation criteria.")]
    #[serde(default)]
    pub evaluation_criteria: Vec<String>,
    #[schemars(description = "Discovery tags.")]
    #[serde(default)]
    pub tags: Vec<String>,
    #[schemars(description = "Guardrail policy JSON configuration.")]
    #[serde(default)]
    pub guardrails: Value,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct EmptyMcpRequest {}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct GetMcpServerRequest {
    #[schemars(description = "UUID of the registered MCP tool server.")]
    pub id: Option<Uuid>,
    #[schemars(description = "Exact server_name of the registered MCP tool server.")]
    pub server_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct SearchAgentContextMcpRequest {
    #[schemars(description = "Natural-language task query to search across agents, skills, traits, tools, and knowledge.")]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (default: 5).")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListBenchesMcpRequest {
    #[schemars(description = "Optional owner UUID to filter workbenches.")]
    pub owner_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct BenchIdMcpRequest {
    #[schemars(description = "UUID of the workbench.")]
    pub bench_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ListThreadsMcpRequest {
    #[schemars(description = "Optional bench UUID to scope returned threads.")]
    pub bench_id: Option<Uuid>,
    #[schemars(description = "Maximum number of threads to return (default: 25).")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct UpdateBenchMemoryMcpRequest {
    #[schemars(description = "UUID of the workbench whose working memory should be updated.")]
    pub bench_id: Uuid,
    #[schemars(description = "Updated markdown content for shared working memory.")]
    pub content: String,
    #[schemars(description = "Optional title for the working memory entry.")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExecuteAgentMcpRequest {
    #[schemars(description = "UUID of the agent or skill to execute.")]
    pub agent_id: Uuid,
    #[schemars(description = "Prompt input to send to the agent.")]
    pub prompt: String,
    #[schemars(description = "Optional model override name.")]
    pub model: Option<String>,
    #[schemars(description = "Optional execution context JSON (e.g. {\"bench_id\": \"...\"}).")]
    pub context: Option<Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize, schemars::JsonSchema)]
pub struct ExecutionIdMcpRequest {
    #[schemars(description = "UUID of the execution record to retrieve.")]
    pub id: Uuid,
}

// ============================================================================
// Unified AadMcpServer
// ============================================================================

#[derive(Clone)]
pub struct AadMcpServer {
    pool: PgPool,
    state: Option<AppState>,
    tool_router: ToolRouter<Self>,
}

impl AadMcpServer {
    pub fn new(state: AppState) -> Self {
        let pool = state.pool.clone();
        Self {
            pool,
            state: Some(state),
            tool_router: Self::tool_router(),
        }
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            state: None,
            tool_router: Self::tool_router(),
        }
    }

    pub fn list_tools(&self) -> Vec<rmcp::model::Tool> {
        self.tool_router.list_all()
    }

    pub fn info(&self) -> ServerConfig {
        self.get_info()
    }

    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<String, McpError> {
        macro_rules! dispatch {
            ($req_ty:ty, $method:ident) => {{
                let req: $req_ty = serde_json::from_value(arguments)
                    .map_err(|e| McpError::InvalidArguments(name.to_string(), e.to_string()))?;
                self.$method(Parameters(req))
                    .await
                    .map_err(McpError::Execution)
            }};
        }

        match name {
            // Agents
            "list_agents" => dispatch!(ListAgentsMcpRequest, list_agents),
            "get_agent" => dispatch!(GetEntityByIdOrNameRequest, get_agent),
            "search_agents" => dispatch!(SearchAgentsMcpRequest, search_agents),
            "create_agent" => dispatch!(CreateAgentMcpRequest, create_agent),
            "update_agent" => dispatch!(UpdateAgentMcpRequest, update_agent),
            "delete_agent" => dispatch!(DeleteAgentMcpRequest, delete_agent),
            // Skills
            "list_skills" => dispatch!(ListSkillsMcpRequest, list_skills),
            "get_skill" => dispatch!(GetEntityByIdOrNameRequest, get_skill),
            "create_skill" => dispatch!(CreateSkillMcpRequest, create_skill),
            "update_skill" => dispatch!(UpdateSkillMcpRequest, update_skill),
            "delete_skill" => dispatch!(DeleteByIdRequest, delete_skill),
            // Traits
            "list_traits" => dispatch!(ListTraitsMcpRequest, list_traits),
            "get_trait" => dispatch!(GetEntityByIdOrNameRequest, get_trait),
            "create_trait" => dispatch!(CreateTraitMcpRequest, create_trait),
            "update_trait" => dispatch!(UpdateTraitMcpRequest, update_trait),
            "delete_trait" => dispatch!(DeleteByIdRequest, delete_trait),
            // MCP Tool Servers Registry
            "list_mcp_servers" => dispatch!(EmptyMcpRequest, list_mcp_servers),
            "get_mcp_server" => dispatch!(GetMcpServerRequest, get_mcp_server),
            // Semantic Context Search
            "search_agent_context" => dispatch!(SearchAgentContextMcpRequest, search_agent_context),
            // Knowledge & Graph
            "search_knowledge" => dispatch!(KnowledgeSearchRequest, search_knowledge),
            "read_knowledge" => dispatch!(KnowledgeIdRequest, read_knowledge),
            "ingest_knowledge" => dispatch!(KnowledgeNode, ingest_knowledge),
            "update_knowledge" => dispatch!(KnowledgeNode, update_knowledge),
            "delete_knowledge" => dispatch!(KnowledgeIdRequest, delete_knowledge),
            "traverse_knowledge_graph" => {
                dispatch!(GraphTraverseRequest, traverse_knowledge_graph)
            }
            // Workbenches, Threads & Memory
            "list_benches" => dispatch!(ListBenchesMcpRequest, list_benches),
            "get_bench" => dispatch!(BenchIdMcpRequest, get_bench),
            "list_threads" => dispatch!(ListThreadsMcpRequest, list_threads),
            "read_bench_memory" => dispatch!(BenchIdMcpRequest, read_bench_memory),
            "update_bench_memory" => dispatch!(UpdateBenchMemoryMcpRequest, update_bench_memory),
            // Execution
            "execute_agent" => dispatch!(ExecuteAgentMcpRequest, execute_agent),
            "get_execution" => dispatch!(ExecutionIdMcpRequest, get_execution),
            _ => Err(McpError::ToolNotFound(name.to_string())),
        }
    }
}

#[tool_router]
impl AadMcpServer {
    // ------------------------------------------------------------------------
    // Agents Registry
    // ------------------------------------------------------------------------

    #[tool(description = "Lists active (non-archived) declarative agents in Agent-As-Data, optionally filtered by exact name.")]
    pub async fn list_agents(
        &self,
        Parameters(req): Parameters<ListAgentsMcpRequest>,
    ) -> Result<String, String> {
        let Json(agents) = crate::webserver::agents::list_agents(
            State(self.pool.clone()),
            Query(ListAgentsParams { name: req.name }),
        )
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&agents).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves a declarative agent definition by its UUID or exact name.")]
    pub async fn get_agent(
        &self,
        Parameters(req): Parameters<GetEntityByIdOrNameRequest>,
    ) -> Result<String, String> {
        if let Some(id) = req.id {
            let Json(agent) = crate::webserver::agents::get_agent(State(self.pool.clone()), Path(id))
                .await
                .map_err(|e| e.to_string())?;
            return serde_json::to_string(&agent).map_err(|e| e.to_string());
        }

        let Some(name) = req.name.filter(|n| !n.trim().is_empty()) else {
            return Err("Either 'id' or non-empty 'name' must be provided".to_string());
        };

        let Json(agents) = crate::webserver::agents::list_agents(
            State(self.pool.clone()),
            Query(ListAgentsParams {
                name: Some(name.clone()),
            }),
        )
        .await
        .map_err(|e| e.to_string())?;

        let Some(agent) = agents.into_iter().next() else {
            return Err(format!("Agent '{}' not found", name));
        };
        serde_json::to_string(&agent).map_err(|e| e.to_string())
    }

    #[tool(description = "Searches active agents by natural-language or keyword query across names and descriptions.")]
    pub async fn search_agents(
        &self,
        Parameters(req): Parameters<SearchAgentsMcpRequest>,
    ) -> Result<String, String> {
        if req.query.trim().is_empty() {
            return Err("'query' must not be empty".to_string());
        }
        let Json(results) = crate::webserver::agents::search_agents(
            State(self.pool.clone()),
            Json(AgentSearchRequest {
                query: req.query,
                limit: req.limit,
            }),
        )
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    #[tool(description = "Creates a new declarative agent in the AAD registry and syncs its embeddings.")]
    pub async fn create_agent(
        &self,
        Parameters(req): Parameters<CreateAgentMcpRequest>,
    ) -> Result<String, String> {
        if req.name.trim().is_empty() {
            return Err("'name' must not be empty".to_string());
        }
        let agent = Agent {
            id: None,
            name: req.name,
            description: req.description,
            tags: req.tags,
            implements_traits: req.implements_traits,
            uses_traits: req.uses_traits,
            attached_tools: req.attached_tools,
            attached_agents: req.attached_agents,
            attached_skills: req.attached_skills,
            current_version: req.current_version,
            owner_id: req.owner_id,
            judge_threshold: req.judge_threshold,
            input_guardrails: vec![],
            output_guardrails: vec![],
            guardrail_config: None,
            read_groups: vec![],
            write_groups: vec![],
            execute_groups: vec![],
            agent_definition: req.agent_definition,
            model: req.model,
            archived_at: None,
        };
        let (_, Json(created)) =
            crate::webserver::agents::create_agent(State(self.pool.clone()), Json(agent))
                .await
                .map_err(|e| e.to_string())?;
        serde_json::to_string(&created).map_err(|e| e.to_string())
    }

    #[tool(description = "Updates an existing declarative agent in the AAD registry and refreshes its embeddings.")]
    pub async fn update_agent(
        &self,
        Parameters(req): Parameters<UpdateAgentMcpRequest>,
    ) -> Result<String, String> {
        let agent = Agent {
            id: Some(req.id),
            name: req.name,
            description: req.description,
            tags: req.tags,
            implements_traits: req.implements_traits,
            uses_traits: req.uses_traits,
            attached_tools: req.attached_tools,
            attached_agents: req.attached_agents,
            attached_skills: req.attached_skills,
            current_version: req.current_version,
            owner_id: req.owner_id,
            judge_threshold: req.judge_threshold,
            input_guardrails: vec![],
            output_guardrails: vec![],
            guardrail_config: None,
            read_groups: vec![],
            write_groups: vec![],
            execute_groups: vec![],
            agent_definition: req.agent_definition,
            model: req.model,
            archived_at: None,
        };
        let Json(updated) = crate::webserver::agents::update_agent(
            State(self.pool.clone()),
            Path(req.id),
            Json(agent),
        )
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&updated).map_err(|e| e.to_string())
    }

    #[tool(description = "Archives (soft delete) or permanently deletes an agent by UUID.")]
    pub async fn delete_agent(
        &self,
        Parameters(req): Parameters<DeleteAgentMcpRequest>,
    ) -> Result<String, String> {
        let Json(deleted) = crate::webserver::agents::delete_agent(
            State(self.pool.clone()),
            Path(req.id),
            Query(DeleteAgentParams { hard: req.hard }),
        )
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&deleted).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------------
    // Skills Registry
    // ------------------------------------------------------------------------

    #[tool(description = "Lists managed skills in Agent-As-Data, optionally filtered by a case-insensitive name substring.")]
    pub async fn list_skills(
        &self,
        Parameters(req): Parameters<ListSkillsMcpRequest>,
    ) -> Result<String, String> {
        let Json(mut skills) = crate::webserver::skills::list_skills(State(self.pool.clone()))
            .await
            .map_err(|e| e.to_string())?;
        if let Some(filter) = req.name.filter(|s| !s.trim().is_empty()) {
            let needle = filter.to_lowercase();
            skills.retain(|s| s.name.to_lowercase().contains(&needle));
        }
        serde_json::to_string(&skills).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves a managed skill definition by UUID or exact name.")]
    pub async fn get_skill(
        &self,
        Parameters(req): Parameters<GetEntityByIdOrNameRequest>,
    ) -> Result<String, String> {
        if let Some(id) = req.id {
            let Json(skill) = crate::webserver::skills::get_skill(State(self.pool.clone()), Path(id))
                .await
                .map_err(|e| e.to_string())?;
            return serde_json::to_string(&skill).map_err(|e| e.to_string());
        }

        let Some(name) = req.name.filter(|n| !n.trim().is_empty()) else {
            return Err("Either 'id' or non-empty 'name' must be provided".to_string());
        };

        let skill = sqlx::query_as::<_, Skill>(
            "SELECT id, name, description, definition, tags, current_version, owner_id, attached_skills, attached_tools, input_schema, output_schema, implementation, implements_traits, uses_traits FROM skills WHERE name = $1",
        )
        .bind(&name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching skill '{}': {}", name, e))?;

        let Some(skill) = skill else {
            return Err(format!("Skill '{}' not found", name));
        };
        serde_json::to_string(&skill).map_err(|e| e.to_string())
    }

    #[tool(description = "Creates a new managed skill in Agent-As-Data and syncs its embeddings.")]
    pub async fn create_skill(
        &self,
        Parameters(req): Parameters<CreateSkillMcpRequest>,
    ) -> Result<String, String> {
        if req.name.trim().is_empty() {
            return Err("'name' must not be empty".to_string());
        }
        let skill = Skill {
            id: None,
            name: req.name,
            description: req.description,
            definition: req.definition,
            tags: req.tags,
            owner_id: req.owner_id,
            current_version: req.current_version,
            implements_traits: req.implements_traits,
            uses_traits: req.uses_traits,
            attached_skills: req.attached_skills,
            attached_tools: req.attached_tools,
            input_schema: None,
            output_schema: None,
            implementation: None,
        };
        let (_, Json(created)) =
            crate::webserver::skills::create_skill(State(self.pool.clone()), Json(skill))
                .await
                .map_err(|e| e.to_string())?;
        serde_json::to_string(&created).map_err(|e| e.to_string())
    }

    #[tool(description = "Updates an existing managed skill by UUID.")]
    pub async fn update_skill(
        &self,
        Parameters(req): Parameters<UpdateSkillMcpRequest>,
    ) -> Result<String, String> {
        let skill = Skill {
            id: Some(req.id),
            name: req.name,
            description: req.description,
            definition: req.definition,
            tags: req.tags,
            owner_id: req.owner_id,
            current_version: req.current_version,
            implements_traits: req.implements_traits,
            uses_traits: req.uses_traits,
            attached_skills: req.attached_skills,
            attached_tools: req.attached_tools,
            input_schema: None,
            output_schema: None,
            implementation: None,
        };
        let Json(updated) = crate::webserver::skills::update_skill(
            State(self.pool.clone()),
            Path(req.id),
            Json(skill),
        )
        .await
        .map_err(|e| e.to_string())?;
        serde_json::to_string(&updated).map_err(|e| e.to_string())
    }

    #[tool(description = "Deletes a managed skill by UUID.")]
    pub async fn delete_skill(
        &self,
        Parameters(req): Parameters<DeleteByIdRequest>,
    ) -> Result<String, String> {
        crate::webserver::skills::delete_skill(State(self.pool.clone()), Path(req.id))
            .await
            .map_err(|e| e.to_string())?;
        Ok(json!({ "deleted": true, "id": req.id }).to_string())
    }

    // ------------------------------------------------------------------------
    // Traits Registry
    // ------------------------------------------------------------------------

    #[tool(description = "Lists trait contracts defined in Agent-As-Data.")]
    pub async fn list_traits(
        &self,
        Parameters(req): Parameters<ListTraitsMcpRequest>,
    ) -> Result<String, String> {
        let size = req.size.unwrap_or(50).clamp(1, 200);
        let page = req.page.unwrap_or(0).max(0);
        let offset = page * size;
        let traits = sqlx::query_as::<_, TraitContract>(
            "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at FROM trait_contracts ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(size)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("DB error listing traits: {}", e))?;
        serde_json::to_string(&traits).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves a trait contract by UUID or exact name.")]
    pub async fn get_trait(
        &self,
        Parameters(req): Parameters<GetEntityByIdOrNameRequest>,
    ) -> Result<String, String> {
        if let Some(id) = req.id {
            let Json(t) = crate::webserver::traits::get_trait(State(self.pool.clone()), Path(id))
                .await
                .map_err(|(_, msg)| msg)?;
            return serde_json::to_string(&t).map_err(|e| e.to_string());
        }

        let Some(name) = req.name.filter(|n| !n.trim().is_empty()) else {
            return Err("Either 'id' or non-empty 'name' must be provided".to_string());
        };

        let trait_opt = sqlx::query_as::<_, TraitContract>(
            "SELECT id, name, description, version, capability_requirements, behavioral_invariants, evaluation_criteria, tags, guardrails, owner_id, created_at, updated_at FROM trait_contracts WHERE name = $1",
        )
        .bind(&name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching trait '{}': {}", name, e))?;

        let Some(t) = trait_opt else {
            return Err(format!("Trait '{}' not found", name));
        };
        serde_json::to_string(&t).map_err(|e| e.to_string())
    }

    #[tool(description = "Creates a new trait contract in Agent-As-Data.")]
    pub async fn create_trait(
        &self,
        Parameters(req): Parameters<CreateTraitMcpRequest>,
    ) -> Result<String, String> {
        if req.name.trim().is_empty() {
            return Err("'name' must not be empty".to_string());
        }
        let guardrails = if req.guardrails.is_null() {
            json!({})
        } else {
            req.guardrails
        };
        let contract = TraitContract {
            id: None,
            name: req.name,
            description: req.description,
            version: req.version,
            capability_requirements: req.capability_requirements,
            behavioral_invariants: req.behavioral_invariants,
            evaluation_criteria: req.evaluation_criteria,
            tags: req.tags,
            guardrails,
            owner_id: req.owner_id,
            created_at: None,
            updated_at: None,
        };
        let (_, Json(created)) =
            crate::webserver::traits::create_trait(State(self.pool.clone()), Json(contract))
                .await
                .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&created).map_err(|e| e.to_string())
    }

    #[tool(description = "Updates an existing trait contract by UUID.")]
    pub async fn update_trait(
        &self,
        Parameters(req): Parameters<UpdateTraitMcpRequest>,
    ) -> Result<String, String> {
        let guardrails = if req.guardrails.is_null() {
            json!({})
        } else {
            req.guardrails
        };
        let contract = TraitContract {
            id: Some(req.id),
            name: req.name,
            description: req.description,
            version: req.version,
            capability_requirements: req.capability_requirements,
            behavioral_invariants: req.behavioral_invariants,
            evaluation_criteria: req.evaluation_criteria,
            tags: req.tags,
            guardrails,
            owner_id: req.owner_id,
            created_at: None,
            updated_at: None,
        };
        let Json(updated) = crate::webserver::traits::update_trait(
            State(self.pool.clone()),
            Path(req.id),
            Json(contract),
        )
        .await
        .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&updated).map_err(|e| e.to_string())
    }

    #[tool(description = "Deletes a trait contract by UUID.")]
    pub async fn delete_trait(
        &self,
        Parameters(req): Parameters<DeleteByIdRequest>,
    ) -> Result<String, String> {
        let Json(deleted) =
            crate::webserver::traits::delete_trait(State(self.pool.clone()), Path(req.id))
                .await
                .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&deleted).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------------
    // Registered MCP Tool Servers
    // ------------------------------------------------------------------------

    #[tool(description = "Lists all MCP tool servers registered in Agent-As-Data along with their sync status and cached tool capabilities.")]
    pub async fn list_mcp_servers(
        &self,
        Parameters(_req): Parameters<EmptyMcpRequest>,
    ) -> Result<String, String> {
        let (_, Json(servers)) = crate::webserver::tools::list_tools(State(self.pool.clone()))
            .await
            .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&servers).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves a registered MCP tool server by UUID or server_name.")]
    pub async fn get_mcp_server(
        &self,
        Parameters(req): Parameters<GetMcpServerRequest>,
    ) -> Result<String, String> {
        if let Some(id) = req.id {
            let (_, Json(server)) =
                crate::webserver::tools::get_tool(State(self.pool.clone()), Path(id))
                    .await
                    .map_err(|(_, msg)| msg)?;
            return serde_json::to_string(&server).map_err(|e| e.to_string());
        }

        let Some(server_name) = req.server_name.filter(|s| !s.trim().is_empty()) else {
            return Err("Either 'id' or non-empty 'server_name' must be provided".to_string());
        };

        let server = sqlx::query_as::<_, Tool>(
            "SELECT id, server_name, transport_type, endpoint_config, cached_capabilities, owner_id, sync_policy, sync_status, last_synced_at, last_sync_error FROM tools WHERE server_name = $1",
        )
        .bind(&server_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error fetching MCP server '{}': {}", server_name, e))?;

        let Some(server) = server else {
            return Err(format!("MCP tool server '{}' not found", server_name));
        };
        serde_json::to_string(&server).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------------
    // Unified Semantic Context Search
    // ------------------------------------------------------------------------

    #[tool(description = "Performs unified semantic/hybrid search across Agents, Skills, Traits, Registered MCP Tools, and Knowledge entities.")]
    pub async fn search_agent_context(
        &self,
        Parameters(req): Parameters<SearchAgentContextMcpRequest>,
    ) -> Result<String, String> {
        if req.query.trim().is_empty() {
            return Err("'query' must not be empty".to_string());
        }
        let Json(results) = crate::webserver::search::semantic_search(
            State(self.pool.clone()),
            Json(SemanticSearchRequest {
                query: req.query,
                limit: req.limit,
            }),
        )
        .await
        .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------------
    // Knowledge Base & Graph
    // ------------------------------------------------------------------------

    #[tool(description = "Searches the knowledge graph using a text query.")]
    pub async fn search_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeSearchRequest>,
    ) -> Result<String, String> {
        let limit = req.limit.unwrap_or(5) as i64;
        let pattern = format!("%{}%", req.query);

        let rows = sqlx::query(
            r#"
            SELECT node_id, chunk_index, chunk_text
            FROM knowledge_embeddings
            WHERE chunk_text ILIKE $1
            LIMIT $2
            "#,
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Search error: {}", e))?;

        let results: Vec<KnowledgeSearchResult> = rows
            .into_iter()
            .map(|r| KnowledgeSearchResult {
                node_id: r.get("node_id"),
                chunk_index: r.get("chunk_index"),
                chunk_text: r.get("chunk_text"),
                score: 0.95,
                search_type: SearchType::Semantic,
            })
            .collect();

        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    #[tool(description = "Reads a specific knowledge node by its UUID.")]
    pub async fn read_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeIdRequest>,
    ) -> Result<String, String> {
        let node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            FROM knowledge_nodes
            WHERE id = $1
            "#,
        )
        .bind(req.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        match node {
            Some(n) => serde_json::to_string(&n).map_err(|e| e.to_string()),
            None => Err("Knowledge node not found".to_string()),
        }
    }

    #[tool(description = "Ingests a new knowledge node and returns the created node.")]
    pub async fn ingest_knowledge(
        &self,
        Parameters(payload): Parameters<KnowledgeNode>,
    ) -> Result<String, String> {
        if payload.topic.trim().is_empty() || payload.content.trim().is_empty() {
            return Err("Topic and content are required".to_string());
        }
        let node_id = payload.id.unwrap_or_else(Uuid::new_v4);
        let tags = payload.tags;
        let metadata = if payload.metadata.is_null() {
            json!({})
        } else {
            payload.metadata
        };

        let created_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            INSERT INTO knowledge_nodes (id, topic, title, description, tags, content, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#,
        )
        .bind(node_id)
        .bind(&payload.topic)
        .bind(&payload.title)
        .bind(&payload.description)
        .bind(&tags)
        .bind(&payload.content)
        .bind(metadata)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        let chunks = chunk_text(&payload.content, 200);
        for (idx, chunk) in chunks.iter().enumerate() {
            let chunk_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(chunk_id)
            .bind(node_id)
            .bind(idx as i32)
            .bind(chunk)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Chunk Insert Error: {}", e))?;
        }

        for tuple in payload.tuples {
            let tuple_id = Uuid::new_v4();
            let confidence = tuple.confidence.unwrap_or(1.0);
            sqlx::query(
                r#"
                INSERT INTO knowledge_tuples (id, source_node_id, subject, predicate, object, confidence)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(tuple_id)
            .bind(node_id)
            .bind(tuple.triple.subject)
            .bind(tuple.triple.predicate)
            .bind(tuple.triple.object)
            .bind(confidence)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("Tuple Insert Error: {}", e))?;
        }

        serde_json::to_string(&created_node).map_err(|e| e.to_string())
    }

    #[tool(description = "Updates an existing knowledge node and returns the updated node.")]
    pub async fn update_knowledge(
        &self,
        Parameters(payload): Parameters<KnowledgeNode>,
    ) -> Result<String, String> {
        let id = payload
            .id
            .ok_or("Knowledge node id is required for update")?;

        let current_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            SELECT id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            FROM knowledge_nodes
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        let Some(current_node) = current_node else {
            return Err("Knowledge node not found".to_string());
        };

        let new_topic = if payload.topic.is_empty() {
            current_node.topic
        } else {
            payload.topic
        };
        let new_title = if payload.title.trim().is_empty() {
            current_node.title
        } else {
            payload.title
        };
        let new_description = if payload.description.trim().is_empty() {
            current_node.description
        } else {
            payload.description
        };
        let new_tags = if payload.tags.is_empty() {
            current_node.tags
        } else {
            payload.tags
        };
        let new_content = if payload.content.is_empty() {
            current_node.content.clone()
        } else {
            payload.content
        };
        let new_metadata = if payload.metadata.is_null() || payload.metadata == json!({}) {
            current_node.metadata
        } else {
            payload.metadata
        };

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| format!("Tx Error: {}", e))?;

        let updated_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            UPDATE knowledge_nodes
            SET topic = $1, title = $2, description = $3, tags = $4, content = $5, metadata = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#,
        )
        .bind(new_topic)
        .bind(new_title)
        .bind(new_description)
        .bind(&new_tags)
        .bind(new_content.clone())
        .bind(new_metadata)
        .bind(id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("Update Error: {}", e))?;

        if new_content != current_node.content {
            sqlx::query("DELETE FROM knowledge_embeddings WHERE node_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Delete chunks error: {}", e))?;

            let chunks = chunk_text(&new_content, 200);
            for (idx, chunk) in chunks.iter().enumerate() {
                let chunk_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO knowledge_embeddings (id, node_id, chunk_index, chunk_text)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(chunk_id)
                .bind(id)
                .bind(idx as i32)
                .bind(chunk)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Chunk Insert Error: {}", e))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| format!("Commit Error: {}", e))?;

        serde_json::to_string(&updated_node).map_err(|e| e.to_string())
    }

    #[tool(description = "Deletes a knowledge node by its UUID and returns the deleted node.")]
    pub async fn delete_knowledge(
        &self,
        Parameters(req): Parameters<KnowledgeIdRequest>,
    ) -> Result<String, String> {
        let deleted_node = sqlx::query_as::<_, KnowledgeNode>(
            r#"
            DELETE FROM knowledge_nodes
            WHERE id = $1
            RETURNING id, topic, title, COALESCE(description, '') as description, tags, content, metadata, created_at, updated_at
            "#,
        )
        .bind(req.id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB Error: {}", e))?;

        match deleted_node {
            Some(node) => serde_json::to_string(&node).map_err(|e| e.to_string()),
            None => Err("Knowledge node not found".to_string()),
        }
    }

    #[tool(description = "Traverses SPO relational tuples in the knowledge graph starting from a subject.")]
    pub async fn traverse_knowledge_graph(
        &self,
        Parameters(req): Parameters<GraphTraverseRequest>,
    ) -> Result<String, String> {
        let max_depth = req.max_depth.unwrap_or(2);

        let rows = sqlx::query(
            r#"
            SELECT subject, predicate, object, confidence
            FROM knowledge_tuples
            WHERE subject ILIKE $1 OR object ILIKE $1
            LIMIT 10
            "#,
        )
        .bind(&req.subject)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Traverse Error: {}", e))?;

        let results: Vec<GraphTraverseResult> = rows
            .into_iter()
            .map(|r| GraphTraverseResult {
                triple: Triple::new(
                    r.get::<String, _>("subject"),
                    r.get::<String, _>("predicate"),
                    r.get::<String, _>("object"),
                ),
                confidence: r.get("confidence"),
                depth: max_depth,
            })
            .collect();

        serde_json::to_string(&results).map_err(|e| e.to_string())
    }

    // ------------------------------------------------------------------------
    // Workbenches, Threads & Shared Bench Memory
    // ------------------------------------------------------------------------

    #[tool(description = "Lists workbenches in Agent-As-Data, optionally filtered by owner_id.")]
    pub async fn list_benches(
        &self,
        Parameters(req): Parameters<ListBenchesMcpRequest>,
    ) -> Result<String, String> {
        let benches = if let Some(owner_id) = req.owner_id {
            sqlx::query_as::<_, Bench>(
                "SELECT * FROM benches WHERE owner_id = $1 ORDER BY updated_at DESC",
            )
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Bench>("SELECT * FROM benches ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| format!("DB error listing benches: {}", e))?;
        serde_json::to_string(&benches).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves a workbench by its UUID.")]
    pub async fn get_bench(
        &self,
        Parameters(req): Parameters<BenchIdMcpRequest>,
    ) -> Result<String, String> {
        let bench = sqlx::query_as::<_, Bench>("SELECT * FROM benches WHERE id = $1")
            .bind(req.bench_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| format!("DB error fetching bench {}: {}", req.bench_id, e))?;
        let Some(bench) = bench else {
            return Err(format!("Bench '{}' not found", req.bench_id));
        };
        serde_json::to_string(&bench).map_err(|e| e.to_string())
    }

    #[tool(description = "Lists conversation threads, optionally scoped to a specific bench_id.")]
    pub async fn list_threads(
        &self,
        Parameters(req): Parameters<ListThreadsMcpRequest>,
    ) -> Result<String, String> {
        let limit = req.limit.unwrap_or(25).clamp(1, 200);
        let threads = if let Some(bench_id) = req.bench_id {
            sqlx::query_as::<_, Thread>(
                "SELECT * FROM threads WHERE bench_id = $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(bench_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, Thread>(
                "SELECT * FROM threads ORDER BY created_at DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| format!("DB error listing threads: {}", e))?;
        serde_json::to_string(&threads).map_err(|e| e.to_string())
    }

    #[tool(description = "Reads the shared working memory scratchpad for a specific workbench.")]
    pub async fn read_bench_memory(
        &self,
        Parameters(req): Parameters<BenchIdMcpRequest>,
    ) -> Result<String, String> {
        let row = sqlx::query(
            "SELECT title, content, updated_at FROM bench_memory WHERE bench_id = $1 AND memory_type = 'working' LIMIT 1",
        )
        .bind(req.bench_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error reading bench memory: {}", e))?;

        let Some(r) = row else {
            return Ok(json!({
                "bench_id": req.bench_id,
                "title": "Working Memory",
                "content": ""
            })
            .to_string());
        };

        Ok(json!({
            "bench_id": req.bench_id,
            "title": r.get::<String, _>("title"),
            "content": r.get::<String, _>("content"),
            "updated_at": r.get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
        })
        .to_string())
    }

    #[tool(description = "Upserts the shared working memory scratchpad for a specific workbench.")]
    pub async fn update_bench_memory(
        &self,
        Parameters(req): Parameters<UpdateBenchMemoryMcpRequest>,
    ) -> Result<String, String> {
        let title = req
            .title
            .unwrap_or_else(|| "Working Memory".to_string());
        let existing_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM bench_memory WHERE bench_id = $1 AND memory_type = 'working' LIMIT 1",
        )
        .bind(req.bench_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("DB error checking bench memory: {}", e))?;

        if let Some(id) = existing_id {
            sqlx::query(
                "UPDATE bench_memory SET title = $1, content = $2, updated_at = NOW() WHERE id = $3",
            )
            .bind(&title)
            .bind(&req.content)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error updating bench memory: {}", e))?;
        } else {
            sqlx::query(
                "INSERT INTO bench_memory (id, bench_id, memory_type, title, content, metadata, created_at, updated_at) VALUES ($1, $2, 'working', $3, $4, '{}'::jsonb, NOW(), NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(req.bench_id)
            .bind(&title)
            .bind(&req.content)
            .execute(&self.pool)
            .await
            .map_err(|e| format!("DB error inserting bench memory: {}", e))?;
        }

        Ok(json!({
            "success": true,
            "bench_id": req.bench_id,
            "title": title
        })
        .to_string())
    }

    // ------------------------------------------------------------------------
    // Execution
    // ------------------------------------------------------------------------

    #[tool(description = "Executes a registered agent or skill by UUID with the provided prompt and optional context.")]
    pub async fn execute_agent(
        &self,
        Parameters(req): Parameters<ExecuteAgentMcpRequest>,
    ) -> Result<String, String> {
        let Some(state) = self.state.clone() else {
            return Err("execute_agent requires full AppState runtime configuration".to_string());
        };
        if req.prompt.trim().is_empty() {
            return Err("'prompt' must not be empty".to_string());
        }
        let (_, Json(resp)) = crate::webserver::execution::execute_agent(
            State(state),
            Path(req.agent_id),
            Json(ExecuteAgentRequest {
                prompt: req.prompt,
                model: req.model,
                context: req.context,
                webhook_url: None,
            }),
        )
        .await
        .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&resp).map_err(|e| e.to_string())
    }

    #[tool(description = "Retrieves an agent execution record by execution UUID.")]
    pub async fn get_execution(
        &self,
        Parameters(req): Parameters<ExecutionIdMcpRequest>,
    ) -> Result<String, String> {
        let Json(exec) =
            crate::webserver::execution::get_execution(State(self.pool.clone()), Path(req.id))
                .await
                .map_err(|(_, msg)| msg)?;
        serde_json::to_string(&exec).map_err(|e| e.to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for AadMcpServer {
    fn get_info(&self) -> ServerConfig {
        let instructions = self
            .state
            .as_ref()
            .map(|s| s.config.mcp.instructions.clone())
            .unwrap_or_else(|| McpConfig::default().instructions);
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions(instructions)
    }
}
