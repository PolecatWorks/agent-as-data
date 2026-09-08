# Agent Registry & Execution Engine PRD

## Overview
The **Agent Registry & Execution Engine** in **Agent-As-Data (AAD)** treats AI agents as declarative, queryable, version-controlled database records rather than hardcoded imperative logic. It provides dynamic agent serving, version history, RAG discovery, and execution runtimes with strict incoming/outgoing guardrail enforcement.

## Core Capabilities

### 1. Declarative Agent Storage, Versioning & Access Control
- **Agent Specification**: Stores `name`, `description`, `tags`, `agent_definition` (system prompt/persona), `model` config, `attached_tools`, `attached_skills`, `attached_agents`, `incoming_guardrails`, and `outgoing_guardrails`.
- **UI Consistency & Viewport Scrolling**: The Edit views for Traits, Tools, Skills, and Agents must display Name, Owner, Description, and Tags as the top lines on the view, maintaining consistent labels across all 4. Sidebar lists of cards (e.g. skills list on `/skills` or agents on `/agents`) must scroll independently in an isolated container without scrolling the global page or top bar.
- **Ownership & Group-Based Access Control (RBAC)**:
  - `owner_id` (UUID): Primary user or service account owner.
  - `read_groups` (TEXT[]): Array of group names / IDs granted read & discovery access.
  - `write_groups` (TEXT[]): Array of group names / IDs granted modification, refactoring, and deletion permissions.
  - `execute_groups` (TEXT[]): Array of group names / IDs permitted to run/execute the agent.
- **RBAC Delegation Context Inheritance**: When Agent A delegates to Agent B, the caller's identity (`caller_identity`) is inherited. Child invocation is rejected (`423 Forbidden`) if caller identity lacks `execute_groups` access to Agent B. Pre-delegation guardrails sanitize secrets/PII before cross-team transfer.
- **Immutable Revisions**: Any modification increments the agent's version counter and creates an immutable snapshot in `agent_revisions`, ensuring execution determinism.

### 2. Agent Traits, 3-Element Definition & Trait-Inherited Guardrails
- **3-Element Agent Trait Specification (`implements_traits`)**: Agents declare adherence to abstract traits (e.g. `CodeReviewer`, `SecurityAuditor`, `Compiler`). Traits are authored independently from data-type schemas via three core elements:
  1. *Capability Requirements*: Necessary tools, state access, or environmental interaction permissions (e.g. AST parser, read-only repo access).
  2. *Behavioral Invariants*: Strict rules and constraints the agent MUST ALWAYS or MUST NEVER violate (e.g. *MUST NEVER execute untrusted binaries*).
  3. *Evaluation Criteria*: Semantic guidelines and scoring rubrics for LLM judges or evaluators to grade performance.
- **Inherited & Mandatory Trait Guardrails**: Traits attach mandatory pre-execution and post-execution guardrails. When an Agent implements a Trait, it automatically inherits the Trait's baseline guardrails alongside any Agent-specific guardrails.
- **Semantic Compatibility Verification**: When an agent references a concrete sub-agent or trait implementation, AAD executes a semantic similarity and contract check (`POST /{{api_prefix}}/v1/agents/verify-contract`):
  - *Conceptual Fit Check*: Verifies vector similarity (`pgvector`) between the referring agent's prompt intent and the referenced agent's capabilities to ensure the sub-agent conceptually "fits".
  - *Trait Contract Validation*: Ensures the referenced agent satisfies required capability requirements, behavioral invariants, and guardrail boundaries.
- **Dynamic Contract Negotiation & Fallback Resolution**: Trait resolution uses Depth-First Search (DFS) topological cycle detection (`ERR_CIRCULAR_DELEGATION`). If a user's custom `trait_mappings` fail contract verification, AAD attempts fallback negotiation to default trait agents or rejects with `422 Unprocessable Entity` in strict mode.


### 3. Remote MCP Tool Registration, Synchronization & Execution Engine

