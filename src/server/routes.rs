use axum::{
    http::{StatusCode,header},
    response::Html,
    routing::{get, MethodRouter},
    Router,
    Json,
    response::{IntoResponse, Response},
    extract::{Extension,Path},
    body::Body,
};
use axum_extra::extract::TypedHeader;
use headers::Range;
use tokio::{fs::File};
use tokio_util::io::ReaderStream;
use crate::media::files::FileEntry;
use std::sync::{Arc, Mutex};
use super::stream::*;


pub fn create_router(media_tree: Arc<Mutex<Option<FileEntry>>>) -> Router {
    Router::new()
       
        .route("/", static_handler("html/home.html"))
        .route("/media", static_handler("html/media.html"))
        .route("/api/media.json", get(media_json))
         .route("/api/media/{*path}", get(stream_media)) 
        .route("/health", get(health_check))
        .fallback(static_handler("html/error.html"))
        .layer(Extension(media_tree))
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


async fn stream_media(Path(path): Path<String>, range: Option<TypedHeader<Range>>) -> Response {
    // Call safe_path with the path argument and handle the Result
    let safe_path = match safe_path(&path) {
        Ok(p) => p,
        Err(resp) => return resp,
    };

    let file_path = if is_browser_supported(&safe_path) {
        safe_path.clone()
    } else {
        match transcode(&safe_path).await {
            Ok(p) => p,
            Err(resp) => return resp,
        }
    };
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


    

