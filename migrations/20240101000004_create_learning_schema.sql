-- Learning modules per career role
CREATE TABLE learning_modules (
    id           UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    role_id      UUID         NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    title        VARCHAR(255) NOT NULL,
    description  TEXT,
    order_index  INT          NOT NULL,
    is_published BOOLEAN      NOT NULL DEFAULT true,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_learning_modules_role_order ON learning_modules (role_id, order_index);

CREATE TRIGGER set_updated_at_learning_modules
    BEFORE UPDATE ON learning_modules
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();

-- Sub-materials within a module
CREATE TABLE submaterials (
    id                UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id         UUID         NOT NULL REFERENCES learning_modules(id) ON DELETE CASCADE,
    title             VARCHAR(255) NOT NULL,
    content           TEXT,
    order_index       INT          NOT NULL,
    estimated_minutes INT          NOT NULL DEFAULT 10,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    UNIQUE (module_id, order_index)
);

CREATE INDEX idx_submaterials_module_order ON submaterials (module_id, order_index);

CREATE TRIGGER set_updated_at_submaterials
    BEFORE UPDATE ON submaterials
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();

-- User progress: which submaterials have been completed
CREATE TABLE user_submaterial_progress (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    submaterial_id  UUID        NOT NULL REFERENCES submaterials(id) ON DELETE CASCADE,
    completed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, submaterial_id)
);

CREATE INDEX idx_user_sub_progress_user    ON user_submaterial_progress (user_id);
CREATE INDEX idx_user_sub_progress_submat  ON user_submaterial_progress (submaterial_id);
