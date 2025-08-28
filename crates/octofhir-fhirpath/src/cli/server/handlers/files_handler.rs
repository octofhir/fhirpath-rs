//! File management handlers for the FHIRPath server

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::{FileInfo, FileListResponse, FileUploadResponse},
};

use axum::{
    body::Bytes,
    extract::{Multipart, Path},
    response::Json,
};
use serde_json::Value as JsonValue;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};
use tracing::{info, warn};

/// List all files in the storage directory  
pub async fn list_files() -> ServerResult<Json<FileListResponse>> {
    let storage_dir = PathBuf::from("./storage");

    if !storage_dir.exists() {
        return Ok(Json(FileListResponse {
            files: Vec::new(),
            storage_path: storage_dir.display().to_string(),
        }));
    }

    let mut files = Vec::new();

    // Recursively scan storage directory for JSON files
    collect_json_files(&storage_dir, &storage_dir, &mut files).await;

    // Sort files by name
    files.sort_by(|a, b| a.name.cmp(&b.name));

    info!("📁 Listed {} files from storage directory", files.len());

    Ok(Json(FileListResponse {
        files,
        storage_path: storage_dir.display().to_string(),
    }))
}

/// Get a specific file from storage
pub async fn get_file(Path(filename): Path<String>) -> ServerResult<Json<JsonValue>> {
    let storage_dir = PathBuf::from("./storage");
    let file_path = storage_dir.join(&filename);

    // Security check: ensure the resolved path is still within storage directory
    if !file_path.starts_with(&storage_dir) {
        return Err(ServerError::BadRequest {
            message: "Invalid file path".to_string(),
        });
    }

    if !file_path.exists() {
        return Err(ServerError::FileNotFound { filename });
    }

    let content = fs::read_to_string(&file_path).await?;
    let json: JsonValue = serde_json::from_str(&content)?;

    info!("📄 Retrieved file: {}", filename);
    Ok(Json(json))
}

/// Delete a file from storage
pub async fn delete_file(Path(filename): Path<String>) -> ServerResult<Json<JsonValue>> {
    let storage_dir = PathBuf::from("./storage");
    let file_path = storage_dir.join(&filename);

    // Security check: ensure the resolved path is still within storage directory
    if !file_path.starts_with(&storage_dir) {
        return Err(ServerError::BadRequest {
            message: "Invalid file path".to_string(),
        });
    }

    if !file_path.exists() {
        return Err(ServerError::FileNotFound { filename });
    }

    fs::remove_file(&file_path).await?;

    info!("🗑️ Deleted file: {}", filename);
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("File '{}' deleted successfully", filename)
    })))
}

/// Upload a new file to storage
pub async fn upload_file(mut multipart: Multipart) -> ServerResult<Json<FileUploadResponse>> {
    let storage_dir = PathBuf::from("./storage");

    // Ensure storage directory exists
    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir).await?;
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ServerError::BadRequest {
            message: format!("Failed to read multipart field: {}", e),
        })?
    {
        let name = field.name().unwrap_or("unknown");

        if name == "file" {
            let filename = field.file_name().unwrap_or("upload.json").to_string();
            let data = field.bytes().await.map_err(|e| ServerError::BadRequest {
                message: format!("Failed to read file data: {}", e),
            })?;

            return handle_file_upload(filename, data, &storage_dir).await;
        }
    }

    Err(ServerError::BadRequest {
        message: "No file field found in multipart request".to_string(),
    })
}

/// Handle the actual file upload
async fn handle_file_upload(
    filename: String,
    data: Bytes,
    storage_dir: &PathBuf,
) -> ServerResult<Json<FileUploadResponse>> {
    // Sanitize filename
    let safe_filename = sanitize_filename(&filename);
    let file_path = storage_dir.join(&safe_filename);

    // Security check: ensure the resolved path is still within storage directory
    if !file_path.starts_with(storage_dir) {
        return Err(ServerError::BadRequest {
            message: "Invalid file path".to_string(),
        });
    }

    // Validate that it's valid JSON
    let content = String::from_utf8(data.to_vec()).map_err(|_| ServerError::BadRequest {
        message: "File content is not valid UTF-8".to_string(),
    })?;

    // Parse JSON to validate format
    serde_json::from_str::<JsonValue>(&content)?;

    // Write file to storage
    let mut file = fs::File::create(&file_path).await?;
    file.write_all(content.as_bytes()).await?;
    file.flush().await?;

    let file_size = data.len() as u64;
    info!("📤 Uploaded file: {} ({} bytes)", safe_filename, file_size);

    Ok(Json(FileUploadResponse {
        success: true,
        filename: safe_filename,
        size: file_size,
        error: None,
    }))
}

