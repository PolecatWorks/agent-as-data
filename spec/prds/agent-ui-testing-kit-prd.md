# Agent Development UI & Testing Kit PRD

## Overview
The **Agent Development UI & Testing Kit** provides an interactive web dashboard for developing, testing, visualising, refactoring, and debugging declarative AI agents and project memory stored in **Agent-As-Data (AAD)**.

Built using **Angular 18+ (Standalone Components, Angular Material, RxJS, and TailwindCSS)** following the architectural container patterns established in `sward-warden/sw-fe-container`, this container serves as the primary visual IDE and testing studio for agent developers.

---

## Architecture & Container Structure

Following `sward-warden`, the frontend is containerized in `aad-fe-container`:

```mermaid
graph TD
    subgraph Frontend ["aad-fe-container (Angular 18 + Material + TailwindCSS)"]
        TraitRegistry["1. Trait Contracts Registry (/traits)"]
        AgentRegistry["2. Agent Registry & Builder (/agents)"]
        SkillsRegistry["3. Skills Registry (/skills)"]
        TestingKit["4. Interactive Agent Testing Studio"]
        NetworkGraph["5. Mermaid Delegation Graph Visualizer"]
        RefactorStudio["6. Agent Refactoring & Compression Lab"]
        KnowledgeLab["7. Knowledge Base Lab & SPO Triples"]
        ToolManager["8. Remote Tool Manager"]
    end

    subgraph Backend ["aad-be-container (Rust Microservice)"]
        REST["REST API Server (/api/v1)"]
        SSE["SSE Token Stream"]
    end

    TraitRegistry --> REST
    AgentRegistry --> REST
    SkillsRegistry --> REST
    TestingKit <-->|SSE Token Stream| SSE
    NetworkGraph --> REST
    ToolManager --> REST
```

## UI Consistency & Standard Global Navigation

### Global Top Bar & Navigation Menu Specification
All views across the application must share an identical, standardized top bar (`h-14 bg-white border-b border-slate-200 shadow-sm`) and navigation menu to ensure a seamless, uniform developer experience:
- **Left Context / Title Area**: Displays a pill badge with the module icon and current workspace name (e.g. `Trait Contracts`, `Skills Registry`, `Agents Registry`, `Interactive Testing Studio`).
- **Primary View Action (Left of Menu)**: Contextual creation button styled consistently with a solid fill (`mat-flat-button color="primary"`), e.g., `+ New Trait`, `+ New Skill`, `+ New Agent`, or `+ New Thread`. Clicking this initializes a clean form in-place without triggering full route reloads.
- **Global Navigation (Hamburger Menu)**: An `appMenu` triggered by a standard hamburger icon (`menu`) providing one-click routing across all top-level workspaces:
  - `verified` -> `/traits` (Trait Contracts)
  - `dns` -> `/tools` (Tools)
  - `extension` -> `/skills` (Skills)
  - `app_registration` -> `/agents` (Agents)
  - `bug_report` -> `/interactive-testing` (Interactive Testing Studio)
  - `account_tree` -> `/network-visualizer` (Network Graph Visualizer)
  - `build_circle` -> `/refactoring-lab` (Refactoring & Compression Lab)
  - `library_books` -> `/knowledge-inspector` (Knowledge & SPO Tuple Inspector)
  - `work` -> `/workbench` (Workbench)
- **Top Bar Secondary Controls**: Divider line, layout/view toggle icon (`view_column`), and user profile avatar badge (`BG`).

### Appearance & Styling Uniformity
- **Layout Consistency**: 2-column split view across all major registries and studios (collapsible sidebar list with search/filter on the left, full edit/blueprint or execution workspace on the right).
- **Form Layout Standard**: For Traits, Tools, Skills, and Agents edit views, the fields `Name`, `Owner`, `Description`, and `Tags` must be presented as the top lines on the view with consistent labels (`Name`, `Owner`, `Description`, `Tags`).
- **Card Parity**: Sidebar cards across Agents, Skills, Tools, and the Interactive Testing Studio must share unified card styling (icon, title, version/meta tags, description snippet, and attached items summary badges like `N Skills` / `N Tools`).
- **Interactive Testing Studio Alignment with Skills Layout**:
  - **Collapsible Sidebar**: Left-hand target entity selector with smooth collapse/expand toggle (`w-72` expanded / `w-16` collapsed) matching Skills Registry navigation.
  - **Entity Filter & Search**: Pinned search input filtering across both agents and skills.
  - **Entity Context & Prompt Inspector**: Selection of an agent or skill displays the entity's full **Description** and **System Prompt / Definition** (`agent_definition` for agents, `definition` for skills) in an inspectable, collapsible card directly within the testing workspace so developers can reference instructions while testing.
  - **Unified Visual Hierarchy**: Consistent top action bar, status badges, and Material styling.
