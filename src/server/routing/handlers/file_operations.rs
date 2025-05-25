use axum::{
    body::Body,
    extract::{Extension, Path},
    http::{header, StatusCode},
    response::{IntoResponse, Response, Json},
};
use axum_extra::extract::{Multipart, TypedHeader};
use headers::Range;
use tokio::{
    fs::File, 
    sync::Mutex,
    io::{AsyncSeekExt, SeekFrom, AsyncReadExt},
};
use tokio_util::io::ReaderStream;
use std::sync::Arc;
use serde::Deserialize;
use tracing::info;
use mime_guess::mime;

use crate::file_manager::{file_tree::*, file_utils::*};

#[derive(Deserialize)]
pub struct DeleteRequest {
    pub path: String,
}

#[derive(Deserialize)]
pub struct CreateFolderRequest {
    pub path: String,
}

/// Returns the current file tree as JSON.
pub async fn media_json(
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
) -> Json<Option<FileEntry>> {
    let tree = file_tree.lock().await;
    Json(tree.clone())
}

/// Opens a file for browser viewing or streaming (supports range requests).
pub async fn open(
    Path(path): Path<String>,
    range: Option<TypedHeader<Range>>,
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
) -> Response {
    let safe_path = match safe_path(&path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    // Find the file entry in the in-memory tree
    let entry_arc = {
        let mut tree_guard = file_tree.lock().await;
        let tree = tree_guard.as_mut().unwrap();
        if let Some(entry) = find_entry(tree, &path) {
            Some(entry.clone())
        } else {
            None
        }
    };

    if let Some(entry) = entry_arc {
        // Acquire the file lock before reading
        let _file_guard = entry.lock.lock().await;
        // Now open the file as before
        let file_path = safe_path.clone();
        let mime = get_mime_type(&file_path);
        let file = match File::open(&file_path).await {
            Ok(f) => f,
            Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
        };
        let file_size = file_size(&file).await;
        if let Some(TypedHeader(range)) = range {
            return build_range_response(file, file_size, &mime, range).await;
        } else {
            let stream = ReaderStream::new(file);
            Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .header(header::ACCEPT_RANGES, "bytes")
                .body(Body::from_stream(stream))
                .unwrap()
        }
    } else {
        return (StatusCode::NOT_FOUND, "File not found").into_response();
    }
}

/// Handles file uploads via multipart form data.
pub async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    let mut target_path: Option<String> = None;
    let mut file_data: Option<(String, bytes::Bytes)> = None;

    // Parse multipart fields for target path and file data
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "target_path" {
                target_path = Some(field.text().await.unwrap_or_default());
                info!("✅ Received target_path: {:?}", target_path);
                continue;
            }

            if name == "file" {
                if let Some(filename) = field.file_name().map(|s| s.to_string()) {
                    match field.bytes().await {
                        Ok(data) => {
                            file_data = Some((filename, data));
                        }
                        Err(e) => {
                            eprintln!("Failed to read file bytes: {:?}", e);
                            return (StatusCode::BAD_REQUEST, "Failed to read file content").into_response();
                        }
                    }
                }
            }
        }
    }

    // Make sure we got both parts
    let (filename, data) = match file_data {
        Some(pair) => pair,
        None => return (StatusCode::BAD_REQUEST, "Missing file").into_response(),
    };

    // Build the relative path for the uploaded file
    let rel_path = if let Some(ref dir) = target_path {
        if dir.is_empty() {
            filename.clone()
        } else {
            format!("{}/{}", dir.trim_matches('/'), filename)
        }
    } else {
        filename.clone()
    };

    info!("➡️ Final relative path: {}", rel_path);

    let filepath = match safe_path(&rel_path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    info!("🧩 Resolved filesystem path: {:?}", filepath);

    // Prevent overwriting existing files
    if filepath.exists() {
        return (StatusCode::CONFLICT, "File already exists").into_response();
    }

    // Ensure parent directory exists
    if let Some(parent) = filepath.parent() {
        if !parent.exists() {
            return (StatusCode::BAD_REQUEST, "Target folder does not exist").into_response();
        }
    }

    // Write the file to disk
    if let Err(e) = tokio::fs::write(&filepath, &data).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {e}")).into_response();
    }

    (StatusCode::OK, "File uploaded").into_response()
}

pub async fn delete_file(
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
    Json(payload): Json<DeleteRequest>,
) -> impl IntoResponse {
    // Find the file entry in the in-memory tree before deleting
    let entry_arc = {
        let mut tree_guard = file_tree.lock().await;
        let tree = tree_guard.as_mut().unwrap();
        // Clone the Arc to the entry if found
        if let Some(entry) = find_entry(tree, &payload.path) {
            Some(entry.clone())
        } else {
            None
        }
    };

    if let Some(entry) = entry_arc {
        // Lock the file entry to prevent concurrent access
        let _file_guard = entry.lock.lock().await;

        let safe_path = match safe_path(&payload.path) {
            Ok(p) => p,
            Err(resp) => return resp,
        };

        if !safe_path.exists() {
            return (StatusCode::NOT_FOUND, "File not found").into_response();
        }

        if let Err(e) = tokio::fs::remove_file(&safe_path).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to delete file: {e}")).into_response();
        }

        (StatusCode::OK, "File deleted").into_response()
    } else {
        (StatusCode::NOT_FOUND, "File not found in media tree").into_response()
    }
}

