//! Top-level `aad-be ctl` command definitions and execution engine.

use clap::{Args, Subcommand, ValueEnum};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

use crate::cli::client::{AadApiClient, resolve_bearer_token};
use crate::cli::error::CtlError;
use crate::cli::loader::load_manifests;
use crate::cli::manifest::{
    AgentManifest, ResourceKind, ResourceManifest, ResourceSelector, SkillManifest, ToolManifest,
    TraitManifest, slugify_name,
};
use crate::cli::planner::{HydrationPlan, build_hydration_plan};
use crate::cli::template::TemplateVars;
use crate::models::agent::Agent;
use crate::models::skill::Skill;
use crate::models::tool::{Tool, TransportType};
use crate::models::traits::TraitContract;

/// Output serialization format for `aad-be ctl get`.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Yaml,
    Json,
    Table,
}

/// Top-level options and subcommand tree for `aad-be ctl`.
#[derive(Args, Debug)]
pub struct CtlArgs {
    /// Target AAD REST API base URL (e.g. http://localhost:8080/api/aad/v1).
    /// When set, bypasses loading server AppConfig and database secrets.
    #[arg(long, env = "AAD_CTL_URL", global = true)]
    pub url: Option<String>,

    /// Bearer JWT token for Authorization header.
    #[arg(long, env = "AAD_CTL_TOKEN", conflicts_with = "token_file", global = true)]
    pub token: Option<String>,

    /// Path to file containing Bearer JWT token (trimmed).
    #[arg(long, conflicts_with = "token", global = true)]
    pub token_file: Option<PathBuf>,

    /// Fallback owner_id for manifests omitting owner_id.
    #[arg(
        long,
        env = "AAD_CTL_OWNER_ID",
        default_value = "00000000-0000-0000-0000-000000000000",
        global = true
    )]
    pub default_owner_id: Uuid,

    /// HTTP client timeout in seconds.
    #[arg(long, default_value_t = 30, global = true)]
    pub timeout_secs: u64,

    /// Template variable override in KEY=VALUE form (repeatable).
    #[arg(long = "var", value_name = "KEY=VALUE", global = true)]
    pub vars: Vec<String>,

    /// Optional YAML file containing KEY: VALUE template variables.
    #[arg(long, global = true)]
    pub vars_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: CtlCommands,
}

impl CtlArgs {
    /// Returns `true` when the requested subcommand runs purely offline (no network / AppConfig).
    pub fn is_offline(&self) -> bool {
        matches!(self.command, CtlCommands::Validate { .. })
    }
}

/// Supported `aad-be ctl` subcommands.
#[derive(Subcommand, Debug)]
pub enum CtlCommands {
    /// Hydrate (create or update) Traits, Tools, Skills, and Agents from YAML manifests
    Apply {
        /// Manifest file, directory, or '-' for stdin (repeatable)
        #[arg(short, long, required = true)]
        file: Vec<PathBuf>,
        /// Traverse directories recursively for *.yaml / *.yml files
        #[arg(short = 'R', long)]
        recursive: bool,
        /// Validate, resolve references, and print planned actions without mutating AAD
        #[arg(long)]
        dry_run: bool,
        /// Fail fast if any Skill or Agent references a Trait not defined locally or on AAD
        #[arg(long)]
        strict_traits: bool,
        /// Allow unreachable HTTP MCP tools to register in 'degraded' status during bootstrap
        #[arg(long)]
        allow_degraded_tools: bool,
    },
    /// Validate YAML manifests offline (schema, duplicates, DAG cycles, optional --self-contained)
    Validate {
        /// Manifest file, directory, or '-' for stdin (repeatable)
        #[arg(short, long, required = true)]
        file: Vec<PathBuf>,
        /// Traverse directories recursively for *.yaml / *.yml files
        #[arg(short = 'R', long)]
        recursive: bool,
        /// Require all referenced Traits, Tools, Skills, and Agents to exist within these manifests
        #[arg(long)]
        self_contained: bool,
    },
    /// Get details of a resource or list resources (trait, tool, skill, agent, or all)
    Get {
        /// Resource type: trait, tool, skill, agent, or all
        #[arg(value_enum)]
        resource_type: ResourceSelector,
        /// Optional resource UUID or canonical name
        id_or_name: Option<String>,
        /// Output format: yaml, json, or table
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Yaml)]
        output: OutputFormat,
    },
    /// Delete a resource by UUID or canonical name
    Delete {
        /// Resource type: trait, tool, skill, or agent
        #[arg(value_enum)]
        resource_type: ResourceKind,
        /// Resource UUID or canonical name
        id_or_name: String,
        /// Hard-delete an agent (only applies to agents; default is soft-archive)
        #[arg(long)]
        hard: bool,
    },
    /// Export live AAD resources as portable YAML manifests
    Export {
        /// Comma-separated resource types to export (default: trait,tool,skill,agent)
        #[arg(long, value_enum, value_delimiter = ',')]
        types: Option<Vec<ResourceKind>>,
        /// Write all exported documents into a single multi-document YAML file
        #[arg(short, long, conflicts_with = "output_dir")]
        file: Option<PathBuf>,
        /// Write exported documents into a structured directory tree (traits/, tools/, skills/, agents/)
        #[arg(long, conflicts_with = "file")]
        output_dir: Option<PathBuf>,
        /// Keep raw UUID references and resource IDs instead of portable names
        #[arg(long)]
        raw_ids: bool,
    },
    /// Refresh MCP capabilities for a specific tool server or all HTTP tool servers
    SyncTools {
        /// Optional tool server UUID or server_name (omit to sync all HTTP MCP servers)
        id_or_name: Option<String>,
    },
}