- **Independent Card List Scrolling & Viewport Isolation Standard**:
  - **Fixed Viewport Shell**: The global application shell (`app-root`, `html`, `body`, and `router-outlet + *`) is strictly locked to `100%` viewport height with `overflow: hidden`, preventing the outer page, browser window, or top navigation bar from ever scrolling.
  - **Isolated List Scrolling**: Any collection or list of objects/cards (including sidebar lists on `/skills`, `/agents`, `/traits`, `/tools`, `/workbench`, and `/network-visualizer`, cluster/redundancy columns in `/refactoring-lab`, entity selectors in `/interactive-testing`, RAG chunk results, and SPO tuple results in `/knowledge-inspector`) must be contained in a dedicated scroll container (`flex-1 min-h-0 overflow-y-auto`).
  - **User Interaction Expectation**: When a user hovers their mouse over a list of cards (e.g. the list of skills on `/skills` or target entities on `/interactive-testing`) and scrolls with their mouse wheel or trackpad, only the list of cards scrolls vertically. The top bar, search bar, and opposite workspace pane remain entirely stationary.
  - **Detail Form & Inner Container Scrolling**: The main detail/editor workspace pane independently scrolls its own form content when required without bubbling scroll events to the outer viewport shell.

---

## Key UI Modules & Features

### 1. Declarative Agent Registry & Builder Module (`/agents`)
- **Visual Agent Editor**: Form fields for `name`, `description`, `tags`, `implements_traits`, `uses_traits`, `model`, `agent_definition` (system prompt), `tools`, `available_skills`, and `available_agents`.
  - **`implements_traits`**: Traits this agent actively implements/satisfies.
  - **`uses_traits`**: Traits this agent depends on or delegates to (without implementing them).
  - **Active Trait Filtering & Hover Descriptions**: Search filter for active implemented traits and tooltips displaying contract descriptions on hover.
- **Skill & Tool Count on Cards**: Agent sidebar cards display `N Skills` and `N Tools` counts to provide at-a-glance dependency insight.
- **Guardrail Configurator**: Visual JSON/rule builder for `incoming_guardrails` and `outgoing_guardrails`.
- **RBAC Group Assignment**: Interface to select `owner_id`, `read_groups`, `write_groups`, and `execute_groups`.
- **Version Lineage Viewer**: Inspect historical snapshots from `agent_revisions` with visual side-by-side diffing.

### 2. Skills Registry Module (`/skills`)
- **Visual Skill Editor**: Form fields for `name`, `description`, `tags`, `definition` (instructions), `implements_traits`, `uses_traits`, `attached_tools`, and `attached_skills`.
- **Skill & Tool Count on Cards**: Skill sidebar cards mirror the agent card layout — displaying `N Skills` and `N Tools` counts for at-a-glance dependency insight (consistent UI parity with the Agent Registry).
- **Promote to Agent**: One-click `POST /api/v1/skills/:id/promote` converts a skill into a full declarative agent.
- **Schemas & Mappings Tab**: Dedicated sub-tab for editing `input_schema`, `output_schema`, and `implementation` (JSONB) configuration.

### 3. Trait Contracts Registry (`/traits`)
- **Trait Definition Editor**: CRUD workspace for `name`, `description`, `owner_id`, `capability_requirements`, `behavioral_invariants`, `evaluation_criteria`, `tags`, and `guardrails`.
- **Idempotent Create**: Trait creation uses upsert semantics (`ON CONFLICT (name) DO UPDATE`) so duplicate names update in place rather than failing with a constraint error.
- **Sync with Backend**: Trait editor actions (create, update, delete) are verified by integration tests (`test_journey_11_trait_editor_ui.robot`) to confirm UI and backend remain in sync.

