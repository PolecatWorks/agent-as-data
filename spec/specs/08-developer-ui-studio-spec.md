# Spec 08: Developer UI Container & Testing Studio (`aad-fe-container`)

**Status**: `complete`

## Overview & Scope
This specification defines the frontend web application container **`aad-fe-container`** built with **Angular 18+ (Standalone Components, Angular Material, RxJS, TailwindCSS, Fontsource Inter, and Mermaid.js)** mirroring the `sward-warden/sw-fe-container` architecture.

It specifies the comprehensive overhaul of the **Interactive Testing Studio (`/interactive-testing`)** to align with the visual styling, collapsible sidebar layout, and card parity of the Skills Registry, and adds the **Entity Context & Prompt Inspector** and **Rig-powered Ollama Execution** capabilities.

## Dependencies & References
- **Build Order Phase**: **Phase 4 (Developer UI & Workbench)**.
- **Dependencies**: Depends on [04-mcp-server-spec.md](./04-mcp-server-spec.md), [05-execution-guardrails-engine-spec.md](./05-execution-guardrails-engine-spec.md), [06-agent-test-judge-engine-spec.md](./06-agent-test-judge-engine-spec.md), and [07-refactoring-compiler-engine-spec.md](./07-refactoring-compiler-engine-spec.md).
- **PRD References**: [Agent UI & Testing Kit PRD](../prds/agent-ui-testing-kit-prd.md), [Agent Registry PRD](../prds/agent-registry-execution-prd.md), [Agent-As-Data Master PRD](../prds/agent-as-data-prd.md).
- **Research References**: [ui-build-container-reference.md](../research/ui-build-container-reference.md).

```mermaid
flowchart TD
    subgraph TestingStudioLayout ["Interactive Testing Studio Layout (Skills Alignment)"]
        subgraph LeftSidebar ["Left Sidebar (Collapsible w-72 / w-16)"]
            SidebarHeader["Header + Collapse/Expand Toggle"]
            SearchInput["Search Filter (Agents & Skills)"]
            EntityList["Scrollable Entity Cards List\n(Name, Type, Version, Tags, N Skills / Tools)"]
        end
        
        subgraph RightWorkspace ["Right Workspace (Execution & Inspection)"]
            TopActionBar["Top Bar: Entity Title, Model Selector (e.g. qwen2.5-coder:14b), Status (IDLE/RUNNING), Execute Button"]
            
            subgraph EntityInspector ["Selected Entity Context & Prompt Inspector"]
                EntityDesc["Description Panel (Purpose & Metadata)"]
                PromptViewer["System Prompt / Definition Inspector\n(agent_definition / skill definition in formatted block)"]
            end
            
            subgraph ExecutionControls ["Execution & Streaming Console"]
                TestInputs["Prompt Input + Optional Webhook URL + Model Override"]
                TerminalLog["Dark Terminal Window (Real-time SSE token stream, logs, tool invocations)"]
                FinalOutputCard["Final LLM Output Inspector (Parsed response from Rig + LLM)"]
            end
        end
    end

    LeftSidebar -->|Select Agent or Skill| EntityInspector
    ExecutionControls -->|Execute Request| BackendAPI["POST /api/v1/agents/:id/execute\n(or /api/v1/skills/:id/execute)"]
    BackendAPI -->|Rig-Core + Local Ollama (qwen2.5-coder:14b)| LLMRuntime["Ollama Runtime (http://localhost:11434)"]
    LLMRuntime -->|SSE Token Stream & Final Text| TerminalLog
    LLMRuntime -->|Final LLM Response| FinalOutputCard
```

---

## 1. UI Modules & Interactive Testing Studio Specifications

### 1.1 Visual Styling & Layout Parity with Skills Registry
- **Collapsible Sidebar Layout**: The left target entity selector uses a collapsible drawer (`w-72` when expanded, `w-16` when collapsed) with an expand/collapse toggle button (`chevron_left` / `chevron_right`).
- **Unified Entity Cards**:
  - Entity type icons: `extension` for Skills (indigo themed), `smart_toy` for Agents (indigo/blue themed).
  - Header: Entity name, truncate handling, and version badge (`v1.0.0` in emerald badge).
  - Body: Description text snippet with `line-clamp-2` and `leading-relaxed`.
  - Dependency count badges: `N Skills` and `N Tools` matching Skills and Agents registry cards.
  - Tags: `#tag` pills rendered at the bottom of the card.
  - Selection Highlight: Active entity card highlighted with indigo/amber border (`border-indigo-500` / `border-amber-500`), subtle shadow, and ring highlight (`ring-1`).
- **Independent List Scrolling**: The card list container is isolated (`flex-1 min-h-0 overflow-y-auto`) so mouse scrolling over the list never scrolls the outer page or top action bar.

### 1.2 Entity Context & Prompt Inspector
- When any Agent or Skill is selected from the list:
  - **Description**: Displays the entity's complete `description` in a clean metadata card.
  - **Prompt / Definition Viewer**: Displays the entity's full behavioral prompt (`agent_definition` for Agents, `definition` for Skills) in a dedicated, monospace-formatted code block with a one-click copy button.
  - Enables developers to continuously inspect system instructions, persona rules, and constraints while formulating test inputs and analyzing model responses.

### 1.3 Rig-Powered Local Ollama Execution
- **Model Selector**: UI control allowing selection/override of LLM model (defaulting to `qwen2.5-coder:14b`).
- **Backend Execution Flow**:
  - Dispatches `POST /api/v1/agents/:id/execute` (or skill execution endpoint) with `{ prompt, model, webhook_url }`.
  - Backend uses `rig_core::providers::ollama::Client` connecting to local Ollama (`http://localhost:11434` or `OLLAMA_API_BASE_URL`).
  - Injects `agent_definition` (or skill `definition`) as the system prompt.
  - Returns structured `ExecuteAgentResponse` containing the final LLM output, execution status, and audit telemetry.
- **Dark Terminal & Final Output Inspector**:
  - Live execution terminal (`bg-slate-950 text-slate-100 rounded-xl font-mono text-xs`) displaying real-time execution steps, status transitions (`IDLE`, `RUNNING`, `COMPLETED`), and guardrail logs.
  - Final Output card rendering the parsed LLM completion for immediate inspection.

---

## 2. Test Strategy & Verification Plan

### 2.1 Backend Integration & Unit Tests
- **Rig Ollama Execution Unit Test**: Test `execute_agent` handler using `rig-core` with mock/local Ollama responses.
- **Skill Execution Test**: Test executing skills with their `definition` prompt template.

### 2.2 Frontend Unit Tests (Jasmine / Karma)
- **`interactive-testing.component.spec.ts`**:
  - Test sidebar toggle expansion/collapse.
  - Test entity selection updating the active entity, description, and prompt inspector.
  - Test search query filtering across agents and skills.
  - Test `runExecution()` invoking API service and rendering output in the terminal and final output inspector.

### 2.3 End-to-End Robot Framework Tests
- Run `/integration-tests/run-tests-local.sh` and verify all tests pass cleanly.
