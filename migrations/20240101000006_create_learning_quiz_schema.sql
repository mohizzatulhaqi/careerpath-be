-- Mini Quiz: per submaterial (must score 100% to pass)
CREATE TABLE submaterial_quizzes (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submaterial_id UUID NOT NULL REFERENCES submaterials(id) ON DELETE CASCADE,
    question       TEXT NOT NULL,
    order_index    INTEGER NOT NULL DEFAULT 1,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE submaterial_quiz_options (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID NOT NULL REFERENCES submaterial_quizzes(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    is_correct  BOOLEAN NOT NULL DEFAULT false,
    order_index INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE submaterial_quiz_attempts (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    submaterial_id UUID NOT NULL REFERENCES submaterials(id) ON DELETE CASCADE,
    score          DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    passed         BOOLEAN NOT NULL DEFAULT false,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE submaterial_quiz_attempt_answers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id  UUID NOT NULL REFERENCES submaterial_quiz_attempts(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES submaterial_quizzes(id) ON DELETE CASCADE,
    option_id   UUID NOT NULL REFERENCES submaterial_quiz_options(id) ON DELETE CASCADE,
    is_correct  BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (attempt_id, question_id)
);

-- Final Quiz: per module (must score >= 70% to pass, unlimited retries)
CREATE TABLE module_quizzes (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    module_id   UUID NOT NULL REFERENCES learning_modules(id) ON DELETE CASCADE,
    question    TEXT NOT NULL,
    order_index INTEGER NOT NULL DEFAULT 1,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE module_quiz_options (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID NOT NULL REFERENCES module_quizzes(id) ON DELETE CASCADE,
    text        TEXT NOT NULL,
    is_correct  BOOLEAN NOT NULL DEFAULT false,
    order_index INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE module_quiz_attempts (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    module_id  UUID NOT NULL REFERENCES learning_modules(id) ON DELETE CASCADE,
    score      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    passed     BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE module_quiz_attempt_answers (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id  UUID NOT NULL REFERENCES module_quiz_attempts(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES module_quizzes(id) ON DELETE CASCADE,
    option_id   UUID NOT NULL REFERENCES module_quiz_options(id) ON DELETE CASCADE,
    is_correct  BOOLEAN NOT NULL DEFAULT false,
    UNIQUE (attempt_id, question_id)
);

CREATE INDEX idx_submaterial_quizzes_sub ON submaterial_quizzes (submaterial_id);
CREATE INDEX idx_sub_quiz_attempts_user_sub ON submaterial_quiz_attempts (user_id, submaterial_id);
CREATE INDEX idx_module_quizzes_mod ON module_quizzes (module_id);
CREATE INDEX idx_mod_quiz_attempts_user_mod ON module_quiz_attempts (user_id, module_id);
