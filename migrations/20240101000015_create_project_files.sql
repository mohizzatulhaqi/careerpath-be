-- Stores uploaded project submission files as binary data in the database.
-- Used when STORAGE_BACKEND=database (no external storage service required).
CREATE TABLE project_files (
    path        VARCHAR(500) PRIMARY KEY,
    data        BYTEA       NOT NULL,
    mime_type   VARCHAR(100) NOT NULL DEFAULT 'application/zip',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
