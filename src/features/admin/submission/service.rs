use anyhow::Context;
use axum::body::Bytes;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use super::{
    dto::{
        QueueStatsDto, ReviewResultDto, SubmissionDetailDto, SubmissionFilter,
        SubmissionListResponse,
    },
    error::SubmissionError,
    repository,
};
use crate::{
    features::admin::audit::service as audit_svc,
    features::certificate::service::CertificateService,
    features::project::storage::Storage,
    shared::sanitization,
};

/// Returned by `download_file`.
pub struct DownloadPayload {
    pub bytes: Bytes,
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
}

// ── Pure state-transition validator ──────────────────────────────────────────

/// Validates whether transitioning from `from` to `to` is allowed.
/// Pure function — no I/O, easy to unit test.
pub fn validate_transition(from: &str, to: &str) -> Result<(), SubmissionError> {
    match (from, to) {
        ("pending_review", "approved") | ("pending_review", "rejected") => Ok(()),
        ("approved", "rejected") => Ok(()),
        ("rejected", "approved") => Ok(()),
        // no-op same-state transitions
        ("approved", "approved") => Err(SubmissionError::AlreadyApproved),
        ("rejected", "rejected") => Err(SubmissionError::AlreadyRejected),
        // anything → pending is forbidden
        (current, "pending_review") => Err(SubmissionError::InvalidStateTransition {
            from: current.to_string(),
            to: "pending_review".to_string(),
        }),
        (current, target) => Err(SubmissionError::InvalidStateTransition {
            from: current.to_string(),
            to: target.to_string(),
        }),
    }
}

// ── Service struct ────────────────────────────────────────────────────────────

pub struct AdminSubmissionService<'a> {
    pub db: &'a sqlx::PgPool,
    pub storage: Arc<dyn Storage>,
}

impl<'a> AdminSubmissionService<'a> {
    pub fn new(db: &'a sqlx::PgPool, storage: Arc<dyn Storage>) -> Self {
        Self { db, storage }
    }

    // ── List ──────────────────────────────────────────────────────────────────

    pub async fn list(
        &self,
        filter: SubmissionFilter,
    ) -> Result<SubmissionListResponse, SubmissionError> {
        let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);
        let page = filter.page.unwrap_or(1).max(1);

        let (submissions, total) = repository::list_submissions(self.db, &filter)
            .await
            .map_err(SubmissionError::Internal)?;

        let summary = repository::get_summary_counts(self.db)
            .await
            .map_err(SubmissionError::Internal)?;

        let total_pages = (total + per_page - 1) / per_page;

