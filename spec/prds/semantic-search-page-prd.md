# Semantic Search Discovery Page PRD

## Overview
This PRD outlines a new page for the Agent-As-Data UI that allows users to query the vector database and retrieve relevant entities (Agents, Skills, Traits, Tools) via semantic similarity search.

## Features & Requirements
1. **Unified Search Interface**:
   - A dedicated search page (e.g. `/discovery` or `/semantic-search`).
   - A primary search bar to input natural language queries.
2. **Prioritized Results & Matching Metrics**:
   - The page must query the backend vector embeddings (which requires updating the backend search endpoint to use actual vector similarity matching, or using a mock score if pgvector is unavailable).
   - Display the results in a prioritized list sorted by highest similarity score.
   - Each result item must display a visual metric representing how closely it matched the query (e.g. `98% match` or similarity score).
3. **Card-Based UI Consistency**:
   - Follow the existing platform's look and feel for lists and cards.
   - Result cards should display the entity's `name`, `description`, `type` (Agent, Skill, Trait, or Tool), and `tags`.
   - Result cards should *not* display full definitions or prompts (keep it high-level).
4. **Navigation Options**:
   - Each result card must feature a button (e.g., "View Details") or allow the user to click the card itself to navigate to the dedicated detail page for that specific entity.

## Backend Dependencies
- Ensure the backend exposes a semantic search API endpoint capable of returning mixed entity types with similarity scores.
- Return `tags` and the `similarity_score` alongside `name`, `description`, and `entity_type`.
