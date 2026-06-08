use utoipa::ToSchema;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

// ── Top-level response ─────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct DashboardResponse {
    /// true jika user belum menyelesaikan prequiz — data learning tidak tersedia.
    pub prequiz_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prequiz_message: Option<String>,
    pub user: UserSummaryDto,
    pub career_path: CareerPathSummaryDto,
    pub learning_progress: LearningProgressSummaryDto,
    pub final_project: FinalProjectSummaryDto,
    pub next_action: NextActionDto,
    pub recent_activities: Vec<ActivityDto>,
    pub certificates: Vec<CertificateBriefDto>,
}

// ── Section: certificates ──────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CertificateBriefDto {
    pub id: uuid::Uuid,
    pub certificate_code: String,
    pub role_name: String,
    pub issued_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub verification_url: String,
}

// ── Section: user ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct UserSummaryDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub member_since: DateTime<Utc>,
}

// ── Section: career_path ───────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct CareerPathSummaryDto {
    pub has_taken_quiz: bool,
    pub role: Option<RoleDto>,
    pub match_percentage: Option<f64>,
    pub quiz_taken_at: Option<DateTime<Utc>>,
    pub last_quiz_attempt_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct RoleDto {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

// ── Section: learning_progress ─────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct LearningProgressSummaryDto {
    pub overall_percentage: f64,
    pub total_modules: i64,
    pub completed_modules: i64,
    pub total_submaterials: i64,
    pub completed_submaterials: i64,
    pub current_module: Option<CurrentModuleDto>,
}

impl Default for LearningProgressSummaryDto {
    fn default() -> Self {
        Self {
            overall_percentage: 0.0,
            total_modules: 0,
            completed_modules: 0,
            total_submaterials: 0,
            completed_submaterials: 0,
            current_module: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CurrentModuleDto {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
    pub completion_percentage: f64,
    /// First unlocked submaterial that is not yet completed.
    pub current_submaterial: Option<CurrentSubmaterialDto>,
}

#[derive(Debug, Serialize, Clone, ToSchema)]
pub struct CurrentSubmaterialDto {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
}

// ── Section: final_project ─────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct FinalProjectSummaryDto {
    pub is_available: bool,
    pub project: Option<ProjectBriefDto>,
    /// "locked" | "available" | "pending_review" | "approved" | "rejected" | null
    pub access_status: Option<String>,
    pub latest_submission_at: Option<DateTime<Utc>>,
}

impl Default for FinalProjectSummaryDto {
    fn default() -> Self {
        Self { is_available: false, project: None, access_status: None, latest_submission_at: None }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectBriefDto {
    pub id: Uuid,
    pub title: String,
}

// ── Section: next_action ───────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct NextActionDto {
    /// TAKE_QUIZ | CONTINUE_LEARNING | TAKE_FINAL_QUIZ | SUBMIT_PROJECT | WAIT_REVIEW | ALL_DONE
    pub code: String,
    pub title: String,
    pub description: String,
    pub target_url: String,
}

// ── Section: recent_activities ─────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityDto {
    /// submaterial_completed | final_quiz_passed | final_quiz_failed | project_submitted
    pub kind: String,
    pub title: String,
    pub detail: String,
    pub occurred_at: DateTime<Utc>,
}

// ── Learning summary response (GET /dashboard/learning-summary) ────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct LearningSummaryResponse {
    pub role: RoleDto,
    pub overall_percentage: f64,
    pub modules: Vec<LearningSummaryModuleDto>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LearningSummaryModuleDto {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
    pub is_unlocked: bool,
    pub is_completed: bool,
    pub completion_percentage: f64,
    pub submaterials_total: i64,
    pub submaterials_completed: i64,
    pub final_quiz_passed: bool,
    pub final_quiz_unlocked: bool,
}

// ── Activity log response (GET /dashboard/activity) ───────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct ActivityLogResponse {
    pub activities: Vec<ActivityDto>,
    pub has_more: bool,
}

// ── Internal: data gathered before composing response ─────────────────────

/// Minimal info about the "current" module needed for next_action logic.
#[derive(Debug, Clone)]
pub struct CurrentModuleInfo {
    pub id: Uuid,
    pub title: String,
    pub order_index: i32,
    pub completion_percentage: f64,
    /// true = all subs done but final quiz not yet passed
    pub all_subs_done_quiz_pending: bool,
    pub current_submaterial: Option<CurrentSubmaterialDto>,
}

/// What we know about the final project for next_action logic.
#[derive(Debug, Clone)]
pub struct FinalProjectInfo {
    pub id: Uuid,
    pub title: String,
    /// "locked" | "available" | "pending_review" | "approved" | "rejected"
    pub access_status: String,
    pub latest_submission_at: Option<DateTime<Utc>>,
}