/// Handles file updates (replacement) via multipart form data.
/// Updates the in-memory file tree metadata after writing.
#[axum::debug_handler]
pub async fn update_file(
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut replace_path: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;
    let mut uploaded_ext: Option<String> = None;

    // Parse multipart fields for path and file
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("replace_path") => {
                replace_path = Some(field.text().await.unwrap_or_default());
            }
            _ if field.file_name().is_some() => {
                // Get the uploaded file's extension
                if let Some(fname) = field.file_name() {
                    uploaded_ext = std::path::Path::new(fname)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_lowercase());
                }
                match field.bytes().await {
                    Ok(data) => file_bytes = Some(data),
                    Err(e) => {
                        eprintln!("Failed to read file bytes: {:?}", e);
                        return (StatusCode::BAD_REQUEST, "Failed to read file").into_response();
                    }
                }
            }
            _ => {}
        }
    }

    // Ensure we have both the path and file data
    let (rp, data) = match (replace_path, file_bytes) {
        (Some(rp), Some(data)) => (rp, data),
        _ => return (StatusCode::BAD_REQUEST, "Missing file or path").into_response(),
    };

    // Check that the file extension matches the original
    let orig_ext = std::path::Path::new(&rp)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let uploaded_ext = uploaded_ext.unwrap_or_default();

    if orig_ext != uploaded_ext {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "text/plain")],
            format!(
                "Extension mismatch: original is .{}, uploaded is .{}",
                orig_ext, uploaded_ext
            ),
        ).into_response();
    }

    // Find the file entry in the in-memory tree before updating
    let entry_arc = {
        let mut tree_guard = file_tree.lock().await;
        let tree = tree_guard.as_mut().unwrap();
        // Clone the Arc to the entry if found
        if let Some(entry) = find_entry(tree, &rp) {
            Some(entry.clone())
        } else {
            None
        }
    };

    if let Some(entry) = entry_arc {
        // Lock the file entry to prevent concurrent updates
        let _file_guard = entry.lock.lock().await;

        // Resolve the safe path
        let filepath = match safe_path(&rp) {
            Ok(p) => p,
            Err(resp) => return resp, // This will return a proper error response
        };

        // Remove old file if it exists
        if filepath.exists() {
            if let Ok(meta) = std::fs::metadata(&filepath) {
                if meta.is_file() {
                    let _ = std::fs::remove_file(&filepath);
                }
            }
        }

        // Write the new file
        if let Err(e) = tokio::fs::write(&filepath, &data).await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to write file: {e}"),
            )
                .into_response();
        }

        // Update the in-memory FileEntry metadata
        {
            let mut tree_guard = file_tree.lock().await;
            if let Some(tree) = tree_guard.as_mut() {
                if let Some(mut entry) = find_entry(tree, &rp) {
                    entry.size = get_file_size(&filepath);
                    entry.modified = get_modified_time(&filepath);
                }
            }
        }

        return (StatusCode::OK, "File updated successfully").into_response();
    } else {
        return (StatusCode::NOT_FOUND, "File not found in media tree").into_response();
    }
}

pub async fn create_folder (
    Json(payload): Json<CreateFolderRequest>,
) -> impl IntoResponse {
    let safe_path = match safe_path(&payload.path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    if safe_path.exists() {
        return (StatusCode::CONFLICT, "Folder already exists").into_response();
    }

    if let Err(e) = tokio::fs::create_dir_all(&safe_path).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create folder: {e}")).into_response();
    }

    (StatusCode::OK, "Folder created").into_response()
}

pub async fn build_range_response(
    mut file: File,
    file_size: u64,
    mime: &mime::Mime,
    range: Range,
) -> Response {
    let mut ranges = range.satisfiable_ranges(file_size);


    if let Some((start_bound, end_bound)) = ranges.next() {
        

        let start = match start_bound {
            std::ops::Bound::Included(s) => s,
            std::ops::Bound::Excluded(s) => s + 1,
            std::ops::Bound::Unbounded => 0,
        };

        
        let end = match end_bound {
            std::ops::Bound::Included(e) => e,
            std::ops::Bound::Excluded(e) => e - 1,
            std::ops::Bound::Unbounded => file_size - 1,
        };

       
        if start > end || end >= file_size {
            return (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid range").into_response();
        }

        let chunk_size = end - start + 1;

     
        if file.seek(SeekFrom::Start(start)).await.is_err() {
            return (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid range").into_response();
        }

        // Stream the requested chunk with a buffer size of 128KB
        let stream = ReaderStream::with_capacity(file.take(chunk_size), 128 * 1024);

        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::ACCEPT_RANGES, "bytes")
            .header(
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, file_size),
            )
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(Body::from_stream(stream))
            .unwrap()
    } else {
        // No valid range found
        (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid range").into_response()
    }
}

