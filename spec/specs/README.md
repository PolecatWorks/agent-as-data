# Task Specifications (Specs) Registry

This directory contains task specifications and historical build snapshots for **Agent-As-Data (AAD)**. 

## Specification Lifecycle & Status Rules
- **Status Types**:
  - `draft`: Spec is under active authoring or refinement. **Modifications are only permitted in `draft` status**.
  - `complete`: Features and test definitions are fully implemented. **Complete specs are immutable** (cannot be modified except to transition to `deprecated`).
  - `deprecated`: Historical spec superseded by a newer specification.
- **Task-Focused & Ephemeral**: Spec files describe specific implementation tasks, schema designs, API contracts, and user journeys.
- **Historical Auditability**: Past specs are preserved to observe what was built and why design choices evolved.

## Implementation Build Order & Dependency Graph

The modular specifications must be implemented in the following strict dependency sequence:

```mermaid
flowchart TD
    subgraph CrossCuttingSpecs ["Cross-Cutting Architectural Reference Specs"]
        SchemaSpec["agent-schema-spec.md<br/>(Master DDL & REST/MCP Payloads)"]
        JourneysSpec["user-journeys-spec.md<br/>(Sequence Flows & End-to-End User Journeys)"]
    end

    subgraph Phase1 ["Phase 1: Foundation Layer"]
        Spec01["01-core-storage-spec.md<br/>(Database DDL & Migrations)"]:::complete
    end

    subgraph Phase2 ["Phase 2: Core Domain Engines"]
        Spec02["02-knowledge-engine-spec.md<br/>(RAG & Graph Store Engine)"]:::complete
        Spec03["03-declarative-agent-registry-spec.md<br/>(Agent & Skills Registry)"]:::complete
    end



    subgraph Phase3 ["Phase 3: Execution Runtimes & MCP"]
        Spec04["04-mcp-server-spec.md<br/>(Native MCP Server & Remote MCP Caching)"]:::complete
        Spec05["05-execution-guardrails-engine-spec.md<br/>(Sync/Async Runtimes & OCC State Sync)"]:::complete
    end



    subgraph Phase4 ["Phase 4: Quality, Compiler & Developer UI"]
        Spec06["06-agent-test-judge-engine-spec.md<br/>(Unit Testing & LLM-as-a-Judge Engine)"]:::complete
        Spec07["07-refactoring-compiler-engine-spec.md<br/>(Refactoring & Pre-Flight Compiler)"]:::complete
        Spec08["08-developer-ui-studio-spec.md<br/>(Angular 18+ Developer UI Workbench)"]:::complete
        Spec10["10-workbench-spec.md<br/>(Workbench File Management)"]:::complete
    end

    subgraph Phase5 ["Phase 5: Codebase Architecture & Agent Automation"]
        Spec09["09-backend-modular-architecture-spec.md<br/>(Backend Modular Layout & Webservice Routing)"]:::complete
        Spec11["11-workspace-agent-tool-execution-spec.md<br/>(Rig Tool Trait & Autonomous Multi-Turn Execution)"]:::complete
        Spec12["12-workbench-multiturn-context-and-agent-dispatch-spec.md<br/>(Multi-Turn Conversational History & Dynamic Dispatch)"]:::complete
    end

    subgraph Phase6 ["Phase 6: Workbench Benches & Project Memory"]
        Spec13["13-workbench-benches-domain-and-scoped-execution-spec.md<br/>(Bench Domain, Schema & Scoped Execution)"]:::complete
        Spec14["14-workbench-benches-and-threads-ui-navigation-spec.md<br/>(Benches & Scoped Threads UI Navigation)"]:::draft
        Spec15["15-workbench-bench-working-memory-spec.md<br/>(Bench Working Memory & Rig Tools)"]:::draft
    end

    classDef complete fill:#10B981,stroke:#047857,stroke-width:2px,color:#ffffff;
    classDef draft fill:#3B82F6,stroke:#1D4ED8,stroke-width:2px,color:#ffffff;

    SchemaSpec -.->|Informs Schema & Payloads| Spec01
    SchemaSpec -.->|Informs Schema & Payloads| Spec02
    SchemaSpec -.->|Informs Schema & Payloads| Spec03
    SchemaSpec -.->|Informs Schema & Payloads| Spec04
    SchemaSpec -.->|Informs Schema & Payloads| Spec05

    JourneysSpec -.->|Validates User Flows| Spec04
    JourneysSpec -.->|Validates User Flows| Spec05
    JourneysSpec -.->|Validates User Flows| Spec07
    JourneysSpec -.->|Validates User Flows| Spec08

    Spec01 --> Spec02
    Spec01 --> Spec03

    Spec02 --> Spec04
    Spec03 --> Spec04
    Spec03 --> Spec05

    Spec05 --> Spec06
    Spec03 --> Spec07
    
    Spec04 --> Spec08
    Spec05 --> Spec08
    Spec06 --> Spec08
    Spec07 --> Spec08
    Spec08 --> Spec10
    Spec08 --> Spec09
    Spec10 --> Spec09
    Spec09 --> Spec11
    Spec10 --> Spec11
    Spec11 --> Spec12
    Spec12 --> Spec13
    Spec13 --> Spec14
    Spec14 --> Spec15
```

