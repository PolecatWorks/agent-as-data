# Specification: Knowledge Base MCP Journey

## 1. Overview
This document specifies the technical implementation details for exposing Knowledge Base operations via the Model Context Protocol (MCP) in the backend. It derived from `knowledge-mcp-journey-prd.md`.

## 2. Architecture & Components

The implementation requires extending the existing toolset provided to AI agents within the Rust backend (`aad-be-container/src/llm_tools.rs`).

We will implement five new `PortableTool` structs:
1. `KbNodeBrowseTool`
2. `KbNodeReadTool`
3. `KbNodeAddTool`
4. `KbNodeEditTool`
5. `KbNodeDeleteTool`

These tools will interact directly with the database using `sqlx` (since `PortableTool`s are often instantiated with a DB pool, or we will modify the tool traits/context to allow DB operations, similar to how other tools operate, or pass the `PgPool` or backend REST server port to make HTTP calls). *Note*: Since tools run in the backend, they can make local HTTP requests to the Axum server (`http://127.0.0.1:8080/v1/knowledge-base`) or use `PgPool` directly. Using local HTTP requests to the backend's own API ensures business logic (like vector embedding generation on ingest) is reused. We will use `reqwest` to call the local API.

## 3. Tool Specifications

### 3.1. `KbNodeBrowseTool`
- **Name:** `kb_node_browse`
- **Description:** Browse or search knowledge nodes.
- **Parameters:**
  - `query` (string, optional): A search query. If provided, uses the `/v1/agent-context/search` or `/v1/knowledge-base/search` endpoint. If omitted, uses the `/v1/knowledge-base` GET endpoint.
  - `limit` (number, optional): Max results.
- **Action:**
  - If `query` is present, POST `{"query": query, "limit": limit}` to `http://127.0.0.1:8080/v1/knowledge-base/search`.
  - Otherwise, GET `http://127.0.0.1:8080/v1/knowledge-base?limit=X`.

### 3.2. `KbNodeReadTool`
- **Name:** `kb_node_read`
- **Description:** Read the full content of a specific knowledge node.
- **Parameters:**
  - `id` (string, required): The UUID of the knowledge node.
- **Action:**
  - GET `http://127.0.0.1:8080/v1/knowledge-base/{id}`

### 3.3. `KbNodeAddTool`
- **Name:** `kb_node_add`
- **Description:** Create a new knowledge node.
- **Parameters:**
  - `topic` (string, required): The topic or domain.
  - `title` (string, optional): The title.
  - `description` (string, optional): Short summary.
  - `content` (string, required): Full content/article body.
  - `tags` (array of strings, optional): Categorization tags.
- **Action:**
  - POST to `http://127.0.0.1:8080/v1/knowledge-base` with the `IngestKnowledgeRequest` payload.

### 3.4. `KbNodeEditTool`
- **Name:** `kb_node_edit`
- **Description:** Update an existing knowledge node.
- **Parameters:**
  - `id` (string, required): UUID of the node to edit.
  - `topic`, `title`, `description`, `content`, `tags` (all optional).
- **Action:**
  - PUT to `http://127.0.0.1:8080/v1/knowledge-base/{id}` with `UpdateKnowledgeRequest`.

### 3.5. `KbNodeDeleteTool`
- **Name:** `kb_node_delete`
- **Description:** Delete a knowledge node.
- **Parameters:**
  - `id` (string, required): UUID of the node to delete.
- **Action:**
  - DELETE `http://127.0.0.1:8080/v1/knowledge-base/{id}`

## 4. Integration Tests

A Robot Framework test `knowledge_mcp_journey.robot` will be created in `integration-tests/tests/`. It will:
1. Connect to the backend and invoke the MCP tools using the standard MCP tool execution mechanism or by simulating the LLM tool call wrapper if possible (or just directly test the `/v1/llm/...` tool endpoint if exposed, else we will test the APIs directly and verify the tools logic via unit test in Rust).

*Correction:* Since the `PortableTool` logic will be inside the backend, we can write Rust unit tests in `llm_tools.rs` for these tools. We will also write a Robot Framework test that hits a mock agent or an endpoint that can execute these tools, OR if there isn't a direct API to invoke arbitrary PortableTools from the outside, the Robot test will hit the backend APIs to verify BREAD operations are fundamentally sound, while Rust unit tests ensure the Tools work.
