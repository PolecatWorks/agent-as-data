# Knowledge & Data System PRD

## Overview
The Knowledge & Data System in **Agent-As-Data (AAD)** serves as a persistent **Enterprise Brain & Knowledge Engine**. It captures unwritten developer thoughts, architectural decisions, business processes, and domain concepts from employees and AI agents. By transforming tacit knowledge into searchable vector chunks (`pgvector`) and relational concept graphs (`knowledge_tuples`), AAD enables human strategists and AI tools to perform deep reasoning on existing operations and future business ideas grounded in institutional memory.


## Core Capabilities

### 1. Hybrid Knowledge Storage
- **Text & Document Nodes**: Raw narrative notes, design docs, and architectural decisions stored in `knowledge_nodes`. Extended to include `description` and `tags` metadata to facilitate human browsing and categorization.
- **Semantic RAG Embeddings**: Automatic text chunking and vector indexing in `knowledge_embeddings` (`pgvector` with HNSW cosine similarity indices) to enable semantic vector queries (`POST /{{api_prefix}}/v1/knowledge/search`).
- **Graph Relational Triples**: Relational tuple storage (`subject`, `predicate`, `object`, `confidence`, `metadata`) in `knowledge_tuples` to capture concept maps (e.g., `User -> belongs_to -> Tenant`).

### 2. Knowledge Registry UI & Management (BREAD)
- **AI Markdown Import & Full Document Retention**: Ability to paste or upload raw markdown documents, persist the complete source document as a root `knowledge_node`, analyze it via LLM to extract granular knowledge concepts, and bulk ingest both the source document and derived child concepts with full provenance tracking.
- **Unified UI/UX Structure**: Following the identical architectural structure of the Agency Registry, the Knowledge UI implements a dual-pane layout: a collapsible left sidebar for browsing knowledge nodes, and a right-side main workspace for creating, reading, and editing knowledge nodes.
- **BREAD Operations**: Full Browse, Read, Edit, Add, and Delete capabilities over knowledge items directly from the UI, supporting edits to `title`, `description`, `content`, and `tags`.

### 3. Markdown Document Retention & Concept Provenance
- **Full Source Document Persistence**: When importing markdown, the complete document text is preserved intact in `knowledge_nodes` as a primary document node:
  - `topic`: Domain topic selected by user or inferred by LLM (defaults to `"document"` or custom topic).
  - `title`: Extracted from the document's top-level header (`# Title`), user input, or filename.
  - `description`: LLM-generated executive summary / abstract of the overall document.
  - `tags`: Document categorization tags including `["document", "imported"]`.
  - `content`: Unaltered, complete Markdown source text.
  - `metadata`: Carries structured provenance attributes:
    ```json
    {
      "is_source_document": true,
      "source_format": "markdown",
      "extracted_concepts_count": 5,
      "content_length": 4096
    }
    ```
- **Bidirectional Concept Provenance**: Every granular concept proposal extracted by the LLM links back to the originating source document:
  - **Node Metadata**: Each child `knowledge_node` stores `metadata.source_document_id` referencing the parent document's UUID.
  - **Relational Graph Triples**: An explicit tuple is created in `knowledge_tuples`:
    `(ChildConceptNode, derived_from, SourceDocumentNode)` with `confidence = 1.0` and `source_node_id` pointing to the child or parent.
- **Document-Concept Inspection & Navigation**:
  - In the Knowledge Inspector UI, selecting a source document displays a "Derived Concepts" panel listing all extracted child nodes with quick-jump links.
  - Selecting a child concept node displays a clickable "Source Document" chip / badge leading directly to the full source document.
- **Knowledge Explorer Integration**:
  - The Knowledge Explorer (`/knowledge-explorer`) visualizes `derived_from` directed edges on the canvas between child concepts and their root source documents, enabling visual exploration of concept clusters originating from single documents.

### 4. Knowledge Retrieval & Graph Traversal
- **Semantic Vector Search**: Nearest-neighbor retrieval over chunked text context given a user or agent prompt. Both source documents and granular concept nodes are indexed into `knowledge_embeddings` (`pgvector`).
- **Multi-Hop Graph Queries**: Traversal endpoint (`POST /{{api_prefix}}/v1/knowledge/graph/traverse`) to discover connected entities, conceptual dependencies, and document hierarchies.
- **AI / LLM Integration**: Integration with AI via the `rig-core` crate and connection to an Ollama instance to process natural language queries over vector data (RAG).

### 5. Knowledge Editor (BREAD)
- **Interactive UI Workbench**: The Knowledge Inspector UI provides a two-pane layout: a left sidebar for discovering existing knowledge nodes, and a right workspace editor to modify the knowledge node directly.
- **Full BREAD API**: Full support for listing (`GET /v1/knowledge`), retrieving (`GET /v1/knowledge/{id}`), updating (`PUT /v1/knowledge/{id}`), creating (`POST /v1/knowledge`), and deleting (`DELETE /v1/knowledge/{id}`).
- **Graph Triple Editor**: Included in the UI is an explicit Knowledge Graph Triple editor which allows users to explicitly define schema components as Subject-Predicate-Object with adjustable confidence weights.

