# Workspace Filesystem Tools & Agent Tool Execution PRD

## Overview
This document defines the requirements and architectural standards for workspace filesystem tools and agent tool execution within **Agent-As-Data (AAD)**. When a multiuser conversational thread is created in the Workbench, it establishes a dedicated, isolated workspace directory (`/tmp/workspace/<thread_id>`). This PRD formalizes how native Rust tools and external MCP tools are defined, bound to Rig agents via the `Tool` trait and `AgentBuilder`, and executed in an autonomous multi-turn loop.

## Objectives
1. **Thread Isolation**: Each thread must maintain an isolated filesystem directory located at `/tmp/workspace/<thread_id>`.
2. **Standardized Rig Tool Implementations**: Expose safe workspace filesystem tools (`read_file`, `write_file`, `replace_in_file`, `list_files`, `delete_file`, `rename_file`) implemented in accordance with Rig's `Tool` trait architecture and JSON schema definitions.
3. **Autonomous Multi-Turn Execution**: Leverage Rig's `AgentBuilder` multi-turn agent loop (`.tool(...)`, `.max_turns(...)`) so the model autonomously selects tools, executes functions, receives structured outputs or errors, self-corrects, and formulates final user responses.
4. **Self-Correcting Error Recovery**: Format all tool execution errors as informative, instructive feedback for the LLM rather than failing the prompt.
5. **Path Traversal Security**: Strictly enforce workspace containment to prevent any access outside `/tmp/workspace/<thread_id>`.

---

## Agent Tool Execution Architecture

```mermaid
sequenceDiagram
    autonumber
    actor User as Workbench UI / Developer
    participant BE as AAD Webserver (threads.rs)
    participant RigAgent as Rig Agent Loop (AgentBuilder)
    participant LLM as Ollama / Model Provider
    participant Tools as Rig Workspace Tools (Tool Trait)
    participant FS as Local Workspace (/tmp/workspace/thread_id)

    User->>BE: POST /api/v1/threads/{id}/messages (user message)
    BE->>BE: Insert user message into DB
    BE->>RigAgent: agent.prompt(user_content)
    RigAgent->>LLM: Dispatches Prompt + Tool Schemas (JSON Schema)
    
    loop Multi-Turn Tool Execution Loop (up to max_turns)
        LLM-->>RigAgent: Tool Call Request (e.g. list_files, read_file)
        RigAgent->>Tools: Tool::call(args)
        Tools->>FS: Safe Path Sanitization & File I/O
        FS-->>Tools: File Content / Directory Listing / Error
        Tools-->>RigAgent: Tool Result Payload / Instructive Error String
        RigAgent->>LLM: Tool Result Feedback
    end

    LLM-->>RigAgent: Final Response Text
    RigAgent-->>BE: Returns Assistant Message
    BE->>BE: Insert assistant message into DB
    BE-->>User: 201 Created (User Message) & Real-time Assistant Update
```

---

## Core Features & Tool Specifications

### 1. Workspace Lifecycle & Isolation
- **Directory Scaffolding**: Upon creation of a `Thread` (via `POST /{{api_prefix}}/v1/threads/create` or lazy access), the system automatically provisions `/tmp/workspace/<thread_id>`.
- **Idempotency**: Directory creation must be idempotent (`create_dir_all`).
- **Security Boundary**: All file operations must be validated against `resolve_safe_path` to guarantee canonical paths strictly begin with `/tmp/workspace/<thread_id>/`. Path traversal attempts (`..`, symlink escapes, absolute paths) must be rejected with `PermissionDenied`.

### 2. Filesystem Tool Interfaces (`Tool` Trait)

All workspace tools must implement Rig's `rig_core::tool::Tool` trait with typed arguments, JSON schema definitions, and model-oriented descriptions.

| Tool Name | Arguments | Description | Output |
|---|---|---|---|
| `list_files` | `dir_path: Option<String>` | Lists files and subdirectories under the workspace path. | `ListFilesOutput { files: Vec<String> }` |
| `read_file` | `filepath: String` | Reads text content of a file relative to workspace root. | `ReadFileOutput { content: String }` |
| `write_file` | `filepath: String, content: String` | Creates or overwrites a file; automatically scaffolds parent folders. | `WriteFileOutput { success: bool, message: String }` |
| `replace_in_file` | `filepath: String, search_string: String, replace_string: String` | Targeted search-and-replace for modifying specific blocks. | `ReplaceInFileOutput { success: bool, message: String }` |
| `delete_file` | `filepath: String` | Deletes a specified file or directory. | `DeleteFileOutput { success: bool, message: String }` |
| `rename_file` | `filepath: String, new_filepath: String` | Renames or moves a file/directory within workspace. | `RenameFileOutput { success: bool, message: String }` |

### 3. Agent Integration & Execution Loop

```mermaid
flowchart TD
    A[Incoming User Thread Message] --> B[Initialize Ollama / LLM Client]
    B --> C[Scaffold Rig Agent with Preamble]
    C --> D[Attach Workspace Tools via .tool]
    D --> E[Configure Turn Budget via .max_turns]
    E --> F[Execute agent.prompt]
    F --> G{Model Requests Tool?}
    G -->|Yes| H[Execute Tool Function]
    H --> I{Execution Success?}
    I -->|Success| J[Feed Serialized Output to Model]
    I -->|Failure / Invalid Input| K[Feed Instructive Error Message to Model for Self-Correction]
    J --> F
    K --> F
    G -->|No / Complete| L[Persist Assistant Message to Database]
    L --> M[Refresh Workbench UI State]
```

#### Best Practices for Rig Tool Attachment:
1. **`AgentBuilder` Attachment**:
   ```rust
   let agent = client
       .agent(&config.llm.model)
       .preamble("You are a workspace assistant operating within an isolated directory.")
       .tool(ListFilesTool { thread_id })
       .tool(ReadFileTool { thread_id })
       .tool(WriteFileTool { thread_id })
       .tool(ReplaceInFileTool { thread_id })
       .tool(DeleteFileTool { thread_id })
       .tool(RenameFileTool { thread_id })
       .max_turns(5)
       .build();
   ```
2. **Turn Budgeting**: Always configure `.max_turns(5)` (or higher) to give the model headroom to call multiple tools sequentially and self-correct when necessary.
3. **Instructive Error Feedback**: When a tool fails (e.g. file does not exist), return clear contextual guidance (e.g. `File 'notes.txt' not found. Available workspace files are: ['todo.md', 'draft.txt']`) so the model can adjust arguments on the next turn.

### 4. Advanced Tool Capabilities (Roadmap)
- **Tool-RAG (`ToolEmbedding`)**: For agents with large tool catalogs (e.g. tools, skills, database queries), implement `ToolEmbedding` to retrieve only the top `N` relevant tools via vector similarity (`.dynamic_tools(n, index, toolset)`).
- **Model Context Protocol (MCP)**: Attach external tool servers dynamically using `AgentBuilder::rmcp_tool(...)` / `ToolServerHandle`.
- **Tool Servers**: Run high-throughput or shared mutable tools in isolated Tokio tasks via `ToolServer`.

---

## Related Specifications
- [09-backend-modular-architecture-spec.md](../specs/09-backend-modular-architecture-spec.md): Modular router and webserver architecture.
- [10-workbench-spec.md](../specs/10-workbench-spec.md): Workbench UI file management and chat integration.
- [Master PRD](./agent-as-data-prd.md): Core platform capabilities and execution engine.

