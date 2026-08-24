# Agent Development UI & Testing Kit PRD

## Overview
The **Agent Development UI & Testing Kit** provides an interactive web dashboard for developing, testing, visualising, refactoring, and debugging declarative AI agents and project memory stored in **Agent-As-Data (AAD)**.

Built using **Angular 18+ (Standalone Components, Angular Material, RxJS, and TailwindCSS)** following the architectural container patterns established in `sward-warden/sw-fe-container`, this container serves as the primary visual IDE and testing studio for agent developers.

---

## Architecture & Container Structure

Following `sward-warden`, the frontend is containerized in `aad-fe-container`:

```mermaid
graph TD
    subgraph Frontend ["aad-fe-container (Angular 18 + TailwindCSS)"]
        RegistryView["1. Agent Registry & Builder"]
        TestingKit["2. Interactive Agent Testing Studio"]
        NetworkGraph["3. Mermaid Delegation Graph Visualizer"]
        RefactorStudio["4. Agent Refactoring & Compression Lab"]
        KnowledgeLab["5. Knowledge Base Lab & SPO Triples"]
        ToolManager["6. Remote Tool Manager"]
        Visualizer["7. Interactive Graph Visualizer"]
    end

    subgraph Backend ["aad-be-container (Rust Microservice)"]
        REST["REST API Server (/api/v1)"]
        SSE["SSE Token Stream"]
    end

    RegistryView --> REST
    TestingKit <-->|SSE Token Stream| SSE
    NetworkGraph --> REST
    TestingStudio --> REST
    ToolManager --> REST
    Visualizer --> REST
```

---

## Key UI Modules & Features

**UI Consistency Requirement:** For Traits, Tools, Skills, and Agents edit views, the fields `Name`, `Owner`, `Description`, and `Tags` must be presented as the top lines on the view with consistent labels (`Name`, `Owner`, `Description`, `Tags`).

### 1. Declarative Agent Registry & Builder Module
- **Visual Agent Editor**: Form fields for `name`, `description`, `tags`, `implements_traits`, `model`, `agent_definition` (system prompt), `tools`, `available_skills`, and `available_agents`.
  - **Active Trait Filtering & Hover Descriptions**: Search filter for active implemented traits (`implements_traits`) and tooltips displaying contract descriptions on hover.
- **Guardrail Configurator**: Visual JSON/rule builder for `incoming_guardrails` and `outgoing_guardrails`.
- **RBAC Group Assignment**: Interface to select `owner_id`, `read_groups`, `write_groups`, and `execute_groups`.
- **Version Lineage Viewer**: Inspect historical snapshots from `agent_revisions` with visual side-by-side diffing.

### 2. Interactive Agent Testing Studio (Playground)
- **Live Execution Workbench**: Test synchronous execution (`POST /api/v1/agents/:id/execute`) and prompt discovery execution (`search-and-execute`).
- **Real-Time SSE Token Streaming**: Renders streaming tokens, reasoning logs, and tool call invocations in real-time.
- **Dynamic Trait Mapping Overrides**: UI controls to override `trait_mappings` during test executions (e.g. mapping `trait:SecurityAuditor` to a custom test agent UUID).
- **Contract Verification Tester**: Live badge showing pass/fail status of `verify-contract` semantic fit and trait compatibility checks.

### 3. Agent Delegation Network Visualizer
- **Interactive Mermaid / Canvas Graph**: Visualizes agent hierarchies, sub-agent delegation links (`available_agents`), and skill dependencies (`available_skills`).
- **Live Filtering**: Filter visual graph by trait interface, ownership group, or tag.

### 4. Agent Refactoring & Compression Lab
- **Overlap & Duplication Scanner**: Trigger cluster analysis (`POST /api/v1/agents/refactor/analyze`) to discover duplicate or conflicting agents.
- **Harmonization & Merge Diff Viewer**: Review suggested merges or deliberate contradiction labels before applying changes to `agent_revisions`.

### 5. Knowledge & SPO Tuple Inspector
- **Hybrid Knowledge Search**: RAG vector query input (`POST /api/v1/knowledge/search`) displaying semantic chunk similarity scores alongside Subject-Predicate-Object relation tuples (`knowledge_tuples`).
- **Graph Traversal Tree**: Interactive multi-hop entity graph visualizer.

### 6. Remote Tool Manager
- **Tool Ingestion**: Register external MCP servers Stdio commands or SSE transport URLs (`POST /api/v1/agents/tools/register`).
- **Cached Tool & Schema Browser**: Inspect cached tool argument schemas, descriptions, and type signatures retrieved from remote servers.

### 7. Interactive Graph Visualizer
- **Mermaid D3 Network Architecture**: An interactive map rendering live declarative agent architectures, sub-agent delegation (`agent:`), utilized remote tools (`mcp:`), and deterministic skill executions (`skill:`).
- **Clean Labeling & Hover States**: Version numbers, tool counts, and item descriptions are natively injected into node hover tooltips to avoid cluttering visual text labels.

