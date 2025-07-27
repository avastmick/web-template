CREATE TABLE user_invites (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT UNIQUE NOT NULL COLLATE NOCASE,
    invited_by TEXT,
    invited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    used_at DATETIME,
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_invites_email ON user_invites(email);
CREATE INDEX idx_user_invites_used_at ON user_invites(used_at) WHERE used_at IS NULL;
CREATE INDEX idx_user_invites_expires_at ON user_invites(expires_at) WHERE expires_at IS NOT NULL AND used_at IS NULL;
