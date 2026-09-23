# Product Requirements Document: Knowledge Base MCP Journey

## 1. Introduction

This document outlines the product requirements for enabling an AI agent (or external systems) to interface with the platform's knowledge base via the Model Context Protocol (MCP). The goal is to provide a complete BREAD (Browse, Read, Edit, Add, Delete) and Search journey over knowledge nodes so that an agent can autonomously manage and retrieve knowledge.

## 2. Problem Statement

Currently, knowledge nodes are managed via REST APIs and the frontend web UI. While an AI agent can perform semantic searches implicitly through system prompts or backend endpoints, it lacks direct, robust tool-based access to cleanly navigate, ingest, and mutate the knowledge base. To allow agents to build and refine system knowledge over time, they need standard MCP tools that expose these operations.

## 3. Goals & Non-Goals

### 3.1. Goals
- Expose MCP tools for Knowledge Nodes: Browse (including Search), Read, Edit, Add, and Delete.
- Allow AI agents to manage knowledge autonomously.
- Provide a robust user journey that integrates seamless semantic and full-text search with list browsing.

### 3.2. Non-Goals
- Modifying the existing UI for knowledge nodes.
- Introducing a completely new entity for "Articles" (we will reuse the existing `knowledge_nodes` entity).
- Adding complex multi-user RBAC for these MCP tools (operations will execute with default server context permissions).

## 4. User Journey

1. **Discovery / Searching**: The agent receives a query from the user and needs to find related information in the platform's knowledge base. It uses the `kb_node_browse` tool (with a search query parameter) to discover relevant knowledge nodes.
2. **Reading**: The agent identifies a specific knowledge node of interest and uses the `kb_node_read` tool to fetch its full content, tags, and metadata.
3. **Adding / Updating**: Based on newly acquired information or user instruction, the agent realizes the knowledge base needs an update. It uses the `kb_node_add` or `kb_node_edit` tools to ingest new content or modify an existing knowledge node.
4. **Deleting**: If information is obsolete, the agent uses `kb_node_delete` to remove the node.

## 5. Functional Requirements

### 5.1. BREAD & Search Operations (MCP Tools)
The following tools must be exposed via the backend:
- `kb_node_browse`: Returns a list of knowledge nodes. If a `query` parameter is provided, it performs a search (coalescing semantic and full-text search functionality); otherwise, it returns a standard paginated list.
- `kb_node_read`: Retrieves the full details of a specific knowledge node by ID.
- `kb_node_add`: Creates a new knowledge node. Requires title, topic, content, and optional tags/metadata.
- `kb_node_edit`: Updates an existing knowledge node by ID.
- `kb_node_delete`: Deletes a knowledge node by ID.

### 5.2. Tool Implementation Constraints
- Built into the Rust backend (`aad-be-container/src/llm_tools.rs`).
- Implement the `PortableTool` trait from `rig_core`.
- Tool methods must return `Result<String, String>` mapping to JSON output.

## 6. Success Metrics
- Successful execution of a Robot Framework integration test validating the complete Browse -> Read -> Add -> Edit -> Delete cycle using the MCP tools.