/// Summary of a hydration (`apply`) or dry-run execution.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ApplySummary {
    pub dry_run: bool,
    pub traits_created: usize,
    pub traits_updated: usize,
    pub tools_created: usize,
    pub tools_updated: usize,
    pub skills_created: usize,
    pub skills_updated: usize,
    pub agents_created: usize,
    pub agents_updated: usize,
}

impl ApplySummary {
    pub fn total_created(&self) -> usize {
        self.traits_created + self.tools_created + self.skills_created + self.agents_created
    }

    pub fn total_updated(&self) -> usize {
        self.traits_updated + self.tools_updated + self.skills_updated + self.agents_updated
    }
}

/// Executes offline `validate` without initializing a Tokio runtime or loading `AppConfig`.
pub fn run_validate_offline(
    files: &[PathBuf],
    recursive: bool,
    self_contained: bool,
    vars_file: Option<&Path>,
    cli_vars: &[String],
) -> Result<HydrationPlan, CtlError> {
    let vars = TemplateVars::from_sources(vars_file, cli_vars)?;
    let loaded = load_manifests(files, recursive, &vars)?;
    let plan = build_hydration_plan(loaded, self_contained)?;

    println!(
        "Validation OK: {} total resources ({} traits, {} tools, {} skills, {} agents)",
        plan.total_count(),
        plan.traits.len(),
        plan.tools.len(),
        plan.skills.len(),
        plan.agents.len()
    );
    Ok(plan)
}

/// Executes any online `aad-be ctl` command against the normalized `base_url`.
pub async fn run_ctl(args: CtlArgs, base_url: &str) -> Result<(), CtlError> {
    let vars = TemplateVars::from_sources(args.vars_file.as_deref(), &args.vars)?;

    if let CtlCommands::Validate {
        file,
        recursive,
        self_contained,
    } = args.command
    {
        run_validate_offline(
            &file,
            recursive,
            self_contained,
            args.vars_file.as_deref(),
            &args.vars,
        )?;
        return Ok(());
    }

    let token = resolve_bearer_token(args.token.as_deref(), args.token_file.as_deref())?;
    let client = AadApiClient::new(
        base_url.to_string(),
        token,
        Duration::from_secs(args.timeout_secs),
    )?;

    match args.command {
        CtlCommands::Apply {
            file,
            recursive,
            dry_run,
            strict_traits,
            allow_degraded_tools,
        } => {
            let loaded = load_manifests(&file, recursive, &vars)?;
            let plan = build_hydration_plan(loaded, false)?;
            let summary = apply_plan(
                &client,
                &plan,
                args.default_owner_id,
                dry_run,
                strict_traits,
                allow_degraded_tools,
            )
            .await?;

            let mode = if summary.dry_run {
                "Dry-run complete"
            } else {
                "Hydration complete"
            };
            println!(
                "{}: {} created, {} updated (traits: +{} ~{}, tools: +{} ~{}, skills: +{} ~{}, agents: +{} ~{})",
                mode,
                summary.total_created(),
                summary.total_updated(),
                summary.traits_created,
                summary.traits_updated,
                summary.tools_created,
                summary.tools_updated,
                summary.skills_created,
                summary.skills_updated,
                summary.agents_created,
                summary.agents_updated,
            );
        }
        CtlCommands::Validate { .. } => unreachable!(),
        CtlCommands::Get {
            resource_type,
            id_or_name,
            output,
        } => {
            let rendered = get_resources(&client, resource_type, id_or_name.as_deref(), output).await?;
            println!("{}", rendered);
        }
        CtlCommands::Delete {
            resource_type,
            id_or_name,
            hard,
        } => {
            delete_resource(&client, resource_type, &id_or_name, hard).await?;
        }
        CtlCommands::Export {
            types,
            file,
            output_dir,
            raw_ids,
        } => {
            export_resources(
                &client,
                types.as_deref(),
                file.as_deref(),
                output_dir.as_deref(),
                raw_ids,
            )
            .await?;
        }
        CtlCommands::SyncTools { id_or_name } => {
            sync_tools(&client, id_or_name.as_deref()).await?;
        }
    }

    Ok(())
}

