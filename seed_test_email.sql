-- ============================================================================
-- Seed: Test Email Notification
-- Buat user + submission PENDING agar bisa di-approve/reject dari admin
-- dan mengetes pengiriman email ke mihkiki489@gmail.com
--
-- Jalankan:
--   psql "$DATABASE_URL" -f seed_test_email.sql
-- ============================================================================

DO $$
DECLARE
  u_test   UUID := '00000005-0000-0000-0000-000000000001';
  sub_test UUID := '00000005-0000-0000-0000-000000000002';
  r_fe     UUID := '00000001-0000-0000-0000-000000000001';
  proj_fe_id UUID;
BEGIN

IF EXISTS (SELECT 1 FROM users WHERE id = u_test) THEN
    RAISE NOTICE 'Test email user sudah ada — skipping.';
    RETURN;
END IF;

-- User dengan email asli untuk tes notifikasi
INSERT INTO users (id, email, password_hash, name, role, is_active) VALUES (
    u_test,
    'mihkiki489@gmail.com',
    '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
    'Moh. Izzatul Haqi',
    'user',
    true
);

-- Dummy file ZIP
INSERT INTO project_files (path, data, mime_type) VALUES (
    'test_email_submission.zip',
    '\x504b050600000000000000000000000000000000000000000000'::bytea,
    'application/zip'
) ON CONFLICT (path) DO NOTHING;

-- Submission PENDING — tinggal approve/reject dari admin panel
SELECT id INTO proj_fe_id FROM projects WHERE role_id = r_fe;

INSERT INTO project_submissions (
    id, user_id, project_id,
    submission_notes, file_path, file_original_name, file_size_bytes,
    status, submitted_at
) VALUES (
    sub_test,
    u_test,
    proj_fe_id,
    'Ini adalah submission untuk tes notifikasi email. Tolong approve atau reject untuk memverifikasi email terkirim ke mihkiki489@gmail.com.',
    'test_email_submission.zip',
    'test_submission.zip',
    22,
    'pending_review',
    NOW()
);

RAISE NOTICE '✓ User mihkiki489@gmail.com + submission pending siap. Buka admin panel dan approve/reject submission ID: %', sub_test;

END $$;
