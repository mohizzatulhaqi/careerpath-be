-- Add theft-detection columns to refresh_tokens.
-- family_id: all tokens in one rotation chain share the same ID.
-- used: TRUE once the token has been exchanged for a new one.
-- Replaying a used token triggers family revocation (all sessions for that chain).

ALTER TABLE refresh_tokens
  ADD COLUMN family_id UUID    NOT NULL DEFAULT gen_random_uuid(),
  ADD COLUMN used      BOOLEAN NOT NULL DEFAULT FALSE;

-- Fast lookup when revoking an entire family
CREATE INDEX idx_refresh_tokens_family ON refresh_tokens (family_id);
