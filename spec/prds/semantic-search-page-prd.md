# Semantic Search & Agent Context Discovery Page PRD

## Overview
This PRD outlines the **Agent Context Search & Discovery** feature for the Agent-As-Data UI (`/agent-context`), allowing users to query registered agent artifacts (Agents, Skills, Traits, Tools) using natural language task context. The system processes natural language queries against entity representations stored in PostgreSQL (`entity_embeddings`), delivering ranked, deduplicated, and user-friendly discovery cards.

```mermaid
flowchart TD
    A[User Enters NL Task Context] --> B{Enter Key Pressed?}
    B -->|Shift + Enter| C[Insert Newline in Textarea]
    B -->|Enter without Shift| D[Prevent Default Newline & Trim Query]
    D --> E[POST /api/v1/agent-context/search]
    E --> F[Backend Full-Text & Vector Ranking]
    F --> G[Extract Lexemes & Disjunctive Stems]
    G --> H[Query entity_embeddings Table]
    H --> I[Cover-Density Ranking ts_rank_cd]
    I --> J[Deduplicate Matches by Entity Name/ID]
    J --> K[Format Results: Name, Match Reason, Description]
    K --> L[Render Result Cards in UI]
    L --> M[User Clicks 'View Details' -> Navigate to /agents/:id, /skills/:id, etc.]
```

## User Experience & Interface Specifications

### 1. Natural Language Task Context Input
- **Dedicated Textarea**: A multiline text input allowing users to express complex, real-world task goals (e.g., *"i need an agent that can be used to design accounting systems"*).
- **Enter-to-Submit Interaction**:
  - Pressing **Enter** triggers immediate search execution.
  - The default browser behavior of inserting a newline character into `<textarea>` is intercepted on `keydown` (`event.preventDefault()`) to ensure searches do not fail or introduce trailing `\n` characters.
  - **Shift + Enter** continues to allow manual multi-line context formatting.
- **Fail-Safe Trimming**: The search query is trimmed both on the client side before dispatch and on the backend service to strip extraneous whitespace.

### 2. Result Presentation & Aesthetics
- **Card Title**: Resolves and displays the human-readable **name** of the matching object (e.g. `FinancialAuditorAgent`, `ComplianceChecking`), accompanied by a badge for its entity type (`agents`, `skills`, `tools`, `traits`).
- **Match Reason Badge**: An explicit, informative badge detailing why the entity was surfaced (e.g. `✓ Matched on entity description`, `✓ Matched on entity name`, `✓ Matched on agent prompt`, `✓ Matched on skill definition`).
- **Entity Description**: Full descriptive text rendered cleanly below the match badge.
- **Matched Content Snippet**: Displays matching snippet content when the query matches secondary fields (e.g. prompt or skill definition).
- **Omission of Raw IDs**: Internal database UUIDs are omitted from the user-facing card display to maintain a clean, high-level discovery experience.
- **Zero Boilerplate**: Generic placeholder text (e.g. *"Semantic similarity based on..."* or *"Semantic similarity matched well with query"*) is strictly excluded.
- **Direct Navigation**: Each card includes a **View Details** action button that routes directly to the entity's native management view (`/agents/:id`, `/skills/:id`, `/traits/:id`, `/tools/:id`).

## Search Engine & Backend Architecture

### 1. Hybrid Semantic & Full-Text Retrieval
```mermaid
sequenceDiagram
    autonumber
    participant UI as Angular Frontend (/agent-context)
    participant API as Backend Handler (agents::search_agent_context)
    participant DB as PostgreSQL (entity_embeddings)

    UI->>API: POST /api/v1/agent-context/search { query, depth }
    API->>API: Trim query and extract search pattern
    API->>DB: Execute query with ILIKE + plainto_tsquery OR parsing
    Note over DB: Evaluates lexemes, stems, and stop-word filtering.<br/>Ranks by cover-density (ts_rank_cd).<br/>Deduplicates via DISTINCT ON (entity_name/id).
    DB-->>API: Top N matching entity records with score & field info
    API->>API: Map field names to human-readable match reasons
    API-->>UI: 200 OK [ { entity_id, name, description, match_reason, score, ... } ]
    UI->>UI: Render cards with name, badge, description, and View Details
```

- **Query Tokenization & Stemming**: Queries are processed using PostgreSQL's `plainto_tsquery('english', ...)` to strip stop words (*"i"*, *"need"*, *"an"*, *"that"*, *"can"*, *"be"*, *"to"*) and extract linguistic stems (*"accounting"* $\rightarrow$ *"account"*, *"systems"* $\rightarrow$ *"system"*).
- **Disjunctive Stem Evaluation**: Terms are combined into a disjunctive OR query (`|`) to match multi-concept prompts against disparate entity descriptions and prompts.
- **Cover-Density Ranking (`ts_rank_cd`)**: Results are ranked by proximity and density of matching terms, surfacing entities that satisfy multiple concepts (e.g. agents matching both *"accounting"* and *"agent"*) ahead of single-word matches.
- **Exact Match Elevation**: Verbatim substring matches are boosted to top rank with high confidence scores (`98%`), while multi-token matches scale appropriately between `60%` and `96%`.
- **Entity Deduplication**: Query results utilize `DISTINCT ON (COALESCE(name, entity_id::text))` to ensure that an entity matching across multiple fields (e.g. both name and description) is returned only once with its highest-scoring match.

