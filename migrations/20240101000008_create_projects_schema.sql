-- Final projects: one project spec per role
CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id         UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    title           VARCHAR NOT NULL,
    description     TEXT NOT NULL,           -- instruksi project (markdown)
    requirements    TEXT,                    -- kriteria/checklist (markdown)
    estimated_hours INT NOT NULL DEFAULT 20,
    is_published    BOOLEAN NOT NULL DEFAULT true,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_projects_role UNIQUE (role_id)
);

CREATE INDEX idx_projects_role ON projects (role_id);

CREATE TRIGGER set_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();

-- Submissions: users upload their final project ZIP
CREATE TABLE project_submissions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    project_id          UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    submission_notes    TEXT NOT NULL,
    file_path           VARCHAR(500) NOT NULL,       -- relative path dalam storage root
    file_original_name  VARCHAR(255) NOT NULL,       -- nama file asli dari user
    file_size_bytes     BIGINT NOT NULL,
    file_mime_type      VARCHAR(100) NOT NULL DEFAULT 'application/zip',
    status              VARCHAR NOT NULL DEFAULT 'pending_review'
                            CHECK (status IN ('pending_review', 'approved', 'rejected')),
    reviewer_notes      TEXT,
    reviewed_at         TIMESTAMPTZ,
    reviewed_by         UUID REFERENCES users(id),
    submitted_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_project_submissions_user_project ON project_submissions (user_id, project_id, submitted_at DESC);
CREATE INDEX idx_project_submissions_status ON project_submissions (status, submitted_at);
