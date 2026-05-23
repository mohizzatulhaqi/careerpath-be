-- Add soft-delete columns to users
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_active          BOOLEAN     NOT NULL DEFAULT true;
ALTER TABLE users ADD COLUMN IF NOT EXISTS deactivated_at     TIMESTAMPTZ;
ALTER TABLE users ADD COLUMN IF NOT EXISTS deactivated_by     UUID REFERENCES users(id);
ALTER TABLE users ADD COLUMN IF NOT EXISTS deactivation_reason TEXT;

CREATE INDEX IF NOT EXISTS idx_users_is_active ON users (is_active);

-- Admin audit log
CREATE TABLE IF NOT EXISTS admin_audit_logs (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id    UUID        NOT NULL REFERENCES users(id),
    action      VARCHAR(50) NOT NULL,
    target_type VARCHAR(50) NOT NULL,
    target_id   UUID,
    metadata    JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_admin  ON admin_audit_logs (admin_id,   created_at DESC);
CREATE INDEX idx_audit_target ON admin_audit_logs (target_type, target_id);
CREATE INDEX idx_audit_action ON admin_audit_logs (action,      created_at DESC);