Agent-As-Data enables autonomous agents and skills to utilize external capabilities by registering Model Context Protocol (MCP) servers. Registered MCP servers are persisted in the `tools` table, cached for performant prompt construction and RAG discovery, and dynamically invoked during agent execution loops.

#### 1. Transport Protocol: Stateless HTTP JSON-RPC 2.0 (Istio & Gateway Architecture)
In Kubernetes deployments operating behind **Istio Service Mesh**, Ingress Gateways, and Envoy sidecar proxies, persistent streaming transports and local process execution have severe operational liabilities:
- **Why Stdio is Inapplicable**: Stdio transport requires local process spawning on the same filesystem. In a Kubernetes microservice architecture, MCP tools run in isolated containers/pods (e.g. `aad-mcp-container`) across different nodes and namespaces, rendering Stdio impossible across network boundaries.
- **Why SSE is Avoided**: Server-Sent Events (SSE) require long-lived streaming connections that suffer from proxy timeouts, load-balancer reconnect flapping, Envoy buffer stalling, and fragile HTTP/1.1 vs HTTP/2 translation across Istio Ingress Gateways.
- **First-Class Protocol: Stateless HTTP POST JSON-RPC 2.0 (`http`)**:
  - All MCP lifecycle interactions (`initialize`, `notifications/initialized`, `ping`, `tools/list`, `tools/call`) are standard, idempotent HTTP POST requests with `Content-Type: application/json`.
  - Integrates seamlessly with Istio `VirtualService`, Envoy connection pooling, mTLS, circuit breakers, and standard Kubernetes ClusterIP DNS (`http://agent-as-data-mcp:8080/mcp`).
  - Natively implemented by the dedicated [MCP Server Container (`aad-mcp-container`)](./mcp-server-container-prd.md).

#### 2. Tool & Schema Fetching Strategies: Evaluation under Gateway Constraints
Because server-to-client streaming push notifications (`notifications/tools/list_changed`) are eliminated by the stateless gateway architecture, AAD evaluates three viable pull-based fetching strategies:

| Strategy | Description | Latency Impact | Network Overhead | Freshness | Resilience & Failure Handling |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Option 1: Eager Initial Fetch (on Register / Edit)** | Query `tools/list` synchronously via HTTP POST when the server is registered or updated via `POST /v1/agents/tools/register`. | None during agent execution (cache hit). | Zero ongoing network traffic. | High initially; static until explicit update or re-registration. | High: Fails fast on invalid URL, DNS resolution error, or gateway timeout. |
| **Option 2: Periodic Background Polling (Interval Cron)** | Background Tokio worker periodically (e.g. every 5–15 min) sends HTTP POST `tools/list` to all registered MCP services. | None during agent execution (cache hit). | Moderate recurring HTTP traffic across Istio mesh. | High (eventual consistency within interval). | Moderate: Temporary container restarts or cluster rollouts can trigger warning noise. |
| **Option 3: Lazy JIT with TTL (Stale-While-Revalidate)** | Tools cached on register with TTL (e.g. 300s). On agent execution, if `last_synced_at` > TTL, serve cached tools while triggering an async HTTP POST refresh. | Zero blocking latency on agent execution hot path. | Low: Only queries active tool servers during active user sessions. | High for actively utilized tools. | High: Preserves existing schema and serves stale cache if gateway or remote container fails. |
| **Option 4: Manual On-Demand Sync ("Sync Now" & CI/CD Webhook)** | Developer clicks "Sync Now" in UI or CI/CD triggers `POST /v1/agents/tools/{id}/sync` upon deploying a new MCP container version. | None during agent execution. | Zero automated network traffic. | Explicit: Synchronized precisely when new container versions deploy. | Maximum: Full developer and deployment pipeline control; ideal for GitOps/FluxCD. |