## Automated Entity Embeddings Lifecycle via BREAD Operations & Reverse References

To guarantee that semantic and natural language discovery always reflects current platform reality without manual operational friction, entity embeddings for all four core entities (**Agents, Skills, Traits, Tools**) are maintained synchronously and automatically within backend BREAD operations:

```mermaid
sequenceDiagram
    autonumber
    participant UI as Developer UI (/agents, /skills, /traits, /tools)
    participant API as Backend REST Handler
    participant DB as PostgreSQL (agents / skills / trait_contracts / tools)
    participant VEC as PostgreSQL (entity_embeddings)

    alt Record Created or Updated (POST / PUT)
        UI->>API: Save Entity (Create / Edit Form)
        API->>DB: Upsert entity record
        API->>VEC: Purge prior embeddings for entity_id
        API->>VEC: Insert fresh embeddings with reverse references (origin_id, origin_type, origin_uri, origin_name)
        API-->>UI: 200 OK / 201 Created (Vector store strongly consistent)
    else Record Deleted (DELETE)
        UI->>API: Delete Entity
        API->>DB: Delete record or set archived_at = NOW()
        API->>VEC: Cascade / Purge embeddings WHERE entity_id = id
        API-->>UI: 200 OK / 204 No Content (No stale vectors remaining)
    end
```

### 1. Automated Lifecycle Synchronization across All Entities
- **Agents (`/agents`)**:
  - Automatically indexes `name`, `description`, and persona prompt (`agent_definition`) on `create_agent` and `update_agent`.
  - Purged automatically upon soft-deletion/archival or hard deletion.
- **Skills (`/skills`)**:
  - Automatically indexes `name`, `description`, and instructions (`definition`) on `create_skill` and `update_skill`.
  - Purged automatically upon deletion or migrated when promoted to an Agent.
- **Traits (`/traits`)**:
  - Automatically indexes `name`, `description`, `behavioral_invariants`, `evaluation_criteria`, and `capability_requirements` on `create_trait` and `update_trait`.
  - Purged automatically upon deletion (`DELETE /traits/:id`).
- **Tools (`/tools`)**:
  - Automatically indexes `server_name` and individual tool names/descriptions extracted from `cached_capabilities` during `register_tool` or `sync_tool`.
  - Purged automatically upon deletion (`DELETE /agents/tools/:id`).

### 2. Embeddings Reverse References to Originating Entities
Every embedding entry in `entity_embeddings` stores explicit reverse references back to its originating entity record:
- **`origin_id` (UUID)**: The unique identifier of the source entity (`agent_id`, `skill_id`, `trait_id`, or `tool_id`).
- **`origin_type` (VARCHAR)**: Canonical entity type (`agents`, `skills`, `traits`, `tools`).
- **`origin_uri` (VARCHAR)**: Canonical frontend routing URI linking directly back to the originating record (e.g. `/agents/{id}`, `/skills/{id}`, `/traits/{id}`, `/tools/{id}`).
- **`origin_name` (VARCHAR)**: Name of the originating entity at the time of indexing.
- **Referential Integrity & Cascading Purge**: Embeddings are constrained by foreign keys with `ON DELETE CASCADE` (or transactional lifecycle triggers) to guarantee that removing an originating entity immediately eliminates all of its embeddings without leaving orphaned fragments in the vector index.

### 3. Zero UI Sync Button (Frictionless Automation)
- Because backend BREAD operations guarantee strong index consistency on write, no manual "Sync Embeddings" buttons exist in either the top navigation bars or per-record action bars.
- Backend synchronization endpoints (`POST /api/v1/{agents,skills,traits,tools}/{id}/sync-embeddings`) are preserved for headless maintenance, automated testing, and CLI operations, but are not exposed in standard user workflows.

## Backend Route & API Contract

### Request: `POST /api/v1/agent-context/search` (and alias `/api/v1/agents/context/search`)
```json
{
  "query": "i need an agent that can be used to design accounting systems",
  "depth": 5
}
```

### Response: `200 OK`
```json
[
  {
    "entity_id": "276e3a21-6c8d-45ad-a438-d0adf3afb6bb",
    "entity_type": "agents",
    "name": "FinancialAuditorAgent",
    "description": "A detail-oriented accounting agent specialized in financial audits, general ledger reconciliation, and strict adherence to global financial regulations.",
    "field_name": "description",
    "content": "A detail-oriented accounting agent specialized in financial audits, general ledger reconciliation, and strict adherence to global financial regulations.",
    "score": 0.95,
    "match_reason": "Matched on entity description",
    "origin_id": "276e3a21-6c8d-45ad-a438-d0adf3afb6bb",
    "origin_type": "agents",
    "origin_uri": "/agents/276e3a21-6c8d-45ad-a438-d0adf3afb6bb",
    "origin_name": "FinancialAuditorAgent"
  }
]
```

## Cross References
- [Agent UI & Testing Kit PRD](./agent-ui-testing-kit-prd.md)
- [Agent Registry & Execution PRD](./agent-registry-execution-prd.md)
- [Skills Registry Tools PRD](./skills-registry-tools-prd.md)
- [Master PRD](./agent-as-data-prd.md)
