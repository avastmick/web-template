-- Drop cli_refresh_tokens table and related objects
DROP INDEX IF EXISTS idx_cli_refresh_tokens_active;
DROP INDEX IF EXISTS idx_cli_refresh_tokens_expires_at;
DROP INDEX IF EXISTS idx_cli_refresh_tokens_device_id;
DROP INDEX IF EXISTS idx_cli_refresh_tokens_token_hash;
DROP TABLE IF EXISTS cli_refresh_tokens;
