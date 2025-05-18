use axum::{
    body::Body, 
    extract::{DefaultBodyLimit, Extension, Form, Path},
    http::{header, StatusCode}, 
    response::{Html, IntoResponse, Redirect, Response}, 
    routing::{get, MethodRouter}, 
    Json, 
    Router
};
use axum_extra::extract::{Multipart,TypedHeader};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use headers::Range;
use tokio::{fs::File,sync::{Mutex, MutexGuard}};
use tokio_util::io::ReaderStream;
use std::sync::Arc;
use super::streaming::*;
use crate::file_manager::files::*;
use crate::utils::config::Config;
use tracing::info;
use tower_http::services::ServeDir;

#[derive(serde::Deserialize)]
struct LoginForm {
    password: String,
}
// Creates and configures the application router with all routes.
// Accepts a shared `file_tree` state for media file management.
pub fn create_router(file_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
    Router::new()
        .nest_service("/static", ServeDir::new("static"))
        .route("/", static_handler("static/html/home.html"))
        .route("/login", axum::routing::post(login))
        .route("/master", get(media_protected))
        .route("/api/master.json", get(media_json))
        .route("/api/master/{*path}", get(open))
        .route("/health", get(health_check))
        .route("/api/upload", axum::routing::post(upload_file))
        .route("/api/update", axum::routing::post(update_file))
        .fallback(static_handler("static/html/error.html"))
        .layer(CookieManagerLayer::new()) // <-- Add this line
        .layer(Extension(file_tree))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)) 
}

// Generic handler for serving static HTML content.
// Used for routes that do not require dynamic logic
fn static_handler(path: &'static str) -> MethodRouter {
    get(move || async move  {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("⚠️ Critical file not found: {}", path));
        Html(content)
    })
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn media_json(
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
) -> Json<Option<FileEntry>> {
    let tree = file_tree.lock().await;
    Json(tree.clone())
}

// open supported browser files in web view 
async fn open(Path(path): Path<String>, range: Option<TypedHeader<Range>>) -> Response {
    let safe_path = match safe_path(&path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let file_path = safe_path.clone(); 
    let mime = get_mime_type(&file_path);
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };
    
    let file_size = file_size(&file).await;

     if let Some(TypedHeader(range)) = range {
        return build_range_response(file, file_size, &mime, range).await;
    }  else {
        // No range header, stream the whole file
        let stream = ReaderStream::new(file);
        Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::ACCEPT_RANGES, "bytes")
            .body(Body::from_stream(stream))
            .unwrap()
    }
}

pub async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    let mut target_path: Option<String> = None;
    let mut file_data: Option<(String, bytes::Bytes)> = None;

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

    if filepath.exists() {
        return (StatusCode::CONFLICT, "File already exists").into_response();
    }

    if let Some(parent) = filepath.parent() {
        if !parent.exists() {
            return (StatusCode::BAD_REQUEST, "Target folder does not exist").into_response();
        }
    }

    if let Err(e) = tokio::fs::write(&filepath, &data).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {e}")).into_response();
    }

    (StatusCode::OK, "File uploaded").into_response()
}

#[axum::debug_handler]
pub async fn update_file(
    Extension(file_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut replace_path: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;
    let mut uploaded_ext: Option<String> = None;

    // Parse multipart fields
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

    // Ensure we have path and file
    let (rp, data) = match (replace_path, file_bytes) {
        (Some(rp), Some(data)) => (rp, data),
        _ => return (StatusCode::BAD_REQUEST, "Missing file or path").into_response(),
    };

    // Check extension match
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

    // Extract the entry path before any await
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

        return (StatusCode::OK, "File updated successfully").into_response();
    } else {
        return (StatusCode::NOT_FOUND, "File not found in media tree").into_response();
    }
}

//password protected login route and create authentication cookie
async fn login(cookies: Cookies, Form(form): Form<LoginForm>) -> impl IntoResponse {
    if form.password == Config::from_env().password() {
        let mut cookie = Cookie::new("auth", "1");
        cookie.set_path("/");
        cookie.set_max_age(cookie::time::Duration::hours(12)); // 1 day
        cookies.add(cookie);
        Redirect::to("/master").into_response()
    } else {
        (StatusCode::UNAUTHORIZED, "Wrong access code").into_response()
    }
}


//check if the user is authenticated using the cookie created by the login route
async fn media_protected(cookies: Cookies) -> impl IntoResponse {
    if cookies.get("auth").map(|c| c.value().to_owned()) == Some("1".to_string()) {
        let content = std::fs::read_to_string("static/html/master.html")
            .or_else(|_| std::fs::read_to_string("static/html/error.html"))
            .unwrap_or_else(|_| "<h1>Page not found</h1>".to_string());
        Html(content).into_response()
    } else {
        Redirect::to("static/html/error.html").into_response()
    }
}