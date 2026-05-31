-- ============================================================================
-- Demo Seed Data — CareerPath
-- ============================================================================
-- Jalankan SETELAH semua migration selesai:
--   psql $DATABASE_URL -f seed.sql
--
-- Password semua user demo: ChangeMeASAP123!
--
-- Akun yang dibuat:
--   admin@careerpath.local  — admin  (dari migration, sudah ada)
--   budi@demo.com           — user   (Frontend, project APPROVED, punya sertifikat)
--   sari@demo.com           — user   (Backend, project PENDING review)
--   andi@demo.com           — user   (UIUX, project REJECTED)
--   rina@demo.com           — user   (Data Analyst, baru selesai quiz)
--   dimas@demo.com          — user   (Mobile, baru selesai quiz)
-- ============================================================================

DO $$
DECLARE
  -- ── Fixed IDs ──────────────────────────────────────────────────────────────
  u_admin UUID := '00000000-0000-0000-0000-000000000001';

  u_budi  UUID := '00000002-0000-0000-0000-000000000001';
  u_sari  UUID := '00000002-0000-0000-0000-000000000002';
  u_andi  UUID := '00000002-0000-0000-0000-000000000003';
  u_rina  UUID := '00000002-0000-0000-0000-000000000004';
  u_dimas UUID := '00000002-0000-0000-0000-000000000005';

  r_fe   UUID := '00000001-0000-0000-0000-000000000001';
  r_be   UUID := '00000001-0000-0000-0000-000000000002';
  r_uiux UUID := '00000001-0000-0000-0000-000000000003';
  r_da   UUID := '00000001-0000-0000-0000-000000000004';
  r_mob  UUID := '00000001-0000-0000-0000-000000000005';

  sub_budi UUID := '00000003-0000-0000-0000-000000000001';
  sub_sari UUID := '00000003-0000-0000-0000-000000000002';
  sub_andi UUID := '00000003-0000-0000-0000-000000000003';

  cert_budi UUID := '00000004-0000-0000-0000-000000000001';

  -- ── Working vars ───────────────────────────────────────────────────────────
  v_attempt_id UUID;
  v_mod_id     UUID;
  v_mqa_id     UUID;
  proj_fe_id   UUID;
  proj_be_id   UUID;
  proj_uiux_id UUID;
  fe_mod_count INT;

BEGIN

-- ── Idempotency guard ─────────────────────────────────────────────────────────
IF EXISTS (SELECT 1 FROM users WHERE id = u_budi) THEN
    RAISE NOTICE 'Demo seed data already exists — skipping.';
    RETURN;
END IF;

-- ═════════════════════════════════════════════════════════════════════════════
-- 1. USERS
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO users (id, email, password_hash, name, role, is_active) VALUES
  (u_budi,  'budi@demo.com',
   '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
   'Budi Santoso', 'user', true),
  (u_sari,  'sari@demo.com',
   '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
   'Sari Dewi', 'user', true),
  (u_andi,  'andi@demo.com',
   '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
   'Andi Pratama', 'user', true),
  (u_rina,  'rina@demo.com',
   '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
   'Rina Kusuma', 'user', true),
  (u_dimas, 'dimas@demo.com',
   '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
   'Dimas Putra', 'user', true);

-- ═════════════════════════════════════════════════════════════════════════════
-- 2. CAREER QUIZ ATTEMPTS
--    Setiap user punya 1 attempt yang sudah submitted ke role yang sesuai.
--    Jawaban = opsi dengan bobot tertinggi untuk role tersebut.
-- ═════════════════════════════════════════════════════════════════════════════

-- Budi → Frontend (88%)
INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
VALUES (u_budi, 'submitted', NOW() - INTERVAL '10 days', r_fe, 88.0)
RETURNING id INTO v_attempt_id;

INSERT INTO quiz_attempt_answers (attempt_id, question_id, option_id)
SELECT v_attempt_id, q.id,
       (SELECT o.id FROM quiz_options o
        JOIN option_role_weights w ON o.id = w.option_id
        WHERE o.question_id = q.id AND w.role_id = r_fe
        ORDER BY w.weight DESC, o.order_index ASC LIMIT 1)
FROM quiz_questions q WHERE q.is_active = true;

-- Sari → Backend (83%)
INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
VALUES (u_sari, 'submitted', NOW() - INTERVAL '8 days', r_be, 83.0)
RETURNING id INTO v_attempt_id;

