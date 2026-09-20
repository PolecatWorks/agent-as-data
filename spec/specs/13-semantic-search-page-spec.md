---
status: complete
---

# 13. Semantic Search Discovery Page Spec

## Objective
Implement a Semantic Search Discovery page in the Angular frontend (`aad-fe-container`) and the corresponding backend vector search logic in `aad-be-container`.

## 1. Backend Search Endpoint
**File:** `aad-be-container/src/webserver/search.rs` (or `agents.rs`)
- **Route:** `POST /api/v1/search/semantic`
- **Request Payload:** `{"query": "string", "limit": 10}`
- **Behavior:**
  - Convert `query` to an embedding (or mock for now).
  - Execute a SQL query against `entity_embeddings` using vector cosine distance (e.g. `embedding <=> $1`) to get the top `limit` results.
  - Join with the respective tables (`agents`, `skills`, `tools`, `traits`) to retrieve `name`, `description`, `tags`.
- **Response Payload:** Array of objects containing:
  - `id`
  - `entity_type` (agent, skill, trait, tool)
  - `name`
  - `description`
  - `tags` (array of strings)
  - `similarity_score` (float 0.0 - 1.0)

## 2. Frontend Page Implementation
**Location:** `aad-fe-container/src/app/pages/semantic-search/`
- Create a new routable component mapped to `/semantic-search`.
- Use the existing application layout and card components for consistency.
- **UI Elements:**
  - Search input field triggering the backend API.
  - Results list displaying a card for each returned entity.
  - Card Content: Name, description, type badge, tags, and the `similarity_score` displayed visually (e.g. a percentage progress bar or badge).
  - "View Details" button on each card routing to `/<entity_type>/<id>` (e.g., `/agents/:id`, `/skills/:id`).

## 3. Integration & Testing
- Unit tests for the UI component ensuring it renders the required fields and navigation links.
- Backend integration test verifying the `/api/v1/search/semantic` endpoint successfully joins and returns standard entity data and a score.
