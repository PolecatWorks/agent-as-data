//! End-to-end integration tests for Spec 41 (`aad-be ctl`) and Spec 42 (`manifests/ecosystem`).

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use uuid::Uuid;

use aad_be_container::cli::client::AadApiClient;
use aad_be_container::cli::ctl::{
    OutputFormat, apply_plan, delete_resource, export_resources, get_resources,
    run_validate_offline, sync_tools,
};
use aad_be_container::cli::error::CtlError;
use aad_be_container::cli::loader::load_manifests;
use aad_be_container::cli::planner::build_hydration_plan;
use aad_be_container::cli::template::TemplateVars;
use aad_be_container::models::agent::Agent;
use aad_be_container::models::common::{ListPages, PageOptions};
use aad_be_container::models::skill::Skill;
use aad_be_container::models::tool::{
    RegisterToolRequest, RegisterToolResponse, SyncStatus, SyncToolResponse, Tool,
};
use aad_be_container::models::traits::TraitContract;

const EXPECTED_TOKEN: &str = "test-keycloak-jwt-token";

#[derive(Default)]
struct MockStore {
    traits: HashMap<Uuid, TraitContract>,
    tools: HashMap<Uuid, Tool>,
    skills: HashMap<Uuid, Skill>,
    agents: HashMap<Uuid, Agent>,
    sync_calls: Vec<Uuid>,
    hard_deletes: Vec<Uuid>,
    agent_put_count: usize,
    agent_post_count: usize,
}

#[derive(Clone)]
struct MockState {
    store: Arc<Mutex<MockStore>>,
    required_token: Option<String>,
}

