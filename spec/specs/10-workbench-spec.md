# Spec 10: Workbench Thread File Management

**Status**: `complete`

## Overview & Scope
This specification defines the extension of the **Workbench** UI in **`aad-fe-container`** to include file management capabilities within conversational threads. It integrates with the backend filesystem APIs (`/v1/threads/{id}/fs/*`) defined in `aad-be-container/src/webserver/fs.rs`.

Each conversation thread operates within an isolated local directory workspace (`/tmp/workspace/{thread_id}`). This specification implements the UI allowing developers to create, list, read, edit, and delete files directly inside that thread's workspace from the Workbench UI.

## Dependencies & References
- **Build Order Phase**: **Phase 4 (Developer UI & Workbench)**.
- **Dependencies**: Depends on [08-developer-ui-studio-spec.md](./08-developer-ui-studio-spec.md) and Backend Web Server FS APIs (`fs.rs`).
- **PRD References**: [Agent UI & Testing Kit PRD](../prds/agent-ui-testing-kit-prd.md), [Agent-As-Data Master PRD](../prds/agent-as-data-prd.md).

## Requirements

### API Service Enhancements
The Angular `ApiService` must be extended with methods corresponding to the thread FS endpoints:
- `listThreadFiles(threadId: string, dirPath?: string): Observable<string[]>` (POST `/v1/threads/{id}/fs/list`)
- `readThreadFile(threadId: string, filepath: string): Observable<string>` (GET `/v1/threads/{id}/fs/read/{filepath}`)
- `writeThreadFile(threadId: string, filepath: string, content: string): Observable<any>` (POST `/v1/threads/{id}/fs/write`)
- `deleteThreadFile(threadId: string, filepath: string): Observable<any>` (POST `/v1/threads/{id}/fs/delete`)

### UI Implementation (`workbench.component.html`)
The existing mock "Review" diff view in the right-hand **Editor Pane** is replaced by a functional file management layout:
- **Header**: Action buttons (e.g., "Add File") in the Editor Header.
- **File Explorer Sidebar**: A list of files present in the active thread's workspace.
  - Clicking a file selects it and reads its contents.
  - Hovering or viewing a file shows a "Delete" action.
- **Code Editor Area**:
  - Displays the `selectedFileContent` in a `textarea` (or equivalent editable text area).
  - A "Save" button to commit changes back to the filesystem via the `writeThreadFile` API.
  - If no file is selected, displays a placeholder message (e.g., "Select a file to edit").

### Component Logic (`workbench.component.ts`)
- **State**: Track `files` (array of strings), `selectedFile` (string or null), and `selectedFileContent` (string).
- **Hooks**:
  - On thread selection/load (`loadThreadContent`), automatically trigger `loadThreadFiles()` to populate the workspace.
- **Actions**:
  - `loadThreadFiles()`: Calls `listThreadFiles` and updates `files`. If the currently `selectedFile` no longer exists, clear the selection.
  - `selectFile(filename)`: Calls `readThreadFile` and updates `selectedFile` and `selectedFileContent`.
  - `createNewFile()`: Prompts for a filename, calls `writeThreadFile` with empty content, updates the `files` list, and selects the new file.
  - `saveFile()`: Calls `writeThreadFile` with `selectedFile` and `selectedFileContent`. Shows a success toast/alert on completion.
  - `deleteFile(filename)`: Confirms deletion, calls `deleteThreadFile`, reloads files, and clears selection if the deleted file was active.
