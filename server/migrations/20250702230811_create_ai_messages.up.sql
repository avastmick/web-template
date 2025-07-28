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

CREATE INDEX idx_ai_messages_conversation_id ON ai_messages(conversation_id);
CREATE INDEX idx_ai_messages_created_at ON ai_messages(created_at);