### 6. Knowledge Explorer & System Relational Tuples Graph (`/knowledge-explorer`)
- **Holistic System Knowledge Graph**: The Knowledge Explorer visualizes the entire Agent-As-Data ecosystem—Agents, Skills, Tools, Traits, and Knowledge Nodes—as an interconnected knowledge network.
- **Entity Relationships as Relational Tuples (SPO Triples)**: Structural connections between platform entities are modeled and exposed explicitly as Subject-Predicate-Object tuples:
  - `(Agent, has_skill, Skill)`
  - `(Agent, uses_tool, Tool)`
  - `(Agent, delegates_to, Agent)`
  - `(Agent, implements, Trait)`
  - `(Agent, requires_trait, Trait)`
  - `(Skill, uses_tool, Tool)`
  - `(Skill, composes_skill, Skill)`
  - `(Skill, implements, Trait)`
  - `(ChildConceptNode, derived_from, SourceDocumentNode)` capturing document provenance.
  - `(KnowledgeNode, predicate, KnowledgeNode / Concept)` derived from `knowledge_tuples`.
- **Canvas Edge Tuple Representation**:
  - All graph edges in the interactive network canvas (`vis-network`) must display directional arrows (`Subject -> Object`) and predicate labels (`uses_tool`, `has_skill`, `derived_from`, `implements`, etc.).
  - Edge hover tooltips must present the complete formal tuple format: `(Subject) —[Predicate]→ (Object)`.
- **Entity Hydration Standard**: The explorer must fully hydrate agents and skills (retrieving their complete entity payloads including `attached_skills`, `attached_tools`, `attached_agents`, and trait implementations) to ensure zero missing structural relationships.
- **Selected Entity Tuple Inspector**:
  - When an entity node or edge is selected in the canvas, the right-hand inspection drawer displays a dedicated **Tuples / Relationships** section.
  - Groups inbound (`(Source) -> predicate -> [This]`) and outbound (`[This] -> predicate -> (Target)`) triples with one-click navigation to focus connected nodes.
- **Global Tuple Filter & Exploration Controls**:
  - Interactive filter controls allow toggling or isolating specific predicate types (e.g. filter by `uses_tool`, `has_skill`, `derived_from`, `implements`, or factual knowledge triples).
  - Optional drawer/modal providing a tabular registry of all active system tuples.

### 7. Quality & Performance Safeguards
- **Entity Canonicalization & Synonym Resolution**: Vector similarity scans on subject/object names (`subject_canonical`, `object_canonical`) detect synonymous entities (e.g. `PostgreSQL` vs `Postgres`) to prevent graph fragmentation.
- **Graph Tuples Confidence Scoring**: Every extracted tuple carries a mandatory `confidence` score (0.0 to 1.0) to filter noisy or low-certainty relationships during reasoning queries.
- **Source Traceability & Cascade Pruning**: Tuples reference their originating `knowledge_nodes` via `source_node_id`. If a document is modified or deleted, stale tuples are automatically re-evaluated or cascade-deleted.
- **Automated Orphan & Decay Pruning**: Background pruning jobs purge unreferenced tuples with zero traversal hits and low confidence after a configurable retention period.
- **HNSW Vector Acceleration**: `knowledge_embeddings` uses PostgreSQL HNSW vector indexing to maintain sub-millisecond similarity lookup speeds as the knowledge base grows.
- **Reversible Migration Safeguards**: All DDL tables (`knowledge_nodes`, `knowledge_embeddings`, `knowledge_tuples`) and indices MUST be defined with paired forward (`.up.sql`) and reverse (`.down.sql`) migration scripts to support clean schema rollbacks.

## Knowledge System Flow

```mermaid
flowchart TD
    subgraph ImportWorkflow["Markdown Ingestion & LLM Analysis"]
        RawDoc["Raw Markdown Document Input"] --> Extractor["LLM Document & Concept Extractor"]
        Extractor --> DocProps["Parent Document Node (Full Text & Metadata)"]
        Extractor --> ConceptProps["Extracted Concept Proposals (Chunks & Summaries)"]
    end

    subgraph Storage["Dual Hybrid Store (PostgreSQL)"]
        DocProps -->|1. Ingest Parent Document| KNDoc[("knowledge_nodes (Parent Document)")]
        ConceptProps -->|2. Ingest with source_document_id| KNChild[("knowledge_nodes (Child Concepts)")]
        KNDoc -.->|3. (Child) -[derived_from]-> (Parent)| Tuples[("knowledge_tuples (Graph Store)")]
        KNChild -.->|3. (Child) -[derived_from]-> (Parent)| Tuples
        KNDoc -->|HNSW Chunk Embeddings| Embeddings[("knowledge_embeddings (pgvector)")]
        KNChild -->|HNSW Chunk Embeddings| Embeddings
    end

    subgraph SystemEntities["System Entity Graph"]
        Agents["Agents"]
        Skills["Skills"]
        Tools["Tools"]
        Traits["Traits"]
        Agents -->|has_skill| Skills
        Agents -->|uses_tool| Tools
        Agents -->|implements| Traits
        Skills -->|uses_tool| Tools
    end

    subgraph Visualization["Knowledge Explorer & Inspector"]
        Explorer["Knowledge Explorer (/knowledge-explorer)"]
        Inspector["Knowledge Inspector (/knowledge-inspector)"]
        Tuples -->|SPO & derived_from Tuples| Explorer
        SystemEntities -->|Relational Tuples| Explorer
        KNDoc -->|Browse Documents & Concepts| Inspector
        KNChild -->|View Concept & Source Link| Inspector
    end

    subgraph Retrieval["RAG + Graph Retrieval"]
        Query["AI Context Request"] -->|search_knowledge| Embeddings
        Query -->|query_knowledge_graph| Tuples
    end
```

## Related PRDs & Specs
- [Agent-As-Data Core PRD](./agent-as-data-prd.md)
- [Agent Registry & Execution Engine PRD](./agent-registry-execution-prd.md)
- [Agent UI & Testing Kit PRD](./agent-ui-testing-kit-prd.md)
- [Detailed Schema Specification](../specs/agent-schema-spec.md)

