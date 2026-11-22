//! Declarative YAML manifest models for Traits, Tools, Skills, and Agents.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use crate::cli::error::CtlError;
use crate::models::agent::{Agent, InputGuardrailType, OutputGuardrailType};
use crate::models::common::default_version;
use crate::models::skill::Skill;
use crate::models::tool::{RegisterToolRequest, SyncPolicy, Tool, TransportType};
use crate::models::traits::TraitContract;

/// Canonical registry entity kind managed by `aad-be ctl`.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceKind {
    #[value(alias = "traits")]
    Trait,
    #[value(alias = "tools")]
    Tool,
    #[value(alias = "skills")]
    Skill,
    #[value(alias = "agents")]
    Agent,
}

impl ResourceKind {
    /// Parses a user-supplied resource type string (singular or plural, case-insensitive).
    pub fn parse_singular_or_plural(input: &str) -> Result<Self, CtlError> {
        match input.trim().to_ascii_lowercase().as_str() {
            "trait" | "traits" => Ok(Self::Trait),
            "tool" | "tools" => Ok(Self::Tool),
            "skill" | "skills" => Ok(Self::Skill),
            "agent" | "agents" => Ok(Self::Agent),
            _ => Err(CtlError::UnsupportedResourceType(input.to_string())),
        }
    }

    /// Directory name used by `aad-be ctl export --output-dir`.
    pub fn dir_name(self) -> &'static str {
        match self {
            Self::Trait => "traits",
            Self::Tool => "tools",
            Self::Skill => "skills",
            Self::Agent => "agents",
        }
    }

    /// Phase rank for deterministic dependency-ordered hydration.
    pub fn phase_rank(self) -> u8 {
        match self {
            Self::Trait => 0,
            Self::Tool => 1,
            Self::Skill => 2,
            Self::Agent => 3,
        }
    }
}

impl fmt::Display for ResourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trait => write!(f, "Trait"),
            Self::Tool => write!(f, "Tool"),
            Self::Skill => write!(f, "Skill"),
            Self::Agent => write!(f, "Agent"),
        }
    }
}

/// Resource target selector for `get` / `export` commands (includes `all`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSelector {
    Single(ResourceKind),
    All,
}

impl ValueEnum for ResourceSelector {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Single(ResourceKind::Trait),
            Self::Single(ResourceKind::Tool),
            Self::Single(ResourceKind::Skill),
            Self::Single(ResourceKind::Agent),
            Self::All,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Self::Single(ResourceKind::Trait) => {
                clap::builder::PossibleValue::new("trait").alias("traits")
            }
            Self::Single(ResourceKind::Tool) => {
                clap::builder::PossibleValue::new("tool").alias("tools")
            }
            Self::Single(ResourceKind::Skill) => {
                clap::builder::PossibleValue::new("skill").alias("skills")
            }
            Self::Single(ResourceKind::Agent) => {
                clap::builder::PossibleValue::new("agent").alias("agents")
            }
            Self::All => clap::builder::PossibleValue::new("all"),
        })
    }
}

impl ResourceSelector {
    pub fn parse(input: &str) -> Result<Self, CtlError> {
        if input.trim().eq_ignore_ascii_case("all") {
            return Ok(Self::All);
        }
        Ok(Self::Single(ResourceKind::parse_singular_or_plural(input)?))
    }
}

/// Strongly-typed declarative manifest union discriminated by `type:`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceManifest {
    Trait(TraitManifest),
    Tool(ToolManifest),
    Skill(SkillManifest),
    Agent(AgentManifest),
}

impl ResourceManifest {
    pub fn kind(&self) -> ResourceKind {
        match self {
            Self::Trait(_) => ResourceKind::Trait,
            Self::Tool(_) => ResourceKind::Tool,
            Self::Skill(_) => ResourceKind::Skill,
            Self::Agent(_) => ResourceKind::Agent,
        }
    }

    pub fn canonical_name(&self) -> &str {
        match self {
            Self::Trait(t) => &t.name,
            Self::Tool(t) => &t.server_name,
            Self::Skill(s) => &s.name,
            Self::Agent(a) => &a.name,
        }
    }
}

/// Loaded manifest tagged with its source file and 1-based document index.
#[derive(Debug, Clone)]
pub struct LoadedManifest {
    pub source_name: String,
    pub doc_index: usize,
    pub manifest: ResourceManifest,
}

fn default_sync_policy() -> SyncPolicy {
    SyncPolicy::Manual
}

fn default_json_object() -> serde_json::Value {
    serde_json::json!({})
}

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub capability_requirements: Vec<String>,
    #[serde(default)]
    pub behavioral_invariants: Vec<String>,
    #[serde(default)]
    pub evaluation_criteria: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_json_object")]
    pub guardrails: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
}