#### 3. Recommended Best Practice: Multi-Tier Gateway-Friendly Architecture
For Istio-managed Kubernetes environments, AAD implements a **Multi-Tier Pull & Webhook Strategy**:
- **Tier 1 — Mandatory Eager Discovery & Validation (On Register/Edit)**:
  - Immediately dispatch HTTP POST `initialize` and `tools/list` to the MCP endpoint upon receiving `POST /{{api_prefix}}/v1/agents/tools/register`.
  - Validate that the target responds with `200 OK` and conformant JSON Schema v7 tool definitions. If unreachable or non-conformant, registration fails immediately with `422 Unprocessable Entity`.
  - Store `cached_capabilities`, `cached_tools_count`, `sync_status = 'synced'`, and `last_synced_at = NOW()` in the `tools` database record.
- **Tier 2 — Explicit Manual & CI/CD Webhook Sync (`POST /{{api_prefix}}/v1/agents/tools/{id}/sync`)**:
  - Expose a dedicated sync endpoint and UI action. When GitOps (FluxCD) or CI/CD deploys an updated image of an MCP server container, the deployment pipeline calls `/sync` to immediately re-index tool schemas without container restarts.
- **Tier 3 — Configurable Synchronization Policy (`sync_policy`)**:
  - Each registered tool record declares its `sync_policy`:
    - `manual` (Default for stable production microservices): Schemas are only refreshed on registration, edit, or explicit `/sync` webhook trigger.
    - `lazy_revalidate`: If `last_synced_at` exceeds `cache_ttl_seconds` (default 300s), AAD uses the cached schema for the immediate turn while dispatching a non-blocking background Tokio task to revalidate via HTTP POST.
    - `interval`: Background Tokio worker sends HTTP POST `tools/list` every `sync_interval_seconds` (e.g., 300s).
- **Fault Tolerance & Schema Preservation**:
  - If a sync attempt fails (e.g., target pod restarting, gateway 503, connection timeout), AAD **MUST NOT wipe out the existing cached capabilities**. Instead, it marks `sync_status = "degraded"`, logs `last_sync_error`, and retains the last known good schema so ongoing agent workflows are unaffected.

#### 4. Tool Registration & Discovery Flow

```mermaid
sequenceDiagram
    autonumber
    actor User as Developer / UI / CI/CD
    participant BE as AAD Backend (aad-be-container)
    participant Istio as Istio Ingress / Service Mesh
    participant MCP as MCP Pod (aad-mcp-container)
    participant DB as PostgreSQL (tools table)

    User->>BE: POST /v1/agents/tools/register (server_name, transport="http", url)
    BE->>Istio: HTTP POST /mcp: JSON-RPC initialize
    Istio->>MCP: Forward to http://agent-as-data-mcp:8080/mcp
    alt Gateway Error / Pod Unreachable
        Istio-->>BE: 503 Service Unavailable or Timeout
        BE-->>User: 422 Unprocessable Entity (error: MCP server unreachable)
    else Success
        MCP-->>BE: 200 OK (capabilities, serverInfo)
        BE->>Istio: HTTP POST /mcp: JSON-RPC notifications/initialized
        BE->>Istio: HTTP POST /mcp: JSON-RPC tools/list
        Istio->>MCP: Forward tools/list
        MCP-->>BE: 200 OK tools/list: ["hello"] with JSON Schema
        BE->>DB: INSERT INTO tools (server_name, transport_type, endpoint_config, cached_capabilities, sync_status, last_synced_at)
        DB-->>BE: Row saved (UUID)
        BE-->>User: 201 Created (id, server_name, cached_tools_count=1, sync_status="synced")
    end
```

#### 5. Tool Ingestion Synchronization Modes

```mermaid
flowchart TD
    A["Tool Ingestion Trigger"] --> B{"Trigger Mechanism?"}
    B -->|"Registration or Edit"| C["Eager Synchronous Handshake and tools/list"]
    B -->|"UI Click or GitOps Webhook"| D["POST /tools/:id/sync Immediate Refresh"]
    B -->|"Execution Access"| E{"sync_policy is lazy_revalidate?"}
    B -->|"Periodic Interval Worker"| F{"sync_policy is interval?"}

    E -->|"Age exceeds TTL"| G["Serve Cached Tools Immediately and Trigger Async HTTP POST"]
    E -->|"Within TTL"| H["Serve Cached Tools Directly"]

    F -->|"Yes"| I["Background Tokio Task: HTTP POST tools/list"]
    F -->|"No"| J["Skip"]

    C --> K["HTTP POST JSON-RPC tools/list via Gateway"]
    D --> K
    G --> K
    I --> K

    K --> L{"HTTP 200 and Valid Schema?"}
    L -->|"Yes"| M["Update cached_capabilities, sync_status='synced', last_synced_at=NOW()"]
    L -->|"No, 5xx, or Timeout"| N["Preserve last known good schema, set sync_status='degraded'"]
```

