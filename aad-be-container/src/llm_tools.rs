use rig_core::tool::PortableTool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::webserver::fs::{get_workspace_root, resolve_safe_path};

#[derive(Deserialize)]
pub struct ReadFileArgs {
    pub filepath: String,
}

#[derive(Serialize)]
pub struct ReadFileOutput {
    pub content: String,
}

#[derive(Deserialize)]
pub struct WriteFileArgs {
    pub filepath: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct WriteFileOutput {
    pub success: bool,
    pub message: String,
}

pub struct ReadFileTool {
    pub bench_id: Uuid,
}

impl PortableTool for ReadFileTool {
    const NAME: &'static str = "read_file";
    type Error = std::io::Error;
    type Args = ReadFileArgs;
    type Output = ReadFileOutput;

    fn description(&self) -> String {
        "Reads the content of a file in the workspace.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "Path to the file to read, relative to the workspace root."
                }
            },
            "required": ["filepath"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", args.filepath),
            ));
        }

        if safe_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path is a directory: {}", args.filepath),
            ));
        }

        let content = std::fs::read_to_string(&safe_path)?;

        Ok(ReadFileOutput { content })
    }
}

pub struct WriteFileTool {
    pub bench_id: Uuid,
}

impl PortableTool for WriteFileTool {
    const NAME: &'static str = "write_file";
    type Error = std::io::Error;
    type Args = WriteFileArgs;
    type Output = WriteFileOutput;

    fn description(&self) -> String {
        "Overwrites or creates a file with the provided content in the workspace. Will create parent directories if they don't exist.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "Path to the file to create or overwrite, relative to the workspace root."
                },
                "content": {
                    "type": "string",
                    "description": "The full content to write to the file."
                }
            },
            "required": ["filepath", "content"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            std::fs::create_dir_all(&workspace_root)?;
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if let Some(parent) = safe_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::write(&safe_path, &args.content)?;

        Ok(WriteFileOutput {
            success: true,
            message: format!("Successfully wrote to {}", args.filepath),
        })
    }
}

#[derive(Deserialize)]
pub struct ReplaceInFileArgs {
    pub filepath: String,
    pub search_string: String,
    pub replace_string: String,
}

#[derive(Serialize)]
pub struct ReplaceInFileOutput {
    pub success: bool,
    pub message: String,
}

pub struct ReplaceInFileTool {
    pub bench_id: Uuid,
}

impl PortableTool for ReplaceInFileTool {
    const NAME: &'static str = "replace_in_file";
    type Error = std::io::Error;
    type Args = ReplaceInFileArgs;
    type Output = ReplaceInFileOutput;

    fn description(&self) -> String {
        "Performs a targeted search-and-replace to modify specific blocks in a file without rewriting the entire file. `search_string` must match exactly the text to be replaced.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "Path to the file to modify, relative to the workspace root."
                },
                "search_string": {
                    "type": "string",
                    "description": "The exact string block to search for."
                },
                "replace_string": {
                    "type": "string",
                    "description": "The exact string block to replace it with."
                }
            },
            "required": ["filepath", "search_string", "replace_string"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", args.filepath),
            ));
        }

        if safe_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path is a directory: {}", args.filepath),
            ));
        }

        let mut content = std::fs::read_to_string(&safe_path)?;

        if !content.contains(&args.search_string) {
            return Ok(ReplaceInFileOutput {
                success: false,
                message: format!("Search string not found in {}", args.filepath),
            });
        }

        content = content.replace(&args.search_string, &args.replace_string);

        std::fs::write(&safe_path, content)?;

        Ok(ReplaceInFileOutput {
            success: true,
            message: format!("Successfully replaced string in {}", args.filepath),
        })
    }
}

#[derive(Deserialize)]
pub struct ListFilesArgs {
    pub dir_path: Option<String>,
}

#[derive(Serialize)]
pub struct ListFilesOutput {
    pub files: Vec<String>,
}

pub struct ListFilesTool {
    pub bench_id: Uuid,
}

impl PortableTool for ListFilesTool {
    const NAME: &'static str = "list_files";
    type Error = std::io::Error;
    type Args = ListFilesArgs;
    type Output = ListFilesOutput;

