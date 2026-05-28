use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    config::Config,
    features::admin::audit::service as audit_svc,
    shared::certificate_code,
};

use super::{
    dto::{
        AdminCertificateFilter, AdminCertificateListResponse, AchievementDto,
        CertificateDetailDto, CertificateListResponse, CertificateSummaryDto, FinalProjectAchievementDto,
        IssuedCertificate, PublicCertificateDto, RevokeRequest, VerificationResponse,
        AdminCertificateSummaryDto,
    },
    eligibility::{check_eligibility, EligibilityInput, EligibilityResult},
    error::CertificateError,
    pdf::{generate_pdf, CertificatePdfData},
    qr,
    repository,
};
use crate::features::admin::audit::dto::PaginationDto;

pub struct CertificateService<'a> {
    pub db: &'a PgPool,
    pub config: Arc<Config>,
}

impl<'a> CertificateService<'a> {
    pub fn new(db: &'a PgPool, config: Arc<Config>) -> Self {
        Self { db, config }
    }

    fn verification_url(&self, code: &str) -> String {
        format!("{}/verify/{}", self.config.app_base_url, code)
    }

    fn preview_url(&self, cert_id: Uuid) -> String {
        format!("{}/api/certificates/me/{}", self.config.app_base_url, cert_id)
    }

    fn download_url(&self, cert_id: Uuid) -> String {
        format!("{}/api/certificates/me/{}/download.pdf", self.config.app_base_url, cert_id)
    }

    // ── Auto-issue (called inside approve transaction) ──────────────────────────