#### 6. Remote Tool Execution Bridge & Runtime Flow

```mermaid
sequenceDiagram
    autonumber
    actor Client as User / Workbench Thread
    participant BE as AAD Execution Engine (Rig / Ollama)
    participant DB as PostgreSQL (tools & agents)
    participant MCP as Remote MCP Server (aad-mcp-container)

    Client->>BE: Execute Agent with attached_tools: [tool_uuid]
    BE->>DB: Fetch Agent and attached tools (cached_capabilities)
    DB-->>BE: Tool definitions (hello: name string)
    BE->>BE: Hydrate LLM System Prompt with tool definitions
    BE->>BE: LLM generates tool call: hello(name="Antigravity")
    BE->>MCP: JSON-RPC tools/call (name="hello", arguments: {name: "Antigravity"})
    MCP-->>BE: Result content: "Hello, Antigravity!" (isError: false)
    BE->>BE: Feed tool result back into LLM completion context
    BE-->>Client: Final response streaming with greeting
```

#### 7. Direct Remote Tool Verification Endpoint & Interactive Testing Flow
To enable developers and automated smoke tests to verify MCP tools in isolation without needing to execute full multi-turn LLM reasoning loops, the backend provides an interactive tool verification endpoint:
- **Endpoint**: `POST /{{api_prefix}}/v1/agents/tools/:id/test`
- **Request Payload**:
  ```json
  {
    "tool_name": "hello",
    "arguments": {
      "name": "Developer"
    }
  }
  ```
- **Execution Bridge**:
  1. Validates that the requested `tool_name` exists within the server's `cached_capabilities.tools`.
  2. Forwards the stateless JSON-RPC `tools/call` to the remote server's `endpoint_config.url`.
  3. Returns execution timing (latency in ms), formatted text content, and full JSON-RPC response envelope.

```mermaid
sequenceDiagram
    autonumber
    actor Dev as Developer / UI Studio
    participant BE as AAD Backend (tools API)
    participant DB as PostgreSQL (tools table)
    participant MCP as Remote MCP Server (aad-mcp-container)

    Dev->>BE: POST /tools/:id/test {tool_name: "hello", arguments: {name: "Alice"}}
    BE->>DB: Fetch tool server by :id and verify tool exists in cached_capabilities
    DB-->>BE: Server record & endpoint_config.url
    BE->>MCP: HTTP POST / {jsonrpc: "2.0", method: "tools/call", params: {name: "hello", arguments: {name: "Alice"}}}
    MCP-->>BE: 200 OK {result: {content: [{type: "text", text: "Hello, Alice!"}], isError: false}}
    BE-->>Dev: 200 OK {success: true, output: "Hello, Alice!", raw_result: {...}, latency_ms: 42}
```

#### 8. Integration Verification with Sample MCP Container (`aad-mcp-container`)
The dedicated [MCP Server Container (`aad-mcp-container`)](./mcp-server-container-prd.md) serves as the primary verification target for testing MCP tool ingestion, synchronization, and agent execution:
- **Server Address**: `http://127.0.0.1:8082` or `http://127.0.0.1:8082/mcp` (or internal Kubernetes DNS `http://agent-as-data-mcp:8080/mcp`).
- **Target Verification Tool**: `hello`, which accepts a `name` string input and returns a friendly greeting.
- **Automated Test Flow**: Registration -> Eager `tools/list` discovery -> Verification of cached schema in `tools` table -> Direct verification via `/test` -> Execution via agent turn -> Verifying greeting in LLM response.

