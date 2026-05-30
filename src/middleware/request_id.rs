use axum::{body::Body, http::Request, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn layer(req: Request<Body>, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Record into the active TraceLayer span so every log line in this
    // request automatically carries the ID.
    tracing::Span::current().record("request_id", request_id.as_str());

    let mut res = next.run(req).await;
    if let Ok(val) = request_id.parse() {
        res.headers_mut().insert("x-request-id", val);
    }
    res
}