    /// Checks eligibility and issues a certificate if all conditions are met.
    /// Idempotent: returns existing cert if already issued for (user, role).
    /// Called inside the approve transaction — tx is passed in for atomicity.
    pub async fn try_issue(
        tx: &mut Transaction<'_, Postgres>,
        admin_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<IssuedCertificate>, CertificateError> {
        // 1. Resolve user's role
        let role_id_opt = repository::get_user_role_id(tx, user_id)
            .await
            .map_err(CertificateError::Internal)?;

        let role_id = match role_id_opt {
            Some(id) => id,
            None => return Ok(None),
        };

        // 2. Gather eligibility data
        let total_modules = repository::count_total_modules(tx, role_id)
            .await
            .map_err(CertificateError::Internal)? as usize;

        let completed_modules = repository::count_completed_modules(tx, role_id, user_id)
            .await
            .map_err(CertificateError::Internal)? as usize;

        let approved_submission_id = repository::get_approved_submission_id(tx, user_id, role_id)
            .await
            .map_err(CertificateError::Internal)?;

        let input = EligibilityInput {
            has_submitted_quiz: true, // we already resolved role_id above
            role_id: Some(role_id),
            total_modules,
            completed_modules,
            final_project_approved: approved_submission_id.is_some(),
            final_project_submission_id: approved_submission_id,
        };

        let (elig_role_id, submission_id) = match check_eligibility(input) {
            EligibilityResult::Eligible { role_id, submission_id } => (role_id, submission_id),
            EligibilityResult::NotEligible { .. } => return Ok(None),
        };

        // 3. Idempotency: return existing if already issued
        if let Some((existing_id, existing_code)) =
            repository::find_existing(tx, user_id, elig_role_id)
                .await
                .map_err(CertificateError::Internal)?
        {
            return Ok(Some(IssuedCertificate {
                id: existing_id,
                certificate_code: existing_code,
            }));
        }

        // 4. Snapshot data
        let recipient_name = repository::get_user_name(tx, user_id)
            .await
            .map_err(CertificateError::Internal)?;
        let (role_name, role_code) = repository::get_role_snapshot(tx, elig_role_id)
            .await
            .map_err(CertificateError::Internal)?;

        // 5. Insert with retry on unique code collision
        let year = Utc::now().year();
        for _ in 0..5 {
            let code = certificate_code::generate_code(year);
            match repository::insert_certificate(
                tx,
                &code,
                user_id,
                elig_role_id,
                &recipient_name,
                &role_name,
                &role_code,
                completed_modules as i32,
                submission_id,
            )
            .await
            {
                Ok((cert_id, _)) => {
                    // 6. Audit log
                    let _ = audit_svc::log(
                        tx,
                        admin_id,
                        "certificate.issued",
                        "certificate",
                        Some(cert_id),
                        Some(json!({
                            "user_id": user_id,
                            "role_id": elig_role_id,
                            "certificate_code": code,
                        })),
                    )
                    .await;

                    return Ok(Some(IssuedCertificate {
                        id: cert_id,
                        certificate_code: code,
                    }));
                }
                Err(sqlx::Error::Database(e)) if e.is_unique_violation() => continue,
                Err(e) => return Err(CertificateError::Database(e)),
            }
        }

        Err(CertificateError::CodeGenerationFailed)
    }

    // ── User-facing list ────────────────────────────────────────────────────────

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<CertificateListResponse, CertificateError> {
        let certs = repository::list_for_user(self.db, user_id)
            .await
            .map_err(CertificateError::Internal)?;

        let summaries = certs
            .into_iter()
            .map(|c| CertificateSummaryDto {
                verification_url: self.verification_url(&c.certificate_code),
                preview_url: self.preview_url(c.id),
                download_url: self.download_url(c.id),
                id: c.id,
                certificate_code: c.certificate_code,
                recipient_name: c.recipient_name,
                role_name: c.role_name,
                role_code: c.role_code,
                issued_at: c.issued_at,
                is_revoked: c.is_revoked,
            })
            .collect();

        Ok(CertificateListResponse { certificates: summaries })
    }

    // ── User-facing detail ──────────────────────────────────────────────────────

    pub async fn get_for_user(
        &self,
        user_id: Uuid,
        cert_id: Uuid,
    ) -> Result<CertificateDetailDto, CertificateError> {
        let cert = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        if cert.user_id != user_id {
            return Err(CertificateError::NotOwned);
        }

        let modules = repository::get_completed_modules(self.db, cert.role_id, user_id)
            .await
            .map_err(CertificateError::Internal)?;

        let (project_title, approved_at) =
            repository::get_project_title_and_approved_at(self.db, cert.final_project_submission_id)
                .await
                .map_err(CertificateError::Internal)?;

        let verification_url = self.verification_url(&cert.certificate_code);
        let qr_code_data_url = qr::generate_qr_data_url(&verification_url)?;

        Ok(CertificateDetailDto {
            id: cert.id,
            certificate_code: cert.certificate_code,
            recipient_name: cert.recipient_name,
            role_name: cert.role_name,
            role_code: cert.role_code,
            issued_at: cert.issued_at,
            achievement: AchievementDto {
                modules_completed_count: cert.modules_completed_count,
                modules,
                final_project: FinalProjectAchievementDto {
                    title: project_title,
                    approved_at,
                },
            },
            is_revoked: cert.is_revoked,
            revoked_at: cert.revoked_at,
            revocation_reason: cert.revocation_reason,
            verification_url,
            qr_code_data_url,
        })
    }

    // ── PDF download ────────────────────────────────────────────────────────────

    pub async fn generate_pdf_bytes(
        &self,
        user_id: Uuid,
        cert_id: Uuid,
    ) -> Result<(Vec<u8>, String), CertificateError> {
        let cert = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        if cert.user_id != user_id {
            return Err(CertificateError::NotOwned);
        }

        let verification_url = self.verification_url(&cert.certificate_code);
        let qr_png = qr::generate_qr_png(&verification_url)?;

        let pdf_bytes = generate_pdf(&CertificatePdfData {
            certificate_code: cert.certificate_code.clone(),
            recipient_name: cert.recipient_name,
            role_name: cert.role_name,
            issued_at: cert.issued_at,
            modules_count: cert.modules_completed_count,
            verification_url,
            qr_code_png: qr_png,
        })?;

        Ok((pdf_bytes, cert.certificate_code))
    }

    // ── Public verification ─────────────────────────────────────────────────────

    pub async fn verify_by_code(&self, code: &str) -> Result<VerificationResponse, CertificateError> {
        let cert = repository::get_by_code(self.db, code)
            .await
            .map_err(CertificateError::Internal)?;

        match cert {
            None => Ok(VerificationResponse {
                is_valid: false,
                certificate: None,
                status: "not_found".into(),
                message: "Certificate tidak ditemukan".into(),
            }),
            Some(c) if c.is_revoked => Ok(VerificationResponse {
                is_valid: false,
                certificate: Some(PublicCertificateDto {
                    certificate_code: c.certificate_code,
                    recipient_name: c.recipient_name,
                    role_name: c.role_name,
                    issued_at: c.issued_at,
                    modules_completed_count: c.modules_completed_count,
                }),
                status: "revoked".into(),
                message: "Certificate ini telah dicabut".into(),
            }),
            Some(c) => Ok(VerificationResponse {
                is_valid: true,
                certificate: Some(PublicCertificateDto {
                    certificate_code: c.certificate_code,
                    recipient_name: c.recipient_name,
                    role_name: c.role_name,
                    issued_at: c.issued_at,
                    modules_completed_count: c.modules_completed_count,
                }),
                status: "valid".into(),
                message: "Certificate valid".into(),
            }),
        }
    }

    // ── Admin: list ─────────────────────────────────────────────────────────────

    pub async fn admin_list(
        &self,
        filter: AdminCertificateFilter,
    ) -> Result<AdminCertificateListResponse, CertificateError> {
        let page = filter.page.unwrap_or(1).max(1);
        let per_page = filter.per_page.unwrap_or(20).clamp(1, 100);

        let (items, total) = repository::admin_list(self.db, &filter)
            .await
            .map_err(CertificateError::Internal)?;

        let total_pages = (total + per_page - 1) / per_page;

        Ok(AdminCertificateListResponse {
            certificates: items,
            pagination: PaginationDto {
                page,
                per_page,
                total_items: total,
                total_pages,
            },
        })
    }

    // ── Admin: revoke ───────────────────────────────────────────────────────────

    pub async fn revoke(
        &self,
        admin_id: Uuid,
        cert_id: Uuid,
        req: RevokeRequest,
    ) -> Result<AdminCertificateSummaryDto, CertificateError> {
        let cert = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        if cert.is_revoked {
            return Err(CertificateError::AlreadyRevoked);
        }

        let reason = crate::shared::sanitization::sanitize_plain_text(&req.reason, 20, 2000)
            .map_err(|e| CertificateError::Internal(anyhow::anyhow!("{e}")))?;

        let mut tx = self.db.begin().await.map_err(CertificateError::Database)?;

        repository::apply_revoke(&mut tx, cert_id, admin_id, &reason)
            .await
            .map_err(CertificateError::Internal)?;

        audit_svc::log(
            &mut tx,
            admin_id,
            "certificate.revoked",
            "certificate",
            Some(cert_id),
            Some(json!({ "certificate_code": cert.certificate_code, "reason": reason })),
        )
        .await
        .map_err(CertificateError::Internal)?;

        tx.commit().await.map_err(CertificateError::Database)?;

        // Re-fetch to return updated state
        let updated = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        let user_email = sqlx::query_scalar!(
            "SELECT email FROM users WHERE id = $1",
            updated.user_id
        )
        .fetch_one(self.db)
        .await
        .map_err(CertificateError::Database)?;

        Ok(AdminCertificateSummaryDto {
            id: updated.id,
            certificate_code: updated.certificate_code,
            recipient_name: updated.recipient_name,
            user_id: updated.user_id,
            user_email,
            role_name: updated.role_name,
            role_code: updated.role_code,
            issued_at: updated.issued_at,
            modules_completed_count: updated.modules_completed_count,
            is_revoked: updated.is_revoked,
            revoked_at: updated.revoked_at,
            revocation_reason: updated.revocation_reason,
        })
    }

    // ── Admin: restore ──────────────────────────────────────────────────────────

    pub async fn restore(
        &self,
        admin_id: Uuid,
        cert_id: Uuid,
    ) -> Result<AdminCertificateSummaryDto, CertificateError> {
        let cert = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        if !cert.is_revoked {
            return Err(CertificateError::NotRevoked);
        }

        let mut tx = self.db.begin().await.map_err(CertificateError::Database)?;

        repository::apply_restore(&mut tx, cert_id)
            .await
            .map_err(CertificateError::Internal)?;

        audit_svc::log(
            &mut tx,
            admin_id,
            "certificate.restored",
            "certificate",
            Some(cert_id),
            Some(json!({ "certificate_code": cert.certificate_code })),
        )
        .await
        .map_err(CertificateError::Internal)?;

        tx.commit().await.map_err(CertificateError::Database)?;

        let updated = repository::get_by_id(self.db, cert_id)
            .await
            .map_err(CertificateError::Internal)?
            .ok_or(CertificateError::NotFound)?;

        let user_email = sqlx::query_scalar!(
            "SELECT email FROM users WHERE id = $1",
            updated.user_id
        )
        .fetch_one(self.db)
        .await
        .map_err(CertificateError::Database)?;

        Ok(AdminCertificateSummaryDto {
            id: updated.id,
            certificate_code: updated.certificate_code,
            recipient_name: updated.recipient_name,
            user_id: updated.user_id,
            user_email,
            role_name: updated.role_name,
            role_code: updated.role_code,
            issued_at: updated.issued_at,
            modules_completed_count: updated.modules_completed_count,
            is_revoked: updated.is_revoked,
            revoked_at: updated.revoked_at,
            revocation_reason: updated.revocation_reason,
        })
    }
}

// chrono::Datelike for .year()
use chrono::Datelike;
