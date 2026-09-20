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

## Integration
These tools must be attached to the agent pipeline in `execute_workspace_tool` and `llm_tools.rs` to make them available across Workbench threads.
