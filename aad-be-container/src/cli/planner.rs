//! Dependency planner, duplicate validator, Kahn's DAG topological sorter, and offline reference checker.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use uuid::Uuid;

use crate::cli::error::CtlError;
use crate::cli::manifest::{
    AgentManifest, LoadedManifest, ResourceKind, ResourceManifest, SkillManifest, ToolManifest,
    TraitManifest,
};

/// Ordered, validated hydration plan partitioned into the 4 strict execution phases.
#[derive(Debug, Clone)]
pub struct HydrationPlan {
    pub traits: Vec<TraitManifest>,
    pub tools: Vec<ToolManifest>,
    pub skills: Vec<SkillManifest>,
    pub agents: Vec<AgentManifest>,
}

impl HydrationPlan {
    pub fn total_count(&self) -> usize {
        self.traits.len() + self.tools.len() + self.skills.len() + self.agents.len()
    }
}

/// Validates uniqueness, verifies intra-manifest DAG acyclicity, optionally enforces
/// `--self-contained` reference completeness, and produces a deterministic 4-phase plan.
pub fn build_hydration_plan(
    loaded: Vec<LoadedManifest>,
    self_contained: bool,
) -> Result<HydrationPlan, CtlError> {
    let mut seen: HashMap<(ResourceKind, String), String> = HashMap::new();
    let mut traits = Vec::new();
    let mut tools = Vec::new();
    let mut skills = Vec::new();
    let mut agents = Vec::new();

    for item in loaded {
        let kind = item.manifest.kind();
        let name = item.manifest.canonical_name().to_string();
        let key = (kind, name.clone());

        if let Some(first_source) = seen.get(&key) {
            return Err(CtlError::DuplicateResource {
                kind: kind.to_string(),
                name,
                first_source: first_source.clone(),
                second_source: format!("{} (doc #{})", item.source_name, item.doc_index),
            });
        }
        seen.insert(key, format!("{} (doc #{})", item.source_name, item.doc_index));

        match item.manifest {
            ResourceManifest::Trait(t) => traits.push(t),
            ResourceManifest::Tool(t) => tools.push(t),
            ResourceManifest::Skill(s) => skills.push(s),
            ResourceManifest::Agent(a) => agents.push(a),
        }
    }

    // Phase 1 & 2: deterministic lexicographical sort
    traits.sort_by(|a, b| a.name.cmp(&b.name));
    tools.sort_by(|a, b| a.server_name.cmp(&b.server_name));

    // Phase 3: topological sort on Skill.attached_skills
    let sorted_skills = topo_sort_named(
        skills,
        ResourceKind::Skill,
        |s| s.name.as_str(),
        |s| &s.attached_skills,
    )?;

    // Phase 4: topological sort on Agent.attached_agents
    let sorted_agents = topo_sort_named(
        agents,
        ResourceKind::Agent,
        |a| a.name.as_str(),
        |a| &a.attached_agents,
    )?;

    if self_contained {
        validate_self_contained_references(&traits, &tools, &sorted_skills, &sorted_agents)?;
    }

    Ok(HydrationPlan {
        traits,
        tools,
        skills: sorted_skills,
        agents: sorted_agents,
    })
}

