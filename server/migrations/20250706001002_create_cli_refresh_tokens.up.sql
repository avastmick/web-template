-- Create cli_refresh_tokens table for managing refresh tokens
CREATE TABLE cli_refresh_tokens (
    id TEXT PRIMARY KEY NOT NULL,                    -- UUID
    device_id TEXT NOT NULL,                         -- Foreign key to cli_devices
    token_hash TEXT NOT NULL UNIQUE,                 -- SHA256 hash of the refresh token
    expires_at DATETIME NOT NULL,                    -- Token expiration time
    last_used_at DATETIME,                           -- Last time token was used
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at DATETIME,                             -- When token was revoked (if applicable)
    FOREIGN KEY (device_id) REFERENCES cli_devices(id) ON DELETE CASCADE
);

-- Indexes for efficient querying
CREATE INDEX idx_cli_refresh_tokens_token_hash ON cli_refresh_tokens(token_hash);
CREATE INDEX idx_cli_refresh_tokens_device_id ON cli_refresh_tokens(device_id);
CREATE INDEX idx_cli_refresh_tokens_expires_at ON cli_refresh_tokens(expires_at);
-- Index for finding active tokens (not revoked, not expired)
CREATE INDEX idx_cli_refresh_tokens_active ON cli_refresh_tokens(device_id, expires_at)
    WHERE revoked_at IS NULL;