/// Live server inventory snapshot used for name-to-UUID resolution and upsert matching.
struct LiveInventory {
    traits_by_name: HashMap<String, Uuid>,
    tools_by_name: HashMap<String, Uuid>,
    skills_by_name: HashMap<String, Uuid>,
    agents_by_name: HashMap<String, Uuid>,
    agent_ids: HashSet<Uuid>,
}

async fn fetch_inventory(client: &AadApiClient) -> Result<LiveInventory, CtlError> {
    let (traits, tools, skills, agents) = tokio::try_join!(
        client.list_traits(),
        client.list_tools(),
        client.list_skills(),
        client.list_agents(),
    )?;

    let mut inv = LiveInventory {
        traits_by_name: HashMap::new(),
        tools_by_name: HashMap::new(),
        skills_by_name: HashMap::new(),
        agents_by_name: HashMap::new(),
        agent_ids: HashSet::new(),
    };

    for t in traits {
        if let Some(id) = t.id {
            inv.traits_by_name.insert(t.name, id);
        }
    }
    for t in tools {
        inv.tools_by_name.insert(t.server_name, t.id);
    }
    for s in skills {
        if let Some(id) = s.id {
            inv.skills_by_name.insert(s.name, id);
        }
    }
    for a in agents {
        if let Some(id) = a.id {
            inv.agent_ids.insert(id);
            inv.agents_by_name.insert(a.name, id);
        }
    }

    Ok(inv)
}

fn resolve_uuid_refs(
    refs: &[String],
    symbol_table: &HashMap<String, Uuid>,
    owner_kind: &str,
    owner_name: &str,
    ref_kind: &str,
) -> Result<Vec<Uuid>, CtlError> {
    let mut resolved = Vec::with_capacity(refs.len());
    for reference in refs {
        if let Ok(uuid) = Uuid::parse_str(reference) {
            resolved.push(uuid);
            continue;
        }
        let Some(&uuid) = symbol_table.get(reference) else {
            return Err(CtlError::UnresolvedReference {
                owner_kind: owner_kind.to_string(),
                owner_name: owner_name.to_string(),
                ref_kind: ref_kind.to_string(),
                reference: reference.clone(),
            });
        };
        resolved.push(uuid);
    }
    Ok(resolved)
}

fn verify_trait_refs(
    trait_refs: impl Iterator<Item = impl AsRef<str>>,
    known_traits: &HashMap<String, Uuid>,
    owner_kind: &str,
    owner_name: &str,
    strict_traits: bool,
) -> Result<(), CtlError> {
    for tr in trait_refs {
        let tr_name = tr.as_ref();
        if known_traits.contains_key(tr_name) {
            continue;
        }
        if strict_traits {
            return Err(CtlError::UnresolvedReference {
                owner_kind: owner_kind.to_string(),
                owner_name: owner_name.to_string(),
                ref_kind: "Trait".to_string(),
                reference: tr_name.to_string(),
            });
        }
        eprintln!(
            "Warning: {} '{}' references unknown Trait '{}' (pass --strict-traits to fail fast)",
            owner_kind, owner_name, tr_name
        );
    }
    Ok(())
}

