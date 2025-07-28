CREATE TABLE ai_conversations (
	id TEXT PRIMARY KEY NOT NULL,                     -- UUID as string
	user_id TEXT NOT NULL,                            -- UUID as string, references users.id
	title TEXT,                                       -- Optional conversation title
	model TEXT NOT NULL,                              -- AI model name used
	system_prompt TEXT,                               -- Optional system prompt
	created_at TEXT NOT NULL,                         -- ISO8601 timestamp
	updated_at TEXT NOT NULL,                         -- ISO8601 timestamp
	archived_at TEXT,                                 -- ISO8601 timestamp for soft delete
	metadata TEXT,                                    -- JSON string for additional data
	FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_conversations_user_id ON ai_conversations(user_id);
CREATE INDEX idx_ai_conversations_archived_at ON ai_conversations(archived_at);
