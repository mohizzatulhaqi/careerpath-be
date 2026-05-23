/// Project access gating logic.
///
/// A project is unlocked only when ALL modules for the user's role are completed.
/// After that, the access status depends on the latest submission status.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectAccessStatus {
    /// Some modules are not yet completed.
    Locked,
    /// All modules completed, user can submit (never submitted or last rejected).
    Available,
    /// Latest submission is pending admin review.
    PendingReview,
    /// Latest submission was approved.
    Approved,
    /// Latest submission was rejected — user may resubmit.
    Rejected,
}

impl ProjectAccessStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Locked => "locked",
            Self::Available => "available",
            Self::PendingReview => "pending_review",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

pub struct ModuleCompletionSummary {
    pub total: i64,
    pub completed: i64,
}

/// Compute access status from module completion + latest submission.
pub fn compute_project_access(
    summary: &ModuleCompletionSummary,
    latest_submission_status: Option<&str>,
) -> ProjectAccessStatus {
    // Must have > 0 modules AND all completed
    if summary.total == 0 || summary.completed < summary.total {
        return ProjectAccessStatus::Locked;
    }
    match latest_submission_status {
        None => ProjectAccessStatus::Available,
        Some("pending_review") => ProjectAccessStatus::PendingReview,
        Some("approved") => ProjectAccessStatus::Approved,
        Some("rejected") => ProjectAccessStatus::Rejected,
        _ => ProjectAccessStatus::Available, // defensive fallback
    }
}

/// Whether the user is allowed to submit (or resubmit) right now.
pub fn can_submit(status: &ProjectAccessStatus) -> bool {
    matches!(status, ProjectAccessStatus::Available | ProjectAccessStatus::Rejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locked_when_not_all_done() {
        let s = ModuleCompletionSummary { total: 5, completed: 4 };
        assert_eq!(compute_project_access(&s, None), ProjectAccessStatus::Locked);
    }

    #[test]
    fn available_when_all_done_no_submission() {
        let s = ModuleCompletionSummary { total: 5, completed: 5 };
        assert_eq!(compute_project_access(&s, None), ProjectAccessStatus::Available);
    }

    #[test]
    fn pending_review() {
        let s = ModuleCompletionSummary { total: 5, completed: 5 };
        assert_eq!(
            compute_project_access(&s, Some("pending_review")),
            ProjectAccessStatus::PendingReview
        );
    }

    #[test]
    fn approved() {
        let s = ModuleCompletionSummary { total: 5, completed: 5 };
        assert_eq!(
            compute_project_access(&s, Some("approved")),
            ProjectAccessStatus::Approved
        );
    }

    #[test]
    fn rejected() {
        let s = ModuleCompletionSummary { total: 5, completed: 5 };
        assert_eq!(
            compute_project_access(&s, Some("rejected")),
            ProjectAccessStatus::Rejected
        );
    }

    #[test]
    fn locked_when_no_modules() {
        let s = ModuleCompletionSummary { total: 0, completed: 0 };
        assert_eq!(compute_project_access(&s, None), ProjectAccessStatus::Locked);
    }

    #[test]
    fn can_submit_only_available_and_rejected() {
        assert!(can_submit(&ProjectAccessStatus::Available));
        assert!(can_submit(&ProjectAccessStatus::Rejected));
        assert!(!can_submit(&ProjectAccessStatus::Locked));
        assert!(!can_submit(&ProjectAccessStatus::PendingReview));
        assert!(!can_submit(&ProjectAccessStatus::Approved));
    }
}
