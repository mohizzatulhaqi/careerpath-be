use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use governor::{
    clock::{Clock, DefaultClock},
    state::keyed::DefaultKeyedStateStore,
    Quota, RateLimiter,
};
use serde_json::json;
use std::{net::IpAddr, num::NonZeroU32, sync::Arc};

pub type KeyedLimiter = Arc<RateLimiter<IpAddr, DefaultKeyedStateStore<IpAddr>, DefaultClock>>;

/// Create a limiter that allows `per_minute` requests per IP per minute.
pub fn new_limiter(per_minute: u32) -> KeyedLimiter {
    Arc::new(RateLimiter::keyed(Quota::per_minute(
        NonZeroU32::new(per_minute).expect("rate must be > 0"),
    )))
}

/// Extract client IP from X-Forwarded-For (Render proxy) or X-Real-IP.
fn client_ip(req: &Request) -> IpAddr {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse().ok())
        .or_else(|| {
            req.headers()
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse().ok())
        })
        .unwrap_or(IpAddr::from([127, 0, 0, 1]))
}

pub async fn check(limiter: KeyedLimiter, req: Request, next: Next) -> Response {
    let ip = client_ip(&req);
    match limiter.check_key(&ip) {
        Ok(_) => next.run(req).await,
        Err(e) => {
            let retry_after = e
                .wait_time_from(DefaultClock::default().now())
                .as_secs()
                .max(1);
            (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_after.to_string())],
                Json(json!({
                    "success": false,
                    "error": {
                        "code": "TOO_MANY_REQUESTS",
                        "message": "Terlalu banyak percobaan, coba lagi beberapa saat"
                    }
                })),
            )
                .into_response()
        }
    }
}