/// Applies (or dry-runs) a pre-validated 4-phase `HydrationPlan` against `client`.
pub async fn apply_plan(
    client: &AadApiClient,
    plan: &HydrationPlan,
    default_owner_id: Uuid,
    dry_run: bool,
    strict_traits: bool,
    allow_degraded_tools: bool,
) -> Result<ApplySummary, CtlError> {
    let mut inv = fetch_inventory(client).await?;
    let mut summary = ApplySummary {
        dry_run,
        ..Default::default()
    };

    // Phase 1: Traits
    for trait_manifest in &plan.traits {
        let existed = inv.traits_by_name.contains_key(&trait_manifest.name);
        if dry_run {
            let action = if existed { "UPDATE" } else { "CREATE" };
            println!("[DRY-RUN] {:<6} Trait '{}'", action, trait_manifest.name);
            inv.traits_by_name
                .entry(trait_manifest.name.clone())
                .or_insert_with(Uuid::nil);
        } else {
            let payload = trait_manifest.to_contract(default_owner_id);
            let saved = client.upsert_trait(&payload).await?;
            let final_id = saved.id.unwrap_or_else(Uuid::nil);
            inv.traits_by_name.insert(saved.name.clone(), final_id);
            let action = if existed { "UPDATED" } else { "CREATED" };
            println!("{:<7} Trait '{}' ({})", action, saved.name, final_id);
        }

        if existed {
            summary.traits_updated += 1;
        } else {
            summary.traits_created += 1;
        }
    }

    // Phase 2: Tools
    for tool_manifest in &plan.tools {
        let existed = inv.tools_by_name.contains_key(&tool_manifest.server_name);
        if dry_run {
            let action = if existed { "UPDATE" } else { "CREATE" };
            println!(
                "[DRY-RUN] {:<6} Tool '{}'",
                action, tool_manifest.server_name
            );
            inv.tools_by_name
                .entry(tool_manifest.server_name.clone())
                .or_insert_with(Uuid::nil);
        } else {
            let payload =
                tool_manifest.to_register_request(default_owner_id, allow_degraded_tools);
            let saved = client.register_tool(&payload).await?;
            inv.tools_by_name
                .insert(saved.server_name.clone(), saved.id);
            let action = if existed { "UPDATED" } else { "CREATED" };
            println!(
                "{:<7} Tool '{}' ({}, status: {:?}, {} tools)",
                action, saved.server_name, saved.id, saved.sync_status, saved.cached_tools_count
            );
        }

        if existed {
            summary.tools_updated += 1;
        } else {
            summary.tools_created += 1;
        }
    }

    // Phase 3: Skills (topological order)
    for skill_manifest in &plan.skills {
        verify_trait_refs(
            skill_manifest
                .implements_traits
                .iter()
                .chain(skill_manifest.uses_traits.iter()),
            &inv.traits_by_name,
            "Skill",
            &skill_manifest.name,
            strict_traits,
        )?;
        let attached_tools = resolve_uuid_refs(
            &skill_manifest.attached_tools,
            &inv.tools_by_name,
            "Skill",
            &skill_manifest.name,
            "Tool",
        )?;
        let attached_skills = resolve_uuid_refs(
            &skill_manifest.attached_skills,
            &inv.skills_by_name,
            "Skill",
            &skill_manifest.name,
            "Skill",
        )?;

        let existed = inv.skills_by_name.contains_key(&skill_manifest.name);
        if dry_run {
            let action = if existed { "UPDATE" } else { "CREATE" };
            println!("[DRY-RUN] {:<6} Skill '{}'", action, skill_manifest.name);
            inv.skills_by_name
                .entry(skill_manifest.name.clone())
                .or_insert_with(Uuid::nil);
        } else {
            let payload =
                skill_manifest.to_skill(default_owner_id, attached_skills, attached_tools);
            let saved = client.upsert_skill(&payload).await?;
            let final_id = saved.id.unwrap_or_else(Uuid::nil);
            inv.skills_by_name.insert(saved.name.clone(), final_id);
            let action = if existed { "UPDATED" } else { "CREATED" };
            println!("{:<7} Skill '{}' ({})", action, saved.name, final_id);
        }

        if existed {
            summary.skills_updated += 1;
        } else {
            summary.skills_created += 1;
        }
    }

    // Phase 4: Agents (topological order)
    for agent_manifest in &plan.agents {
        verify_trait_refs(
            agent_manifest
                .implements_traits
                .iter()
                .chain(agent_manifest.uses_traits.iter()),
            &inv.traits_by_name,
            "Agent",
            &agent_manifest.name,
            strict_traits,
        )?;
        let attached_tools = resolve_uuid_refs(
            &agent_manifest.attached_tools,
            &inv.tools_by_name,
            "Agent",
            &agent_manifest.name,
            "Tool",
        )?;
        let attached_skills = resolve_uuid_refs(
            &agent_manifest.attached_skills,
            &inv.skills_by_name,
            "Agent",
            &agent_manifest.name,
            "Skill",
        )?;
        let attached_agents = resolve_uuid_refs(
            &agent_manifest.attached_agents,
            &inv.agents_by_name,
            "Agent",
            &agent_manifest.name,
            "Agent",
        )?;

        let existing_id = agent_manifest
            .id
            .filter(|id| inv.agent_ids.contains(id))
            .or_else(|| inv.agents_by_name.get(&agent_manifest.name).copied());
        let existed = existing_id.is_some();

        if dry_run {
            let action = if existed { "UPDATE" } else { "CREATE" };
            println!("[DRY-RUN] {:<6} Agent '{}'", action, agent_manifest.name);
            inv.agents_by_name
                .entry(agent_manifest.name.clone())
                .or_insert_with(Uuid::nil);
        } else if let Some(target_id) = existing_id {
            let payload = agent_manifest.to_agent(
                Some(target_id),
                default_owner_id,
                attached_tools,
                attached_skills,
                attached_agents,
            );
            let saved = client.update_agent(target_id, &payload).await?;
            let final_id = saved.id.unwrap_or(target_id);
            inv.agents_by_name.insert(saved.name.clone(), final_id);
            inv.agent_ids.insert(final_id);
            println!("UPDATED Agent '{}' ({})", saved.name, final_id);
        } else {
            let payload = agent_manifest.to_agent(
                None,
                default_owner_id,
                attached_tools,
                attached_skills,
                attached_agents,
            );
            let saved = client.create_agent(&payload).await?;
            let final_id = saved.id.unwrap_or_else(Uuid::nil);
            inv.agents_by_name.insert(saved.name.clone(), final_id);
            inv.agent_ids.insert(final_id);
            println!("CREATED Agent '{}' ({})", saved.name, final_id);
        }

        if existed {
            summary.agents_updated += 1;
        } else {
            summary.agents_created += 1;
        }
    }

    Ok(summary)
}