    fn description(&self) -> String {
        "Lists files and directories under the given directory in the workspace (defaults to workspace root).".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "dir_path": {
                    "type": "string",
                    "description": "The directory path to list files from, relative to the workspace root. Defaults to the root of the workspace if omitted or empty."
                }
            }
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        }

        let target_dir = match args.dir_path {
            Some(ref p) if !p.is_empty() => p.clone(),
            _ => "".to_string(), // Root of workspace
        };

        let safe_path = resolve_safe_path(&workspace_root, &target_dir)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Directory not found: {}", target_dir),
            ));
        }

        if !safe_path.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", target_dir),
            ));
        }

        let mut files = Vec::new();
        let entries = std::fs::read_dir(safe_path)?;

        for entry in entries {
            let Ok(entry) = entry else { continue };
            let Ok(file_name) = entry.file_name().into_string() else { continue };
            
            let mut suffix = "";
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    suffix = "/";
                }
            }
            files.push(format!("{}{}", file_name, suffix));
        }

        files.sort();

        Ok(ListFilesOutput { files })
    }
}

#[derive(Deserialize)]
pub struct DeleteFileArgs {
    pub filepath: String,
}

#[derive(Serialize)]
pub struct DeleteFileOutput {
    pub success: bool,
    pub message: String,
}

pub struct DeleteFileTool {
    pub bench_id: Uuid,
}

impl PortableTool for DeleteFileTool {
    const NAME: &'static str = "delete_file";
    type Error = std::io::Error;
    type Args = DeleteFileArgs;
    type Output = DeleteFileOutput;

    fn description(&self) -> String {
        "Deletes the specified file or directory in the workspace.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "Path to the file or directory to delete, relative to the workspace root."
                }
            },
            "required": ["filepath"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File or directory not found: {}", args.filepath),
            ));
        }

        if safe_path.is_dir() {
            std::fs::remove_dir_all(&safe_path)?;
        } else {
            std::fs::remove_file(&safe_path)?;
        }

        Ok(DeleteFileOutput {
            success: true,
            message: format!("Successfully deleted {}", args.filepath),
        })
    }
}

#[derive(Deserialize)]
pub struct RenameFileArgs {
    pub filepath: String,
    pub new_filepath: String,
}

#[derive(Serialize)]
pub struct RenameFileOutput {
    pub success: bool,
    pub message: String,
}

pub struct RenameFileTool {
    pub bench_id: Uuid,
}

impl PortableTool for RenameFileTool {
    const NAME: &'static str = "rename_file";
    type Error = std::io::Error;
    type Args = RenameFileArgs;
    type Output = RenameFileOutput;

    fn description(&self) -> String {
        "Renames or moves a file or directory in the workspace.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "The current path of the file or directory, relative to the workspace root."
                },
                "new_filepath": {
                    "type": "string",
                    "description": "The new path for the file or directory, relative to the workspace root."
                }
            },
            "required": ["filepath", "new_filepath"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let workspace_root = get_workspace_root(self.bench_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Workspace not found",
            ));
        }

        let old_safe_path = resolve_safe_path(&workspace_root, &args.filepath).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Invalid original path: {}", e),
            )
        })?;

        let new_safe_path =
            resolve_safe_path(&workspace_root, &args.new_filepath).map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("Invalid new path: {}", e),
                )
            })?;

        if !old_safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Original file or directory not found: {}", args.filepath),
            ));
        }

        if new_safe_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "Target file or directory already exists: {}",
                    args.new_filepath
                ),
            ));
        }

        if let Some(parent) = new_safe_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::rename(&old_safe_path, &new_safe_path)?;

        Ok(RenameFileOutput {
            success: true,
            message: format!(
                "Successfully renamed {} to {}",
                args.filepath, args.new_filepath
            ),
        })
    }
}

#[derive(Deserialize)]
pub struct ReadBenchMemoryArgs {}

#[derive(Serialize)]
pub struct ReadBenchMemoryOutput {
    pub content: String,
}

pub struct ReadBenchMemoryTool {
    pub bench_id: Uuid,
    pub pool: sqlx::PgPool,
}

