-- migrate:up
-- Create user_payments table to track payment status for non-invited users
CREATE TABLE IF NOT EXISTS user_payments (
	id TEXT PRIMARY KEY NOT NULL,
	user_id TEXT NOT NULL,
	stripe_customer_id TEXT UNIQUE,
	stripe_subscription_id TEXT,
	stripe_payment_intent_id TEXT,
	payment_status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'active', 'cancelled', 'expired', 'failed'
	payment_type TEXT NOT NULL DEFAULT 'subscription', -- 'subscription', 'one_time'
	amount_cents INTEGER,
	currency TEXT DEFAULT 'usd',
	subscription_start_date TEXT,
	subscription_end_date TEXT,
	subscription_cancelled_at TEXT,
	last_payment_date TEXT,
	created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
	updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
	FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for performance
CREATE INDEX idx_user_payments_user_id ON user_payments(user_id);
CREATE INDEX idx_user_payments_stripe_customer_id ON user_payments(stripe_customer_id);
CREATE INDEX idx_user_payments_stripe_subscription_id ON user_payments(stripe_subscription_id);
CREATE INDEX idx_user_payments_payment_status ON user_payments(payment_status);
CREATE INDEX idx_user_payments_subscription_end_date ON user_payments(subscription_end_date);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_user_payments_updated_at
AFTER UPDATE ON user_payments
FOR EACH ROW
BEGIN
	UPDATE user_payments
	SET updated_at = datetime('now', 'utc')
	WHERE id = NEW.id;
END;

-- migrate:down
DROP TRIGGER IF EXISTS update_user_payments_updated_at;
DROP TABLE IF EXISTS user_payments;