/// Fetches and formats resources for `aad-be ctl get`.
pub async fn get_resources(
    client: &AadApiClient,
    selector: ResourceSelector,
    id_or_name: Option<&str>,
    output: OutputFormat,
) -> Result<String, CtlError> {
    match selector {
        ResourceSelector::All => {
            let (traits, tools, skills, agents) = tokio::try_join!(
                client.list_traits(),
                client.list_tools(),
                client.list_skills(),
                client.list_agents(),
            )?;
            if output == OutputFormat::Table {
                return Ok(render_table_all(&traits, &tools, &skills, &agents));
            }
            let combined = serde_json::json!({
                "traits": traits,
                "tools": tools,
                "skills": skills,
                "agents": agents,
            });
            format_value(&combined, output)
        }
        ResourceSelector::Single(ResourceKind::Trait) => {
            if let Some(ident) = id_or_name {
                let item = find_trait(client, ident).await?;
                if output == OutputFormat::Table {
                    return Ok(render_table_all(&[item], &[], &[], &[]));
                }
                let val = serde_json::to_value(&item)
                    .map_err(|e| CtlError::Serialization(e.to_string()))?;
                return format_value(&val, output);
            }
            let items = client.list_traits().await?;
            if output == OutputFormat::Table {
                return Ok(render_table_all(&items, &[], &[], &[]));
            }
            let val = serde_json::to_value(&items)
                .map_err(|e| CtlError::Serialization(e.to_string()))?;
            format_value(&val, output)
        }
        ResourceSelector::Single(ResourceKind::Tool) => {
            if let Some(ident) = id_or_name {
                let item = find_tool(client, ident).await?;
                if output == OutputFormat::Table {
                    return Ok(render_table_all(&[], &[item], &[], &[]));
                }
                let val = serde_json::to_value(&item)
                    .map_err(|e| CtlError::Serialization(e.to_string()))?;
                return format_value(&val, output);
            }
            let items = client.list_tools().await?;
            if output == OutputFormat::Table {
                return Ok(render_table_all(&[], &items, &[], &[]));
            }
            let val = serde_json::to_value(&items)
                .map_err(|e| CtlError::Serialization(e.to_string()))?;
            format_value(&val, output)
        }
        ResourceSelector::Single(ResourceKind::Skill) => {
            if let Some(ident) = id_or_name {
                let item = find_skill(client, ident).await?;
                if output == OutputFormat::Table {
                    return Ok(render_table_all(&[], &[], &[item], &[]));
                }
                let val = serde_json::to_value(&item)
                    .map_err(|e| CtlError::Serialization(e.to_string()))?;
                return format_value(&val, output);
            }
            let items = client.list_skills().await?;
            if output == OutputFormat::Table {
                return Ok(render_table_all(&[], &[], &items, &[]));
            }
            let val = serde_json::to_value(&items)
                .map_err(|e| CtlError::Serialization(e.to_string()))?;
            format_value(&val, output)
        }
        ResourceSelector::Single(ResourceKind::Agent) => {
            if let Some(ident) = id_or_name {
                let item = find_agent(client, ident).await?;
                if output == OutputFormat::Table {
                    return Ok(render_table_all(&[], &[], &[], &[item]));
                }
                let val = serde_json::to_value(&item)
                    .map_err(|e| CtlError::Serialization(e.to_string()))?;
                return format_value(&val, output);
            }
            let items = client.list_agents().await?;
            if output == OutputFormat::Table {
                return Ok(render_table_all(&[], &[], &[], &items));
            }
            let val = serde_json::to_value(&items)
                .map_err(|e| CtlError::Serialization(e.to_string()))?;
            format_value(&val, output)
        }
    }
}

