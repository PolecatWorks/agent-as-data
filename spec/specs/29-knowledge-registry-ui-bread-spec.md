# Spec 29: Knowledge Registry UI & BREAD Operations

**Status**: `draft`

## Overview & Scope
This specification details the implementation of a full BREAD (Browse, Read, Edit, Add, Delete) feature set for the Knowledge System, bringing it to architectural UI/UX parity with the Agent Registry. It expands the underlying `knowledge_nodes` data model and introduces corresponding REST endpoints and Angular UI components.

## Dependencies & References
- **Build Order Phase**: **Phase 12 (Knowledge BREAD)**.
- **Dependencies**: Depends on [02-knowledge-engine-spec.md](./02-knowledge-engine-spec.md) and [08-developer-ui-studio-spec.md](./08-developer-ui-studio-spec.md).
- **PRD References**: [Knowledge & Data System PRD](../prds/knowledge-data-system-prd.md)

## 1. Schema DDL Migrations
- Create a forward migration (`.up.sql`) to add two new columns to the `knowledge_nodes` table:
  - `description TEXT`
  - `tags TEXT[] NOT NULL DEFAULT '{}'`
- Create a reverse migration (`.down.sql`) to drop the `description` and `tags` columns.

## 2. Backend APIs & Models (`models/knowledge.rs` & `webserver/knowledge.rs`)
- **Data Models**:
  - Add `description` and `tags` to `IngestKnowledgeRequest`.
  - Create a new `KnowledgeNode` struct representing the fully expanded DB record.
  - Create a new `UpdateKnowledgeRequest` struct.
- **API Endpoints**:
  - `GET /` -> Retrieve a list of all `knowledge_nodes` (Browse).
  - `GET /{id}` -> Retrieve a specific `knowledge_nodes` record (Read).
  - `POST /` -> Enhance existing `ingest_knowledge` to accept and insert `description` and `tags` (Add).
  - `PUT /{id}` -> Update `title`, `description`, `content`, `tags`, and `metadata` for a given node. Triggers re-chunking/embedding of vector data when `content` changes (Edit).
  - `DELETE /{id}` -> Existing delete endpoint to be verified (Delete).

## 3. Frontend UI (`knowledge-inspector.component`)
- **Layout Refactoring**:
  - Migrate from the current 2-column RAG/Graph layout to the Agency Registry's dual-pane layout.
  - Left Sidebar: A list to browse all knowledge nodes, displaying title, tags, and description snippet.
  - Right Main Pane: A detailed workspace view with editable fields for `Title`, `Description`, `Tags`, and `Content`.
- **API Service**: Update `api.service.ts` to include `getKnowledgeNodes`, `getKnowledgeNode`, `updateKnowledgeNode`, and modify `ingestKnowledge`.
- **Concept Guide Integration**: Maintain the zero-footprint reusable concept guide in the top bar.

## 4. Test Strategy & Verification Plan
- **Backend Unit Tests**: Verify `GET`, `PUT`, and `POST` handlers behave correctly.
- **Integration Tests**: Verify database migrations successfully alter schema, and robot tests pass BREAD flow.
- **Frontend Verification**: UI should successfully map to mock/API data.
