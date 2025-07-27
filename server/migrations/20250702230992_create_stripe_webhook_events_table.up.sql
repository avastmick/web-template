-- Create stripe_webhook_events table to track and handle webhook events
CREATE TABLE IF NOT EXISTS stripe_webhook_events (
	id TEXT PRIMARY KEY NOT NULL,
	stripe_event_id TEXT UNIQUE NOT NULL,
	event_type TEXT NOT NULL,
	processed BOOLEAN NOT NULL DEFAULT FALSE,
	processing_attempts INTEGER NOT NULL DEFAULT 0,
	last_error TEXT,
	event_data TEXT NOT NULL, -- JSON string containing the full event data
	created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
	processed_at TEXT
);

-- Create indexes for performance
CREATE INDEX idx_stripe_webhook_events_stripe_event_id ON stripe_webhook_events(stripe_event_id);
CREATE INDEX idx_stripe_webhook_events_event_type ON stripe_webhook_events(event_type);
CREATE INDEX idx_stripe_webhook_events_processed ON stripe_webhook_events(processed);
CREATE INDEX idx_stripe_webhook_events_created_at ON stripe_webhook_events(created_at);
