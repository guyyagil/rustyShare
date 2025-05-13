use std::path::{Path,PathBuf};
use axum::{
    http::{StatusCode,header},
    response::{ IntoResponse, Response},
    body::Body,
    };

use headers::Range;
use crate::cache::cache_path_for;

use tokio::{
            io::{AsyncSeekExt, SeekFrom, AsyncReadExt},
            process::Command,
            fs::File,
        };
use tokio_util::io::ReaderStream;
use mime_guess::{mime,from_path};


pub fn safe_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, Response> {
    let media_dir = std::fs::canonicalize("media")
        .unwrap_or_else(|_| PathBuf::from("media"));

    let joined_path = media_dir.join(path.as_ref());

    match joined_path.canonicalize() {
        Ok(canonical) if canonical.starts_with(&media_dir) => Ok(canonical),
        _ => Err((StatusCode::FORBIDDEN, "Forbidden").into_response()),
    }
}


pub fn is_browser_supported<P:AsRef<Path>>(path:P) -> bool {
    let path = path.as_ref();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "mp4" | "webm" | "mp3" | "ogg" | "wav")
}


pub async fn transcode<P: AsRef<Path>>(path: P) -> Result<PathBuf, Response> {
    let path = path.as_ref();
    let cache_path = cache_path_for(path);

    if !cache_path.exists() {
        let input = path.to_string_lossy().to_string();
        let output = cache_path.to_string_lossy().to_string();

        let status = Command::new("ffmpeg")
            .args([
                "-i", &input,
                "-f", "mp4",
                "-c:v", "libx264",
                "-c:a", "aac",
                "-movflags", "frag_keyframe+empty_moov",
                "-preset", "veryfast",
                "-tune", "zerolatency",
                &output,
            ])
            .status()
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Transcoding failed").into_response())?;

        if !status.success() {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Transcoding failed").into_response());
        }
    }

    Ok(cache_path)
}

pub fn get_mime_type<P: AsRef<Path>>(path: P) -> mime::Mime {
    from_path(path.as_ref()).first_or_octet_stream()
}

pub async fn file_size(f :&File) -> u64 {
    match f.metadata().await {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    }
}


pub async fn build_range_response(mut file: File,file_size: u64,mime: &mime::Mime,range: Range,) -> Response {
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

        let stream = ReaderStream::with_capacity(file.take(chunk_size), 16 * 1024);

        Response::builder()
            .status(StatusCode::PARTIAL_CONTENT)
            .header(header::CONTENT_TYPE, mime.as_ref())
            .header(header::ACCEPT_RANGES, "bytes")
            .header(
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, file_size),
            )
            .body(Body::from_stream(stream))
            .unwrap()
    } else {
        (StatusCode::RANGE_NOT_SATISFIABLE, "Invalid range").into_response()
    }
}

