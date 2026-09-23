# Spec 30: Workbench Multi-Turn Journeys

**Status**: `draft`

## Overview
This specification outlines the implementation details for supporting the three multi-turn conversational journeys defined in [workbench-multiturn-journeys-prd.md](../prds/workbench-multiturn-journeys-prd.md). It focuses on ensuring the foundational capabilities—context retention across chat turns and workspace file manipulation—are robustly implemented and tested.

Crucially, the actual "Code Execution MCP Server" (Docker-based sandbox) is out of scope for this initial implementation phase. Code execution interactions (Journeys 1 and 3) will be handled via a mocked/stubbed MCP tool response within the backend to allow testing the agent's *intent* to execute code without requiring the actual secure runtime.

## Dependencies & References
- **PRD Reference**: [workbench-multiturn-journeys-prd.md](../prds/workbench-multiturn-journeys-prd.md)
- **Previous Specs**: [10-workbench-spec.md](./10-workbench-spec.md) (Thread File Management), [11-workspace-agent-tool-execution-spec.md](./11-workspace-agent-tool-execution-spec.md) (Tool execution loop).

## Requirements

### 1. Mocked Code Execution Tool
Until the dedicated MCP server is built, the backend must inject a placeholder tool into the agent's execution context to simulate code execution capability.
- **Tool Name**: `mcp:code_runner:execute_python` (or similar stub).
- **Description**: Instructs the agent that it can run python code files present in the workspace.
- **Behavior**: When the agent invokes this tool, the backend should intercept it and return a static, successful mock response (e.g., `stdout: "Hello Workbench\n"` for Journey 1) rather than attempting actual execution.

### 2. Context Retention (Multi-Turn)
The backend `POST /v1/threads/{id}/chat` endpoint must effectively load previous `Message` history from the database and feed it into the `rig-core` LLM prompt context to ensure the agent remembers constraints and outputs from Turn 1 when executing Turn 2.

### 3. Workspace File Tool Integration
The agent must have access to the existing workspace filesystem tools (`read_file`, `write_file`) injected into its prompt context during thread execution.

## Testing Strategy (Robot Framework)
A new test suite `test_journey_18_workbench_multiturn.robot` will be created to validate these capabilities via API calls.

*   **Test Case 1: Brainstorming and File Writing (Journey 2)**
    *   *Step 1*: Send chat request: "Brainstorm 3 ideas for a feature." Validate response contains ideas.
    *   *Step 2*: Send chat request: "Write the second idea to feature_idea.md."
    *   *Step 3*: Use the backend FS read API (`/v1/threads/{id}/fs/read/feature_idea.md`) to assert the file was successfully created by the agent and contains markdown text.
*   **Test Case 2: Code Generation and (Mocked) Execution (Journey 1/3 Hybrid)**
    *   *Step 1*: Send chat request: "Write a python script `hello.py` that prints 'Hello'."
    *   *Step 2*: Verify via FS API that `hello.py` exists.
    *   *Step 3*: Send chat request: "Run hello.py."
    *   *Step 4*: Assert the agent's text response includes the mocked execution output, confirming it successfully attempted to use the mock execution tool.