impl TraitManifest {
    pub fn to_contract(&self, default_owner_id: Uuid) -> TraitContract {
        TraitContract {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            version: self.version.clone(),
            capability_requirements: self.capability_requirements.clone(),
            behavioral_invariants: self.behavioral_invariants.clone(),
            evaluation_criteria: self.evaluation_criteria.clone(),
            tags: self.tags.clone(),
            guardrails: self.guardrails.clone(),
            owner_id: self.owner_id.unwrap_or(default_owner_id),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn from_contract(t: &TraitContract, raw_ids: bool) -> Self {
        Self {
            id: if raw_ids { t.id } else { None },
            name: t.name.clone(),
            description: t.description.clone(),
            version: t.version.clone(),
            capability_requirements: t.capability_requirements.clone(),
            behavioral_invariants: t.behavioral_invariants.clone(),
            evaluation_criteria: t.evaluation_criteria.clone(),
            tags: t.tags.clone(),
            guardrails: t.guardrails.clone(),
            owner_id: Some(t.owner_id),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub server_name: String,
    pub transport_type: TransportType,
    pub endpoint_config: serde_json::Value,
    #[serde(default = "default_sync_policy")]
    pub sync_policy: SyncPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub allow_degraded_sync: bool,
}

impl ToolManifest {
    pub fn to_register_request(
        &self,
        default_owner_id: Uuid,
        cli_allow_degraded: bool,
    ) -> RegisterToolRequest {
        RegisterToolRequest {
            id: self.id,
            server_name: self.server_name.clone(),
            transport_type: self.transport_type.clone(),
            endpoint_config: self.endpoint_config.clone(),
            owner_id: self.owner_id.unwrap_or(default_owner_id),
            sync_policy: self.sync_policy.clone(),
            allow_degraded_sync: self.allow_degraded_sync || cli_allow_degraded,
        }
    }

    pub fn from_tool(t: &Tool, raw_ids: bool) -> Self {
        Self {
            id: if raw_ids { Some(t.id) } else { None },
            server_name: t.server_name.clone(),
            transport_type: t.transport_type.clone(),
            endpoint_config: t.endpoint_config.clone(),
            sync_policy: t.sync_policy.clone(),
            owner_id: Some(t.owner_id),
            allow_degraded_sync: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub definition: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
    #[serde(default = "default_version")]
    pub current_version: String,
    #[serde(default)]
    pub implements_traits: Vec<String>,
    #[serde(default)]
    pub uses_traits: Vec<String>,
    /// Accepts canonical skill names OR raw UUID strings.
    #[serde(default)]
    pub attached_skills: Vec<String>,
    /// Accepts canonical tool server_names OR raw UUID strings.
    #[serde(default)]
    pub attached_tools: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<serde_json::Value>,
}

impl SkillManifest {
    pub fn to_skill(
        &self,
        default_owner_id: Uuid,
        attached_skills: Vec<Uuid>,
        attached_tools: Vec<Uuid>,
    ) -> Skill {
        Skill {
            id: self.id,
            name: self.name.clone(),
            description: self.description.clone(),
            definition: self.definition.clone(),
            tags: self.tags.clone(),
            owner_id: self.owner_id.unwrap_or(default_owner_id),
            current_version: self.current_version.clone(),
            implements_traits: self.implements_traits.clone(),
            uses_traits: self.uses_traits.clone(),
            attached_skills,
            attached_tools,
            input_schema: self.input_schema.clone(),
            output_schema: self.output_schema.clone(),
            implementation: self.implementation.clone(),
        }
    }

    pub fn from_skill(
        s: &Skill,
        tool_id_to_name: &HashMap<Uuid, String>,
        skill_id_to_name: &HashMap<Uuid, String>,
        raw_ids: bool,
    ) -> Self {
        let attached_tools = s
            .attached_tools
            .iter()
            .map(|id| map_id_for_export(*id, tool_id_to_name, raw_ids))
            .collect();
        let attached_skills = s
            .attached_skills
            .iter()
            .map(|id| map_id_for_export(*id, skill_id_to_name, raw_ids))
            .collect();

        Self {
            id: if raw_ids { s.id } else { None },
            name: s.name.clone(),
            description: s.description.clone(),
            definition: s.definition.clone(),
            tags: s.tags.clone(),
            owner_id: Some(s.owner_id),
            current_version: s.current_version.clone(),
            implements_traits: s.implements_traits.clone(),
            uses_traits: s.uses_traits.clone(),
            attached_skills,
            attached_tools,
            input_schema: s.input_schema.clone(),
            output_schema: s.output_schema.clone(),
            implementation: s.implementation.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
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
    /// Accepts canonical tool server_names OR raw UUID strings.
    #[serde(default)]
    pub attached_tools: Vec<String>,
    /// Accepts canonical agent names OR raw UUID strings.
    #[serde(default)]
    pub attached_agents: Vec<String>,
    /// Accepts canonical skill names OR raw UUID strings.
    #[serde(default)]
    pub attached_skills: Vec<String>,
    #[serde(default = "default_version")]
    pub current_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Uuid>,
    #[serde(default)]
    pub judge_threshold: f64,
    #[serde(default)]
    pub input_guardrails: Vec<InputGuardrailType>,
    #[serde(default)]
    pub output_guardrails: Vec<OutputGuardrailType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guardrail_config: Option<serde_json::Value>,
    #[serde(default)]
    pub read_groups: Vec<String>,
    #[serde(default)]
    pub write_groups: Vec<String>,
    #[serde(default)]
    pub execute_groups: Vec<String>,
    #[serde(default = "default_json_object")]
    pub agent_definition: serde_json::Value,
    #[serde(default = "default_json_object")]
    pub model: serde_json::Value,
}

impl AgentManifest {
    pub fn to_agent(
        &self,
        resolved_id: Option<Uuid>,
        default_owner_id: Uuid,
        attached_tools: Vec<Uuid>,
        attached_skills: Vec<Uuid>,
        attached_agents: Vec<Uuid>,
    ) -> Agent {
        Agent {
            id: resolved_id.or(self.id),
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            implements_traits: self.implements_traits.clone(),
            uses_traits: self.uses_traits.clone(),
            attached_tools,
            attached_agents,
            attached_skills,
            current_version: self.current_version.clone(),
            owner_id: self.owner_id.unwrap_or(default_owner_id),
            judge_threshold: self.judge_threshold,
            input_guardrails: self.input_guardrails.clone(),
            output_guardrails: self.output_guardrails.clone(),
            guardrail_config: self.guardrail_config.clone(),
            read_groups: self.read_groups.clone(),
            write_groups: self.write_groups.clone(),
            execute_groups: self.execute_groups.clone(),
            agent_definition: self.agent_definition.clone(),
            model: self.model.clone(),
            archived_at: None,
        }
    }

    pub fn from_agent(
        a: &Agent,
        tool_id_to_name: &HashMap<Uuid, String>,
        skill_id_to_name: &HashMap<Uuid, String>,
        agent_id_to_name: &HashMap<Uuid, String>,
        raw_ids: bool,
    ) -> Self {
        let attached_tools = a
            .attached_tools
            .iter()
            .map(|id| map_id_for_export(*id, tool_id_to_name, raw_ids))
            .collect();
        let attached_skills = a
            .attached_skills
            .iter()
            .map(|id| map_id_for_export(*id, skill_id_to_name, raw_ids))
            .collect();
        let attached_agents = a
            .attached_agents
            .iter()
            .map(|id| map_id_for_export(*id, agent_id_to_name, raw_ids))
            .collect();

        Self {
            id: if raw_ids { a.id } else { None },
            name: a.name.clone(),
            description: a.description.clone(),
            tags: a.tags.clone(),
            implements_traits: a.implements_traits.clone(),
            uses_traits: a.uses_traits.clone(),
            attached_tools,
            attached_agents,
            attached_skills,
            current_version: a.current_version.clone(),
            owner_id: Some(a.owner_id),
            judge_threshold: a.judge_threshold,
            input_guardrails: a.input_guardrails.clone(),
            output_guardrails: a.output_guardrails.clone(),
            guardrail_config: a.guardrail_config.clone(),
            read_groups: a.read_groups.clone(),
            write_groups: a.write_groups.clone(),
            execute_groups: a.execute_groups.clone(),
            agent_definition: a.agent_definition.clone(),
            model: a.model.clone(),
        }
    }
}

fn map_id_for_export(id: Uuid, id_to_name: &HashMap<Uuid, String>, raw_ids: bool) -> String {
    if raw_ids {
        return id.to_string();
    }
    id_to_name
        .get(&id)
        .cloned()
        .unwrap_or_else(|| id.to_string())
}

/// Converts an arbitrary resource name into a clean lowercase filesystem slug.
pub fn slugify_name(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    let mut prev_dash = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    let trimmed = slug.trim_end_matches('-');
    if trimmed.is_empty() {
        "resource".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_name() {
        assert_eq!(
            slugify_name("Semantic Data Steward Agent"),
            "semantic-data-steward-agent"
        );
        assert_eq!(
            slugify_name("Knowledge & Ontology Curator Agent"),
            "knowledge-ontology-curator-agent"
        );
        assert_eq!(slugify_name("___"), "resource");
    }

    #[test]
    fn test_resource_kind_parsing() {
        assert_eq!(
            ResourceKind::parse_singular_or_plural("Traits").unwrap(),
            ResourceKind::Trait
        );
        assert_eq!(
            ResourceKind::parse_singular_or_plural("tool").unwrap(),
            ResourceKind::Tool
        );
        assert_eq!(
            ResourceKind::parse_singular_or_plural("SKILLS").unwrap(),
            ResourceKind::Skill
        );
        assert_eq!(
            ResourceKind::parse_singular_or_plural("agent").unwrap(),
            ResourceKind::Agent
        );
        assert!(ResourceKind::parse_singular_or_plural("bogus").is_err());
    }
}
