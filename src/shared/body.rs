use axum::body::Body;
use bytes::Bytes;
use futures::stream;
use std::convert::Infallible;

const CHUNK_SIZE: usize = 256 * 1024; // 256 KB per chunk

/// Convert a `Bytes` buffer into a chunked streaming `Body`.
/// Uses zero-copy `Bytes::slice()` — no extra allocation per chunk.
/// Allows hyper to release each chunk from memory once sent.
pub fn chunked_body(data: Bytes) -> Body {
    let len = data.len();
    Body::from_stream(stream::iter(
        (0..len)
            .step_by(CHUNK_SIZE)
            .map(move |start| Ok::<Bytes, Infallible>(data.slice(start..(start + CHUNK_SIZE).min(len)))),
    ))
}