async fn find_trait(client: &AadApiClient, ident: &str) -> Result<TraitContract, CtlError> {
    if let Ok(uuid) = Uuid::parse_str(ident) {
        return client.get_trait(uuid).await;
    }
    let items = client.list_traits().await?;
    items
        .into_iter()
        .find(|t| t.name == ident)
        .ok_or_else(|| CtlError::ResourceNotFound {
            kind: "Trait".to_string(),
            identifier: ident.to_string(),
        })
}

async fn find_tool(client: &AadApiClient, ident: &str) -> Result<Tool, CtlError> {
    if let Ok(uuid) = Uuid::parse_str(ident) {
        return client.get_tool(uuid).await;
    }
    let items = client.list_tools().await?;
    items
        .into_iter()
        .find(|t| t.server_name == ident)
        .ok_or_else(|| CtlError::ResourceNotFound {
            kind: "Tool".to_string(),
            identifier: ident.to_string(),
        })
}

async fn find_skill(client: &AadApiClient, ident: &str) -> Result<Skill, CtlError> {
    if let Ok(uuid) = Uuid::parse_str(ident) {
        return client.get_skill(uuid).await;
    }
    let items = client.list_skills().await?;
    items
        .into_iter()
        .find(|s| s.name == ident)
        .ok_or_else(|| CtlError::ResourceNotFound {
            kind: "Skill".to_string(),
            identifier: ident.to_string(),
        })
}

async fn find_agent(client: &AadApiClient, ident: &str) -> Result<Agent, CtlError> {
    if let Ok(uuid) = Uuid::parse_str(ident) {
        return client.get_agent(uuid).await;
    }
    let items = client.list_agents().await?;
    items
        .into_iter()
        .find(|a| a.name == ident)
        .ok_or_else(|| CtlError::ResourceNotFound {
            kind: "Agent".to_string(),
            identifier: ident.to_string(),
        })
}

/// Deletes a resource by UUID or canonical name.
pub async fn delete_resource(
    client: &AadApiClient,
    kind: ResourceKind,
    id_or_name: &str,
    hard: bool,
) -> Result<Uuid, CtlError> {
    let id = match kind {
        ResourceKind::Trait => {
            let uuid = match Uuid::parse_str(id_or_name) {
                Ok(u) => u,
                Err(_) => find_trait(client, id_or_name)
                    .await?
                    .id
                    .ok_or_else(|| CtlError::ResourceNotFound {
                        kind: "Trait".to_string(),
                        identifier: id_or_name.to_string(),
                    })?,
            };
            client.delete_trait(uuid).await?;
            uuid
        }
        ResourceKind::Tool => {
            let uuid = match Uuid::parse_str(id_or_name) {
                Ok(u) => u,
                Err(_) => find_tool(client, id_or_name).await?.id,
            };
            client.delete_tool(uuid).await?;
            uuid
        }
        ResourceKind::Skill => {
            let uuid = match Uuid::parse_str(id_or_name) {
                Ok(u) => u,
                Err(_) => find_skill(client, id_or_name)
                    .await?
                    .id
                    .ok_or_else(|| CtlError::ResourceNotFound {
                        kind: "Skill".to_string(),
                        identifier: id_or_name.to_string(),
                    })?,
            };
            client.delete_skill(uuid).await?;
            uuid
        }
        ResourceKind::Agent => {
            let uuid = match Uuid::parse_str(id_or_name) {
                Ok(u) => u,
                Err(_) => find_agent(client, id_or_name)
                    .await?
                    .id
                    .ok_or_else(|| CtlError::ResourceNotFound {
                        kind: "Agent".to_string(),
                        identifier: id_or_name.to_string(),
                    })?,
            };
            client.delete_agent(uuid, hard).await?;
            uuid
        }
    };

    println!("DELETED {} '{}' ({})", kind, id_or_name, id);
    Ok(id)
}

