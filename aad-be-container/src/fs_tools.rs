use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};
use uuid::Uuid;

// Note: These tool handlers could be implemented as Axum routes for an internal API
// or invoked directly. For now, they return JSON responses matching standard Axum handlers.

#[derive(Deserialize)]
pub struct WriteFileRequest {
    pub filepath: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct FileOperationResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct ReadFileResponse {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ListFilesRequest {
    pub dir_path: Option<String>,
}

#[derive(Serialize)]
pub struct ListFilesResponse {
    pub files: Vec<String>,
}

#[derive(Deserialize)]
pub struct DeleteFileRequest {
    pub filepath: String,
}

/// Helper function to safely resolve paths and prevent path traversal.
/// Takes the base workspace root and the user-provided relative path.
fn resolve_safe_path(workspace_root: &StdPath, relative_path: &str) -> Result<PathBuf, String> {
    // 1. Construct the intended path
    let mut intended_path = workspace_root.to_path_buf();

    // Strip leading slashes to ensure it's treated as relative to the workspace root
    let relative = relative_path.trim_start_matches('/');
    intended_path.push(relative);

    // 2. Canonicalize the path.
    // `canonicalize` requires the file/directory to exist. If it doesn't, we can't canonicalize it directly.
    // So we'll iterate through ancestors until we find one that exists, canonicalize that, and then append the rest.

    let mut existing_part = intended_path.as_path();
    let mut non_existing_parts = Vec::new();

    while !existing_part.exists() {
        if let Some(parent) = existing_part.parent() {
            if let Some(file_name) = existing_part.file_name() {
                non_existing_parts.push(file_name);
            }
            existing_part = parent;
        } else {
            return Err("Invalid path resolution".to_string());
        }
    }

    let canonical_existing = match existing_part.canonicalize() {
        Ok(path) => path,
        Err(_) => return Err("Failed to resolve path".to_string()),
    };

    // Reconstruct the full path
    let mut final_path = canonical_existing;
    for part in non_existing_parts.into_iter().rev() {
        final_path.push(part);
    }

    // 3. Verify boundary check
    let canonical_root = workspace_root.canonicalize().map_err(|_| "Failed to resolve workspace root".to_string())?;

    if final_path.starts_with(&canonical_root) {
        Ok(final_path)
    } else {
        Err("Path traversal detected: Access denied".to_string())
    }
}

fn get_workspace_root(thread_id: Uuid) -> PathBuf {
    PathBuf::from(format!("/tmp/workspace/{}", thread_id))
}


pub async fn write_file(
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<WriteFileRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), (StatusCode, String)> {
    let workspace_root = get_workspace_root(thread_id);

    // Ensure workspace exists before attempting to write inside it
    if !workspace_root.exists() {
         std::fs::create_dir_all(&workspace_root)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create workspace root: {}", e)))?;
    }

    let safe_path = resolve_safe_path(&workspace_root, &payload.filepath)
        .map_err(|e| (StatusCode::FORBIDDEN, e))?;

    if let Some(parent) = safe_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create directories: {}", e)))?;
        }
    }

    std::fs::write(&safe_path, payload.content)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write file: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(FileOperationResponse {
            message: format!("Successfully wrote to {}", payload.filepath),
        }),
    ))
}

pub async fn read_file(
    Path(thread_id): Path<Uuid>,
    Path(filepath): Path<String>,
) -> Result<(StatusCode, Json<ReadFileResponse>), (StatusCode, String)> {
    let workspace_root = get_workspace_root(thread_id);

    if !workspace_root.exists() {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }

    let safe_path = resolve_safe_path(&workspace_root, &filepath)
        .map_err(|e| (StatusCode::FORBIDDEN, e))?;

    if !safe_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File not found".to_string()));
    }

    if safe_path.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Path is a directory".to_string()));
    }

    let content = std::fs::read_to_string(safe_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read file: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(ReadFileResponse { content }),
    ))
}


pub async fn list_files(
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<ListFilesRequest>,
) -> Result<(StatusCode, Json<ListFilesResponse>), (StatusCode, String)> {
    let workspace_root = get_workspace_root(thread_id);

    if !workspace_root.exists() {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }

    let target_dir = match payload.dir_path {
        Some(ref p) if !p.is_empty() => p.clone(),
        _ => "".to_string(), // Root of workspace
    };

    let safe_path = resolve_safe_path(&workspace_root, &target_dir)
        .map_err(|e| (StatusCode::FORBIDDEN, e))?;

    if !safe_path.exists() {
        return Err((StatusCode::NOT_FOUND, "Directory not found".to_string()));
    }

    if !safe_path.is_dir() {
        return Err((StatusCode::BAD_REQUEST, "Path is not a directory".to_string()));
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(safe_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read directory: {}", e)))?;

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

    // Sort to ensure deterministic output
    files.sort();

    Ok((
        StatusCode::OK,
        Json(ListFilesResponse { files }),
    ))
}

pub async fn delete_file(
    Path(thread_id): Path<Uuid>,
    Json(payload): Json<DeleteFileRequest>,
) -> Result<(StatusCode, Json<FileOperationResponse>), (StatusCode, String)> {
    let workspace_root = get_workspace_root(thread_id);

    if !workspace_root.exists() {
        return Err((StatusCode::NOT_FOUND, "Workspace not found".to_string()));
    }

    let safe_path = resolve_safe_path(&workspace_root, &payload.filepath)
        .map_err(|e| (StatusCode::FORBIDDEN, e))?;

    if !safe_path.exists() {
        return Err((StatusCode::NOT_FOUND, "File or directory not found".to_string()));
    }

    if safe_path.is_dir() {
        std::fs::remove_dir_all(&safe_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete directory: {}", e)))?;
    } else {
        std::fs::remove_file(&safe_path)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete file: {}", e)))?;
    }

    Ok((
        StatusCode::OK,
        Json(FileOperationResponse {
            message: format!("Successfully deleted {}", payload.filepath),
        }),
    ))
}