impl PortableTool for ReadBenchMemoryTool {
    const NAME: &'static str = "read_bench_memory";
    type Error = std::io::Error;
    type Args = ReadBenchMemoryArgs;
    type Output = ReadBenchMemoryOutput;

    fn description(&self) -> String {
        "Reads the shared working memory document for the current bench.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let rec = sqlx::query_scalar::<_, String>(
            "SELECT content FROM bench_memory WHERE bench_id = $1 AND memory_type = 'working' LIMIT 1"
        )
        .bind(self.bench_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Database error reading bench memory: {}", e)))?;

        Ok(ReadBenchMemoryOutput {
            content: rec.unwrap_or_default(),
        })
    }
}

#[derive(Deserialize)]
pub struct UpdateBenchMemoryArgs {
    pub content: String,
    pub append: Option<bool>,
}

#[derive(Serialize)]
pub struct UpdateBenchMemoryOutput {
    pub success: bool,
    pub message: String,
}

pub struct UpdateBenchMemoryTool {
    pub bench_id: Uuid,
    pub pool: sqlx::PgPool,
}

#[derive(Deserialize)]
pub struct ListSkillsArgs {}

#[derive(Serialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
}

#[derive(Serialize)]
pub struct ListSkillsOutput {
    pub skills: Vec<SkillMetadata>,
}

pub struct ListSkillsTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for ListSkillsTool {
    const NAME: &'static str = "list_skills";
    type Error = std::io::Error;
    type Args = ListSkillsArgs;
    type Output = ListSkillsOutput;

    fn description(&self) -> String {
        "Lists all available skills in the database.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let rows: Vec<(String, String)> = sqlx::query_as("SELECT name, description FROM skills")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Database error: {}", e))
            })?;

        let skills = rows
            .into_iter()
            .map(|(name, description)| SkillMetadata { name, description })
            .collect();

        Ok(ListSkillsOutput { skills })
    }
}

#[derive(Deserialize)]
pub struct ViewSkillArgs {
    pub skill_name: String,
}

#[derive(Serialize)]
pub struct ViewSkillOutput {
    pub skill: Option<crate::models::skill::Skill>,
    pub error: Option<String>,
}

pub struct ViewSkillTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for ViewSkillTool {
    const NAME: &'static str = "view_skill";
    type Error = std::io::Error;
    type Args = ViewSkillArgs;
    type Output = ViewSkillOutput;

    fn description(&self) -> String {
        "Views the full definition of a specific skill.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "skill_name": {
                    "type": "string",
                    "description": "The name of the skill to view."
                }
            },
            "required": ["skill_name"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let skill_res = sqlx::query_as::<_, crate::models::skill::Skill>(
            "SELECT * FROM skills WHERE name = $1",
        )
        .bind(&args.skill_name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Database error: {}", e))
        })?;

        if let Some(skill) = skill_res {
            Ok(ViewSkillOutput {
                skill: Some(skill),
                error: None,
            })
        } else {
            Ok(ViewSkillOutput {
                skill: None,
                error: Some("Skill not found".to_string()),
            })
        }
    }
}

impl PortableTool for UpdateBenchMemoryTool {
    const NAME: &'static str = "update_bench_memory";
    type Error = std::io::Error;
    type Args = UpdateBenchMemoryArgs;
    type Output = UpdateBenchMemoryOutput;

    fn description(&self) -> String {
        "Updates or appends content to the shared working memory document for the current bench."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Content to write or append to the working memory."
                },
                "append": {
                    "type": "boolean",
                    "description": "If true, appends content to existing memory instead of replacing it. Defaults to false."
                }
            },
            "required": ["content"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let should_append = args.append.unwrap_or(false);

        if should_append {
            sqlx::query(
                "INSERT INTO bench_memory (bench_id, memory_type, title, content)
                 VALUES ($1, 'working', 'Active Working Memory', $2)
                 ON CONFLICT (bench_id) WHERE memory_type = 'working'
                 DO UPDATE SET
                     content = CASE 
                         WHEN bench_memory.content = '' THEN EXCLUDED.content
                         ELSE bench_memory.content || E'\\n\\n' || EXCLUDED.content
                     END,
                     updated_at = NOW()",
            )
            .bind(self.bench_id)
            .bind(&args.content)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database error appending bench memory: {}", e),
                )
            })?;
        } else {
            sqlx::query(
                "INSERT INTO bench_memory (bench_id, memory_type, title, content)
                 VALUES ($1, 'working', 'Active Working Memory', $2)
                 ON CONFLICT (bench_id) WHERE memory_type = 'working'
                 DO UPDATE SET
                     content = EXCLUDED.content,
                     updated_at = NOW()",
            )
            .bind(self.bench_id)
            .bind(&args.content)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database error updating bench memory: {}", e),
                )
            })?;
        }

        Ok(UpdateBenchMemoryOutput {
            success: true,
            message: "Working memory updated successfully.".to_string(),
        })
    }
}

