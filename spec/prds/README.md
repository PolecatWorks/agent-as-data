# Product Requirements Documents (PRDs) Registry

This directory contains the long-term, persistent Product Requirements Documents (PRDs) defining the architecture, objectives, data models, and features of **Agent-As-Data (AAD)**.

## Document Hierarchy & Structure

| Document | Description |
| :--- | :--- |
| [agent-as-data-prd.md](./agent-as-data-prd.md) | **Master PRD**: High-level platform architecture, core objectives, fail-fast startup validation, HaMS sidecar health/metrics with baseline startup telemetry (`tokio-metrics`), and Pod Endpoint Resolution Abstraction for telemetry tests. |
| [knowledge-data-system-prd.md](./knowledge-data-system-prd.md) | **Knowledge System PRD**: Long-term Project Memory, RAG vector chunks (`pgvector`), Graph Tuples (SPO), BREAD Operations UI, and native MCP tools. |
| [agent-registry-execution-prd.md](./agent-registry-execution-prd.md) | **Agent Registry PRD**: Declarative agent storage, versioning (`agent_revisions`), top `n` RAG search, execution engine (sync/async & guardrails), and Remote MCP Tool Ingestion, Synchronization & Execution Engine. |
| [agent-ui-testing-kit-prd.md](./agent-ui-testing-kit-prd.md) | **Agent UI PRD**: Developer UI container (`aad-fe-container`) built with Angular 18+, TailwindCSS, and Angular Material for agent development, live testing, SSE token streaming, Mermaid network graph visualization, MCP server management, and Dual-Mode Telemetry Endpoint Resolution Abstraction (Local vs. In-Cluster Pod IPs). |
| [workspace-filesystem-tools-prd.md](./workspace-filesystem-tools-prd.md) | **Workspace Tools & Agent Execution PRD**: Standardized Rig `Tool` trait implementations (`read_file`, `write_file`, `list_files`, etc.), autonomous multi-turn `AgentBuilder` execution loops, conversational history retention, structured tool execution presentation standards, and distributed pre-tool cancellation safeguards. |
| [workbench-bench-thread-prd.md](./workbench-bench-thread-prd.md) | **Workbench Benches, Threads & Workspace Memory PRD**: Isolated bench project workspaces (`/tmp/workspace/benches/<bench_id>`), immutable thread-to-bench scoping, modal-free inline management, active bench visual context & smart URL routing, phased bench memory, persistent action tracking with distributed cancellation (`thread_runs`), and UI Tool Execution Card rendering. |
| [workbench-multiturn-journeys-prd.md](./workbench-multiturn-journeys-prd.md) | **Workbench Multi-Turn Journeys PRD**: Details 3 multi-turn chat interaction journeys to validate file manipulation, context retention, and introduces a Code Execution MCP Server sandbox for running dynamic code securely. |
| [cli-manifest-tool-prd.md](./cli-manifest-tool-prd.md) | **CLI Manifest Tool PRD**: Defines the CLI extension `aad-be ctl` for applying, listing, and deleting Agents and Skills via YAML definitions interacting directly through the REST API. |
| [mcp-server-container-prd.md](./mcp-server-container-prd.md) | **MCP Server Container PRD**: Dedicated container build (`aad-mcp-container`) and Helm chart (`charts/agent-as-data-mcp`) with backend architectural parity (Clap CLI, centralized config/secrets, HaMS health monitoring on `:8079`, baseline startup metrics, Tokio runtime metrics, Pod Endpoint Resolution Abstraction & `hello` greeting tool). |
| [semantic-search-page-prd.md](./semantic-search-page-prd.md) | **Semantic Search Discovery PRD**: Natural language task context discovery (`/agent-context`), PostgreSQL full-text/vector hybrid cover-density ranking (`ts_rank_cd`), enter-to-submit keydown handling, entity deduplication, and refined discovery cards with direct entity routing. |
| [skills-registry-tools-prd.md](./skills-registry-tools-prd.md) | **Skills Registry Tools PRD**: Declarative skill schemas, embedding synchronization (`/skills/{id}/sync-embeddings`), and MCP tool integration. |

## Governance & Lifecycle Rules
1. **Source of Truth**: PRD files in this directory define the permanent vision and requirements for the platform.
2. **Iterative Evolution**: As new features or capabilities are conceptualized, PRDs are updated to reflect the expanded scope.
3. **Relation to Specs**: Ephemeral execution tasks and historical implementation snapshots live in `spec/specs/` and reference these PRDs.

- [Knowledge Base MCP Journey PRD](./knowledge-mcp-journey-prd.md)
