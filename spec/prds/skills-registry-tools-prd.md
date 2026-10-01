# Skills Registry Tools PRD

## Overview
This PRD defines a set of backend tools to allow the Agent-As-Data LLM threads (Workbench) to query and view platform `skills`.

## Requirements
1. **List Skills Tool**: The LLM must be able to call `list_skills` to retrieve all available skills in the database, including their name and description.
2. **View Skill Tool**: The LLM must be able to call `view_skill(skill_name: String)` to retrieve the full definition, tags, and traits of a specific skill.

## Tool Specifications
| Tool Name | Arguments | Description | Output |
|---|---|---|---|
| `list_skills` | `None` | Lists all available skills registered in the system database. | `ListSkillsOutput { skills: Vec<SkillInfo> }` |
| `view_skill` | `skill_name: String` | Views the detailed definition and metadata of a specific skill. | `ViewSkillOutput { skill: Skill }` |

## Skill Lifecycle & Embedding Synchronization
- **Automated BREAD Lifecycle Sync**:
  - **Creation (`POST /skills`)**: Automatically parses and persists corresponding records (`name`, `description`, `definition`/prompt) to `entity_embeddings` with reverse references (`origin_id = skills.id`, `origin_type = 'skills'`, `origin_uri = '/skills/{id}'`, `origin_name = skills.name`).
  - **Modification (`PUT /skills/:id`)**: Replaces existing entries in `entity_embeddings` with updated content maintaining reverse references.
  - **Deletion (`DELETE /skills/:id`)**: Automatically purges all associated embeddings from `entity_embeddings` via cascading deletion to prevent orphaned or stale search hits.
  - **Promotion (`POST /skills/:id/promote`)**: Migrates entity records and synchronizes embeddings under the target `agents` entity type with updated `origin_type = 'agents'` and `origin_uri = '/agents/{id}'`.
- **Zero UI Sync Button**: Because BREAD operations keep embeddings strongly consistent on write, manual "Sync Embeddings" buttons are omitted from the user interface. Backend maintenance endpoint `POST /skills/:id/sync-embeddings` is retained for automated testing and CLI usage.

## Cross References
- [Agent UI & Testing Kit PRD](./agent-ui-testing-kit-prd.md)
- [Agent Registry & Execution PRD](./agent-registry-execution-prd.md)
- [Semantic Search Discovery PRD](./semantic-search-page-prd.md)
- [Master PRD](./agent-as-data-prd.md)
