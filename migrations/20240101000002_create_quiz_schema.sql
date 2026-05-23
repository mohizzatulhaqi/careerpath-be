-- Career roles
CREATE TABLE roles (
    id          UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    code        VARCHAR(50)  NOT NULL UNIQUE,
    name        VARCHAR(100) NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Quiz questions
CREATE TABLE quiz_questions (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    question_text TEXT        NOT NULL,
    order_index   INT         NOT NULL,
    is_active     BOOLEAN     NOT NULL DEFAULT true,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_quiz_questions_active_order ON quiz_questions (is_active, order_index);

CREATE TRIGGER set_updated_at_quiz_questions
    BEFORE UPDATE ON quiz_questions
    FOR EACH ROW EXECUTE FUNCTION trigger_set_updated_at();

-- Options per question
CREATE TABLE quiz_options (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    question_id UUID        NOT NULL REFERENCES quiz_questions(id) ON DELETE CASCADE,
    option_text TEXT        NOT NULL,
    order_index INT         NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_quiz_options_question_id ON quiz_options (question_id);

-- Weight of each option toward each role
CREATE TABLE option_role_weights (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    option_id UUID NOT NULL REFERENCES quiz_options(id) ON DELETE CASCADE,
    role_id   UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    weight    INT  NOT NULL CHECK (weight >= 0),
    UNIQUE (option_id, role_id)
);
CREATE INDEX idx_option_role_weights_option_id ON option_role_weights (option_id);

-- A single quiz run by a user
CREATE TABLE quiz_attempts (
    id               UUID          PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id          UUID          NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status           VARCHAR(20)   NOT NULL CHECK (status IN ('in_progress', 'submitted')),
    started_at       TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    submitted_at     TIMESTAMPTZ,
    result_role_id   UUID          REFERENCES roles(id),
    match_percentage DOUBLE PRECISION
);
CREATE INDEX idx_quiz_attempts_user_status     ON quiz_attempts (user_id, status);
CREATE INDEX idx_quiz_attempts_user_submitted  ON quiz_attempts (user_id, submitted_at DESC NULLS LAST);

-- One answer per question per attempt (UNIQUE enforces single answer per question)
CREATE TABLE quiz_attempt_answers (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    attempt_id  UUID        NOT NULL REFERENCES quiz_attempts(id) ON DELETE CASCADE,
    question_id UUID        NOT NULL REFERENCES quiz_questions(id),
    option_id   UUID        NOT NULL REFERENCES quiz_options(id),
    answered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (attempt_id, question_id)
);
CREATE INDEX idx_quiz_attempt_answers_attempt_id ON quiz_attempt_answers (attempt_id);
