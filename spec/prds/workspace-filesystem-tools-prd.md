# Workspace Filesystem Tools PRD

## Overview
This document defines the requirements for isolated workspace filesystem tools within **Agent-As-Data (AAD)**. When a multiuser conversational thread is created, it requires a dedicated, isolated filesystem directory. This enables AI agents and tools to perform safe filesystem operations (read, write, list, delete) bounded strictly to that thread's contextual workspace.

## Objectives
1. **Thread Isolation**: Each thread must have its own isolated filesystem directory located at `/tmp/workspace/<thread_id>`.
2. **Filesystem Tools**: Expose specific tools (`read_file`, `write_file`, `list_files`, `delete_file`) that operate exclusively within a thread's workspace.
3. **Security & Path Constraints**: Strictly prevent path traversal vulnerabilities. No operation should be able to access, read, or modify files outside of its designated `/tmp/workspace/<thread_id>` directory.

## Core Features

### 1. Workspace Initialization
- Upon successful creation of a `Thread` (via the `POST /api/v1/threads/create` endpoint or equivalent backend logic), the system must automatically create a physical directory at `/tmp/workspace/<thread_id>`.
- If the directory already exists, it should not fail, ensuring idempotency.
- Proper filesystem permissions should be enforced such that only the AAD backend process can access these directories.

### 2. Filesystem Tool Interfaces
The following operations will be provided as tools. They accept a `thread_id` and a `filepath` (which is always treated as relative to the thread's workspace root).

- **`write_file(thread_id, filepath, content)`**:
  - Writes the provided `content` (string/bytes) to the specified `filepath`.
  - Automatically creates any missing parent directories within the workspace to fulfill the `filepath`.
  - Overwrites existing files.

- **`read_file(thread_id, filepath)`**:
  - Reads the content of the file at `filepath`.
  - Returns an error if the file does not exist or is a directory.

- **`list_files(thread_id, dir_path)`**:
  - Lists the contents of `dir_path` (defaults to the root of the workspace if empty).
  - Returns a list of filenames/directory names.

- **`delete_file(thread_id, filepath)`**:
  - Deletes the file or directory at `filepath`.

### 3. Security & Path Traversal Prevention
- **Strict Canonicalization**: All input paths must be resolved and canonicalized.
- **Boundary Check**: After resolving an absolute path, the system must verify that the resulting path starts with exactly `/tmp/workspace/<thread_id>/`.
- Any attempt to use `../` or absolute paths like `/etc/passwd` that resolve outside the workspace root must be immediately rejected with a definitive security error.

## Architecture Integration
- **Backend Container**: The Rust backend (`aad-be-container`) will expose these capabilities. The tools can be invoked programmatically during agent execution.
- **Directory**: `/tmp/workspace/` is ephemeral. This aligns with containerized deployments where persistent data lives in the database, and local thread workspaces are treated as temporary scratchpads.

## Implementation Phases
1. Update thread creation logic to scaffold `/tmp/workspace/<thread_id>`.
2. Implement robust path sanitization and boundary checking utility in Rust.
3. Implement the tool functions (`read_file`, `write_file`, etc.).
4. Expose these functions to the internal agent execution context or via REST API endpoints as needed.
