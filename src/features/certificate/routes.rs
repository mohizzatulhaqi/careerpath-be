use crate::{features::certificate::handler, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

/// Routes mounted under `/api/certificates`
pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/me", get(handler::list_my_certificates))
        .route("/me/:id", get(handler::get_my_certificate))
        .route("/me/:id/download.pdf", get(handler::download_certificate_pdf))
}

/// Routes mounted under `/api/admin/certificates`
pub fn admin_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler::admin_list_certificates))
        .route("/:id/revoke", post(handler::admin_revoke_certificate))
        .route("/:id/restore", post(handler::admin_restore_certificate))
}

/// Public verify route — mounted under `/api/verify`
pub fn verify_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/:code", get(handler::verify_certificate))
}
