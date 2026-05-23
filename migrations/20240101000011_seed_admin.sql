-- Seed default admin user.
-- Password: ChangeMeASAP123!  ← WAJIB GANTI SETELAH FIRST LOGIN

INSERT INTO users (id, email, password_hash, name, role, is_active, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'admin@careerpath.local',
    '$argon2id$v=19$m=19456,t=2,p=1$PQCqgwKrqIKtpXmBcdlNgQ$rFbmPWGDGGMyPDPf66kf9/51L0xDSQyq/RnQtYWQflw',
    'System Administrator',
    'admin',
    true,
    NOW(),
    NOW()
) ON CONFLICT (id) DO NOTHING;
