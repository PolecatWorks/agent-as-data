# Specification: Knowledge Markdown Import

## 1. Overview
This specification details the addition of a Markdown Import feature to the Knowledge Registry. This feature allows users to paste raw markdown text, which is then analyzed by an LLM to extract structured knowledge nodes (Concepts/Topics) for bulk ingestion.

## 2. Architecture & Components

### 2.1. Backend (`aad-be-container/src/webserver/knowledge.rs`)
- **New Endpoint**: `POST /v1/knowledge/analyze-markdown`
- **Request Model**: `AnalyzeMarkdownRequest { markdown: String }`
- **Response Model**: `AnalyzeMarkdownResponse { proposals: Vec<KnowledgeNodeProposal> }`
- **Logic**: Uses `rig_core` to interact with Ollama. The LLM is prompted to extract distinct concepts and return a JSON array containing the `topic`, `title`, `description`, `tags`, and specific `content` for each concept.

### 2.2. Frontend (`aad-fe-container/src/app/components/knowledge-inspector/`)
- **UI Flow**:
  1. User clicks "Import Markdown".
  2. A textarea is displayed to paste raw markdown.
  3. Clicking "Analyze Document" shows a loading spinner and calls the new backend endpoint.
  4. The LLM response is parsed and presented as a list of proposals.
  5. The user can review the proposals, deselecting any they do not want to import.
  6. Clicking "Finalize Import" sends a `POST /v1/knowledge` request for each selected proposal.

## 3. Related PRDs
- [Knowledge & Data System PRD](../prds/knowledge-data-system-prd.md)
