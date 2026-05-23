use crate::{
    features::learning::{
        module::handler as module_handler,
        module_quiz::handler as mod_quiz_handler,
        progress::handler as progress_handler,
        submaterial::handler as sub_handler,
        submaterial_quiz::handler as sub_quiz_handler,
    },
    state::AppState,
};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // Module endpoints
        .route("/modules",     get(module_handler::list_modules))
        .route("/modules/:id", get(module_handler::get_module))
        // Final quiz endpoints
        .route("/modules/:id/quiz",         get(mod_quiz_handler::get_final_quiz))
        .route("/modules/:id/quiz/submit",  post(mod_quiz_handler::submit_final_quiz))
        .route("/modules/:id/quiz/history", get(mod_quiz_handler::get_final_quiz_history))
        // Submaterial endpoints
        .route("/submaterials/:id",               get(sub_handler::get_submaterial))
        .route("/submaterials/:id/complete",      post(sub_handler::complete_submaterial))
        // Mini quiz endpoints
        .route("/submaterials/:id/quiz",          get(sub_quiz_handler::get_mini_quiz))
        .route("/submaterials/:id/quiz/submit",   post(sub_quiz_handler::submit_mini_quiz))
        // Progress endpoint
        .route("/progress", get(progress_handler::get_progress))
}
