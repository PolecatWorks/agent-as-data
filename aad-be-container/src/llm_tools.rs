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
    pub thread_id: Uuid,
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
        let workspace_root = get_workspace_root(self.thread_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Workspace not found"));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", args.filepath)));
        }

        if safe_path.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Path is a directory: {}", args.filepath)));
        }

        let content = std::fs::read_to_string(&safe_path)?;

        Ok(ReadFileOutput { content })
    }
}

pub struct WriteFileTool {
    pub thread_id: Uuid,
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
        let workspace_root = get_workspace_root(self.thread_id);

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
    pub thread_id: Uuid,
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
        let workspace_root = get_workspace_root(self.thread_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Workspace not found"));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File not found: {}", args.filepath)));
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
    pub thread_id: Uuid,
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
        let workspace_root = get_workspace_root(self.thread_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Workspace not found"));
        }

        let target_dir = match args.dir_path {
            Some(ref p) if !p.is_empty() => p.clone(),
            _ => "".to_string(), // Root of workspace
        };

        let safe_path = resolve_safe_path(&workspace_root, &target_dir)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Directory not found: {}", target_dir)));
        }

        if !safe_path.is_dir() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Path is not a directory: {}", target_dir)));
        }

        let mut files = Vec::new();
        let entries = std::fs::read_dir(safe_path)?;

        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(file_name) = entry.file_name().into_string() {
                    let mut suffix = "";
                    if let Ok(ft) = entry.file_type() {
                        if ft.is_dir() {
                            suffix = "/";
                        }
                    }
                    files.push(format!("{}{}", file_name, suffix));
                }
            }
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
    pub thread_id: Uuid,
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
        let workspace_root = get_workspace_root(self.thread_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Workspace not found"));
        }

        let safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, e))?;

        if !safe_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("File or directory not found: {}", args.filepath)));
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
    pub thread_id: Uuid,
}

impl PortableTool for RenameFileTool {
    const NAME: &'static str = "rename_file";
    type Error = std::io::Error;
    type Args = RenameFileArgs;
    type Output = RenameFileOutput;

    fn description(&self) -> String {
        "Renames and/or moves a file or directory in the workspace.".to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filepath": {
                    "type": "string",
                    "description": "The original path of the file or directory, relative to the workspace root."
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
        let workspace_root = get_workspace_root(self.thread_id);

        if !workspace_root.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Workspace not found"));
        }

        let old_safe_path = resolve_safe_path(&workspace_root, &args.filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, format!("Invalid original path: {}", e)))?;

        let new_safe_path = resolve_safe_path(&workspace_root, &args.new_filepath)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::PermissionDenied, format!("Invalid new path: {}", e)))?;

        if !old_safe_path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("Original file or directory not found: {}", args.filepath)));
        }

        if new_safe_path.exists() {
             return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("Target file or directory already exists: {}", args.new_filepath)));
        }

        if let Some(parent) = new_safe_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        std::fs::rename(&old_safe_path, &new_safe_path)?;

        Ok(RenameFileOutput {
            success: true,
            message: format!("Successfully renamed {} to {}", args.filepath, args.new_filepath),
        })
    }
}

pub async fn execute_workspace_tool(thread_id: Uuid, tool_name: &str, args_json: &serde_json::Value) -> Result<String, String> {
    match tool_name {
        ReadFileTool::NAME => {
            let args: ReadFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = ReadFileTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        WriteFileTool::NAME => {
            let args: WriteFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = WriteFileTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ReplaceInFileTool::NAME => {
            let args: ReplaceInFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = ReplaceInFileTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        ListFilesTool::NAME => {
            let args: ListFilesArgs = serde_json::from_value(args_json.clone())
                .unwrap_or(ListFilesArgs { dir_path: None });
            let tool = ListFilesTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        DeleteFileTool::NAME => {
            let args: DeleteFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = DeleteFileTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        RenameFileTool::NAME => {
            let args: RenameFileArgs = serde_json::from_value(args_json.clone())
                .map_err(|e| format!("Invalid arguments for {}: {}", tool_name, e))?;
            let tool = RenameFileTool { thread_id };
            let res = tool.call(args).await.map_err(|e| e.to_string())?;
            serde_json::to_string(&res).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown workspace tool: {}", tool_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_portable_fs_tools() {
        let thread_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(thread_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        let write_tool = WriteFileTool { thread_id };
        let read_tool = ReadFileTool { thread_id };
        let replace_tool = ReplaceInFileTool { thread_id };
        let list_tool = ListFilesTool { thread_id };
        let rename_tool = RenameFileTool { thread_id };
        let delete_tool = DeleteFileTool { thread_id };

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
        let thread_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(thread_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        let write_tool = WriteFileTool { thread_id };
        let read_tool = ReadFileTool { thread_id };

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
        let thread_id = Uuid::new_v4();
        let workspace_root = get_workspace_root(thread_id);
        let _ = std::fs::remove_dir_all(&workspace_root);

        // 1. Dispatch write_file
        let write_args = json!({
            "filepath": "sample.txt",
            "content": "Hello via dispatcher!"
        });
        let write_out = execute_workspace_tool(thread_id, "write_file", &write_args).await;
        assert!(write_out.is_ok());

        // 2. Dispatch list_files
        let list_args = json!({});
        let list_out = execute_workspace_tool(thread_id, "list_files", &list_args).await;
        assert!(list_out.is_ok());
        assert!(list_out.unwrap().contains("sample.txt"));

        // 3. Dispatch read_file
        let read_args = json!({
            "filepath": "sample.txt"
        });
        let read_out = execute_workspace_tool(thread_id, "read_file", &read_args).await;
        assert!(read_out.is_ok());
        assert!(read_out.unwrap().contains("Hello via dispatcher!"));

        // 4. Dispatch unknown tool
        let unknown_out = execute_workspace_tool(thread_id, "nonexistent_tool", &json!({})).await;
        assert!(unknown_out.is_err());

        let _ = std::fs::remove_dir_all(&workspace_root);
    }
}