fn check_auth(headers: &HeaderMap, required: &Option<String>) -> Result<(), StatusCode> {
    let Some(expected) = required else {
        return Ok(());
    };
    let expected_header = format!("Bearer {}", expected);
    let actual = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    if actual == Some(expected_header.as_str()) {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn mock_list_traits(
    State(state): State<MockState>,
    headers: HeaderMap,
    Query(opts): Query<PageOptions>,
) -> Result<Json<ListPages>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    let opts = PageOptions::defaulting(opts);
    let page = opts.page.unwrap_or(0) as usize;
    let size = opts.size.unwrap_or(10) as usize;
    let mut all_ids: Vec<Uuid> = store.traits.keys().copied().collect();
    all_ids.sort();
    let slice = all_ids
        .into_iter()
        .skip(page * size)
        .take(size)
        .collect();
    Ok(Json(ListPages {
        ids: slice,
        pagination: opts,
    }))
}

async fn mock_get_trait(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<TraitContract>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    store
        .traits
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_upsert_trait(
    State(state): State<MockState>,
    headers: HeaderMap,
    Json(mut payload): Json<TraitContract>,
) -> Result<(StatusCode, Json<TraitContract>), StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    let existing_id = store
        .traits
        .values()
        .find(|t| t.name == payload.name)
        .and_then(|t| t.id);
    let id = existing_id.unwrap_or_else(|| payload.id.unwrap_or_else(Uuid::new_v4));
    payload.id = Some(id);
    payload.created_at = Some(Utc::now());
    payload.updated_at = Some(Utc::now());
    store.traits.insert(id, payload.clone());
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn mock_delete_trait(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    store
        .traits
        .remove(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_list_tools(
    State(state): State<MockState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Tool>>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    Ok(Json(store.tools.values().cloned().collect()))
}

async fn mock_get_tool(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Tool>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    store
        .tools
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_register_tool(
    State(state): State<MockState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterToolRequest>,
) -> Result<(StatusCode, Json<RegisterToolResponse>), StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    let existing_id = store
        .tools
        .values()
        .find(|t| t.server_name == payload.server_name)
        .map(|t| t.id);
    let id = existing_id.unwrap_or_else(|| payload.id.unwrap_or_else(Uuid::new_v4));
    let sync_status = if payload.allow_degraded_sync {
        SyncStatus::Degraded
    } else {
        SyncStatus::Synced
    };
    let tool = Tool {
        id,
        server_name: payload.server_name.clone(),
        transport_type: payload.transport_type.clone(),
        endpoint_config: payload.endpoint_config,
        cached_capabilities: serde_json::json!({"tools": [], "resources": [], "prompts": []}),
        owner_id: payload.owner_id,
        sync_policy: payload.sync_policy.clone(),
        sync_status: sync_status.clone(),
        last_synced_at: Utc::now(),
        last_sync_error: if sync_status == SyncStatus::Degraded {
            Some("bootstrap offline".to_string())
        } else {
            None
        },
    };
    store.tools.insert(id, tool);
    Ok((
        StatusCode::CREATED,
        Json(RegisterToolResponse {
            id,
            server_name: payload.server_name,
            transport_type: payload.transport_type,
            cached_tools_count: 0,
            sync_status,
            sync_policy: payload.sync_policy,
        }),
    ))
}

async fn mock_sync_tool(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<SyncToolResponse>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    let Some(tool) = store.tools.get_mut(&id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    tool.sync_status = SyncStatus::Synced;
    tool.last_sync_error = None;
    let name = tool.server_name.clone();
    store.sync_calls.push(id);
    Ok(Json(SyncToolResponse {
        id,
        server_name: name,
        cached_tools_count: 5,
        sync_status: SyncStatus::Synced,
        last_synced_at: Utc::now(),
        last_sync_error: None,
    }))
}

async fn mock_delete_tool(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    store
        .tools
        .remove(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_list_skills(
    State(state): State<MockState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Skill>>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    Ok(Json(store.skills.values().cloned().collect()))
}

async fn mock_get_skill(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Skill>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    store
        .skills
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_upsert_skill(
    State(state): State<MockState>,
    headers: HeaderMap,
    Json(mut payload): Json<Skill>,
) -> Result<(StatusCode, Json<Skill>), StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    let existing_id = store
        .skills
        .values()
        .find(|s| s.name == payload.name)
        .and_then(|s| s.id);
    let id = existing_id.unwrap_or_else(|| payload.id.unwrap_or_else(Uuid::new_v4));
    payload.id = Some(id);
    store.skills.insert(id, payload.clone());
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn mock_delete_skill(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    store
        .skills
        .remove(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_list_agents(
    State(state): State<MockState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Agent>>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    let active = store
        .agents
        .values()
        .filter(|a| a.archived_at.is_none())
        .cloned()
        .collect();
    Ok(Json(active))
}

async fn mock_get_agent(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Agent>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let store = state.store.lock().unwrap();
    store
        .agents
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn mock_create_agent(
    State(state): State<MockState>,
    headers: HeaderMap,
    Json(mut payload): Json<Agent>,
) -> Result<(StatusCode, Json<Agent>), StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    let id = payload.id.unwrap_or_else(Uuid::new_v4);
    payload.id = Some(id);
    store.agents.insert(id, payload.clone());
    store.agent_post_count += 1;
    Ok((StatusCode::CREATED, Json(payload)))
}

async fn mock_update_agent(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(mut payload): Json<Agent>,
) -> Result<Json<Agent>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    if !store.agents.contains_key(&id) {
        return Err(StatusCode::NOT_FOUND);
    }
    payload.id = Some(id);
    store.agents.insert(id, payload.clone());
    store.agent_put_count += 1;
    Ok(Json(payload))
}

#[derive(serde::Deserialize, Default)]
struct DeleteParams {
    hard: Option<bool>,
}

async fn mock_delete_agent(
    State(state): State<MockState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Query(params): Query<DeleteParams>,
) -> Result<Json<Agent>, StatusCode> {
    check_auth(&headers, &state.required_token)?;
    let mut store = state.store.lock().unwrap();
    if params.hard.unwrap_or(false) {
        store.hard_deletes.push(id);
        return store
            .agents
            .remove(&id)
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND);
    }
    let Some(agent) = store.agents.get_mut(&id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    agent.archived_at = Some(Utc::now());
    Ok(Json(agent.clone()))
}

async fn spawn_mock_aad_server(required_token: Option<String>) -> (String, Arc<Mutex<MockStore>>) {
    let store = Arc::new(Mutex::new(MockStore::default()));
    let state = MockState {
        store: Arc::clone(&store),
        required_token,
    };

    let api = Router::new()
        .route("/traits", get(mock_list_traits).post(mock_upsert_trait))
        .route(
            "/traits/{id}",
            get(mock_get_trait).delete(mock_delete_trait),
        )
        .route("/agents/tools", get(mock_list_tools))
        .route("/agents/tools/register", post(mock_register_tool))
        .route(
            "/agents/tools/{id}",
            get(mock_get_tool).delete(mock_delete_tool),
        )
        .route("/agents/tools/{id}/sync", post(mock_sync_tool))
        .route("/skills", get(mock_list_skills).post(mock_upsert_skill))
        .route(
            "/skills/{id}",
            get(mock_get_skill).delete(mock_delete_skill),
        )
        .route("/agents", get(mock_list_agents).post(mock_create_agent))
        .route(
            "/agents/{id}",
            get(mock_get_agent)
                .put(mock_update_agent)
                .delete(mock_delete_agent),
        )
        .with_state(state);

    let app = Router::new().nest("/api/aad/v1", api);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://{}/api/aad/v1", addr), store)
}

fn create_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_synthetic_ecosystem_fixtures(root: &std::path::Path) {
    let traits_dir = root.join("manifests/ecosystem/traits");
    let tools_dir = root.join("manifests/ecosystem/tools");
    let skills_dir = root.join("manifests/ecosystem/skills");
    let agents_dir = root.join("manifests/ecosystem/agents");
    let vars_dir = root.join("manifests/vars");
    for d in [&traits_dir, &tools_dir, &skills_dir, &agents_dir, &vars_dir] {
        std::fs::create_dir_all(d).unwrap();
    }

    std::fs::write(
        traits_dir.join("semantic-steward.yaml"),
        r#"type: Trait
name: SemanticSteward
description: Semantic stewardship trait
behavioral_invariants:
  - Maintain canonical domain terminology
"#,
    )
    .unwrap();

    std::fs::write(
        tools_dir.join("data-engine-mcp.yaml"),
        r#"type: Tool
server_name: data-engine-mcp
transport_type: http
endpoint_config:
  url: ${DATA_ENGINE_MCP_URL:-http://localhost:8083/api/data-engine/v1/mcp}
"#,
    )
    .unwrap();

    std::fs::write(
        skills_dir.join("plan-semantic-join-path.yaml"),
        r#"type: Skill
name: plan_semantic_join_path
description: Plan semantic join paths
implements_traits:
  - SemanticSteward
attached_tools:
  - data-engine-mcp
definition: Plan join paths.
"#,
    )
    .unwrap();

    std::fs::write(
        skills_dir.join("validate-and-execute-federated-sql.yaml"),
        r#"type: Skill
name: validate_and_execute_federated_sql
description: Execute federated SQL
implements_traits:
  - SemanticSteward
attached_tools:
  - data-engine-mcp
attached_skills:
  - plan_semantic_join_path
definition: Execute SQL.
"#,
    )
    .unwrap();

    std::fs::write(
        agents_dir.join("semantic-data-steward-agent.yaml"),
        r#"type: Agent
name: Semantic Data Steward Agent
description: Semantic steward
agent_definition: You are a semantic steward.
implements_traits:
  - SemanticSteward
attached_skills:
  - plan_semantic_join_path
"#,
    )
    .unwrap();

    std::fs::write(
        agents_dir.join("epistemic-benchmark-governance-agent.yaml"),
        r#"type: Agent
name: Epistemic Benchmark & Governance Agent
description: Governance agent
agent_definition: You govern epistemic benchmarks.
attached_skills:
  - validate_and_execute_federated_sql
attached_agents:
  - Semantic Data Steward Agent
"#,
    )
    .unwrap();

    std::fs::write(
        vars_dir.join("local.yaml"),
        "DATA_ENGINE_MCP_URL: http://localhost:8083/api/data-engine/v1/mcp\n",
    )
    .unwrap();
    std::fs::write(
        vars_dir.join("cluster-dev.yaml"),
        "DATA_ENGINE_MCP_URL: http://data-engine-be.data-engine-dev.svc.cluster.local/api/data-engine/v1/mcp\n",
    )
    .unwrap();
}

#[test]
fn test_ecosystem_pack_offline_validation_all_env_presets() {
    let root = create_temp_dir("aad-ecosystem-test");
    write_synthetic_ecosystem_fixtures(&root);
    let ecosystem_dir = root.join("manifests/ecosystem");

    let presets = [
        (None, "http://localhost:8083/api/data-engine/v1/mcp"),
        (
            Some(root.join("manifests/vars/local.yaml")),
            "http://localhost:8083/api/data-engine/v1/mcp",
        ),
        (
            Some(root.join("manifests/vars/cluster-dev.yaml")),
            "http://data-engine-be.data-engine-dev.svc.cluster.local/api/data-engine/v1/mcp",
        ),
    ];

    for (preset_path, expected_de_url) in presets {
        let plan = run_validate_offline(
            std::slice::from_ref(&ecosystem_dir),
            true,
            true,
            preset_path.as_deref(),
            &[],
        )
        .expect("ecosystem manifest pack must validate cleanly with --self-contained");

        assert_eq!(plan.traits.len(), 1);
        assert_eq!(plan.tools.len(), 1);
        assert_eq!(plan.skills.len(), 2);
        assert_eq!(plan.agents.len(), 2);
        assert_eq!(plan.total_count(), 6);

        let de_tool = plan
            .tools
            .iter()
            .find(|t| t.server_name == "data-engine-mcp")
            .unwrap();
        assert_eq!(
            de_tool.endpoint_config.get("url").and_then(|v| v.as_str()),
            Some(expected_de_url)
        );

        let skill_pos = |name: &str| plan.skills.iter().position(|s| s.name == name).unwrap();
        assert!(
            skill_pos("plan_semantic_join_path") < skill_pos("validate_and_execute_federated_sql")
        );

        let agent_pos = |name: &str| plan.agents.iter().position(|a| a.name == name).unwrap();
        assert!(
            agent_pos("Semantic Data Steward Agent")
                < agent_pos("Epistemic Benchmark & Governance Agent")
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_legacy_sample_agents_backward_compatibility() {
    let root = create_temp_dir("aad-legacy-test");
    let sample_dir = root.join("sample-agents");
    std::fs::create_dir_all(&sample_dir).unwrap();
    std::fs::write(
        sample_dir.join("legacy-skill.yaml"),
        r#"type: Skill
name: legacy_skill
description: Legacy skill
definition: Do legacy work.
"#,
    )
    .unwrap();
    std::fs::write(
        sample_dir.join("legacy-agent.yaml"),
        r#"type: Agent
name: Legacy Agent
description: Legacy agent
agent_definition: You are a legacy agent.
attached_skills:
  - legacy_skill
"#,
    )
    .unwrap();

    let plan = run_validate_offline(&[sample_dir], false, false, None, &[])
        .expect("legacy sample-agents YAML manifests should remain valid");
    assert_eq!(plan.skills.len(), 1);
    assert_eq!(plan.agents.len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn test_full_ecosystem_hydration_dry_run_idempotency_and_export_roundtrip() {
    let (base_url, store) = spawn_mock_aad_server(Some(EXPECTED_TOKEN.to_string())).await;
    let client = AadApiClient::new(
        base_url.clone(),
        Some(EXPECTED_TOKEN.to_string()),
        Duration::from_secs(10),
    )
    .unwrap();

    let root = create_temp_dir("aad-hydration-test");
    write_synthetic_ecosystem_fixtures(&root);
    let ecosystem_dir = root.join("manifests/ecosystem");
    let vars = TemplateVars::default();
    let loaded = load_manifests(&[ecosystem_dir], true, &vars).unwrap();
    let plan = build_hydration_plan(loaded, true).unwrap();

    // 1. Dry-run must plan 6 creations and make ZERO mutations
    let dry_summary = apply_plan(&client, &plan, Uuid::nil(), true, true, true)
        .await
        .unwrap();
    assert!(dry_summary.dry_run);
    assert_eq!(dry_summary.total_created(), 6);
    assert_eq!(dry_summary.total_updated(), 0);
    {
        let s = store.lock().unwrap();
        assert!(s.traits.is_empty());
        assert!(s.tools.is_empty());
        assert!(s.skills.is_empty());
        assert!(s.agents.is_empty());
    }

    // 2. First live apply creates all 27 resources in dependency order with UUID resolution
    let first_summary = apply_plan(&client, &plan, Uuid::nil(), false, true, true)
        .await
        .unwrap();
    assert!(!first_summary.dry_run);
    assert_eq!(first_summary.traits_created, 1);
    assert_eq!(first_summary.tools_created, 1);
    assert_eq!(first_summary.skills_created, 2);
    assert_eq!(first_summary.agents_created, 2);
    assert_eq!(first_summary.total_updated(), 0);

    {
        let s = store.lock().unwrap();
        assert_eq!(s.traits.len(), 1);
        assert_eq!(s.tools.len(), 1);
        assert_eq!(s.skills.len(), 2);
        assert_eq!(s.agents.len(), 2);
        assert_eq!(s.agent_post_count, 2);
        assert_eq!(s.agent_put_count, 0);

        // Verify governance agent has resolved 1 sub-agent UUID + 1 skill UUID
        let orch = s
            .agents
            .values()
            .find(|a| a.name == "Epistemic Benchmark & Governance Agent")
            .unwrap();
        assert_eq!(orch.attached_agents.len(), 1);
        assert_eq!(orch.attached_skills.len(), 1);
        for sub_id in &orch.attached_agents {
            assert!(s.agents.contains_key(sub_id));
        }

        // Verify tools registered in degraded status when allow_degraded_tools=true
        assert!(
            s.tools
                .values()
                .all(|t| t.sync_status == SyncStatus::Degraded)
        );
    }

    // 3. Second live apply must be 100% idempotent (0 created, 27 updated, Agents use PUT)
    let second_summary = apply_plan(&client, &plan, Uuid::nil(), false, true, true)
        .await
        .unwrap();
    assert_eq!(second_summary.total_created(), 0);
    assert_eq!(second_summary.total_updated(), 6);
    {
        let s = store.lock().unwrap();
        assert_eq!(s.traits.len(), 1);
        assert_eq!(s.tools.len(), 1);
        assert_eq!(s.skills.len(), 2);
        assert_eq!(s.agents.len(), 2);
        assert_eq!(s.agent_post_count, 2);
        assert_eq!(s.agent_put_count, 2);
    }

    // 4. Get by name and table output
    let table = get_resources(&client, "all", None, OutputFormat::Table)
        .await
        .unwrap();
    assert!(table.contains("SemanticSteward"));
    assert!(table.contains("data-engine-mcp"));
    assert!(table.contains("plan_semantic_join_path"));
    assert!(table.contains("Epistemic Benchmark & Governance Agent"));

    let single_agent_json = get_resources(
        &client,
        "agent",
        Some("Semantic Data Steward Agent"),
        OutputFormat::Json,
    )
    .await
    .unwrap();
    assert!(single_agent_json.contains("Semantic Data Steward Agent"));

    // 5. Sync tools (single by name and all)
    let single_synced = sync_tools(&client, Some("data-engine-mcp")).await.unwrap();
    assert_eq!(single_synced, 1);
    let all_synced = sync_tools(&client, None).await.unwrap();
    assert_eq!(all_synced, 1);
    {
        let s = store.lock().unwrap();
        assert!(s.tools.values().all(|t| t.sync_status == SyncStatus::Synced));
    }

    // 6. Export to multi-doc file AND structured directory, then validate offline --self-contained
    let tmp_dir = std::env::temp_dir().join(format!("aad-ctl-export-{}", Uuid::new_v4()));
    let export_file = tmp_dir.join("bundle.yaml");
    let export_tree = tmp_dir.join("tree");

    let exported = export_resources(&client, None, Some(&export_file), None, false)
        .await
        .unwrap();
    assert_eq!(exported.len(), 6);
    let file_plan = run_validate_offline(&[export_file], false, true, None, &[]).unwrap();
    assert_eq!(file_plan.total_count(), 6);

    export_resources(&client, None, None, Some(&export_tree), false)
        .await
        .unwrap();
    let tree_plan = run_validate_offline(&[export_tree.clone()], true, true, None, &[]).unwrap();
    assert_eq!(tree_plan.total_count(), 6);
    assert!(export_tree.join("traits").is_dir());
    assert!(export_tree.join("tools").is_dir());
    assert!(export_tree.join("skills").is_dir());
    assert!(export_tree.join("agents").is_dir());

    let _ = std::fs::remove_dir_all(&tmp_dir);

    // 7. Delete by canonical name (including --hard for Agent)
    let deleted_agent_id = delete_resource(
        &client,
        "agent",
        "Epistemic Benchmark & Governance Agent",
        true,
    )
    .await
    .unwrap();
    {
        let s = store.lock().unwrap();
        assert!(s.hard_deletes.contains(&deleted_agent_id));
        assert_eq!(s.agents.len(), 1);
    }

    delete_resource(&client, "skill", "plan_semantic_join_path", false)
        .await
        .unwrap();
    delete_resource(&client, "tool", "data-engine-mcp", false)
        .await
        .unwrap();
    delete_resource(&client, "trait", "SemanticSteward", false)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_auth_rejection_returns_401_without_leaking_token() {
    let (base_url, _store) = spawn_mock_aad_server(Some(EXPECTED_TOKEN.to_string())).await;
    let bad_client = AadApiClient::new(
        base_url,
        Some("wrong-secret-token".to_string()),
        Duration::from_secs(5),
    )
    .unwrap();

    let err = bad_client.list_agents().await.unwrap_err();
    match &err {
        CtlError::ApiError { status, .. } => assert_eq!(*status, 401),
        other => panic!("expected 401 ApiError, got {:?}", other),
    }
    assert!(!err.to_string().contains("wrong-secret-token"));
}

#[test]
fn test_register_tool_request_allow_degraded_sync_defaults_false() {
    let json = serde_json::json!({
        "server_name": "test-mcp",
        "transport_type": "http",
        "endpoint_config": {"url": "http://localhost:8082/mcp"},
        "owner_id": "00000000-0000-0000-0000-000000000000"
    });
    let req: RegisterToolRequest = serde_json::from_value(json).unwrap();
    assert!(!req.allow_degraded_sync);
}