### 4. Interactive Agent Testing Studio (`/interactive-testing`)

```mermaid
flowchart TD
    subgraph TestingStudioLayout ["Interactive Testing Studio Layout (Skills Registry Alignment)"]
        subgraph LeftSidebar ["Left Sidebar (Collapsible: w-72 / w-16)"]
            SidebarHeader["Header + Collapse/Expand Toggle"]
            SearchInput["Search Filter (Agents & Skills)"]
            EntityList["Scrollable Entity Cards List\n(Name, Type, Version, Tags, N Skills / Tools)"]
        end
        
        subgraph RightWorkspace ["Right Workspace (Execution & Inspection)"]
            TopActionBar["Top Bar: Entity Title, Model Selector (e.g. qwen2.5-coder:14b), Status (IDLE/RUNNING), Execute Button"]
            
            subgraph EntityInspector ["Selected Entity Context & Prompt Inspector"]
                EntityDesc["Description Panel (Expanded metadata & purpose)"]
                PromptViewer["System Prompt / Definition Inspector\n(agent_definition / skill definition in formatted block)"]
            end
            
            subgraph ExecutionControls ["Execution & Streaming Console"]
                TestInputs["Prompt Input + Optional Webhook URL + Model Override"]
                TerminalLog["Dark Terminal Window (Real-time SSE token stream, logs, tool invocations)"]
                FinalOutputCard["Final LLM Output Inspector (Parsed output from LLM + Agent/Skill)"]
            end
        end
    end

    LeftSidebar -->|Select Agent or Skill| EntityInspector
    ExecutionControls -->|Execute Request| BackendAPI["POST /api/v1/agents/:id/execute\n(or /api/v1/skills/:id/execute)"]
    BackendAPI -->|Rig-Core + Local Ollama (qwen2.5-coder:14b)| LLMRuntime["Ollama Runtime (http://localhost:11434)"]
    LLMRuntime -->|SSE Token Stream & Final Text| TerminalLog
    LLMRuntime -->|Final LLM Response| FinalOutputCard
```

- **Skills-Aligned Layout & Styling**:
  - **Collapsible Entity Sidebar**: Responsive left panel (`w-72` expanded / `w-16` collapsed) with toggle control and scroll isolation.
  - **Entity Cards with Metadata Badges**: Renders unified card styling with icons (`smart_toy` for agents, `extension` for skills), name, version pills (`v1.0.0`), description snippets, and attached items badges (`N Skills`, `N Tools`).
- **Live Description & System Prompt / Definition Inspector**:
  - Automatically loads and displays the active entity's **Description** and full **Prompt / Definition** (`agent_definition` for agents, `definition` for skills).
  - Presented in an inspectable, collapsible context card with monospace formatting and easy copy/reference controls, allowing developers to inspect behavioral instructions and constraints directly alongside test executions.
- **Rig-Powered Execution with Local Ollama Integration**:
  - Execution requests dispatch to the backend execution engine powered by `rig-core`.
  - Configured to connect to the developer's local Ollama instance (`http://localhost:11434` / `OLLAMA_API_BASE_URL`), defaulting to models such as `qwen2.5-coder:14b` or the agent's defined model.
  - Injects `agent_definition` / `definition` as the system context prompt, passing the developer's test input through the Rig completion pipeline.
  - Directly supports executing both Agents and Skills, providing unified inspection across deterministic and reasoning entities.
- **Final LLM Output & Telemetry Inspector**:
  - Renders the complete, finalized output text from the LLM + Agent/Skill execution.
  - Accompanied by real-time SSE token streaming, status badges (`IDLE`, `RUNNING`, `COMPLETED`, `FAILED`), execution latency, and guardrail validation audit checks in the dark terminal console (`bg-slate-950`).
- **Dynamic Trait Mapping Overrides**: UI controls to override `trait_mappings` during test executions (e.g. mapping `trait:SecurityAuditor` to a custom test agent UUID).
- **Contract Verification Tester**: Live badge showing pass/fail status of `verify-contract` semantic fit and trait compatibility checks.

