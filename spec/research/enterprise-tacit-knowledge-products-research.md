# Market Analysis: Enterprise Tacit Knowledge & Reasoning Products

This research document analyzes existing market products and platforms aimed at capturing **enterprise tacit knowledge, informal developer intuition, and process intelligence**, comparing their paradigms against **Agent-As-Data (AAD)**.

---

## 1. Market Overview & Problem Space

Enterprise tacit knowledge—unwritten operational rules, historical decision trade-offs, and informal team conventions—is traditionally lost due to employee churn or buried across fragmented communication tools.

A new market segment of **Work AI, Enterprise Knowledge Graphs, and Tacit Knowledge Extraction platforms** has emerged to address this challenge.

```mermaid
graph TD
    subgraph Enterprise Tools Ecosystem
        Glean["Glean Work AI<br/>(Enterprise Connector RAG)"]
        MicrosoftGraph["Microsoft GraphRAG<br/>(LLM Extracted Ontologies)"]
        NotionAI["Notion AI / Slite<br/>(Document-Centric Wiki Knowledge)"]
        Guru["Guru / Starmind<br/>(Expert Q&A Capture)"]
    end

    subgraph AAD Core ["Agent-As-Data (AAD)"]
        DualEngine["Dual Hybrid Store:<br/>pgvector RAG + SPO Triples"]
        DeclarativeAgents["Declarative Agent Registry"]
        NativeMCP["Native MCP Interoperability<br/>(IDE & AI Tooling)"]
    end

    Glean -->|Mines Chat Logs| DualEngine
    MicrosoftGraph -->|Extracts Knowledge Graph| DualEngine
    NativeMCP <-->|Direct Context & Action| Enterprise Tools Ecosystem
```

---

## 2. Competitive Product Analysis Matrix

| Product / Platform | Primary Focus | How It Captures Tacit Knowledge | Strengths | Limitations / Gaps | Comparison to AAD |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Glean (Work AI)** | Enterprise-wide search & AI assistant across 100+ connectors (Slack, Jira, Docs). | Indexes informal chat back-and-forth, meeting transcripts, and developer PR comments. | Powerful workplace search, personalized context ranking, native enterprise permissions. | Read-focused document/chat index; lacks declarative agent execution & native MCP IDE context stream. | **Complementary / Overlapping**: Glean indexes passive chat logs; AAD provides a lightweight active project brain with agent execution & MCP API tools. |
| **Microsoft GraphRAG** | Automated LLM-extracted Knowledge Graph creation from unstructured text. | Uses LLMs to detect entities, extract relationships, and build hierarchical community summaries. | Deep global dataset reasoning, multi-hop prompt understanding, handles complex corpus queries. | High LLM extraction token cost; slow indexing times; no native declarative agent registry. | **Technology Alignment**: AAD adopts GraphRAG principles using lightweight `knowledge_tuples` inside Postgres without heavy external graph DB dependencies. |
| **Starmind / Guru** | Tacit Knowledge Management & Expert Identification. | Uses AI to route questions to human subject-matter experts and save verified Q&A pairs. | Captures expert intuition directly at the source; strong human-in-the-loop validation. | Manual question-answer workflow; static text retrieval; no autonomous agent execution. | **Divergent**: Starmind is human Q&A software; AAD is developer/agent data infrastructure exposing knowledge via vector/graph/MCP. |
| **Notion AI / Slite Knowledge Base** | AI-assisted documentation and wiki search. | Asks users to write structured wiki pages and uses RAG for question answering. | Excellent UI/UX; easy human editing; widely adopted by product teams. | Requires humans to manually document tacit knowledge; passive wikis often decay into data graveyards. | **Divergent**: Notion is a human document editor; AAD is an active declarative backend engine built for AI tools & agents. |

---

## 3. Key Differentiators of Agent-As-Data (AAD)

1. **Active MCP Developer & IDE Integration**:
   - Products like Glean or Notion require opening a web browser or enterprise app. AAD streams project memory directly into developer IDEs (Cursor, Antigravity) and assistants (Claude Desktop) via native Model Context Protocol (MCP over Stdio/SSE).
2. **Unified Tacit Knowledge + Agent Execution Engine**:
   - Existing enterprise tools only *search* knowledge. AAD combines tacit knowledge capture (`knowledge_nodes`, `knowledge_tuples`) with **declarative agent execution** (`agents`, `agent_revisions`, `executions`), allowing captured rules to directly govern agent execution guardrails.
3. **Lightweight Postgres-Native Architecture**:
   - Avoids expensive multi-vendor SaaS stacks or complex graph databases by housing `pgvector` RAG embeddings and Subject-Predicate-Object relation tuples in a single, containerized PostgreSQL database.

---

## 4. Summary & Strategic Recommendation

- **Positioning**: AAD occupies a unique position as a **Developer & AI-Native Project Memory Engine**, bridging the gap between enterprise search (Glean) and graph reasoning (Microsoft GraphRAG) while supplying a declarative agent registry.
- **PRD Cross-Reference**:
  - [Master PRD](../prds/agent-as-data-prd.md)
  - [Knowledge System PRD](../prds/knowledge-data-system-prd.md)
  - [Landscape Research Document](./agent-as-data-market-landscape-research.md)