        Ok(SubmissionListResponse {
            submissions,
            pagination: crate::features::admin::audit::dto::PaginationDto {
                page,
                per_page,
                total_items: total,
                total_pages,
            },
            summary,
        })
    }

    // ── Detail ────────────────────────────────────────────────────────────────

    pub async fn get_detail(
        &self,
        submission_id: Uuid,
    ) -> Result<SubmissionDetailDto, SubmissionError> {
        repository::get_submission_detail(self.db, submission_id)
            .await
            .map_err(SubmissionError::Internal)?
            .ok_or(SubmissionError::NotFound)
    }

    // ── Download ──────────────────────────────────────────────────────────────

    pub async fn download_file(
        &self,
        admin_id: Uuid,
        submission_id: Uuid,
    ) -> Result<DownloadPayload, SubmissionError> {
        let file_info = repository::get_file_info(self.db, submission_id)
            .await
            .map_err(SubmissionError::Internal)?
            .ok_or(SubmissionError::NotFound)?;

        let (file_path, original_name, size_bytes, mime_type) = file_info;

        let stored = self
            .storage
            .retrieve(&file_path)
            .await
            .map_err(|e| SubmissionError::Internal(anyhow::anyhow!("storage error: {e}")))?;
        let bytes = stored.data;

        // Audit log for download (no status change, so no review_history entry)
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(SubmissionError::Database)?;

        audit_svc::log(
            &mut tx,
            admin_id,
            "submission.downloaded",
            "project_submission",
            Some(submission_id),
            Some(json!({ "submission_id": submission_id, "file_size": size_bytes })),
        )
        .await
        .map_err(|e| SubmissionError::Internal(anyhow::anyhow!("audit log: {e}")))?;

        tx.commit().await.map_err(SubmissionError::Database)?;

        Ok(DownloadPayload {
            bytes,
            original_name,
            mime_type,
            size: size_bytes as u64,
        })
    }

    // ── Approve ───────────────────────────────────────────────────────────────

    pub async fn approve(
        &self,
        admin_id: Uuid,
        submission_id: Uuid,
        raw_notes: String,
    ) -> Result<ReviewResultDto, SubmissionError> {
        let notes = sanitization::sanitize_plain_text(&raw_notes, 10, 5000)
            .map_err(|e| SubmissionError::ValidationFailed(e.to_string()))?;

        self.do_review(admin_id, submission_id, "approved", notes).await
    }

    // ── Reject ────────────────────────────────────────────────────────────────

    pub async fn reject(
        &self,
        admin_id: Uuid,
        submission_id: Uuid,
        raw_notes: String,
    ) -> Result<ReviewResultDto, SubmissionError> {
        let notes = sanitization::sanitize_plain_text(&raw_notes, 20, 5000)
            .map_err(|e| SubmissionError::ValidationFailed(e.to_string()))?;

        self.do_review(admin_id, submission_id, "rejected", notes).await
    }

    // ── Core review transaction ───────────────────────────────────────────────

    async fn do_review(
        &self,
        admin_id: Uuid,
        submission_id: Uuid,
        new_status: &str,
        notes: String,
    ) -> Result<ReviewResultDto, SubmissionError> {
        let mut tx = self
            .db
            .begin()
            .await
            .map_err(SubmissionError::Database)?;

        // 1. SELECT FOR UPDATE (row-level lock, prevents race condition)
        let current = repository::find_for_update(&mut tx, submission_id)
            .await
            .map_err(SubmissionError::Internal)?
            .ok_or(SubmissionError::NotFound)?;

        let old_status = current.status.clone();

        // 2. Validate state transition
        validate_transition(&old_status, new_status)?;

        // 3. Update + insert history (single DB call pair)
        let reviewed_at = repository::apply_review(
            &mut tx,
            submission_id,
            admin_id,
            &old_status,
            new_status,
            &notes,
        )
        .await
        .map_err(SubmissionError::Internal)?;

        // 4. Audit log
        let was_previously_approved = old_status == "approved";
        audit_svc::log(
            &mut tx,
            admin_id,
            &format!("submission.{new_status}"),
            "project_submission",
            Some(submission_id),
            Some(json!({
                "old_status": old_status,
                "project_id": current.project_id,
                "user_id": current.user_id,
                "was_previously_approved": was_previously_approved,
            })),
        )
        .await
        .context("audit log")
        .map_err(SubmissionError::Internal)?;

        // 5. Auto-issue certificate if this is an approval
        let issued_cert = if new_status == "approved" {
            CertificateService::try_issue(&mut tx, admin_id, current.user_id)
                .await
                .map_err(|e| SubmissionError::Internal(anyhow::anyhow!("certificate: {e}")))?
        } else {
            None
        };

        tx.commit().await.map_err(SubmissionError::Database)?;

        // 6. Build response (re-query brief info outside tx, lock released)
        let (user, project) = repository::get_review_brief(self.db, submission_id)
            .await
            .map_err(SubmissionError::Internal)?
            .ok_or(SubmissionError::NotFound)?;

        Ok(ReviewResultDto {
            submission_id,
            status: new_status.to_string(),
            reviewed_at,
            reviewer_notes: notes,
            user,
            project,
            certificate_issued: issued_cert.is_some(),
            certificate_id: issued_cert.as_ref().map(|c| c.id),
            certificate_code: issued_cert.map(|c| c.certificate_code),
        })
    }

    // ── Queue stats ───────────────────────────────────────────────────────────

    pub async fn queue_stats(&self) -> Result<QueueStatsDto, SubmissionError> {
        repository::get_queue_stats(self.db)
            .await
            .map_err(SubmissionError::Internal)
    }
}

// ── Unit tests for validate_transition ───────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_to_approved_ok() {
        assert!(validate_transition("pending_review", "approved").is_ok());
    }

    #[test]
    fn pending_to_rejected_ok() {
        assert!(validate_transition("pending_review", "rejected").is_ok());
    }

    #[test]
    fn approved_to_rejected_ok() {
        assert!(validate_transition("approved", "rejected").is_ok());
    }

    #[test]
    fn rejected_to_approved_ok() {
        assert!(validate_transition("rejected", "approved").is_ok());
    }

    #[test]
    fn approved_to_approved_is_err() {
        assert!(matches!(
            validate_transition("approved", "approved"),
            Err(SubmissionError::AlreadyApproved)
        ));
    }

    #[test]
    fn rejected_to_rejected_is_err() {
        assert!(matches!(
            validate_transition("rejected", "rejected"),
            Err(SubmissionError::AlreadyRejected)
        ));
    }

    #[test]
    fn approved_to_pending_is_invalid_transition() {
        assert!(matches!(
            validate_transition("approved", "pending_review"),
            Err(SubmissionError::InvalidStateTransition { .. })
        ));
    }

    #[test]
    fn rejected_to_pending_is_invalid_transition() {
        assert!(matches!(
            validate_transition("rejected", "pending_review"),
            Err(SubmissionError::InvalidStateTransition { .. })
        ));
    }
}
