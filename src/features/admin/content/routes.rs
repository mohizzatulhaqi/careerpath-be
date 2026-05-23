use crate::{
    features::admin::content::{
        module::handler as module_h,
        module_quiz::handler as module_quiz_h,
        pre_quiz::handler as pre_quiz_h,
        project::handler as project_h,
        role::handler as role_h,
        submaterial::handler as sub_h,
        submaterial_quiz::handler as sub_quiz_h,
    },
    state::AppState,
};
use axum::{
    routing::{get, patch, post},
    Router,
};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        // ── Roles ──────────────────────────────────────────────────────────
        .route("/roles",                get(role_h::list_roles).post(role_h::create_role))
        .route("/roles/:id",            get(role_h::get_role).patch(role_h::update_role))
        .route("/roles/:id/deactivate", post(role_h::deactivate_role))
        .route("/roles/:id/restore",    post(role_h::restore_role))

        // ── Modules ────────────────────────────────────────────────────────
        .route("/modules",              get(module_h::list_modules).post(module_h::create_module))
        .route("/modules/:id",          get(module_h::get_module).patch(module_h::update_module).delete(module_h::delete_module))
        .route("/modules/:id/restore",  post(module_h::restore_module))

        // ── Final quiz per module ──────────────────────────────────────────
        .route("/modules/:module_id/final-quiz", get(module_quiz_h::list_questions))

        // ── Submaterials ───────────────────────────────────────────────────
        .route("/submaterials",             get(sub_h::list_submaterials).post(sub_h::create_submaterial))
        .route("/submaterials/:id",         get(sub_h::get_submaterial).patch(sub_h::update_submaterial).delete(sub_h::delete_submaterial))
        .route("/submaterials/:id/restore", post(sub_h::restore_submaterial))

        // ── Mini quiz per submaterial ──────────────────────────────────────
        .route("/submaterials/:submaterial_id/quiz", get(sub_quiz_h::list_questions))

        // ── Mini quiz questions CRUD ───────────────────────────────────────
        .route("/submaterial-quiz-questions",             post(sub_quiz_h::create_question))
        .route("/submaterial-quiz-questions/:id",         patch(sub_quiz_h::update_question).delete(sub_quiz_h::delete_question))
        .route("/submaterial-quiz-questions/:id/options", patch(sub_quiz_h::replace_options))

        // ── Final quiz questions CRUD ──────────────────────────────────────
        .route("/module-quiz-questions",             post(module_quiz_h::create_question))
        .route("/module-quiz-questions/:id",         patch(module_quiz_h::update_question).delete(module_quiz_h::delete_question))
        .route("/module-quiz-questions/:id/options", patch(module_quiz_h::replace_options))

        // ── Pre-quiz (role-determining) ────────────────────────────────────
        .route("/pre-quiz-questions",                  get(pre_quiz_h::list_questions).post(pre_quiz_h::create_question))
        .route("/pre-quiz-questions/:id",              get(pre_quiz_h::get_question).patch(pre_quiz_h::update_question))
        .route("/pre-quiz-questions/:id/options",      patch(pre_quiz_h::replace_options))
        .route("/pre-quiz-questions/:id/deactivate",   post(pre_quiz_h::deactivate_question))
        .route("/pre-quiz-questions/:id/restore",      post(pre_quiz_h::restore_question))

        // ── Projects ───────────────────────────────────────────────────────
        .route("/projects",              get(project_h::list_projects).post(project_h::create_project))
        .route("/projects/:id",          get(project_h::get_project).patch(project_h::update_project))
        .route("/projects/:id/unpublish",post(project_h::unpublish_project))
        .route("/projects/:id/restore",  post(project_h::restore_project))
}
