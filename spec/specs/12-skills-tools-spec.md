---
status: complete
---

# 12. Skills Registry Tools Spec

## Objective
Implement `list_skills` and `view_skill` tools in `aad-be` for LLM agents to interact with the skills registry.

## Tool Definitions (`aad-be-container/src/llm_tools.rs`)

### 1. `ListSkillsTool`
**Description:** Lists all available skills in the database.
**Parameters:** None (or empty struct).
**Behavior:** 
- Execute `SELECT id, name, description FROM skills`
- Return `ListSkillsOutput { skills: Vec<SkillMetadata> }` where `SkillMetadata` has `name` and `description`.

### 2. `ViewSkillTool`
**Description:** Views the full definition of a specific skill.
**Parameters:** `skill_name: String`
**Behavior:**
- Execute `SELECT * FROM skills WHERE name = $1`
- Return `ViewSkillOutput` containing the skill details (definition, description, traits).
- If not found, return an error message to the LLM (e.g., "Skill not found").

## Integration Points
1. **`execute_workspace_tool`**: Add routing for `list_skills` and `view_skill`.
2. **Agent Build**: Attach `ListSkillsTool` and `ViewSkillTool` to the `AgentBuilder` in `aad-be-container/src/main.rs` (or where the agent is constructed).

## Tests
- Integration test in `llm_tools.rs` to verify `list_skills` and `view_skill` function as expected.
