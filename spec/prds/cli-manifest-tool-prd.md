# CLI Manifest Tool PRD

## Overview
The CLI Manifest Tool will provide a set of subcommands under the `aad-be` (Agent-As-Data Backend) application to manage Agents and Skills using declarative YAML manifests. This tooling behaves similarly to `kubectl` for Kubernetes, allowing users to define their desired state in a YAML file and apply it, as well as fetch and delete resources. It will interact with the system strictly via the existing REST API.

## Objectives
- Allow users to manage (create, update, read, delete, list) Agents and Skills from the command line without interacting directly with the database.
- Support reading multi-part YAML documents containing multiple resource definitions (e.g., both Agents and Skills).
- Validate YAML schemas locally before dispatching to the API, failing fast on parse or schema errors.
- Ensure the CLI tool is built as an extension (subcommand tree) of the existing `aad-be` CLI.

## Scope
### Supported Resources
- `Agent`
- `Skill`

### New CLI Structure
A new top-level subcommand, for example `ctl`, added to `aad-be`:

```
aad-be ctl [COMMAND]
```

#### Subcommands
- `apply -f <file.yaml>`: Reads a multi-part YAML file and creates/updates the resources defined in it by calling the backend API.
- `get <resource_type> [id]`: Lists all resources of the given type or fetches a specific resource by ID/name.
- `delete <resource_type> <id>`: Deletes a specific resource by ID/name.

### Resource Definition Schema (YAML)
Each document in a multi-part YAML file should define its type and properties. For example:

```yaml
---
type: Skill
name: search_docs
description: Searches the knowledge base
# ... other skill properties
---
type: Agent
name: help_desk_agent
description: Answers customer questions
skills: ["search_docs"]
# ... other agent properties
```

## Technical Architecture
- **Command Parsing**: Extend `clap` usage in `aad-be-container/src/main.rs` to include the `Ctl` command tree.
- **API Client**: Implement a simple HTTP client (e.g., using `reqwest` or `hyper` + `tower`) that uses the server's configured API prefix and port to send requests.
- **YAML Parsing**: Use `serde_yaml` to parse and deserialize the multi-part documents.
- **Schema Validation**: Deserialize directly into strongly-typed DTOs to leverage `serde`'s strict parsing, ensuring schema adherence before API execution.

## Out of Scope
- Direct database interactions (all operations must use the REST API).
- Deploying the backend service (this tool assumes the service is running).