/// Dehydrates live AAD resources into portable YAML manifests.
pub async fn export_resources(
    client: &AadApiClient,
    types: Option<&[ResourceKind]>,
    file: Option<&Path>,
    output_dir: Option<&Path>,
    raw_ids: bool,
) -> Result<Vec<ResourceManifest>, CtlError> {
    let selected_kinds: HashSet<ResourceKind> = match types {
        Some(list) if !list.is_empty() => list.iter().copied().collect(),
        _ => [
            ResourceKind::Trait,
            ResourceKind::Tool,
            ResourceKind::Skill,
            ResourceKind::Agent,
        ]
        .into_iter()
        .collect(),
    };

    // Always fetch full inventories when exporting skills/agents so UUID -> name resolution is complete
    let (traits, tools, skills, agents) = tokio::try_join!(
        client.list_traits(),
        client.list_tools(),
        client.list_skills(),
        client.list_agents(),
    )?;

    let tool_id_to_name: HashMap<Uuid, String> = tools
        .iter()
        .map(|t| (t.id, t.server_name.clone()))
        .collect();
    let skill_id_to_name: HashMap<Uuid, String> = skills
        .iter()
        .filter_map(|s| s.id.map(|id| (id, s.name.clone())))
        .collect();
    let agent_id_to_name: HashMap<Uuid, String> = agents
        .iter()
        .filter_map(|a| a.id.map(|id| (id, a.name.clone())))
        .collect();

    let mut raw_loaded = Vec::new();
    let mut doc_idx = 0usize;

    if selected_kinds.contains(&ResourceKind::Trait) {
        for t in &traits {
            doc_idx += 1;
            raw_loaded.push(crate::cli::manifest::LoadedManifest {
                source_name: "<export>".to_string(),
                doc_index: doc_idx,
                manifest: ResourceManifest::Trait(TraitManifest::from_contract(t, raw_ids)),
            });
        }
    }
    if selected_kinds.contains(&ResourceKind::Tool) {
        for t in &tools {
            doc_idx += 1;
            raw_loaded.push(crate::cli::manifest::LoadedManifest {
                source_name: "<export>".to_string(),
                doc_index: doc_idx,
                manifest: ResourceManifest::Tool(ToolManifest::from_tool(t, raw_ids)),
            });
        }
    }
    if selected_kinds.contains(&ResourceKind::Skill) {
        for s in &skills {
            doc_idx += 1;
            raw_loaded.push(crate::cli::manifest::LoadedManifest {
                source_name: "<export>".to_string(),
                doc_index: doc_idx,
                manifest: ResourceManifest::Skill(SkillManifest::from_skill(
                    s,
                    &tool_id_to_name,
                    &skill_id_to_name,
                    raw_ids,
                )),
            });
        }
    }
    if selected_kinds.contains(&ResourceKind::Agent) {
        for a in &agents {
            doc_idx += 1;
            raw_loaded.push(crate::cli::manifest::LoadedManifest {
                source_name: "<export>".to_string(),
                doc_index: doc_idx,
                manifest: ResourceManifest::Agent(AgentManifest::from_agent(
                    a,
                    &tool_id_to_name,
                    &skill_id_to_name,
                    &agent_id_to_name,
                    raw_ids,
                )),
            });
        }
    }

    // Order exported manifests in deterministic hydration dependency order
    let ordered: Vec<ResourceManifest> = match build_hydration_plan(raw_loaded.clone(), false) {
        Ok(plan) => {
            let mut manifests = Vec::with_capacity(plan.total_count());
            manifests.extend(plan.traits.into_iter().map(ResourceManifest::Trait));
            manifests.extend(plan.tools.into_iter().map(ResourceManifest::Tool));
            manifests.extend(plan.skills.into_iter().map(ResourceManifest::Skill));
            manifests.extend(plan.agents.into_iter().map(ResourceManifest::Agent));
            manifests
        }
        Err(_) => {
            // Fall back to deterministic phase+name sort if live server contains pre-existing duplicates/cycles
            let mut manifests: Vec<ResourceManifest> =
                raw_loaded.into_iter().map(|l| l.manifest).collect();
            manifests.sort_by(|a, b| {
                a.kind()
                    .phase_rank()
                    .cmp(&b.kind().phase_rank())
                    .then_with(|| a.canonical_name().cmp(b.canonical_name()))
            });
            manifests
        }
    };

    if let Some(dir) = output_dir {
        write_export_dir(dir, &ordered)?;
        println!(
            "Exported {} manifests to directory '{}'",
            ordered.len(),
            dir.display()
        );
    } else {
        let yaml_stream = render_multidoc_yaml(&ordered)?;
        if let Some(out_file) = file {
            if let Some(parent) = out_file.parent()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent).map_err(|source| CtlError::Io {
                    path: parent.display().to_string(),
                    source,
                })?;
            }
            fs::write(out_file, &yaml_stream).map_err(|source| CtlError::Io {
                path: out_file.display().to_string(),
                source,
            })?;
            println!(
                "Exported {} manifests to file '{}'",
                ordered.len(),
                out_file.display()
            );
        } else {
            print!("{}", yaml_stream);
        }
    }

    Ok(ordered)
}

