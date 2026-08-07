# Task Specifications (Specs) Registry

This directory contains task specifications and historical build snapshots for **Agent-As-Data (AAD)**. 

## Specification Lifecycle & Status Rules
- **Status Types**:
  - `draft`: Spec is under active authoring or refinement. **Modifications are only permitted in `draft` status**.
  - `complete`: Features and test definitions are fully implemented. **Complete specs are immutable** (cannot be modified except to transition to `deprecated`).
  - `deprecated`: Historical spec superseded by a newer specification.
- **Task-Focused & Ephemeral**: Spec files describe specific implementation tasks, schema designs, API contracts, and user journeys.
- **Historical Auditability**: Past specs are preserved to observe what was built and why design choices evolved.

## Index of Spec Files

| Specification Document | Status | Category / Scope | Primary PRD Reference |
| :--- | :--- | :--- | :--- |
| [agent-schema-spec.md](./agent-schema-spec.md) | `draft` | PostgreSQL Schema, DDL Tables (`pgvector`, embeddings, tuples, revisions, tests, mcp_servers), & API Payloads | [Knowledge PRD](../prds/knowledge-data-system-prd.md), [Agent Registry PRD](../prds/agent-registry-execution-prd.md) |
| [user-journeys-spec.md](./user-journeys-spec.md) | `draft` | Sequence flows & end-to-end user journeys for Project Memory, RAG/Graph, & MCP Execution | [Agent-As-Data Master PRD](../prds/agent-as-data-prd.md) |