// ============================================================================
// In-Process AAD Capability Tools (backed directly by AadMcpServer)
// ============================================================================

#[derive(Deserialize, Default)]
pub struct ListAgentsToolArgs {
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct AadCapabilityToolOutput {
    pub result: serde_json::Value,
}

fn parse_mcp_json_output(raw: String) -> AadCapabilityToolOutput {
    let parsed = serde_json::from_str(&raw).unwrap_or_else(|_| serde_json::Value::String(raw));
    AadCapabilityToolOutput { result: parsed }
}

pub struct ListAgentsTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for ListAgentsTool {
    const NAME: &'static str = "list_agents";
    type Error = std::io::Error;
    type Args = ListAgentsToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Lists active declarative agents registered in Agent-As-Data, optionally filtered by exact name.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Optional exact agent name filter."
                }
            }
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool("list_agents", json!({ "name": args.name }))
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize)]
pub struct GetAgentToolArgs {
    pub id: Option<Uuid>,
    pub name: Option<String>,
}

pub struct GetAgentTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for GetAgentTool {
    const NAME: &'static str = "get_agent";
    type Error = std::io::Error;
    type Args = GetAgentToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Retrieves a declarative agent definition from Agent-As-Data by UUID or exact name.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "UUID of the agent." },
                "name": { "type": "string", "description": "Exact name of the agent." }
            }
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool("get_agent", json!({ "id": args.id, "name": args.name }))
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize)]
pub struct SearchAgentsToolArgs {
    pub query: String,
    pub limit: Option<usize>,
}

pub struct SearchAgentsTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for SearchAgentsTool {
    const NAME: &'static str = "search_agents";
    type Error = std::io::Error;
    type Args = SearchAgentsToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Searches active agents in Agent-As-Data by keyword or natural-language query.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query for matching agents." },
                "limit": { "type": "integer", "description": "Maximum number of results." }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool(
                "search_agents",
                json!({ "query": args.query, "limit": args.limit }),
            )
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize, Default)]
pub struct ListTraitsToolArgs {
    pub page: Option<i64>,
    pub size: Option<i64>,
}

pub struct ListTraitsTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for ListTraitsTool {
    const NAME: &'static str = "list_traits";
    type Error = std::io::Error;
    type Args = ListTraitsToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Lists trait contracts defined in Agent-As-Data.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "page": { "type": "integer", "description": "Zero-based page index." },
                "size": { "type": "integer", "description": "Maximum traits per page." }
            }
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool("list_traits", json!({ "page": args.page, "size": args.size }))
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize, Default)]
pub struct ListMcpServersToolArgs {}

pub struct ListMcpServersTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for ListMcpServersTool {
    const NAME: &'static str = "list_mcp_servers";
    type Error = std::io::Error;
    type Args = ListMcpServersToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Lists registered MCP tool servers in Agent-As-Data and their cached tool capabilities.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool("list_mcp_servers", json!({}))
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize)]
pub struct SearchAgentContextToolArgs {
    pub query: String,
    pub limit: Option<usize>,
}

pub struct SearchAgentContextTool {
    pub pool: sqlx::PgPool,
}