INSERT INTO quiz_attempt_answers (attempt_id, question_id, option_id)
SELECT v_attempt_id, q.id,
       (SELECT o.id FROM quiz_options o
        JOIN option_role_weights w ON o.id = w.option_id
        WHERE o.question_id = q.id AND w.role_id = r_be
        ORDER BY w.weight DESC, o.order_index ASC LIMIT 1)
FROM quiz_questions q WHERE q.is_active = true;

-- Andi → UIUX (79%)
INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
VALUES (u_andi, 'submitted', NOW() - INTERVAL '7 days', r_uiux, 79.0)
RETURNING id INTO v_attempt_id;

INSERT INTO quiz_attempt_answers (attempt_id, question_id, option_id)
SELECT v_attempt_id, q.id,
       (SELECT o.id FROM quiz_options o
        JOIN option_role_weights w ON o.id = w.option_id
        WHERE o.question_id = q.id AND w.role_id = r_uiux
        ORDER BY w.weight DESC, o.order_index ASC LIMIT 1)
FROM quiz_questions q WHERE q.is_active = true;

-- Rina → Data Analyst (91%)
INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
VALUES (u_rina, 'submitted', NOW() - INTERVAL '5 days', r_da, 91.0)
RETURNING id INTO v_attempt_id;

INSERT INTO quiz_attempt_answers (attempt_id, question_id, option_id)
SELECT v_attempt_id, q.id,
       (SELECT o.id FROM quiz_options o
        JOIN option_role_weights w ON o.id = w.option_id
        WHERE o.question_id = q.id AND w.role_id = r_da
        ORDER BY w.weight DESC, o.order_index ASC LIMIT 1)
FROM quiz_questions q WHERE q.is_active = true;

-- Dimas → Mobile (85%)
INSERT INTO quiz_attempts (user_id, status, submitted_at, result_role_id, match_percentage)
VALUES (u_dimas, 'submitted', NOW() - INTERVAL '3 days', r_mob, 85.0)
RETURNING id INTO v_attempt_id;

INSERT INTO quiz_attempt_answers (attempt_id, question_id, option_id)
SELECT v_attempt_id, q.id,
       (SELECT o.id FROM quiz_options o
        JOIN option_role_weights w ON o.id = w.option_id
        WHERE o.question_id = q.id AND w.role_id = r_mob
        ORDER BY w.weight DESC, o.order_index ASC LIMIT 1)
FROM quiz_questions q WHERE q.is_active = true;

-- ═════════════════════════════════════════════════════════════════════════════
-- 3. LEARNING PROGRESS — BUDI (selesaikan SEMUA modul Frontend)
-- ═════════════════════════════════════════════════════════════════════════════

-- Tandai semua submaterial Frontend sebagai selesai
INSERT INTO user_submaterial_progress (user_id, submaterial_id, completed_at)
SELECT
    u_budi,
    s.id,
    NOW() - (INTERVAL '1 hour' * ROW_NUMBER() OVER (
        PARTITION BY lm.id ORDER BY s.order_index
    ))
FROM learning_modules lm
JOIN submaterials s ON s.module_id = lm.id
WHERE lm.role_id = r_fe AND lm.is_published = true AND s.is_published = true
ON CONFLICT (user_id, submaterial_id) DO NOTHING;

-- Mini quiz: passed setiap submaterial (score 100%)
INSERT INTO submaterial_quiz_attempts (user_id, submaterial_id, score, passed, created_at)
SELECT
    u_budi, s.id, 100.0, true,
    NOW() - (INTERVAL '1 hour' * ROW_NUMBER() OVER (
        PARTITION BY lm.id ORDER BY s.order_index
    ))
FROM learning_modules lm
JOIN submaterials s ON s.module_id = lm.id
WHERE lm.role_id = r_fe AND lm.is_published = true AND s.is_published = true;

-- Jawaban mini quiz (semua benar)
INSERT INTO submaterial_quiz_attempt_answers (attempt_id, question_id, option_id, is_correct)
SELECT
    a.id,
    sq.id,
    (SELECT sqo.id FROM submaterial_quiz_options sqo
     WHERE sqo.question_id = sq.id AND sqo.is_correct = true
     ORDER BY sqo.order_index LIMIT 1),
    true
FROM submaterial_quiz_attempts a
JOIN submaterial_quizzes sq ON sq.submaterial_id = a.submaterial_id
WHERE a.user_id = u_budi
ON CONFLICT (attempt_id, question_id) DO NOTHING;

-- Final quiz: passed setiap modul Frontend (score 80%)
INSERT INTO module_quiz_attempts (user_id, module_id, score, passed, created_at)
SELECT
    u_budi, lm.id, 80.0, true,
    NOW() - (INTERVAL '30 minutes' * ROW_NUMBER() OVER (ORDER BY lm.order_index))
