-- migrate:up
-- Update all existing invites to expire 12 months from now
UPDATE user_invites
SET expires_at = datetime('now', '+12 months')
WHERE expires_at IS NULL
  AND used_at IS NULL;

-- migrate:down
-- Reset expiry dates to NULL
UPDATE user_invites
SET expires_at = NULL
WHERE expires_at = datetime('now', '+12 months');
