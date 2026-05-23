-- Migration: create submission_review_history table
-- The project_submissions table (reviewed_by, reviewed_at, reviewer_notes) already
-- exists from migration 008. This only adds the review history log.

CREATE TABLE IF NOT EXISTS submission_review_history (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID        NOT NULL REFERENCES project_submissions(id) ON DELETE CASCADE,
    reviewed_by   UUID        NOT NULL REFERENCES users(id),
    old_status    VARCHAR(20),           -- NULL on first review
    new_status    VARCHAR(20) NOT NULL,
    reviewer_notes TEXT       NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_review_history_submission
    ON submission_review_history (submission_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_review_history_admin
    ON submission_review_history (reviewed_by, created_at DESC);
