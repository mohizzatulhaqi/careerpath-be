use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(
        title = "Career Path API",
        version = "1.0.0",
        description = "API for the Career Path learning platform"
    ),
    paths(
        // Auth
        crate::features::auth::handler::register,
        crate::features::auth::handler::login,
        crate::features::auth::handler::me,
        crate::features::auth::handler::refresh,
        crate::features::auth::handler::logout,
        // Quiz
        crate::features::quiz::handler::get_questions,
        crate::features::quiz::handler::create_attempt,
        crate::features::quiz::handler::save_answer,
        crate::features::quiz::handler::submit_attempt,
        crate::features::quiz::handler::get_result,
        crate::features::quiz::handler::get_history,
        // Dashboard
        crate::features::dashboard::handler::get_dashboard,
        crate::features::dashboard::handler::get_learning_summary,
        crate::features::dashboard::handler::get_activity_log,
        // Project
        crate::features::project::handler::get_my_project,
        crate::features::project::handler::get_project,
        crate::features::project::handler::submit_project,
        crate::features::project::handler::get_submissions,
        crate::features::project::handler::download_submission,
        // Learning - Modules
        crate::features::learning::module::handler::list_modules,
        crate::features::learning::module::handler::get_module,
        // Learning - Progress
        crate::features::learning::progress::handler::get_progress,
        // Learning - Final Quiz
        crate::features::learning::module_quiz::handler::get_final_quiz,
        crate::features::learning::module_quiz::handler::submit_final_quiz,
        crate::features::learning::module_quiz::handler::get_final_quiz_history,
        // Learning - Submaterials
        crate::features::learning::submaterial::handler::get_submaterial,
        crate::features::learning::submaterial::handler::complete_submaterial,
        // Learning - Mini Quiz
        crate::features::learning::submaterial_quiz::handler::get_mini_quiz,
        crate::features::learning::submaterial_quiz::handler::submit_mini_quiz,
        // Admin - Users
        crate::features::admin::user::handler::list_users,
        crate::features::admin::user::handler::get_user,
        crate::features::admin::user::handler::update_user,
        crate::features::admin::user::handler::deactivate_user,
        crate::features::admin::user::handler::activate_user,
        crate::features::admin::user::handler::get_user_audit_logs,
        crate::features::admin::user::handler::get_audit_logs,
        // Admin - Submissions
        crate::features::admin::submission::handler::list_submissions,
        crate::features::admin::submission::handler::get_submission,
        crate::features::admin::submission::handler::download_submission,
        crate::features::admin::submission::handler::approve_submission,
        crate::features::admin::submission::handler::reject_submission,
        crate::features::admin::submission::handler::queue_stats,
        // Admin - Content - Roles
        crate::features::admin::content::role::handler::list_roles,
        crate::features::admin::content::role::handler::get_role,
        crate::features::admin::content::role::handler::create_role,
        crate::features::admin::content::role::handler::update_role,
        crate::features::admin::content::role::handler::deactivate_role,
        crate::features::admin::content::role::handler::restore_role,
        // Admin - Content - Modules
        crate::features::admin::content::module::handler::list_modules,
        crate::features::admin::content::module::handler::get_module,
        crate::features::admin::content::module::handler::create_module,
        crate::features::admin::content::module::handler::update_module,
        crate::features::admin::content::module::handler::delete_module,
        crate::features::admin::content::module::handler::restore_module,
        // Admin - Content - Submaterials
        crate::features::admin::content::submaterial::handler::list_submaterials,
        crate::features::admin::content::submaterial::handler::get_submaterial,
        crate::features::admin::content::submaterial::handler::create_submaterial,
        crate::features::admin::content::submaterial::handler::update_submaterial,
        crate::features::admin::content::submaterial::handler::delete_submaterial,
        crate::features::admin::content::submaterial::handler::restore_submaterial,
        // Admin - Content - Submaterial Quiz
        crate::features::admin::content::submaterial_quiz::handler::list_questions,
        crate::features::admin::content::submaterial_quiz::handler::create_question,
        crate::features::admin::content::submaterial_quiz::handler::update_question,
        crate::features::admin::content::submaterial_quiz::handler::replace_options,
        crate::features::admin::content::submaterial_quiz::handler::delete_question,
        // Admin - Content - Module Quiz
        crate::features::admin::content::module_quiz::handler::list_questions,
        crate::features::admin::content::module_quiz::handler::create_question,
        crate::features::admin::content::module_quiz::handler::update_question,
        crate::features::admin::content::module_quiz::handler::replace_options,
        crate::features::admin::content::module_quiz::handler::delete_question,
        // Admin - Content - Pre Quiz
        crate::features::admin::content::pre_quiz::handler::list_questions,
        crate::features::admin::content::pre_quiz::handler::get_question,
        crate::features::admin::content::pre_quiz::handler::create_question,
        crate::features::admin::content::pre_quiz::handler::update_question,
        crate::features::admin::content::pre_quiz::handler::replace_options,
        crate::features::admin::content::pre_quiz::handler::deactivate_question,
        crate::features::admin::content::pre_quiz::handler::restore_question,
        // Certificates
        crate::features::certificate::handler::list_my_certificates,
        crate::features::certificate::handler::get_my_certificate,
        crate::features::certificate::handler::download_certificate_pdf,
        crate::features::certificate::handler::verify_certificate,
        // Admin - Certificates
        crate::features::certificate::handler::admin_list_certificates,
        crate::features::certificate::handler::admin_revoke_certificate,
        crate::features::certificate::handler::admin_restore_certificate,
        // Admin - Content - Projects
        crate::features::admin::content::project::handler::list_projects,
        crate::features::admin::content::project::handler::get_project,
        crate::features::admin::content::project::handler::create_project,
        crate::features::admin::content::project::handler::update_project,
        crate::features::admin::content::project::handler::unpublish_project,
        crate::features::admin::content::project::handler::restore_project,
    ),
    components(schemas(
        // Auth
        crate::features::auth::dto::RegisterRequest,
        crate::features::auth::dto::LoginRequest,
        crate::features::auth::dto::RefreshRequest,
        crate::features::auth::dto::LogoutRequest,
        crate::features::auth::dto::UserResponse,
        crate::features::auth::dto::AuthResponse,
        crate::features::auth::dto::TokenResponse,
        // Quiz
        crate::features::quiz::dto::SubmitAnswerRequest,
        crate::features::quiz::dto::OptionDto,
        crate::features::quiz::dto::QuestionDto,
        crate::features::quiz::dto::AttemptCreatedResponse,
        crate::features::quiz::dto::AnswerSavedResponse,
        crate::features::quiz::dto::RoleDto,
        crate::features::quiz::dto::TopReason,
        crate::features::quiz::dto::RoleScoreDto,
        crate::features::quiz::dto::QuizResultResponse,
        crate::features::quiz::dto::HistoryItem,
        // Dashboard
        crate::features::dashboard::dto::DashboardResponse,
        crate::features::dashboard::dto::UserSummaryDto,
        crate::features::dashboard::dto::CareerPathSummaryDto,
        crate::features::dashboard::dto::RoleDto,
        crate::features::dashboard::dto::LearningProgressSummaryDto,
        crate::features::dashboard::dto::CurrentModuleDto,
        crate::features::dashboard::dto::CurrentSubmaterialDto,
        crate::features::dashboard::dto::FinalProjectSummaryDto,
        crate::features::dashboard::dto::ProjectBriefDto,
        crate::features::dashboard::dto::NextActionDto,
        crate::features::dashboard::dto::ActivityDto,
        crate::features::dashboard::dto::LearningSummaryResponse,
        crate::features::dashboard::dto::LearningSummaryModuleDto,
        crate::features::dashboard::dto::ActivityLogResponse,
        // Project
        crate::features::project::dto::RoleDto,
        crate::features::project::dto::ModuleProgressDto,
        crate::features::project::dto::LatestSubmissionDto,
        crate::features::project::dto::ProjectDetailDto,
        crate::features::project::dto::MyProjectResponse,
        crate::features::project::dto::SubmitResponse,
        crate::features::project::dto::SubmissionHistoryItem,
        // Learning - Module
        crate::features::learning::module::dto::RoleInfo,
        crate::features::learning::module::dto::ModuleListItem,
        crate::features::learning::module::dto::ModuleListResponse,
        crate::features::learning::module::dto::SubmaterialInModule,
        crate::features::learning::module::dto::ModuleDetailResponse,
        // Learning - Module Quiz
        crate::features::learning::module_quiz::dto::FinalQuizAnswerInput,
        crate::features::learning::module_quiz::dto::SubmitFinalQuizRequest,
        crate::features::learning::module_quiz::dto::FinalQuizOptionResponse,
        crate::features::learning::module_quiz::dto::FinalQuizQuestionResponse,
        crate::features::learning::module_quiz::dto::FinalQuizResponse,
        crate::features::learning::module_quiz::dto::FinalQuizResultOption,
        crate::features::learning::module_quiz::dto::FinalQuizResultQuestion,
        crate::features::learning::module_quiz::dto::FinalQuizSubmitResponse,
        crate::features::learning::module_quiz::dto::FinalQuizAttemptItem,
        crate::features::learning::module_quiz::dto::FinalQuizHistoryResponse,
        // Learning - Progress
        crate::features::learning::progress::dto::CurrentModuleInfo,
        crate::features::learning::progress::dto::ProgressModuleItem,
        crate::features::learning::progress::dto::OverallProgressResponse,
        // Learning - Submaterial
        crate::features::learning::submaterial::dto::SubmaterialModuleInfo,
        crate::features::learning::submaterial::dto::SubmaterialDetailResponse,
        crate::features::learning::submaterial::dto::CompleteModuleStatus,
        crate::features::learning::submaterial::dto::NextModuleUnlocked,
        crate::features::learning::submaterial::dto::CompleteSubmaterialResponse,
        // Learning - Submaterial Quiz
        crate::features::learning::submaterial_quiz::dto::MiniQuizAnswerInput,
        crate::features::learning::submaterial_quiz::dto::SubmitMiniQuizRequest,
        crate::features::learning::submaterial_quiz::dto::MiniQuizOptionResponse,
        crate::features::learning::submaterial_quiz::dto::MiniQuizQuestionResponse,
        crate::features::learning::submaterial_quiz::dto::MiniQuizResponse,
        crate::features::learning::submaterial_quiz::dto::MiniQuizResultOption,
        crate::features::learning::submaterial_quiz::dto::MiniQuizResultQuestion,
        crate::features::learning::submaterial_quiz::dto::MiniQuizSubmitResponse,
        // Admin - Audit
        crate::features::admin::audit::dto::AuditAdminDto,
        crate::features::admin::audit::dto::AuditLogDto,
        crate::features::admin::audit::dto::PaginationDto,
        crate::features::admin::audit::dto::AuditLogPageDto,
        crate::features::admin::audit::dto::AuditLogFilter,
        // Admin - User
        crate::features::admin::user::dto::UserStatsDto,
        crate::features::admin::user::dto::UserSummaryDto,
        crate::features::admin::user::dto::ModuleProgressDto,
        crate::features::admin::user::dto::QuizAttemptDto,
        crate::features::admin::user::dto::ProjectSubmissionDto,
        crate::features::admin::user::dto::UserDetailDto,
        crate::features::admin::user::dto::UserListResponse,
        crate::features::admin::user::dto::UserListQuery,
        crate::features::admin::user::dto::UpdateUserRequest,
        crate::features::admin::user::dto::DeactivateRequest,
        crate::features::admin::user::dto::ActivateRequest,
        // Admin - Submission
        crate::features::admin::submission::dto::UserBriefDto,
        crate::features::admin::submission::dto::RoleBriefDto,
        crate::features::admin::submission::dto::ProjectBriefDto,
        crate::features::admin::submission::dto::FileBriefDto,
        crate::features::admin::submission::dto::ReviewerBriefDto,
        crate::features::admin::submission::dto::SubmissionSummaryDto,
        crate::features::admin::submission::dto::QueueSummaryDto,
        crate::features::admin::submission::dto::SubmissionListResponse,
        crate::features::admin::submission::dto::SubmissionFilter,
        crate::features::admin::submission::dto::UserDetailForReviewDto,
        crate::features::admin::submission::dto::ProjectDetailForReviewDto,
        crate::features::admin::submission::dto::CurrentReviewDto,
        crate::features::admin::submission::dto::ReviewHistoryEntryDto,
        crate::features::admin::submission::dto::SubmissionHistoryItemDto,
        crate::features::admin::submission::dto::SubmissionDetailDto,
        crate::features::admin::submission::dto::ReviewResultDto,
        crate::features::admin::submission::dto::RolePendingCountDto,
        crate::features::admin::submission::dto::QueueStatsDto,
        crate::features::admin::submission::dto::ReviewRequest,
        // Admin - Content - Role
        crate::features::admin::content::role::dto::RoleDto,
        crate::features::admin::content::role::dto::RoleDetailDto,
        crate::features::admin::content::role::dto::RoleStatsDto,
        crate::features::admin::content::role::dto::RoleListResponse,
        crate::features::admin::content::role::dto::RoleListQuery,
        crate::features::admin::content::role::dto::CreateRoleRequest,
        crate::features::admin::content::role::dto::UpdateRoleRequest,
        // Admin - Content - Module
        crate::features::admin::content::module::dto::AdminModuleDto,
        crate::features::admin::content::module::dto::AdminModuleStatsDto,
        crate::features::admin::content::module::dto::AdminModuleListResponse,
        crate::features::admin::content::module::dto::AdminModuleFilter,
        crate::features::admin::content::module::dto::CreateModuleRequest,
        crate::features::admin::content::module::dto::UpdateModuleRequest,
        // Admin - Content - Module Quiz
        crate::features::admin::content::module_quiz::dto::AdminModuleQuizOptionDto,
        crate::features::admin::content::module_quiz::dto::ModuleQuizOptionInput,
        crate::features::admin::content::module_quiz::dto::AdminModuleQuizQuestionDto,
        crate::features::admin::content::module_quiz::dto::CreateModuleQuizQuestionRequest,
        crate::features::admin::content::module_quiz::dto::UpdateModuleQuizQuestionRequest,
        crate::features::admin::content::module_quiz::dto::ReplaceModuleQuizOptionsRequest,
        // Admin - Content - Pre Quiz
        crate::features::admin::content::pre_quiz::dto::WeightDto,
        crate::features::admin::content::pre_quiz::dto::PreQuizOptionDto,
        crate::features::admin::content::pre_quiz::dto::PreQuizQuestionDto,
        crate::features::admin::content::pre_quiz::dto::PreQuizListResponse,
        crate::features::admin::content::pre_quiz::dto::PreQuizFilter,
        crate::features::admin::content::pre_quiz::dto::WeightInput,
        crate::features::admin::content::pre_quiz::dto::PreQuizOptionInput,
        crate::features::admin::content::pre_quiz::dto::CreatePreQuizQuestionRequest,
        crate::features::admin::content::pre_quiz::dto::UpdatePreQuizQuestionRequest,
        crate::features::admin::content::pre_quiz::dto::ReplacePreQuizOptionsRequest,
        // Admin - Content - Project
        crate::features::admin::content::project::dto::AdminProjectDto,
        crate::features::admin::content::project::dto::AdminProjectListResponse,
        crate::features::admin::content::project::dto::AdminProjectFilter,
        crate::features::admin::content::project::dto::CreateProjectRequest,
        crate::features::admin::content::project::dto::UpdateProjectRequest,
        // Admin - Content - Submaterial
        crate::features::admin::content::submaterial::dto::AdminSubmaterialDto,
        crate::features::admin::content::submaterial::dto::AdminSubmaterialListResponse,
        crate::features::admin::content::submaterial::dto::AdminSubmaterialFilter,
        crate::features::admin::content::submaterial::dto::CreateSubmaterialRequest,
        crate::features::admin::content::submaterial::dto::UpdateSubmaterialRequest,
        // Certificate
        crate::features::certificate::dto::CertificateSummaryDto,
        crate::features::certificate::dto::CertificateListResponse,
        crate::features::certificate::dto::CompletedModuleDto,
        crate::features::certificate::dto::FinalProjectAchievementDto,
        crate::features::certificate::dto::AchievementDto,
        crate::features::certificate::dto::CertificateDetailDto,
        crate::features::certificate::dto::PublicCertificateDto,
        crate::features::certificate::dto::VerificationResponse,
        crate::features::certificate::dto::AdminCertificateSummaryDto,
        crate::features::certificate::dto::AdminCertificateListResponse,
        crate::features::certificate::dto::AdminCertificateFilter,
        crate::features::certificate::dto::RevokeRequest,
        // Dashboard certificate brief
        crate::features::dashboard::dto::CertificateBriefDto,
        // Admin - Content - Submaterial Quiz
        crate::features::admin::content::submaterial_quiz::dto::AdminQuizOptionDto,
        crate::features::admin::content::submaterial_quiz::dto::QuizOptionInput,
        crate::features::admin::content::submaterial_quiz::dto::AdminSubQuizQuestionDto,
        crate::features::admin::content::submaterial_quiz::dto::CreateSubQuizQuestionRequest,
        crate::features::admin::content::submaterial_quiz::dto::UpdateSubQuizQuestionRequest,
        crate::features::admin::content::submaterial_quiz::dto::ReplaceOptionsRequest,
    )),
    tags(
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Quiz", description = "Career path quiz endpoints"),
        (name = "Learning - Modules", description = "Learning module endpoints"),
        (name = "Learning - Final Quiz", description = "Module final quiz endpoints"),
        (name = "Learning - Submaterials", description = "Learning submaterial endpoints"),
        (name = "Learning - Mini Quiz", description = "Submaterial mini quiz endpoints"),
        (name = "Project", description = "Project submission endpoints"),
        (name = "Dashboard", description = "Dashboard & activity endpoints"),
        (name = "Admin - Users", description = "Admin user management"),
        (name = "Admin - Audit", description = "Admin audit log management"),
        (name = "Admin - Content", description = "Admin content management"),
        (name = "Admin - Submissions", description = "Admin submission review"),
        (name = "Certificates", description = "Certificate endpoints"),
        (name = "Admin - Certificates", description = "Admin certificate management"),
    )
)]
pub struct ApiDoc;

pub fn build() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
