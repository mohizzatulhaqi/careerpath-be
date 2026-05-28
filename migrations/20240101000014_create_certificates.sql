-- ── Certificate table ─────────────────────────────────────────────────────────

CREATE TABLE certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Unique human-readable code for verification URL (format: CERT-YYYY-XXXXXXXX)
    certificate_code VARCHAR(32) UNIQUE NOT NULL,

    -- Owner
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,

    -- Snapshot data (immutable — unaffected if user/role is later updated)
    recipient_name  VARCHAR(255) NOT NULL,
    role_name       VARCHAR(255) NOT NULL,
    role_code       VARCHAR(50)  NOT NULL,

    -- Achievement details
    issued_at                    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    modules_completed_count      INT         NOT NULL,
    final_project_submission_id  UUID        NOT NULL REFERENCES project_submissions(id) ON DELETE RESTRICT,

    -- Revocation (e.g. plagiarism found later)
    is_revoked        BOOLEAN    NOT NULL DEFAULT false,
    revoked_at        TIMESTAMPTZ,
    revoked_by        UUID REFERENCES users(id),
    revocation_reason TEXT,

    UNIQUE(user_id, role_id)
);

CREATE INDEX idx_certificates_user   ON certificates(user_id);
CREATE INDEX idx_certificates_code   ON certificates(certificate_code);
CREATE INDEX idx_certificates_issued ON certificates(issued_at DESC);