impl PortableTool for SearchAgentContextTool {
    const NAME: &'static str = "search_agent_context";
    type Error = std::io::Error;
    type Args = SearchAgentContextToolArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Performs unified semantic/hybrid search across Agents, Skills, Traits, MCP Tool Servers, and Knowledge.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Natural-language context query." },
                "limit": { "type": "integer", "description": "Maximum number of results." }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::from_pool(self.pool.clone());
        let raw = server
            .call_tool(
                "search_agent_context",
                json!({ "query": args.query, "limit": args.limit }),
            )
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

#[derive(Deserialize, Default)]
pub struct ListAadCapabilitiesArgs {}

pub struct ListAadCapabilitiesTool {
    pub state: crate::state::AppState,
}

impl PortableTool for ListAadCapabilitiesTool {
    const NAME: &'static str = "list_aad_capabilities";
    type Error = std::io::Error;
    type Args = ListAadCapabilitiesArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Lists all Agent-As-Data platform MCP tools and their JSON input schemas available for invocation via invoke_aad_capability.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::new(self.state.clone());
        let tools = serde_json::to_value(server.list_tools()).map_err(std::io::Error::other)?;
        Ok(AadCapabilityToolOutput { result: tools })
    }
}

#[derive(Deserialize)]
pub struct InvokeAadCapabilityArgs {
    pub tool_name: String,
    #[serde(default)]
    pub arguments: serde_json::Value,
}

pub struct InvokeAadCapabilityTool {
    pub state: crate::state::AppState,
}

impl PortableTool for InvokeAadCapabilityTool {
    const NAME: &'static str = "invoke_aad_capability";
    type Error = std::io::Error;
    type Args = InvokeAadCapabilityArgs;
    type Output = AadCapabilityToolOutput;

    fn description(&self) -> String {
        "Invokes any Agent-As-Data platform MCP tool (agents, skills, traits, tools, knowledge, benches, threads, execution) directly in-process.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "tool_name": {
                    "type": "string",
                    "description": "Name of the AAD MCP tool to invoke (discover via list_aad_capabilities)."
                },
                "arguments": {
                    "type": "object",
                    "description": "JSON arguments matching the tool's inputSchema."
                }
            },
            "required": ["tool_name"]
        })
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let server = crate::mcp::server::AadMcpServer::new(self.state.clone());
        let arguments = if args.arguments.is_null() {
            json!({})
        } else {
            args.arguments
        };
        let raw = server
            .call_tool(&args.tool_name, arguments)
            .await
            .map_err(std::io::Error::other)?;
        Ok(parse_mcp_json_output(raw))
    }
}