/// Deterministic Kahn's topological sort over items referenced by canonical name (or explicit UUID).
/// Dependencies outside the local batch are ignored here (they may exist on the live server
/// or be flagged by `validate_self_contained_references`).
fn topo_sort_named<T, NameFn, DepsFn>(
    items: Vec<T>,
    kind: ResourceKind,
    name_fn: NameFn,
    deps_fn: DepsFn,
) -> Result<Vec<T>, CtlError>
where
    NameFn: Fn(&T) -> &str,
    DepsFn: Fn(&T) -> &[String],
{
    let mut by_name: BTreeMap<String, T> = BTreeMap::new();
    for item in items {
        by_name.insert(name_fn(&item).to_string(), item);
    }

    let mut in_degree: BTreeMap<String, usize> = BTreeMap::new();
    let mut dependents: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for name in by_name.keys() {
        in_degree.insert(name.clone(), 0);
        dependents.insert(name.clone(), Vec::new());
    }

    for (owner_name, item) in &by_name {
        let mut unique_deps: BTreeSet<String> = BTreeSet::new();
        for dep in deps_fn(item) {
            if dep == owner_name {
                return Err(CtlError::DependencyCycle {
                    kind: kind.to_string(),
                    cycle: format!("{} -> {}", owner_name, owner_name),
                });
            }
            if by_name.contains_key(dep) && unique_deps.insert(dep.clone()) {
                *in_degree.entry(owner_name.clone()).or_insert(0) += 1;
                dependents
                    .entry(dep.clone())
                    .or_default()
                    .push(owner_name.clone());
            }
        }
    }

    // Use BTreeSet as a deterministic min-heap queue (alphabetical tie-breaking)
    let mut ready: BTreeSet<String> = in_degree
        .iter()
        .filter_map(|(name, &deg)| if deg == 0 { Some(name.clone()) } else { None })
        .collect();

    let mut ordered_names = Vec::with_capacity(by_name.len());

    while let Some(next) = ready.pop_first() {
        ordered_names.push(next.clone());
        if let Some(children) = dependents.get(&next) {
            for child in children {
                let Some(deg) = in_degree.get_mut(child) else {
                    continue;
                };
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    ready.insert(child.clone());
                }
            }
        }
    }

    if ordered_names.len() != by_name.len() {
        let stuck: Vec<String> = in_degree
            .into_iter()
            .filter_map(|(name, deg)| if deg > 0 { Some(name) } else { None })
            .collect();
        return Err(CtlError::DependencyCycle {
            kind: kind.to_string(),
            cycle: stuck.join(" <-> "),
        });
    }

    let mut result = Vec::with_capacity(ordered_names.len());
    for name in ordered_names {
        if let Some(item) = by_name.remove(&name) {
            result.push(item);
        }
    }
    Ok(result)
}