## Index of Task Specifications

| Phase / Scope | Specification Document | Status | Category / Scope | Primary PRD Reference | Dependencies & Role |
| :---: | :--- | :---: | :--- | :--- | :--- |
| **Reference** | [agent-schema-spec.md](./agent-schema-spec.md) | `draft` | Consolidated Schema DDL Reference, Indices, & REST/MCP Payload Specifications | [Master PRD](../prds/agent-as-data-prd.md) | **Cross-Cutting Spec**: Source of truth for database DDL tables and JSON payloads across Phases 1-4 |
| **Reference** | [user-journeys-spec.md](./user-journeys-spec.md) | `draft` | Sequence Diagrams & End-to-End User Journeys (Knowledge, RAG, MCP, Compiler) | [Master PRD](../prds/agent-as-data-prd.md) | **Cross-Cutting Spec**: E2E integration test criteria and sequence flows validating Phases 2-4 |
| **Phase 1** | [01-core-storage-spec.md](./01-core-storage-spec.md) | `complete` | Database DDL Tables, Extension Init (`pgvector`), `sqlx` Migrations, & Seed Engine | [Master PRD](../prds/agent-as-data-prd.md) | Root Phase 1 Spec (Informed by `agent-schema-spec.md`) |
| **Phase 2** | [02-knowledge-engine-spec.md](./02-knowledge-engine-spec.md) | `complete` | Text Chunks RAG, SPO Graph Store, Canonical Entity Resolution, & Pruning | [Knowledge PRD](../prds/knowledge-data-system-prd.md) | Depends on [01-core-storage-spec.md](./01-core-storage-spec.md) |
| **Phase 2** | [03-declarative-agent-registry-spec.md](./03-declarative-agent-registry-spec.md) | `complete` | Declarative Agents, Managed Skills, Skill Promotion, & Trait Verification | [Agent Registry PRD](../prds/agent-registry-execution-prd.md) | Depends on [01-core-storage-spec.md](./01-core-storage-spec.md) |
| **Phase 3** | [04-mcp-server-spec.md](./04-mcp-server-spec.md) | `complete` | Native MCP Server (Stdio/SSE), 11 Tools, & Remote MCP Registration | [Master PRD](../prds/agent-as-data-prd.md) | Depends on [02-knowledge-engine-spec.md](./02-knowledge-engine-spec.md) & [03-declarative-agent-registry-spec.md](./03-declarative-agent-registry-spec.md) |
| **Phase 3** | [05-execution-guardrails-engine-spec.md](./05-execution-guardrails-engine-spec.md) | `complete` | Sync/Async Execution Engine, Guardrails, OCC State Sync, & Webhooks | [Agent Registry PRD](../prds/agent-registry-execution-prd.md) | Depends on [01-core-storage-spec.md](./01-core-storage-spec.md) & [03-declarative-agent-registry-spec.md](./03-declarative-agent-registry-spec.md) |
| **Phase 4** | [06-agent-test-judge-engine-spec.md](./06-agent-test-judge-engine-spec.md) | `complete` | Probabilistic Unit Testing, LLM-as-a-Judge Evaluation, & CI Quality Gates | [Agent Registry PRD](../prds/agent-registry-execution-prd.md) | Depends on [03-declarative-agent-registry-spec.md](./03-declarative-agent-registry-spec.md) & [05-execution-guardrails-engine-spec.md](./05-execution-guardrails-engine-spec.md) |
| **Phase 4** | [07-refactoring-compiler-engine-spec.md](./07-refactoring-compiler-engine-spec.md) | `complete` | Overlap Cluster Refactoring Engine & Pre-Flight Agent Network Compiler | [Agent Registry PRD](../prds/agent-registry-execution-prd.md) | Depends on [03-declarative-agent-registry-spec.md](./03-declarative-agent-registry-spec.md) |
| **Phase 4** | [08-developer-ui-studio-spec.md](./08-developer-ui-studio-spec.md) | `complete` | Angular 18+ Web Dashboard (`aad-fe-container`), Interactive Testing Studio (Skills Layout & Prompt Inspector), & Rig Ollama Execution | [Agent UI PRD](../prds/agent-ui-testing-kit-prd.md) | Depends on [04-mcp-server-spec.md](./04-mcp-server-spec.md), [05-execution-guardrails-engine-spec.md](./05-execution-guardrails-engine-spec.md), [06-agent-test-judge-engine-spec.md](./06-agent-test-judge-engine-spec.md), & [07-refactoring-compiler-engine-spec.md](./07-refactoring-compiler-engine-spec.md) |
| **Phase 4** | [10-workbench-spec.md](./10-workbench-spec.md) | `complete` | Workbench Thread File Management UI | [Agent UI PRD](../prds/agent-ui-testing-kit-prd.md) | Depends on [08-developer-ui-studio-spec.md](./08-developer-ui-studio-spec.md) |
| **Phase 5** | [09-backend-modular-architecture-spec.md](./09-backend-modular-architecture-spec.md) | `complete` | Backend Codebase Layout, CLI Separation (`main.rs`), App Runtime (`lib.rs`), Webservice Router (`webserver/`), & Domain Models (`models/`) | [Master PRD](../prds/agent-as-data-prd.md) | Depends on Phases 1-4 |
| **Phase 5** | [11-workspace-agent-tool-execution-spec.md](./11-workspace-agent-tool-execution-spec.md) | `complete` | Rig `Tool` Trait Implementation, Autonomous Multi-Turn `AgentBuilder` Execution Loop, & Self-Correcting Tool Error Recovery | [Workspace Tools PRD](../prds/workspace-filesystem-tools-prd.md) | Depends on [09-backend-modular-architecture-spec.md](./09-backend-modular-architecture-spec.md) & [10-workbench-spec.md](./10-workbench-spec.md) |
| **Phase 5** | [12-workbench-multiturn-context-and-agent-dispatch-spec.md](./12-workbench-multiturn-context-and-agent-dispatch-spec.md) | `complete` | Multi-Turn Conversational History Retention, Adaptive Inference Timeout, & Thread-Aware Prompt Dispatch | [Workspace Tools PRD](../prds/workspace-filesystem-tools-prd.md) | Depends on [11-workspace-agent-tool-execution-spec.md](./11-workspace-agent-tool-execution-spec.md) |
| **Phase 6** | [13-workbench-benches-domain-and-scoped-execution-spec.md](./13-workbench-benches-domain-and-scoped-execution-spec.md) | `complete` | Benches Domain Models, Database Migrations, Scoped Filesystem APIs (`/tmp/workspace/benches/<id>`), and Rig Tool Scoping | [Workbench Bench & Thread PRD](../prds/workbench-bench-thread-prd.md) | Step 1 of Phase 6 (Depends on Spec 12) |
| **Phase 6** | [14-workbench-benches-and-threads-ui-navigation-spec.md](./14-workbench-benches-and-threads-ui-navigation-spec.md) | `draft` | Scoped Bench Switcher Dropdown, Inline Non-Modal Management, Offset Deletion Confirmation, Top Bar Breadcrumbs & Smart Routing | [Workbench Bench & Thread PRD](../prds/workbench-bench-thread-prd.md) | Step 2 of Phase 6 (Depends on Spec 13) |
| **Phase 6** | [15-workbench-bench-working-memory-spec.md](./15-workbench-bench-working-memory-spec.md) | `draft` | Bench Working Memory Schema (`bench_memory`), UI Scratchpad Editor Tab, and Rig Memory Tools (`read_bench_memory`, `update_bench_memory`) | [Workbench Bench & Thread PRD](../prds/workbench-bench-thread-prd.md) | Step 3 of Phase 6 (Depends on Spec 13 & 14) |
