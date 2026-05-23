-- All DEFAULT true so existing records stay visible.

-- submaterials did not have is_published yet
ALTER TABLE submaterials
    ADD COLUMN IF NOT EXISTS is_published BOOLEAN NOT NULL DEFAULT true;

-- per-question toggle for mini and final quiz
ALTER TABLE submaterial_quizzes
    ADD COLUMN IF NOT EXISTS is_published BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE module_quizzes
    ADD COLUMN IF NOT EXISTS is_published BOOLEAN NOT NULL DEFAULT true;

-- roles can be deactivated (soft delete)
ALTER TABLE roles
    ADD COLUMN IF NOT EXISTS is_active BOOLEAN NOT NULL DEFAULT true;

-- supporting indexes
CREATE INDEX IF NOT EXISTS idx_submaterials_published
    ON submaterials (module_id, is_published);

CREATE INDEX IF NOT EXISTS idx_submaterial_quizzes_published
    ON submaterial_quizzes (submaterial_id, is_published);

CREATE INDEX IF NOT EXISTS idx_module_quizzes_published
    ON module_quizzes (module_id, is_published);

CREATE INDEX IF NOT EXISTS idx_roles_active
    ON roles (is_active);
