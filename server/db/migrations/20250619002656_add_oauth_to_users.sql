-- migrate:up
-- Add OAuth support to users table
ALTER TABLE users ADD COLUMN provider TEXT NOT NULL DEFAULT 'local';
ALTER TABLE users ADD COLUMN provider_user_id TEXT;

-- Create index for OAuth lookups
CREATE INDEX idx_users_provider_oauth ON users(provider, provider_user_id) WHERE provider != 'local';

-- migrate:down
-- Remove OAuth support from users table
DROP INDEX IF EXISTS idx_users_provider_oauth;
ALTER TABLE users DROP COLUMN provider_user_id;
ALTER TABLE users DROP COLUMN provider;