fn render_multidoc_yaml(manifests: &[ResourceManifest]) -> Result<String, CtlError> {
    let mut out = String::new();
    for manifest in manifests {
        out.push_str("---\n");
        let doc = serde_yaml::to_string(manifest)
            .map_err(|e| CtlError::Serialization(e.to_string()))?;
        out.push_str(&doc);
    }
    Ok(out)
}

fn write_export_dir(root: &Path, manifests: &[ResourceManifest]) -> Result<(), CtlError> {
    fs::create_dir_all(root).map_err(|source| CtlError::Io {
        path: root.display().to_string(),
        source,
    })?;

    let mut used_paths: HashMap<PathBuf, usize> = HashMap::new();

    for manifest in manifests {
        let subdir = root.join(manifest.kind().dir_name());
        fs::create_dir_all(&subdir).map_err(|source| CtlError::Io {
            path: subdir.display().to_string(),
            source,
        })?;

        let base_slug = slugify_name(manifest.canonical_name());
        let mut candidate = subdir.join(format!("{}.yaml", base_slug));
        let counter = used_paths.entry(candidate.clone()).or_insert(0);
        *counter += 1;
        if *counter > 1 {
            candidate = subdir.join(format!("{}-{}.yaml", base_slug, *counter));
        }

        let doc = serde_yaml::to_string(manifest)
            .map_err(|e| CtlError::Serialization(e.to_string()))?;
        fs::write(&candidate, doc).map_err(|source| CtlError::Io {
            path: candidate.display().to_string(),
            source,
        })?;
    }

    Ok(())
}

/// Triggers `POST /agents/tools/{id}/sync` for one or all HTTP MCP tool servers.
pub async fn sync_tools(
    client: &AadApiClient,
    id_or_name: Option<&str>,
) -> Result<usize, CtlError> {
    let targets: Vec<Tool> = match id_or_name {
        Some(ident) => vec![find_tool(client, ident).await?],
        None => client
            .list_tools()
            .await?
            .into_iter()
            .filter(|t| t.transport_type == TransportType::Http)
            .collect(),
    };

    let mut synced_count = 0;
    for tool in targets {
        let res = client.sync_tool(tool.id).await?;
        println!(
            "SYNCED Tool '{}' ({}): status={:?}, tools={}{}",
            res.server_name,
            res.id,
            res.sync_status,
            res.cached_tools_count,
            res.last_sync_error
                .as_deref()
                .map(|e| format!(", error={}", e))
                .unwrap_or_default()
        );
        synced_count += 1;
    }

    Ok(synced_count)
}

fn format_value(val: &Value, output: OutputFormat) -> Result<String, CtlError> {
    match output {
        OutputFormat::Yaml => {
            serde_yaml::to_string(val).map_err(|e| CtlError::Serialization(e.to_string()))
        }
        OutputFormat::Json => {
            serde_json::to_string_pretty(val).map_err(|e| CtlError::Serialization(e.to_string()))
        }
        OutputFormat::Table => {
            serde_yaml::to_string(val).map_err(|e| CtlError::Serialization(e.to_string()))
        }
    }
}

fn render_table_all(
    traits: &[TraitContract],
    tools: &[Tool],
    skills: &[Skill],
    agents: &[Agent],
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "{:<8} {:<38} {:<36} {:<12} {}",
        "TYPE", "NAME", "ID", "VER/TRANS", "TAGS/STATUS"
    ));

    for t in traits {
        lines.push(format!(
            "{:<8} {:<38} {:<36} {:<12} {}",
            "Trait",
            t.name,
            t.id.map(|u| u.to_string()).unwrap_or_default(),
            t.version,
            t.tags.join(",")
        ));
    }
    for t in tools {
        lines.push(format!(
            "{:<8} {:<38} {:<36} {:<12} {:?}",
            "Tool",
            t.server_name,
            t.id,
            format!("{:?}", t.transport_type).to_lowercase(),
            t.sync_status
        ));
    }
    for s in skills {
        lines.push(format!(
            "{:<8} {:<38} {:<36} {:<12} {}",
            "Skill",
            s.name,
            s.id.map(|u| u.to_string()).unwrap_or_default(),
            s.current_version,
            s.tags.join(",")
        ));
    }
    for a in agents {
        lines.push(format!(
            "{:<8} {:<38} {:<36} {:<12} {}",
            "Agent",
            a.name,
            a.id.map(|u| u.to_string()).unwrap_or_default(),
            a.current_version,
            a.tags.join(",")
        ));
    }

    lines.join("\n")
}
