-- migrate:up
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

-- Index for fast email lookups
CREATE INDEX idx_user_invites_email ON user_invites(email);

-- Index for finding unused invites
CREATE INDEX idx_user_invites_used_at ON user_invites(used_at) WHERE used_at IS NULL;

-- Index for finding expired invites
CREATE INDEX idx_user_invites_expires_at ON user_invites(expires_at) WHERE expires_at IS NOT NULL;

-- Trigger to update the updated_at timestamp
CREATE TRIGGER update_user_invites_updated_at
    AFTER UPDATE ON user_invites
    FOR EACH ROW
    WHEN OLD.updated_at = NEW.updated_at OR OLD.updated_at IS NULL
BEGIN
    UPDATE user_invites SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;

-- migrate:down
DROP TABLE IF EXISTS user_invites;
