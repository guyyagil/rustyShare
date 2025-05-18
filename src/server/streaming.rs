use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    body::Body,
};
use headers::Range;
use tokio::{
    io::{AsyncSeekExt, SeekFrom,AsyncReadExt},
    fs::File,
};
use tokio_util::io::ReaderStream;
use mime_guess::mime;

/// Builds a streaming HTTP response for a requested byte range of a file.
/// Used for efficient media streaming and seeking support.
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

