# Agent-As-Data Implementation Gaps

After reviewing the current codebase (`aad-be-container`, `aad-fe-container`, `integration-tests`) against the requirements detailed in the PRDs, the following implementation gaps have been identified.

## 1. Agent-As-Data (AAD) Core & Architecture
*   **Usage Audit Telemetry**: The `agent_usage_logs` table (and associated data models) does not exist in the database migrations. The `GET /api/v1/analytics/usage` and `GET /api/v1/agents/:id/logs` endpoints are completely missing. Audit telemetry for agent/tool usage is not being recorded.
*   **Guardrail Enforcement**: The core execution engine (`execute_agent`) does not evaluate `incoming_guardrails` or `outgoing_guardrails`.
*   **Decoupled Async Job Executions**: `POST /api/v1/executions` is not implemented as an asynchronous job queue. Background execution is missing, and status polling (`GET /api/v1/executions/:id`) just reads a static response from the DB.
*   **Execution Webhooks**: The `execute_agent` logic ignores the `webhook_url`. It does not dispatch HTTP POST event payloads upon completion.

## 2. Agent Registry & Execution Engine
*   **Agent Refactoring & Compression**: The `POST /api/v1/agents/refactor/analyze` endpoint is completely mocked and returns hardcoded fake JSON output. It does not perform actual `pgvector` overlap scans or harmonization logic.
*   **Network & Relationship Visualization**: The `GET /api/v1/agents/visualize` endpoint is completely missing. It does not generate `mermaid` strings or `graph_json` formats from agent delegation topologies.
*   **Network Compilation Engine**: The `POST /api/v1/agents/compile` endpoint is a stub returning a hardcoded "clean" status. It performs no structural DAG cycle checks, schema checks, or semantic fit verification.
*   **Contract Negotiation**: The `verify_contract` endpoint returns hardcoded `verified` status and does not perform the required Depth-First Search trait resolution or semantic compatibility scanning.
*   **Skill Demotion**: The `POST /api/v1/agents/:id/demote` endpoint and `GET /api/v1/skills/guidance` are missing entirely.
*   **Mock Judge Testing Engine**: The `POST /api/v1/agents/:id/test` uses a hardcoded `mock_score = 0.9` rather than implementing a deterministic runner or LLM-as-a-judge system.

## 3. Knowledge & Data System
*   **Vector Search & Graph Traversal Shortcuts**: The `search_knowledge` endpoint uses a simple SQL `ILIKE` text search rather than querying the `pgvector` embeddings with cosine similarity (`<->`).
*   **Automated Synonyms & Decay Pruning**: There is no background worker logic to canonicalize entities, resolve synonyms, or prune orphaned graph tuples.
*   **Graph Down Migrations**: No logic exists to gracefully tear down nodes and cascading tuples if an execution reverses.

## 4. Remote MCP Server
*   **Remote Server Auth & Secrets**: There is no credential or secret management configuration implemented for remote MCP servers.
*   **Mock Registration**: The `POST /api/v1/agents/mcp/register` is completely mocked. It does not query `tools/list`, `resources/list`, or `prompts/list` on external servers. It just saves hardcoded JSON to the DB.
*   **Missing Background Sync Cache**: There is no polling process to keep cached MCP remote schemas in sync over time.

## 5. Agent Development UI & Testing Kit
*   **Missing UI Component Implementations**: While the Angular routes are created, every single visual component (Registry Builder, Testing Studio, Mermaid Graph, Refactoring Lab, Knowledge Inspector, MCP Manager) contains only generated CLI scaffold text (e.g., `<p>agent-registry works!</p>`).
*   **Missing API Client Integrations**: The frontend application lacks HTTP services to connect with the Axum backend or listen to SSE streams.

## 6. Integration Testing
*   **Robot Framework Scaffolding Only**: 8 out of the 9 Robot user journey test suites only contain "Preflight Verification" steps that verify the backend URL is reachable. They do not execute the HTTP requests for the user journeys.
