# Product Requirements Documents (PRDs) Registry

This directory contains the long-term, persistent Product Requirements Documents (PRDs) defining the architecture, objectives, data models, and features of **Agent-As-Data (AAD)**.

## Document Hierarchy & Structure

| Document | Description |
| :--- | :--- |
| [agent-as-data-prd.md](./agent-as-data-prd.md) | **Master PRD**: High-level platform architecture, core objectives, and system overview. |
| [knowledge-data-system-prd.md](./knowledge-data-system-prd.md) | **Knowledge System PRD**: Long-term Project Memory, RAG vector chunks (`pgvector`), Graph Tuples (SPO), and native MCP tools. |
| [agent-registry-execution-prd.md](./agent-registry-execution-prd.md) | **Agent Registry PRD**: Declarative agent storage, versioning (`agent_revisions`), top `n` RAG search, and execution engine (sync/async & guardrails). |
| [agent-ui-testing-kit-prd.md](./agent-ui-testing-kit-prd.md) | **Agent UI PRD**: Developer UI container (`aad-fe-container`) built with Angular 18+, TailwindCSS, and Angular Material for agent development, live testing, SSE token streaming, Mermaid network graph visualization, and MCP server management. |
| [workspace-filesystem-tools-prd.md](./workspace-filesystem-tools-prd.md) | **Workspace Tools & Agent Execution PRD**: Secure, thread-isolated ephemeral workspace directories (`/tmp/workspace/<thread_id>`), standardized Rig `Tool` trait implementations (`read_file`, `write_file`, `list_files`, etc.), autonomous multi-turn `AgentBuilder` execution loops, conversational history retention, and thread-aware contextual answering. |


## Governance & Lifecycle Rules
1. **Source of Truth**: PRD files in this directory define the permanent vision and requirements for the platform.
2. **Iterative Evolution**: As new features or capabilities are conceptualized, PRDs are updated to reflect the expanded scope.
3. **Relation to Specs**: Ephemeral execution tasks and historical implementation snapshots live in `spec/specs/` and reference these PRDs.