FROM learning_modules lm
WHERE lm.role_id = r_fe AND lm.is_published = true;

-- Jawaban final quiz (semua benar)
FOR v_mod_id IN
    SELECT id FROM learning_modules WHERE role_id = r_fe AND is_published = true
LOOP
    SELECT id INTO v_mqa_id
    FROM module_quiz_attempts
    WHERE user_id = u_budi AND module_id = v_mod_id
    ORDER BY created_at DESC LIMIT 1;

    INSERT INTO module_quiz_attempt_answers (attempt_id, question_id, option_id, is_correct)
    SELECT
        v_mqa_id, mq.id,
        (SELECT mqo.id FROM module_quiz_options mqo
         WHERE mqo.question_id = mq.id AND mqo.is_correct = true
         ORDER BY mqo.order_index LIMIT 1),
        true
    FROM module_quizzes mq
    WHERE mq.module_id = v_mod_id AND mq.is_published = true
    ON CONFLICT (attempt_id, question_id) DO NOTHING;
END LOOP;

-- ═════════════════════════════════════════════════════════════════════════════
-- 4. LEARNING PROGRESS — SARI (sebagian Backend Module 1)
-- ═════════════════════════════════════════════════════════════════════════════

SELECT id INTO v_mod_id
FROM learning_modules WHERE role_id = r_be AND is_published = true
ORDER BY order_index LIMIT 1;

-- Selesaikan 2 submaterial pertama
INSERT INTO user_submaterial_progress (user_id, submaterial_id, completed_at)
SELECT u_sari, s.id, NOW() - INTERVAL '2 days'
FROM submaterials s
WHERE s.module_id = v_mod_id AND s.is_published = true
ORDER BY s.order_index
LIMIT 2
ON CONFLICT (user_id, submaterial_id) DO NOTHING;

-- Mini quiz passed untuk 2 submaterial tersebut
INSERT INTO submaterial_quiz_attempts (user_id, submaterial_id, score, passed)
SELECT u_sari, s.id, 100.0, true
FROM submaterials s
WHERE s.module_id = v_mod_id AND s.is_published = true
ORDER BY s.order_index
LIMIT 2;

INSERT INTO submaterial_quiz_attempt_answers (attempt_id, question_id, option_id, is_correct)
SELECT
    a.id, sq.id,
    (SELECT sqo.id FROM submaterial_quiz_options sqo
     WHERE sqo.question_id = sq.id AND sqo.is_correct = true
     ORDER BY sqo.order_index LIMIT 1),
    true
FROM submaterial_quiz_attempts a
JOIN submaterial_quizzes sq ON sq.submaterial_id = a.submaterial_id
WHERE a.user_id = u_sari
ON CONFLICT (attempt_id, question_id) DO NOTHING;

-- ═════════════════════════════════════════════════════════════════════════════
-- 5. PROJECT FILES (dummy ZIP — minimal valid ZIP, 22 bytes)
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO project_files (path, data, mime_type) VALUES
  ('demo_budi_fe.zip',   '\x504b050600000000000000000000000000000000000000000000'::bytea, 'application/zip'),
  ('demo_sari_be.zip',   '\x504b050600000000000000000000000000000000000000000000'::bytea, 'application/zip'),
  ('demo_andi_uiux.zip', '\x504b050600000000000000000000000000000000000000000000'::bytea, 'application/zip')
ON CONFLICT (path) DO NOTHING;

-- ═════════════════════════════════════════════════════════════════════════════
-- 6. PROJECT SUBMISSIONS
-- ═════════════════════════════════════════════════════════════════════════════

SELECT id INTO proj_fe_id   FROM projects WHERE role_id = r_fe;
SELECT id INTO proj_be_id   FROM projects WHERE role_id = r_be;
SELECT id INTO proj_uiux_id FROM projects WHERE role_id = r_uiux;

-- Budi: APPROVED
INSERT INTO project_submissions (
    id, user_id, project_id,
    submission_notes, file_path, file_original_name, file_size_bytes,
    status, reviewer_notes, reviewed_at, reviewed_by, submitted_at
) VALUES (
    sub_budi, u_budi, proj_fe_id,
    'Saya membangun e-commerce mini menggunakan React + Redux. Fitur: katalog produk, keranjang belanja, dan checkout. Routing dengan React Router, integrasi REST API, dan desain responsif.',
    'demo_budi_fe.zip', 'capstone_frontend_budi.zip', 22,
    'approved',
    'Project sangat baik! Implementasi routing dan state management sudah tepat. Kode bersih, folder terstruktur, dan README lengkap. Selamat, project disetujui!',
    NOW() - INTERVAL '2 days', u_admin,
    NOW() - INTERVAL '5 days'
);