/// Collect JSON files from directory tree using iterative approach
async fn collect_json_files(
    start_dir: &PathBuf,
    storage_root: &PathBuf,
    files: &mut Vec<FileInfo>,
) {
    let mut dirs_to_scan = vec![start_dir.clone()];

    while let Some(current_dir) = dirs_to_scan.pop() {
        let mut dir_entries = match fs::read_dir(&current_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                // Skip directories we can't read (permissions, etc.)
                warn!("Cannot read directory {:?}: {}", current_dir, e);
                continue;
            }
        };

        while let Some(entry) = dir_entries.next_entry().await.unwrap_or(None) {
            let path = entry.path();

            if path.is_file() {
                // Only process JSON files
                if let Some(extension) = path.extension() {
                    if extension.to_string_lossy().to_lowercase() == "json" {
                        if let Some(file_info) = create_file_info(&path, storage_root).await {
                            files.push(file_info);
                        }
                    }
                }
            } else if path.is_dir() {
                // Add directory to scan queue
                dirs_to_scan.push(path);
            }
        }
    }
}

/// Create file info from path
async fn create_file_info(path: &PathBuf, storage_root: &PathBuf) -> Option<FileInfo> {
    let metadata = fs::metadata(path).await.ok()?;

    // Get relative path from storage root
    let filename = if let Ok(relative_path) = path.strip_prefix(storage_root) {
        relative_path.to_string_lossy().to_string()
    } else {
        path.file_name()?.to_string_lossy().to_string()
    };

    // Note: JSON filtering is done earlier in the scan process

    let modified = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;

    let modified_iso = chrono::DateTime::from_timestamp(modified.as_secs() as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Determine file type by attempting to parse as JSON
    let file_type = detect_file_type(path).await;

    Some(FileInfo {
        name: filename,
        size: metadata.len(),
        modified: modified_iso,
        file_type,
    })
}

/// Detect file type by examining content
async fn detect_file_type(path: &PathBuf) -> Option<String> {
    let content = fs::read_to_string(path).await.ok()?;
    let json: JsonValue = serde_json::from_str(&content).ok()?;

    // Try to detect FHIR resource type
    if let JsonValue::Object(obj) = &json {
        if let Some(JsonValue::String(resource_type)) = obj.get("resourceType") {
            return Some(format!("FHIR {}", resource_type));
        }
    }

    Some("JSON".to_string())
}

/// Sanitize filename to prevent directory traversal attacks
fn sanitize_filename(filename: &str) -> String {
    // Remove directory traversal attempts and normalize path separators
    let cleaned = filename
        .replace("../", "")
        .replace("..\\", "")
        .replace(['/', '\\'], "_");

    // Determine if we need to be more aggressive with sanitization
    let has_spaces_or_special = cleaned
        .chars()
        .any(|c| !c.is_alphanumeric() && c != '.' && c != '-' && c != '_');

    // Replace dangerous characters with underscores
    let sanitized = cleaned
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else if c == '.' && !has_spaces_or_special {
                // Keep dots only if there are no other special characters
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_start_matches('.') // Always remove leading dots
        .trim_start_matches('_')
        .to_string();

    // Return default name if result is empty, otherwise limit length
    if sanitized.is_empty() {
        "upload.json".to_string()
    } else if sanitized.len() > 255 {
        sanitized[..255].to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("patient.json"), "patient.json");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
        assert_eq!(
            sanitize_filename("file with spaces.json"),
            "file_with_spaces_json"
        );
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename(""), "upload.json");
    }

    #[test]
    fn test_filename_length_limit() {
        let long_name = "a".repeat(300) + ".json";
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 255);
    }
}
