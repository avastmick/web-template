-- Create cli_auth_flows table for managing OAuth flows with PKCE
CREATE TABLE cli_auth_flows (
    id TEXT PRIMARY KEY NOT NULL,                    -- UUID
    device_id TEXT NOT NULL,                         -- Foreign key to cli_devices
    state TEXT NOT NULL UNIQUE,                      -- OAuth state parameter for CSRF protection
    code_challenge TEXT NOT NULL,                    -- PKCE code challenge (SHA256 of verifier)
    challenge_method TEXT NOT NULL DEFAULT 'S256',   -- PKCE challenge method (always S256)
    status TEXT NOT NULL DEFAULT 'pending',          -- Flow status: pending, completed, expired
    auth_code TEXT,                                  -- OAuth authorization code (set after auth)
    expires_at DATETIME NOT NULL,                    -- Flow expiration time
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,                           -- When the flow was completed
    FOREIGN KEY (device_id) REFERENCES cli_devices(id) ON DELETE CASCADE,
    CHECK (status IN ('pending', 'completed', 'expired')),
    CHECK (challenge_method = 'S256')
);

-- Indexes for efficient querying
CREATE INDEX idx_cli_auth_flows_state ON cli_auth_flows(state);
CREATE INDEX idx_cli_auth_flows_device_id ON cli_auth_flows(device_id);
CREATE INDEX idx_cli_auth_flows_expires_at ON cli_auth_flows(expires_at);
CREATE INDEX idx_cli_auth_flows_status ON cli_auth_flows(status) WHERE status = 'pending';
