# CLI Manifest Tool Technical Specification

## 1. Overview
The `ctl` tool will be added to the `aad-be` CLI to parse multi-part YAML definitions of Agents and Skills, validating schemas using existing models, and executing API calls to create, read, delete, and list resources.

## 2. CLI Structure Extension (`aad-be-container/src/main.rs`)
Add a new `Ctl` variant to the `Commands` enum:
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...
    /// Manage resources declaratively via API
    Ctl {
        #[command(subcommand)]
        command: CtlCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum CtlCommands {
    /// Apply a resource definition from a YAML file
    Apply {
        #[arg(short, long)]
        file: std::path::PathBuf,
    },
    /// Get details of a resource or list resources
    Get {
        resource_type: String, // e.g. "agent", "skill"
        id: Option<String>,
    },
    /// Delete a resource
    Delete {
        resource_type: String,
        id: String,
    },
}
```

## 3. Modular Implementation
Create a new module `aad-be-container/src/cli_commands/mod.rs` (or similar) to isolate CLI logic.

### 3.1 `Apply` Command Logic
1. Load `AppConfig` to determine the `api_prefix` and listening port. (For a CLI client, we might assume the server is running on the configured `webservice.port`).
2. Read the file provided.
3. Parse multi-part YAML using `serde_yaml::Deserializer::from_reader`.
4. Parse each document into an untyped `serde_json::Value` or a custom `ResourceWrapper` to detect the `type` field.
5. Deserialize into the respective strongly-typed payload struct (e.g., `models::agent::CreateAgentRequest` or `models::skill::CreateSkillRequest`).
6. Dispatch an HTTP request using `reqwest` to `POST /api_prefix/v1/agents` or `POST /api_prefix/v1/skills`.

### 3.2 `Get` and `Delete` Command Logic
1. Route the command based on `resource_type` (case-insensitive: "agent", "agents", "skill", "skills").
2. Construct the URL (`GET /api_prefix/v1/agents/{id}`, `GET /api_prefix/v1/agents`, `DELETE /api_prefix/v1/agents/{id}`).
3. Print the JSON response (or formatted YAML/table) to `stdout`.

## 4. Multi-part YAML Parsing Strategy
```rust
use serde::Deserialize;
use serde_yaml::Deserializer;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ResourceManifest {
    Agent(aad_be_container::models::agent::CreateAgentRequest),
    Skill(aad_be_container::models::skill::CreateSkillRequest),
}
```
Using an enum with `tag = "type"` allows `serde` to automatically route parsing based on the `type` field.

## 5. API Client Configuration
The CLI needs to connect to the running API. It will construct the base URL using the `AppConfig` loaded via `--config-path`.
Base URL format: `http://{bind_addr}:{port}/{api_prefix}`

## 6. Error Handling
- The tool must fail fast if a YAML document fails schema validation.
- If an API request fails (non-2xx status), print the response error and exit with a non-zero code.