fn validate_self_contained_references(
    traits: &[TraitManifest],
    tools: &[ToolManifest],
    skills: &[SkillManifest],
    agents: &[AgentManifest],
) -> Result<(), CtlError> {
    let trait_names: HashSet<&str> = traits.iter().map(|t| t.name.as_str()).collect();
    let tool_names: HashSet<&str> = tools.iter().map(|t| t.server_name.as_str()).collect();
    let tool_ids: HashSet<Uuid> = tools.iter().filter_map(|t| t.id).collect();
    let skill_names: HashSet<&str> = skills.iter().map(|s| s.name.as_str()).collect();
    let skill_ids: HashSet<Uuid> = skills.iter().filter_map(|s| s.id).collect();
    let agent_names: HashSet<&str> = agents.iter().map(|a| a.name.as_str()).collect();
    let agent_ids: HashSet<Uuid> = agents.iter().filter_map(|a| a.id).collect();

    let exists_ref = |reference: &str, names: &HashSet<&str>, ids: &HashSet<Uuid>| -> bool {
        if names.contains(reference) {
            return true;
        }
        if let Ok(uuid) = Uuid::parse_str(reference) {
            return ids.contains(&uuid);
        }
        false
    };

    for skill in skills {
        for tr in skill.implements_traits.iter().chain(skill.uses_traits.iter()) {
            if !trait_names.contains(tr.as_str()) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Skill".to_string(),
                    owner_name: skill.name.clone(),
                    ref_kind: "Trait".to_string(),
                    reference: tr.clone(),
                });
            }
        }
        for tool_ref in &skill.attached_tools {
            if !exists_ref(tool_ref, &tool_names, &tool_ids) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Skill".to_string(),
                    owner_name: skill.name.clone(),
                    ref_kind: "Tool".to_string(),
                    reference: tool_ref.clone(),
                });
            }
        }
        for skill_ref in &skill.attached_skills {
            if !exists_ref(skill_ref, &skill_names, &skill_ids) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Skill".to_string(),
                    owner_name: skill.name.clone(),
                    ref_kind: "Skill".to_string(),
                    reference: skill_ref.clone(),
                });
            }
        }
    }

    for agent in agents {
        for tr in agent.implements_traits.iter().chain(agent.uses_traits.iter()) {
            if !trait_names.contains(tr.as_str()) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Agent".to_string(),
                    owner_name: agent.name.clone(),
                    ref_kind: "Trait".to_string(),
                    reference: tr.clone(),
                });
            }
        }
        for tool_ref in &agent.attached_tools {
            if !exists_ref(tool_ref, &tool_names, &tool_ids) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Agent".to_string(),
                    owner_name: agent.name.clone(),
                    ref_kind: "Tool".to_string(),
                    reference: tool_ref.clone(),
                });
            }
        }
        for skill_ref in &agent.attached_skills {
            if !exists_ref(skill_ref, &skill_names, &skill_ids) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Agent".to_string(),
                    owner_name: agent.name.clone(),
                    ref_kind: "Skill".to_string(),
                    reference: skill_ref.clone(),
                });
            }
        }
        for agent_ref in &agent.attached_agents {
            if !exists_ref(agent_ref, &agent_names, &agent_ids) {
                return Err(CtlError::UnresolvedReference {
                    owner_kind: "Agent".to_string(),
                    owner_name: agent.name.clone(),
                    ref_kind: "Agent".to_string(),
                    reference: agent_ref.clone(),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::loader::parse_yaml_stream;
    use crate::cli::template::TemplateVars;

    #[test]
    fn test_plan_orders_skills_and_agents_topologically() {
        let yaml = r#"
---
type: Agent
name: Orchestrator
attached_agents: ["Specialist"]
attached_skills: ["HighSkill"]
implements_traits: ["CoreTrait"]
---
type: Skill
name: HighSkill
description: depends on BaseSkill
attached_skills: ["BaseSkill"]
attached_tools: ["core-mcp"]
implements_traits: ["CoreTrait"]
---
type: Agent
name: Specialist
attached_skills: ["BaseSkill"]
implements_traits: ["CoreTrait"]
---
type: Skill
name: BaseSkill
description: base
attached_tools: ["core-mcp"]
implements_traits: ["CoreTrait"]
---
type: Tool
server_name: core-mcp
transport_type: http
endpoint_config:
  url: "http://localhost:8082/mcp"
---
type: Trait
name: CoreTrait
description: core trait
"#;
        let mut loaded = Vec::new();
        parse_yaml_stream(yaml, "bundle.yaml", &TemplateVars::default(), &mut loaded).unwrap();
        let plan = build_hydration_plan(loaded, true).unwrap();

        assert_eq!(plan.traits.len(), 1);
        assert_eq!(plan.tools.len(), 1);
        assert_eq!(
            plan.skills.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            vec!["BaseSkill", "HighSkill"]
        );
        assert_eq!(
            plan.agents.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec!["Specialist", "Orchestrator"]
        );
    }

    #[test]
    fn test_plan_rejects_duplicate_resource() {
        let yaml = r#"
---
type: Trait
name: DupTrait
description: first
---
type: Trait
name: DupTrait
description: second
"#;
        let mut loaded = Vec::new();
        parse_yaml_stream(yaml, "dup.yaml", &TemplateVars::default(), &mut loaded).unwrap();
        let err = build_hydration_plan(loaded, false).unwrap_err();
        assert!(matches!(err, CtlError::DuplicateResource { ref name, .. } if name == "DupTrait"));
    }

    #[test]
    fn test_plan_rejects_skill_cycle() {
        let yaml = r#"
---
type: Skill
name: SkillA
description: a
attached_skills: ["SkillB"]
---
type: Skill
name: SkillB
description: b
attached_skills: ["SkillA"]
"#;
        let mut loaded = Vec::new();
        parse_yaml_stream(yaml, "cycle.yaml", &TemplateVars::default(), &mut loaded).unwrap();
        let err = build_hydration_plan(loaded, false).unwrap_err();
        assert!(matches!(err, CtlError::DependencyCycle { ref kind, .. } if kind == "Skill"));
    }

    #[test]
    fn test_plan_rejects_agent_self_cycle() {
        let yaml = r#"
---
type: Agent
name: SelfAgent
attached_agents: ["SelfAgent"]
"#;
        let mut loaded = Vec::new();
        parse_yaml_stream(yaml, "self.yaml", &TemplateVars::default(), &mut loaded).unwrap();
        let err = build_hydration_plan(loaded, false).unwrap_err();
        assert!(matches!(err, CtlError::DependencyCycle { ref kind, .. } if kind == "Agent"));
    }

    #[test]
    fn test_self_contained_rejects_dangling_tool_reference() {
        let yaml = r#"
---
type: Skill
name: OrphanSkill
description: missing tool
attached_tools: ["missing-mcp"]
"#;
        let mut loaded = Vec::new();
        parse_yaml_stream(yaml, "orphan.yaml", &TemplateVars::default(), &mut loaded).unwrap();
        let err = build_hydration_plan(loaded, true).unwrap_err();
        assert!(matches!(
            err,
            CtlError::UnresolvedReference { ref ref_kind, ref reference, .. }
            if ref_kind == "Tool" && reference == "missing-mcp"
        ));
    }
}
