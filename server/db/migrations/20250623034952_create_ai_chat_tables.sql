-- migrate:up

-- AI Conversations table
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

-- AI Messages table
CREATE TABLE ai_messages (
	id TEXT PRIMARY KEY NOT NULL,                     -- UUID as string
	conversation_id TEXT NOT NULL,                    -- UUID as string, references ai_conversations.id
	role TEXT NOT NULL,                               -- 'system', 'user', 'assistant'
	content TEXT NOT NULL,                            -- Message content
	token_count INTEGER,                              -- Optional token count
	created_at TEXT NOT NULL,                         -- ISO8601 timestamp
	metadata TEXT,                                    -- JSON string for additional data
	FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
);

-- AI Usage tracking table
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

-- Indexes for better query performance
CREATE INDEX idx_ai_conversations_user_id ON ai_conversations(user_id);
CREATE INDEX idx_ai_conversations_archived_at ON ai_conversations(archived_at);
CREATE INDEX idx_ai_messages_conversation_id ON ai_messages(conversation_id);
CREATE INDEX idx_ai_messages_created_at ON ai_messages(created_at);
CREATE INDEX idx_ai_usage_user_id ON ai_usage(user_id);
CREATE INDEX idx_ai_usage_conversation_id ON ai_usage(conversation_id);
CREATE INDEX idx_ai_usage_created_at ON ai_usage(created_at);

-- migrate:down

DROP INDEX IF EXISTS idx_ai_usage_created_at;
DROP INDEX IF EXISTS idx_ai_usage_conversation_id;
DROP INDEX IF EXISTS idx_ai_usage_user_id;
DROP INDEX IF EXISTS idx_ai_messages_created_at;
DROP INDEX IF EXISTS idx_ai_messages_conversation_id;
DROP INDEX IF EXISTS idx_ai_conversations_archived_at;
DROP INDEX IF EXISTS idx_ai_conversations_user_id;

DROP TABLE IF EXISTS ai_usage;
DROP TABLE IF EXISTS ai_messages;
DROP TABLE IF EXISTS ai_conversations;
