CREATE TABLE ai_usage (
	id TEXT PRIMARY KEY NOT NULL,                     -- UUID as string
	conversation_id TEXT,                             -- UUID as string, references ai_conversations.id (optional)
	user_id TEXT NOT NULL,                            -- UUID as string, references users.id
	model TEXT NOT NULL,                              -- AI model name used
	prompt_tokens INTEGER NOT NULL,                   -- Number of tokens in prompt
	completion_tokens INTEGER NOT NULL,               -- Number of tokens in completion
	total_tokens INTEGER NOT NULL,                    -- Total tokens used
	cost_cents INTEGER,                               -- Cost in cents (optional)
	created_at TEXT NOT NULL,                         -- ISO8601 timestamp
	request_id TEXT,                                  -- Optional request identifier
	duration_ms INTEGER,                              -- Optional duration in milliseconds
	FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE SET NULL,
	FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_ai_usage_user_id ON ai_usage(user_id);
CREATE INDEX idx_ai_usage_conversation_id ON ai_usage(conversation_id);
CREATE INDEX idx_ai_usage_created_at ON ai_usage(created_at);
