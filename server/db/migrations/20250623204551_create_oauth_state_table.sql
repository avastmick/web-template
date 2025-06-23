-- migrate:up
CREATE TABLE oauth_states (
    state TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,
    redirect_uri TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);

-- Index for cleanup of expired states
CREATE INDEX idx_oauth_states_expires_at ON oauth_states(expires_at);

-- Index for provider lookup
CREATE INDEX idx_oauth_states_provider ON oauth_states(provider);

-- migrate:down
DROP TABLE IF EXISTS oauth_states;
