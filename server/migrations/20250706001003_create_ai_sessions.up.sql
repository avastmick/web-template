-- Create AI sessions table for conversational issue creation
CREATE TABLE ai_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('issue_creation', 'issue_update')),
    status TEXT NOT NULL CHECK (status IN ('active', 'completed', 'cancelled', 'expired')),
    context TEXT NOT NULL, -- JSON string storing conversation context
    draft TEXT, -- JSON string storing current issue draft
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    expires_at TIMESTAMP NOT NULL, -- For automatic cleanup
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index for user lookups
CREATE INDEX idx_ai_sessions_user_id ON ai_sessions(user_id);

-- Create index for cleanup queries
CREATE INDEX idx_ai_sessions_expires_at ON ai_sessions(expires_at) WHERE status = 'active';

-- Create index for status queries
CREATE INDEX idx_ai_sessions_status ON ai_sessions(status);

-- Create AI session messages table
CREATE TABLE ai_session_messages (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'function')),
    content TEXT NOT NULL,
    function_call TEXT, -- JSON string for function calls
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES ai_sessions(id) ON DELETE CASCADE
);

-- Create index for session message lookups
CREATE INDEX idx_ai_session_messages_session_id ON ai_session_messages(session_id);

-- Create AI session assets table
CREATE TABLE ai_session_assets (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    file_path TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size INTEGER NOT NULL CHECK (size > 0),
    metadata TEXT, -- JSON string for additional metadata
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES ai_sessions(id) ON DELETE CASCADE
);

-- Create index for session asset lookups
CREATE INDEX idx_ai_session_assets_session_id ON ai_session_assets(session_id);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_ai_sessions_updated_at
    AFTER UPDATE ON ai_sessions
    FOR EACH ROW
    WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ai_sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