### 5. Agent Delegation Network Visualizer (`/network-visualizer`)
- **Interactive Mermaid / Canvas Graph**: Visualizes agent hierarchies, sub-agent delegation links (`available_agents`), and skill dependencies (`available_skills`).
- **Live Filtering**: Filter visual graph by trait interface, ownership group, or tag.

### 6. Agent Refactoring & Compression Lab (`/refactoring-lab`)
- **Overlap & Duplication Scanner**: Trigger cluster analysis (`POST /api/v1/agents/refactor/analyze`) to discover duplicate or conflicting agents.
- **Harmonization & Merge Diff Viewer**: Review suggested merges or deliberate contradiction labels before applying changes to `agent_revisions`.

### 7. Knowledge & SPO Tuple Inspector (`/knowledge-inspector`)
- **Hybrid Knowledge Search**: RAG vector query input (`POST /api/v1/knowledge/search`) displaying semantic chunk similarity scores alongside Subject-Predicate-Object relation tuples (`knowledge_tuples`).
- **Graph Traversal Tree**: Interactive multi-hop entity graph visualizer.

### 8. Remote Tool Manager (`/tools`)
- **Tool Ingestion**: Register external MCP servers Stdio commands or SSE transport URLs (`POST /api/v1/agents/tools/register`).
- **Cached Tool & Schema Browser**: Inspect cached tool argument schemas, descriptions, and type signatures retrieved from remote servers.

### 9. Workbench (Multiuser Conversational Threads) (`/workbench`)
- **2-Column Layout**:
  - **Left Sidebar**: Collapsible threads list showing historical conversations (filtered by current active `userid`) with a global search filter. Each thread is rendered as a rich card displaying the thread title, short-form date badge, description (if any), and string tags.
  - **Right Workspace**: A resizable split-pane area containing the conversation and editor views, topped by a global action bar.
    - **Top Bar**: Contains the global application navigation menu, context badges, a solid blue "New Thread" button, layout view toggles, and a user profile button.
    - **Conversation Pane (Left)**: Conversational chat interface for the active thread. Features a click-to-edit thread title header and a message input area. Supports direct URL routing via `/workbench/:threadId`.
    - **Editor Pane (Right)**: Direct content editing and review area (e.g., code diffs, files, or context modified through conversation).

### 10. Robot Framework Integration Testing Suite (Ref: `sward-warden/integration-tests`)
- **Declarative User Journey Robot Tests**: `/integration-tests/tests/*.robot` test cases mapping 1-to-1 to all 12 user journeys, covering 24 tests total (all currently passing).
- **Python Integration Libraries**: Custom Python helper modules (`AADRequests.py`) extending Robot Framework for authenticated REST requests, database seeding, and state verification.
- **Idempotent Seed Test**: `test_seed_exemplar_data.robot` seeds the database with exemplar data using upsert semantics — safe to re-run at any time without constraint conflicts.
- **UI Journey Tests (Playwright/Browser Library)**: `test_journey_11_trait_editor_ui.robot` drives a headless Chromium instance to verify the Trait Editor UI lifecycle (create, persist, delete) in sync with the backend REST API.
- **Local Dev & Garden Test Runners**: `run-tests-local.sh` local pre-flight runner verifying backend (`http://localhost:8080`) and frontend (`http://localhost:4200`) before running `robot` suites, integrated into Kubernetes via `garden.yml` (`kind: Test`).

---

## User Journeys: Testing Agents, Skills & Traits via UI

### Journey 1: Interactive Agent & Skill Testing via Playground
**Scenario**: A developer finishes modifying an agent's system prompt or a skill's instructions and wants to verify its behavior before saving.
1. The developer navigates to the **Interactive Agent Testing Studio** (`/interactive-testing`).
2. They select the agent or skill from the collapsible left sidebar list.
3. The right workspace displays the entity's complete **Description** and **System Prompt / Definition** (`agent_definition` or `definition`) in an inspectable panel, allowing the developer to examine the exact active prompt guidelines during the test.
4. The developer provides a test input payload and clicks **Execute**.
5. The UI opens a real-time SSE stream, passing the input to the backend, which is powered by the `rig-core` LLM integration.
6. Tokens are streamed back to the dark terminal window in real-time alongside tool call invocations, reasoning logs, and guardrail statuses.

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
