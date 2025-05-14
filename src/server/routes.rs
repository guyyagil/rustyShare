use axum::{
    http::{StatusCode,header},
    response::Html,
    routing::{get, MethodRouter},
    Router,
    Json,
    response::{IntoResponse, Response},
    extract::{Extension,Path},
    body::Body,
    extract::DefaultBodyLimit,
};

use std::path::PathBuf;
use axum_extra::extract::{Multipart,TypedHeader};
use headers::Range;
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use crate::media::files::FileEntry;
use std::sync::{Arc, Mutex};
use super::streaming::*;
use crate::media::files::*;

pub fn create_router(media_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
    Router::new()
       
        .route("/", static_handler("html/home.html"))
        .route("/media", static_handler("html/media.html"))
        .route("/api/media.json", get(media_json))
        .route("/api/media/{*path}", get(open)) 
        .route("/health", get(health_check))
        .route("/api/upload", axum::routing::post(upload_file))
        .fallback(static_handler("html/error.html"))
        .layer(Extension(media_tree))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024)) // 1 GB
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

    let file_path =safe_path.clone(); 
    let mime = get_mime_type(&file_path);
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "File not found").into_response(),
    };
    let file_size = file_size(&file).await;

     if let Some(TypedHeader(range)) = range {
        return build_range_response(file, file_size, &mime, range).await ;
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
    let upload_dir = PathBuf::from("media");

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