### 4. Semantic RAG Discovery
- **Vector Embeddings**: Generates embeddings for agent definitions and tags in `agent_embeddings` (`pgvector`).
- **Prompt Search**: Endpoint `POST /{{api_prefix}}/v1/agents/search` returns the top `n` most relevant agents for a natural language task description.
- **Embeddings Synchronization**: A manual sync button located on the top bar of the Agents and Skills registries. When triggered, the system processes and separates the embeddings for the entity's `name`, `description`, and `prompt` (`agent_definition` or `definition`) into distinct vectordb entries (or graph nodes). This granular separation improves the retrieval system's ability to locate the most appropriate skill or agent based on specific context matching.

### 5. Execution Engine & Guardrails
- **Synchronous / Streaming**: Immediate execution (`POST /{{api_prefix}}/v1/agents/:id/execute`, `POST /{{api_prefix}}/v1/skills/:id/execute`, and `POST /{{api_prefix}}/v1/agents/search-and-execute`) streaming output tokens and tool calls via SSE.
- **Asynchronous Execution Jobs**: Job queue creation (`POST /{{api_prefix}}/v1/executions`) and status tracking (`GET /{{api_prefix}}/v1/executions/:id`) for long-running agent workflows.
- **AI Execution via Rig & Local Ollama Runtime**:
  - The backend integrates `rig-core` with the Ollama provider (`rig_core::providers::ollama::Client`).
  - Configured for local testing against Ollama (`http://localhost:11434` or environment-configured `OLLAMA_API_BASE_URL`), defaulting to `qwen2.5-coder:14b` (or the model declared in the agent's `model` field).
  - Injects the agent's `agent_definition` (system prompt) or skill's `definition` as the completion instructions, passing the user prompt through Rig's completion agent pipeline.
  - Returns the final LLM response text along with execution logs and guardrail validation status for live developer inspection.
### 6. Agent Refactoring & Compression Engine
- **Overlap & Duplication Detection**: Vector similarity scans across `agent_embeddings` identify candidate clusters of overlapping, duplicate, or conflicting agents (`POST /{{api_prefix}}/v1/agents/refactor/analyze`).
- **Conflict Resolution & Harmonization**: Analyzes candidate agent definitions to identify:
  - *Redundant Agents*: Merges duplicate agent prompt capabilities into unified master definitions.
  - *Unintended Contradictions*: Harmonizes conflicting system prompts, tool bindings, or guardrail rules.
  - *Deliberate Contradictions*: For agents intentionally designed with opposing viewpoints (e.g. `optimist-reviewer` vs `pessimist-security-auditor`), updates definitions to explicitly highlight the deliberate contrast and re-defines input/output payloads so they harmoniously co-exist.
- **Automated Versioning**: Applied changes create new version records in `agent_revisions`, preserving historical lineage.

### 7. Agent Network & Relationship Visualization Engine
- **Delegation & Skill Graph Generation**: Endpoint `GET /{{api_prefix}}/v1/agents/visualize` (or `GET /{{api_prefix}}/v1/agents/:id/visualize`) traverses the `attached_agents` sub-agent hierarchy and `attached_skills` bindings.
- **Dual Representation Output**: Returns the relationship graph in both formats in a single API response payload:
  - **`mermaid` (String)**: Renderable Mermaid flowchart diagram (e.g. `graph TD; AgentA -->|delegates| AgentB`).
  - **`graph_json` (JSON Object)**: Structured nodes and edges representation (`{ "nodes": [...], "edges": [...] }`) for programmatically rendering custom UI network graphs.

### 8. Probabilistic Agent Unit Testing & LLM-as-a-Judge Evaluation Engine
- **Declarative Test Suite Registration**: Agents can store associated test suites (`agent_test_suites` table) containing input payloads, expected output criteria, and assertion strategies.
- **Dual Evaluation Strategy**:
  - *1. Deterministic Schema & Guardrail Checks*: Validates input/output JSON schema compliance, required keys, regex patterns, and guardrail pass rates.
  - *2. Probabilistic LLM-as-a-Judge Evaluation*: Uses an independent evaluator agent persona ("Judge Agent") to assess probabilistic outputs against natural language rubrics (e.g. accuracy, tone, safety, constraint adherence) scoring 0.0 to 1.0.
- **Regression Detection & CI/CD Gates**: Endpoint `POST /{{api_prefix}}/v1/agents/:id/test` runs test suites against new agent prompt iterations. If pass rate or Judge scores fall below configurable thresholds (e.g., `score < 0.85`), modification is flagged as a regression and blocked from updating `agent_revisions`.
- **Test History Audit**: Test execution runs and Judge evaluation rubrics are persisted in `agent_test_runs` for auditability across version iterations.

### 9. Agent & Tool Usage Audit Logging Subsystem
- **Execution & Tool Call Telemetry**: Automatically records structured audit telemetry into `agent_usage_logs` whenever an agent is discovered, invoked, or executes a native/remote tool call.
- **Logged Attributes**:
  - `agent_id` & `agent_version`: Specific agent revision executed.
  - `caller_identity`: User ID, service account, or referring parent agent UUID.
  - `tool_calls` (JSONB): List of tools invoked, including tool name, arguments, execution duration (ms), status code, and error trace.
  - `token_metrics` (JSONB): Prompt tokens, completion tokens, total tokens, and estimated cost.
  - `guardrail_events` (JSONB): Pass/fail status of pre-execution (`incoming_guardrails`) and post-execution (`outgoing_guardrails`) checks.
- **Audit & Analytics APIs**: Endpoints `GET /{{api_prefix}}/v1/agents/:id/logs` and `GET /{{api_prefix}}/v1/analytics/usage` for querying tool invocation frequencies, token consumption trends, error rates, and caller activity.

### 10. Managed Skills Registry, Builder & Lifecycle Engine
- **Dedicated Skills Registry (`skills` Table)**: Managed database repository for deterministic, single-purpose skills (`name`, `description`, `input_schema`, `output_schema`, `implementation`). Agents bind to skills via `attached_skills`.
- **Skill vs. Agent Distinction**:
  - *Skills*: Direct, single-purpose execution routines without autonomous reasoning loops or sub-agent delegation.
  - *Agents*: Autonomous reasoners with system prompts, guardrails, dynamic tool choice, and sub-agent delegation.
- **Full Skills CRUD APIs**: 
  - `GET /{{api_prefix}}/v1/skills` — Lists all registered skills (with optional tag and name filter).
  - `GET /{{api_prefix}}/v1/skills/{id}` — Fetches the complete specification of a skill.
  - `POST /{{api_prefix}}/v1/skills` — Registers a new skill.
  - `PUT /{{api_prefix}}/v1/skills/{id}` — Updates an existing skill's schemas, tags, or execution template.
  - `DELETE /{{api_prefix}}/v1/skills/{id}` — Deletes/archives a skill.
- **Skill <-> Agent Lifecycle Actions**:
  - **Skill -> Agent Promotion (`POST /{{api_prefix}}/v1/skills/:id/promote`)**: Converts a growing skill into a full declarative agent, creating an `agent_definition`, wrapping it in guardrail defaults, and deprecating the original skill.
  - **Agent -> Skill Demotion (`POST /{{api_prefix}}/v1/agents/:id/demote`)**: Simplifies a prompt-wrapped agent down to a single-purpose deterministic skill entry.
- **Developer Guidance API (`GET /{{api_prefix}}/v1/skills/guidance`)**: Provides actionable feedback to developers on whether a proposed capability should be built as a Skill or an Agent based on complexity metrics.
- **Skills Registry & Builder UI**:
  - **Skills Registry**: An interactive UI dashboard showcasing registered skills, filtering by tag, search capabilities, and usage stats.
  - **Skills Builder**: A schema-driven editor enabling developers to construct and configure new skills, define deterministic input/output JSON schemas, configure code/MCP implementation templates, and trigger lifecycle actions (Promote/Demote) directly from the interface.

### 11. Distributed State Synchronization & Working Memory Persistence Subsystem
- **Three-Tier Memory Persistence**:
  - *Tier 1 (In-Memory)*: Transient working context during active LLM token generation loops.
  - *Tier 2 (Optimistic Concurrency Control - OCC)*: Working memory snapshots stored in PostgreSQL `executions.working_memory` (JSONB) with version locking (`execution_version`), rejecting stale writes during parallel sub-agent executions.
  - *Tier 3 (Append-Only Event Stream)*: Immutable audit telemetry in `agent_usage_logs`.
- **Transactional State Locking**: Critical guardrail transitions acquire temporary distributed locks on `execution_id` to guarantee single-writer safety across distributed agent workers.

### 12. Agent Network Compilation & Conceptual Validation Engine
- **Validation & Compilation API (`POST /{{api_prefix}}/v1/agents/compile` or `POST /{{api_prefix}}/v1/agents/:id/validate`)**: Performs pre-flight structural, semantic, and contractual verification across an agent network prior to execution deployment.
- **Verification Phases**:
  1. **Structural DAG Topology Verification**: Scans `available_agents` delegation trees for circular reference deadlocks, infinite loops, and unresolvable missing sub-agent UUIDs/traits.
  2. **Schema & Guardrail Contract Matching**: Verifies that outgoing response JSON schemas from parent agents align with incoming request JSON schemas and guardrail boundaries of child sub-agents.
  3. **Conceptual Cohesion & Semantic Fit Scoring**: Executes cosine similarity vector scans (`pgvector`) across parent and sub-agent prompt definitions to ensure sub-agents conceptually fit the referring domain context (flagging semantic mismatches).
- **Compilation Report & Diagnostics**: Returns a detailed compilation status (`status: "clean"` or `status: "compilation_errors"`) with line-level warning diagnostic messages (e.g. `ERR_CIRCULAR_DELEGATION`, `WARN_LOW_SEMANTIC_FIT`, `ERR_SCHEMA_MISMATCH`).

### 13. Architectural Safeguards, HaMS & Fail-Fast Validation
- **HaMS Health & Metrics Sidecar (`hams`)**: Serves out-of-band health probes (`/ready` readiness via `ProbeManual`, `/alive` liveness) and Prometheus metrics (`/metrics` / `stats/prometheus`) on dedicated port `8079`.
- **Unified Graceful Shutdown**: Links HaMS shutdown closures to Tokio `CancellationToken` and Axum's `.with_graceful_shutdown()`, cleanly draining active execution requests on `SIGINT`/`SIGTERM`.
- **Prometheus Telemetry Instrumentation**: Records HTTP execution telemetry via `axum-prometheus` and bridges metrics to HaMS via C-FFI callbacks (`prometheus_response_mystate`, `prometheus_response_free`).
- **Fail-Fast Early Startup Validation & Debug Delay**: Application configuration, YAML environment overrides, secret files, database connectivity, and required `pgvector` extensions are validated **at process startup** before opening the main webservice listener (`8080`). If initialization fails, an optional `fail_debug_delay` allows container inspection before exit.
- **Decoupled Job Queue**: Asynchronous background agent runs are isolate-tracked in the `executions` table, preventing long-running agent tasks from blocking vector discovery endpoints.
- **Deterministic Version Snapshots**: All executions bind to explicit `version` snapshots in `agent_revisions`, guaranteeing that agent behavior remains immutable even if an agent prompt is edited mid-task.
- **Guardrail Interceptors**: Strict pre-submission (`incoming_guardrails`) and post-execution (`outgoing_guardrails`) validation steps block malformed JSON or prompt injection attacks.
- **Reversible Schema Rollback**: All agent and execution schema migrations (`agents`, `agent_revisions`, `executions`, `skills`, `tools`) require paired forward (`.up.sql`) and reverse (`.down.sql`) scripts to guarantee safe rollbacks.











## User Journeys: Execution & Automated Testing

### Journey 1: Programmatic Trait Constraint & Guardrail Testing
**Scenario**: An automated CI script needs to verify that an agent correctly respects its inherited Trait constraints and guardrails when evaluating an ambiguous request.
1. The CI test queries the registry for the agent and determines the expected strict Trait behaviors (e.g. MUST NEVER expose PII).
2. It sends a series of edge-case payloads to `POST /{{api_prefix}}/v1/agents/:id/execute` with malicious or borderline inputs.
3. The LLM processes the input, but the `rig-core` powered backend intercepts the response based on the `outgoing_guardrails`.
4. The test verifies that the system appropriately blocks or sanitizes the output, returning a structural diagnostic failure to the caller instead of the raw LLM response.

### Journey 2: Automated Skill and Agent Evaluation via LLM-as-a-Judge
**Scenario**: A deployment gate in a CI/CD pipeline ensures that a newly promoted Agent performs better or equal to the deterministic Skill it replaced.
1. The pipeline triggers `POST /{{api_prefix}}/v1/agents/:id/test` for the newly promoted agent.
2. The engine first validates that the new agent's JSON output strictly conforms to the original Skill's expected schema (Deterministic Schema & Guardrail Check).
3. The engine then passes the probabilistic output to a designated "Judge Agent" (LLM-as-a-Judge) which evaluates the qualitative accuracy and reasoning trace.
4. The test passes if both the deterministic schema assertion and the probabilistic Judge score exceed the defined threshold (e.g. >0.85), allowing promotion to production `agent_revisions`.

### Journey 3: Interactive Developer Verification of Remote MCP Tools
**Scenario**: A developer deploys a new MCP container (or connects to an existing internal microservice) and needs to verify its tools and input schemas directly within the AAD Studio before attaching them to agents.
1. The developer navigates to `/tools` and registers a new tool server selecting `Transport Protocol: HTTP` with endpoint `http://localhost:8082`.
2. The system triggers eager handshake validation (`initialize`, `notifications/initialized`, `tools/list`), discovers available tools (e.g. `hello`), and stores the schemas into PostgreSQL with `sync_status: "synced"`.
3. The UI automatically displays the discovered tools, their parameter requirements, and descriptions.
4. The developer clicks **"Test Tool"**, reviews the prepopulated sample arguments (`{"name": "Alice"}`), and clicks **"Execute Tool"**.
5. The backend dispatches `POST /{{api_prefix}}/v1/agents/tools/:id/test`, executes `tools/call` over JSON-RPC 2.0 to `aad-mcp-container`, and displays the resulting greeting `"Hello, Alice!"` alongside execution duration (ms) in the UI console.
6. The developer clicks **"Sync Now"** at any time to refresh the tool definitions if new tools or schema updates are deployed to the remote server.

## Agent Execution Pipeline

```mermaid
sequenceDiagram
    autonumber
    actor Client as IDE / Client
    participant AAD as AAD Microservice
    participant RAG as pgvector RAG Index
    participant LLM as Target LLM Engine

    Client->>AAD: POST /{{api_prefix}}/v1/agents/search-and-execute {query, payload}
    AAD->>RAG: Vector Search Top 1 Matching Agent
    RAG-->>AAD: Returns Agent Definition & System Prompt
    AAD->>AAD: Evaluate incoming_guardrails
    AAD->>LLM: Hydrated Prompt & Tool Definitions
    LLM-->>AAD: Token & Tool Call Event Stream
    AAD->>AAD: Evaluate outgoing_guardrails
    AAD-->>Client: SSE Streaming Response / Job Completion
```

## Related PRDs & Specs
- [Agent-As-Data Core PRD](./agent-as-data-prd.md)
- [Knowledge & Data System PRD](./knowledge-data-system-prd.md)
- [Detailed Schema Specification](../specs/agent-schema-spec.md)


## UI Behavior Updates

- The **Register New** buttons for **Agents**, **Tools**, and **Traits** now initialize a fresh form without performing a router navigation. This eliminates the brief screen flicker previously observed when creating new items.
- Existing UI flows remain unchanged; the form state is reset and ready for input, improving user experience and stability.
