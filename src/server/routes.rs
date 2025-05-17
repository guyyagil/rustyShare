use axum::{
    body::Body, 
    extract::{DefaultBodyLimit, Extension, Form, Path},
    http::{header, StatusCode}, 
    response::{Html, IntoResponse, Redirect, Response}, 
    routing::{get, MethodRouter}, 
    Json, 
    Router
};

use std::path::PathBuf;
use axum_extra::extract::{Multipart,TypedHeader};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use headers::Range;
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use std::sync::{Arc, Mutex};
use super::streaming::*;
use crate::fileManager::files::*;
use crate::utils::config::Config;


#[derive(serde::Deserialize)]
struct LoginForm {
    password: String,
}



pub fn create_router(media_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
    Router::new()
        .route("/", static_handler("html/home.html"))
        .route("/login", axum::routing::post(login))
        .route("/master", get(media_protected))
        .route("/api/master.json", get(media_json))
        .route("/api/master/{*path}", get(open))
        .route("/health", get(health_check))
        .route("/api/upload", axum::routing::post(upload_file))
        .fallback(static_handler("html/error.html"))
        .layer(CookieManagerLayer::new()) // <-- Add this line
        .layer(Extension(media_tree))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)) 
}

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

async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    let upload_dir = PathBuf::from("master");

    loop {
        let field_result = multipart.next_field().await;
        let field = match field_result {
            Ok(Some(f)) => f,
            Ok(None) => break,
            Err(e) => {
                eprintln!("Multipart field read error: {:?}", e);
                return (StatusCode::BAD_REQUEST, "Invalid multipart stream").into_response();
            }
        };

        if let Some(filename) = field.file_name().map(|s| s.to_string()) {
            let filepath = upload_dir.join(&filename);
            match field.bytes().await {
                Ok(data) => {
                    if let Err(e) = tokio::fs::write(&filepath, &data).await {
                        return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save file: {e}")).into_response();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read file bytes: {:?}", e);
                    return (StatusCode::BAD_REQUEST, "Failed to read file content").into_response();
                }
            }
        }
    }
    (StatusCode::OK, "File uploaded").into_response()
}
#[axum::debug_handler]
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

async fn media_protected(cookies: Cookies) -> impl IntoResponse {
    if cookies.get("auth").map(|c| c.value().to_owned()) == Some("1".to_string()) {
        let content = std::fs::read_to_string("html/master.html")
            .unwrap_or_else(|_| "html/error.html".to_string());
        Html(content).into_response()
    } else {
        Redirect::to("html/error.html").into_response()
    }
}