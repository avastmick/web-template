CREATE TABLE oauth_states (
    state TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    redirect_uri TEXT,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL
);

CREATE INDEX idx_oauth_states_expires_at ON oauth_states(expires_at);
CREATE INDEX idx_oauth_states_provider ON oauth_states(provider);
