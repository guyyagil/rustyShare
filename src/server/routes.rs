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
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use std::sync::{Arc, Mutex};
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
// Accepts a shared `media_tree` state for media file management.
pub fn create_router(media_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
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
        .layer(Extension(media_tree))
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
    Extension(media_tree): Extension<Arc<Mutex<Option<FileEntry>>>>,
) -> Json<Option<FileEntry>> {
    let tree = media_tree.lock().unwrap();
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

// --- New update route handler ---
async fn update_file(mut multipart: Multipart) -> impl IntoResponse {
    let mut replace_path: Option<String> = None;
    let mut file_bytes: Option<bytes::Bytes> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.name() {
            if name == "replace_path" {
                replace_path = Some(field.text().await.unwrap_or_default());
                continue;
            }
        }
        if field.file_name().is_some() {
            match field.bytes().await {
                Ok(data) => file_bytes = Some(data),
                Err(e) => {
                    eprintln!("Failed to read file bytes: {:?}", e);
                    return (StatusCode::BAD_REQUEST, "Failed to read file content").into_response();
                }
            }
        }
    }

    if let (Some(rp), Some(data)) = (replace_path, file_bytes) {
        let filepath = match safe_path(&rp) {
            Ok(p) => p,
            Err(resp) => return resp,
        };

        if filepath.exists() {
            if let Ok(metadata) = std::fs::metadata(&filepath) {
                if metadata.is_file() {
                    let _ = std::fs::remove_file(&filepath);
                }
            }
        }

        if let Err(e) = tokio::fs::write(&filepath, &data).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to update file: {e}")).into_response();
        }
        (StatusCode::OK, "File updated").into_response()
    } else {
        (StatusCode::BAD_REQUEST, "No file updated").into_response()
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