pub async fn execute_workspace_tool(
    bench_id: Uuid,
    tool_name: &str,
    args_json: &serde_json::Value,
    pool: Option<&sqlx::PgPool>,
) -> Result<String, String> {
    match tool_name {
        ReadFileTool::NAME => {
            let args: ReadFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = ReadFileTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        WriteFileTool::NAME => {
            let args: WriteFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = WriteFileTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ReplaceInFileTool::NAME => {
            let args: ReplaceInFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = ReplaceInFileTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ListFilesTool::NAME => {
            let args: ListFilesArgs = serde_json::from_value(args_json.clone())
                .unwrap_or(ListFilesArgs { dir_path: None });
            let tool = ListFilesTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        DeleteFileTool::NAME => {
            let args: DeleteFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = DeleteFileTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        RenameFileTool::NAME => {
            let args: RenameFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = RenameFileTool { bench_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ReadBenchMemoryTool::NAME => {
            let pool =
                pool.ok_or_else(|| "Database pool required for read_bench_memory".to_string())?;
            let args: ReadBenchMemoryArgs =
                serde_json::from_value(args_json.clone()).unwrap_or(ReadBenchMemoryArgs {});
            let tool = ReadBenchMemoryTool {
                bench_id,
                pool: pool.clone(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        UpdateBenchMemoryTool::NAME => {
            let pool =
                pool.ok_or_else(|| "Database pool required for update_bench_memory".to_string())?;
            let args: UpdateBenchMemoryArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = UpdateBenchMemoryTool {
                bench_id,
                pool: pool.clone(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ListSkillsTool::NAME => {
            let pool = pool.ok_or_else(|| "Database pool required for list_skills".to_string())?;
            let args: ListSkillsArgs =
                serde_json::from_value(args_json.clone()).unwrap_or(ListSkillsArgs {});
            let tool = ListSkillsTool { pool: pool.clone() };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ViewSkillTool::NAME => {
            let pool = pool.ok_or_else(|| "Database pool required for view_skill".to_string())?;
            let args: ViewSkillArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = ViewSkillTool { pool: pool.clone() };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        crate::kb_tools::KbNodeBrowseTool::NAME => {
            let args: crate::kb_tools::KbNodeBrowseArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = crate::kb_tools::KbNodeBrowseTool {
                backend_url: "http://127.0.0.1:8080".to_string(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        crate::kb_tools::KbNodeReadTool::NAME => {
            let args: crate::kb_tools::KbNodeReadArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = crate::kb_tools::KbNodeReadTool {
                backend_url: "http://127.0.0.1:8080".to_string(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        crate::kb_tools::KbNodeAddTool::NAME => {
            let args: crate::kb_tools::KbNodeAddArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = crate::kb_tools::KbNodeAddTool {
                backend_url: "http://127.0.0.1:8080".to_string(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        crate::kb_tools::KbNodeEditTool::NAME => {
            let args: crate::kb_tools::KbNodeEditArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = crate::kb_tools::KbNodeEditTool {
                backend_url: "http://127.0.0.1:8080".to_string(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        crate::kb_tools::KbNodeDeleteTool::NAME => {
            let args: crate::kb_tools::KbNodeDeleteArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = crate::kb_tools::KbNodeDeleteTool {
                backend_url: "http://127.0.0.1:8080".to_string(),
            };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        _ => {
            if let Some(pool) = pool {
                // 1. Check in-process unified AAD MCP capabilities first
                let aad_mcp = crate::mcp::server::AadMcpServer::from_pool(pool.clone());
                if aad_mcp.list_tools().iter().any(|t| t.name == tool_name) {
                    return aad_mcp
                        .call_tool(tool_name, args_json.clone())
                        .await
                        .map_err(|e| e.to_string());
                }

                // 2. Fallback to registered remote MCP servers
                let row = sqlx::query(
                    r#"
                    SELECT endpoint_config
                    FROM tools
                    WHERE cached_capabilities->'tools' @> $1::jsonb
                    LIMIT 1
                    "#,
                )
                .bind(serde_json::json!([{"name": tool_name}]).to_string())
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    format!("Database error looking up MCP tool '{}': {}", tool_name, e)
                })?;

                if let Some(row) = row {
                    let endpoint_config: serde_json::Value =
                        sqlx::Row::get(&row, "endpoint_config");
                    if let Some(url) = endpoint_config.get("url").and_then(|u| u.as_str()) {
                        let res = crate::webserver::tools::execute_remote_mcp_tool(
                            url,
                            tool_name,
                            args_json.clone(),
                        )
                        .await?;
                        return serde_json::to_string(&res).map_err(|e| e.to_string());
                    }
                }
            }
            Err(format!("Unknown tool: {}", tool_name))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_portable_fs_tools() {
        let bench_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(bench_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        let write_tool = WriteFileTool { bench_id };
        let read_tool = ReadFileTool { bench_id };
        let replace_tool = ReplaceInFileTool { bench_id };
        let list_tool = ListFilesTool { bench_id };
        let rename_tool = RenameFileTool { bench_id };
        let delete_tool = DeleteFileTool { bench_id };

        // 1. Write file
        let write_res = write_tool
            .call(WriteFileArgs {
                filepath: "subdir/hello.txt".to_string(),
                content: "Hello World!".to_string(),
            })
            .await
            .expect("write failed");
        assert!(write_res.success);

        // 2. Read file
        let read_res = read_tool
            .call(ReadFileArgs {
                filepath: "subdir/hello.txt".to_string(),
            })
            .await
            .expect("read failed");
        assert_eq!(read_res.content, "Hello World!");

        // 3. Replace in file
        let replace_res = replace_tool
            .call(ReplaceInFileArgs {
                filepath: "subdir/hello.txt".to_string(),
                search_string: "World".to_string(),
                replace_string: "Rust".to_string(),
            })
            .await
            .expect("replace failed");
        assert!(replace_res.success);

        let read_res2 = read_tool
            .call(ReadFileArgs {
                filepath: "subdir/hello.txt".to_string(),
            })
            .await
            .expect("read failed");
        assert_eq!(read_res2.content, "Hello Rust!");

        // 4. List files
        let list_res = list_tool
            .call(ListFilesArgs {
                dir_path: Some("subdir".to_string()),
            })
            .await
            .expect("list failed");
        assert_eq!(list_res.files, vec!["hello.txt".to_string()]);

        // 5. Rename file
        let rename_res = rename_tool
            .call(RenameFileArgs {
                filepath: "subdir/hello.txt".to_string(),
                new_filepath: "subdir/renamed.txt".to_string(),
            })
            .await
            .expect("rename failed");
        assert!(rename_res.success);

        // 6. Delete file
        let delete_res = delete_tool
            .call(DeleteFileArgs {
                filepath: "subdir/renamed.txt".to_string(),
            })
            .await
            .expect("delete failed");
        assert!(delete_res.success);

        // Cleanup
        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[tokio::test]
    async fn test_portable_fs_tools_path_traversal() {
        let bench_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(bench_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        let write_tool = WriteFileTool { bench_id };
        let read_tool = ReadFileTool { bench_id };

        // Attempt path traversal via write
        let write_res = write_tool
            .call(WriteFileArgs {
                filepath: "../../etc/shadow".to_string(),
                content: "malicious".to_string(),
            })
            .await;
        assert!(write_res.is_err());

        // Attempt path traversal via read
        let read_res = read_tool
            .call(ReadFileArgs {
                filepath: "../../../etc/passwd".to_string(),
            })
            .await;
        assert!(read_res.is_err());

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[tokio::test]
    async fn test_execute_workspace_tool_dispatch() {
        let bench_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(bench_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        // 1. Dispatch write_file
        let write_args = json!({
            "filepath": "sample.txt",
            "content": "Hello via dispatcher!"
        });
        let write_out = execute_workspace_tool(bench_id, "write_file", &write_args, None).await;
        assert!(write_out.is_ok());

        // 2. Dispatch list_files
        let list_args = json!({});
        let list_out = execute_workspace_tool(bench_id, "list_files", &list_args, None).await;
        assert!(list_out.is_ok());
        assert!(list_out.unwrap().contains("sample.txt"));

        // 3. Dispatch read_file
        let read_args = json!({
            "filepath": "sample.txt"
        });
        let read_out = execute_workspace_tool(bench_id, "read_file", &read_args, None).await;
        assert!(read_out.is_ok());
        assert!(read_out.unwrap().contains("Hello via dispatcher!"));

        // 4. Dispatch unknown tool
        let unknown_out =
            execute_workspace_tool(bench_id, "nonexistent_tool", &json!({}), None).await;
        assert!(unknown_out.is_err());

        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[tokio::test]
    async fn test_list_skills_and_view_skill_tools() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:mysecretpassword@localhost:5432/aaddb".to_string()
        });

        let pool = match sqlx::postgres::PgPoolOptions::new().connect(&db_url).await {
            Ok(p) => p,
            Err(_) => {
                println!(
                    "Note: Database not available, skipping test_list_skills_and_view_skill_tools"
                );
                return;
            }
        };

        // Create a dummy skill
        let skill_id = Uuid::new_v4();
        let owner_id = Uuid::new_v4();
        let skill_name = format!("test-skill-{}", skill_id);

        let insert_res = sqlx::query(
            r#"
            INSERT INTO skills (id, name, description, definition, tags, owner_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(skill_id)
        .bind(&skill_name)
        .bind("A test skill")
        .bind("Test definition")
        .bind(vec!["test"])
        .bind(owner_id)
        .execute(&pool)
        .await;

        if insert_res.is_err() {
            println!("Note: Database not migrated, skipping test_list_skills_and_view_skill_tools");
            return;
        }

        // Test ListSkillsTool
        let list_tool = ListSkillsTool { pool: pool.clone() };
        let list_res = list_tool
            .call(ListSkillsArgs {})
            .await
            .expect("ListSkillsTool failed");

        let found = list_res
            .skills
            .iter()
            .any(|s| s.name == skill_name && s.description == "A test skill");
        assert!(found, "ListSkillsTool did not return the inserted skill");

        // Test ViewSkillTool - Found
        let view_tool = ViewSkillTool { pool: pool.clone() };
        let view_res = view_tool
            .call(ViewSkillArgs {
                skill_name: skill_name.clone(),
            })
            .await
            .expect("ViewSkillTool failed");

        assert!(view_res.error.is_none());
        assert!(view_res.skill.is_some());
        let skill = view_res.skill.unwrap();
        assert_eq!(skill.name, skill_name);
        assert_eq!(skill.description, "A test skill");

        // Test ViewSkillTool - Not Found
        let view_res_not_found = view_tool
            .call(ViewSkillArgs {
                skill_name: "nonexistent_skill_123".to_string(),
            })
            .await
            .expect("ViewSkillTool failed");
        assert!(view_res_not_found.skill.is_none());
        assert_eq!(view_res_not_found.error.unwrap(), "Skill not found");

        // Cleanup
        sqlx::query("DELETE FROM skills WHERE id = $1")
            .bind(skill_id)
            .execute(&pool)
            .await
            .expect("Failed to cleanup test skill");
    }

    #[tokio::test]
    async fn test_aad_capability_rig_tools_definitions_and_discovery() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://user:pass@localhost:5432/test")
            .unwrap();
        let handle = axum_prometheus::metrics_exporter_prometheus::PrometheusBuilder::new()
            .build_recorder()
            .handle();
        let config = crate::config::AppConfig {
            debugging: crate::config::DebuggingConfig {
                environment: "test".into(),
                log_level: "info".into(),
                fail_debug_delay: std::time::Duration::from_secs(0),
            },
            webservice: crate::config::WebServiceConfig {
                address: "127.0.0.1:8080".into(),
                api_prefix: "/api/aad/v1".into(),
            },
            llm: crate::config::LlmConfig {
                ollama_url: "http://localhost:11434".into(),
                model: "llama3".into(),
                timeout: std::time::Duration::from_secs(30),
                default_max_turns: 5,
            },
            mcp: crate::mcp::McpConfig::default(),
            runtime: crate::tokio_tools::ThreadRuntime::default(),
            database: crate::config::DatabaseConfig {
                url: crate::config::UrlWithUsernamePassword {
                    url: url::Url::parse("postgres://localhost:5432/test").unwrap(),
                    username: Some("user".into()),
                    password: Some("pass".into()),
                },
                max_connections: 1,
            },
            hams: ::hams::hams::config::HamsConfig::default(),
        };
        let state = crate::state::AppState {
            pool: pool.clone(),
            config,
            prometheus_handle: std::sync::Arc::new(handle),
            tokio_handle: tokio::runtime::Handle::current(),
        };

        assert_eq!(ListAgentsTool::NAME, "list_agents");
        assert_eq!(GetAgentTool::NAME, "get_agent");
        assert_eq!(SearchAgentsTool::NAME, "search_agents");
        assert_eq!(ListTraitsTool::NAME, "list_traits");
        assert_eq!(ListMcpServersTool::NAME, "list_mcp_servers");
        assert_eq!(SearchAgentContextTool::NAME, "search_agent_context");
        assert_eq!(ListAadCapabilitiesTool::NAME, "list_aad_capabilities");
        assert_eq!(InvokeAadCapabilityTool::NAME, "invoke_aad_capability");

        let list_caps = ListAadCapabilitiesTool {
            state: state.clone(),
        };
        let output = list_caps
            .call(ListAadCapabilitiesArgs {})
            .await
            .expect("list_aad_capabilities should succeed without DB query");
        let tools = output
            .result
            .as_array()
            .expect("capabilities should be a JSON array");
        assert!(tools.iter().any(|t| t["name"] == "list_agents"));
        assert!(tools.iter().any(|t| t["name"] == "list_skills"));
        assert!(tools.iter().any(|t| t["name"] == "list_traits"));
        assert!(tools.iter().any(|t| t["name"] == "search_knowledge"));
        assert!(tools.iter().any(|t| t["name"] == "execute_agent"));

        let invoke_caps = InvokeAadCapabilityTool { state };
        let err = invoke_caps
            .call(InvokeAadCapabilityArgs {
                tool_name: "unknown_tool_xyz".to_string(),
                arguments: json!({}),
            })
            .await;
        assert!(err.is_err());
    }
}