### 8. Workbench (Multiuser Conversational Threads)
- **2-Column Layout**:
  - **Left Sidebar**: Collapsible threads list showing historical conversations (filtered by current active `userid`) with a global search filter.
  - **Right Workspace**: A resizable split-pane area containing the conversation and editor views, topped by a global action bar.
    - **Top Bar**: Contains the global application navigation menu, context badges, a solid blue "New Thread" button, layout view toggles, and a user profile button.
    - **Conversation Pane (Left)**: Conversational chat interface for the active thread. Features a click-to-edit thread title header and a message input area. Supports direct URL routing via `/workbench/:threadId`.
    - **Editor Pane (Right)**: Direct content editing and review area (e.g., code diffs, files, or context modified through conversation).

### 9. Robot Framework Integration Testing Suite (Ref: `sward-warden/integration-tests`)
- **Declarative User Journey Robot Tests**: `/integration-tests/tests/*.robot` test cases mapping 1-to-1 to all 9 user journeys in `user-journeys-spec.md`.
- **Python Integration Libraries**: Custom Python helper modules (`AADRequests.py`, `TestSeed.py`) extending Robot Framework for authenticated REST/SSE requests, database seeding, and OCC state reset.
- **Local Dev & Garden Test Runners**: `run-tests-local.sh` local pre-flight runner verifying backend (`http://localhost:8080`) and frontend (`http://localhost:4200`) before running `robot` suites, integrated into Kubernetes via `garden.yml` (`kind: Test`).

---

## User Journeys: Testing Agents, Skills & Traits via UI

### Journey 1: Interactive Agent Testing via Playground
**Scenario**: A developer finishes modifying an agent's system prompt and wants to verify its behavior before saving.
1. The developer navigates to the **Interactive Agent Testing Studio**.
2. They select the agent from the registry and provide a test input payload.
3. The UI opens a real-time SSE stream, passing the input to the backend, which is now powered by the `rig-core` LLM integration.
4. Tokens are streamed back to the frontend, along with tool call invocations, rendering reasoning logs in real-time.

### Journey 2: Trait Contract Verification Testing
**Scenario**: A developer has built a new agent that is intended to map to the `SecurityAuditor` trait and wants to test if it satisfies the contract.
1. In the **Interactive Agent Testing Studio**, the developer sets up a dynamic trait mapping, overriding the `SecurityAuditor` trait with the new agent's UUID.
2. Before execution, the UI invokes `POST /api/v1/agents/verify-contract`.
3. The live verification tester displays a pass/fail badge indicating if the semantic fit and compatibility checks succeeded, preventing a full run if the contract fails.

### Journey 3: Skill Execution Determinism vs. Agent Reasoning
**Scenario**: A developer wants to verify that a registered Skill operates deterministically compared to an Agent's probabilistic reasoning.
1. The developer runs a deterministic test suite for a `json-log-formatter` Skill within the UI.
2. The UI directly triggers the skill and validates the JSON output against the required schema.
3. Contrastingly, they run a probabilistic test for a reasoning Agent in the playground, observing the varying generated tokens and LLM decision-making process.

---

## Technical Stack Alignment (Ref: `sward-warden/sw-fe-container`)


- **Framework**: Angular 18+ (Standalone Components, Signals, RxJS 7.8+).
- **UI Library**: Angular Material 18 (`@angular/material`), Material Symbols (`material-symbols`).
- **Styling**: TailwindCSS (`tailwindcss`, `@tailwindcss/forms`, `@tailwindcss/container-queries`), Fontsource Inter (`@fontsource/inter`).
- **Visualization**: `mermaid.js` for interactive diagram rendering.
- **Container Build**: Multi-stage `Dockerfile` (`node:22-alpine` build + `nginx:alpine` runtime).
- **SPA Ingress Routing**: Custom `nginx.conf` handling static asset caching (`1y`), CORS map headers, health checks (`/alive`, `/ready`), and fallback SPA routing (`try_files $uri /index.html`).
- **Garden & Kubernetes Orchestration**: Deployment definition in `garden.yml` (`kind: Deploy`, `type: helm`, `oci://ghcr.io/polecatworks/mfe-shell/helm/nginx-view`).
- **Multi-Arch CI/CD Pipeline**: GitHub Actions workflow (`.github/workflows/aad-fe-docker-publish.yml`) featuring `dorny/paths-filter`, GHCR manifest inspection by Git content SHA, parallel `linux/amd64` and `linux/arm64` Buildx compilation, multi-arch manifest merging, and automated dev rollout via `kubectl rollout restart`.

---

## Related PRDs & Specs

- [Master PRD](./agent-as-data-prd.md)
- [Agent Registry PRD](./agent-registry-execution-prd.md)
- [Knowledge System PRD](./knowledge-data-system-prd.md)
