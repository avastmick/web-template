-- Create cli_devices table for tracking CLI installations
CREATE TABLE cli_devices (
    id TEXT PRIMARY KEY NOT NULL,                    -- UUID
    user_id TEXT NOT NULL,                           -- Foreign key to users table
    device_name TEXT NOT NULL,                       -- User-friendly device name
    device_fingerprint TEXT NOT NULL,                -- Hardware/system fingerprint for device identification
    last_used_at DATETIME,                           -- Last activity timestamp
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    revoked_at DATETIME,                             -- When device was revoked (if applicable)
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Indexes for efficient querying
CREATE INDEX idx_cli_devices_user_id ON cli_devices(user_id);
CREATE INDEX idx_cli_devices_last_used_at ON cli_devices(last_used_at);
CREATE INDEX idx_cli_devices_fingerprint ON cli_devices(device_fingerprint);
-- Index for finding active devices (not revoked)
CREATE INDEX idx_cli_devices_active ON cli_devices(user_id, device_fingerprint)
    WHERE revoked_at IS NULL;
