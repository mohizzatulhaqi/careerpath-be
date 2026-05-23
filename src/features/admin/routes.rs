use crate::features::admin::{content, user::handler};
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // ── User management (Bagian 1) ─────────────────────────────────────
        .route("/users",              get(handler::list_users))
        .route("/users/:id",          get(handler::get_user).patch(handler::update_user))
        .route("/users/:id/deactivate", post(handler::deactivate_user))
        .route("/users/:id/activate",   post(handler::activate_user))
        .route("/users/:id/audit-logs", get(handler::get_user_audit_logs))
        .route("/audit-logs",           get(handler::get_audit_logs))

        // ── Content management (Bagian 2) ──────────────────────────────────
        .merge(content::routes::router())

        // ── Submission review (Bagian 3) ───────────────────────────────────
        .merge(crate::features::admin::submission::routes::router())
}