-- Sari: PENDING
INSERT INTO project_submissions (
    id, user_id, project_id,
    submission_notes, file_path, file_original_name, file_size_bytes,
    status, submitted_at
) VALUES (
    sub_sari, u_sari, proj_be_id,
    'REST API dengan autentikasi JWT menggunakan Node.js + Express + PostgreSQL. Endpoint: users, posts, dan comments. Dokumentasi tersedia di README dan Postman collection terlampir.',
    'demo_sari_be.zip', 'capstone_backend_sari.zip', 22,
    'pending_review',
    NOW() - INTERVAL '1 day'
);

-- Andi: REJECTED
INSERT INTO project_submissions (
    id, user_id, project_id,
    submission_notes, file_path, file_original_name, file_size_bytes,
    status, reviewer_notes, reviewed_at, reviewed_by, submitted_at
) VALUES (
    sub_andi, u_andi, proj_uiux_id,
    'High-fidelity design untuk aplikasi e-commerce mobile menggunakan Figma. Terdiri dari 10 screen: onboarding, home, katalog, detail produk, keranjang, checkout, dan profil.',
    'demo_andi_uiux.zip', 'capstone_uiux_andi.zip', 22,
    'rejected',
    'Desain belum konsisten: ukuran font bervariasi tanpa type scale yang jelas, dan kontras warna beberapa tombol tidak memenuhi WCAG AA (rasio minimal 4.5:1). Mohon perbaiki kedua hal tersebut dan submit ulang.',
    NOW() - INTERVAL '3 days', u_admin,
    NOW() - INTERVAL '6 days'
);

-- ═════════════════════════════════════════════════════════════════════════════
-- 7. SUBMISSION REVIEW HISTORY
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO submission_review_history
    (submission_id, reviewed_by, old_status, new_status, reviewer_notes, created_at)
VALUES
  (sub_budi, u_admin, 'pending_review', 'approved',
   'Project sangat baik! Implementasi routing dan state management sudah tepat. Kode bersih, folder terstruktur, dan README lengkap. Selamat, project disetujui!',
   NOW() - INTERVAL '2 days'),
  (sub_andi, u_admin, 'pending_review', 'rejected',
   'Desain belum konsisten: ukuran font bervariasi tanpa type scale yang jelas, dan kontras warna beberapa tombol tidak memenuhi WCAG AA (rasio minimal 4.5:1). Mohon perbaiki kedua hal tersebut dan submit ulang.',
   NOW() - INTERVAL '3 days');

-- ═════════════════════════════════════════════════════════════════════════════
-- 8. AUDIT LOG
-- ═════════════════════════════════════════════════════════════════════════════

INSERT INTO admin_audit_logs (admin_id, action, target_type, target_id, metadata, created_at)
VALUES
  (u_admin, 'submission.approved', 'project_submission', sub_budi,
   '{"old_status":"pending_review","was_previously_approved":false}'::jsonb,
   NOW() - INTERVAL '2 days'),
  (u_admin, 'submission.rejected', 'project_submission', sub_andi,
   '{"old_status":"pending_review","was_previously_approved":false}'::jsonb,
   NOW() - INTERVAL '3 days'),
  (u_admin, 'submission.downloaded', 'project_submission', sub_budi,
   '{"file_size":22}'::jsonb,
   NOW() - INTERVAL '4 days');

-- ═════════════════════════════════════════════════════════════════════════════
-- 9. CERTIFICATE (Budi — selesaikan semua modul Frontend + project approved)
-- ═════════════════════════════════════════════════════════════════════════════

SELECT COUNT(*) INTO fe_mod_count
FROM learning_modules WHERE role_id = r_fe AND is_published = true;

INSERT INTO certificates (
    id, certificate_code,
    user_id, role_id,
    recipient_name, role_name, role_code,
    issued_at, modules_completed_count,
    final_project_submission_id
) VALUES (
    cert_budi, 'CERT-2024-DEMO0001',
    u_budi, r_fe,
    'Budi Santoso', 'Frontend Developer', 'frontend',
    NOW() - INTERVAL '2 days',
    fe_mod_count,
    sub_budi
);

RAISE NOTICE '✓ Demo seed selesai. 5 user, 5 quiz attempt, progress belajar, 3 submission, 1 sertifikat.';

END $$;
