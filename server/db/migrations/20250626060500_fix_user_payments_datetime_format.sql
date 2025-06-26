-- migrate:up
-- Update the trigger to use RFC3339 format with timezone
DROP TRIGGER IF EXISTS update_user_payments_updated_at;

CREATE TRIGGER update_user_payments_updated_at
AFTER UPDATE ON user_payments
FOR EACH ROW
BEGIN
	UPDATE user_payments
	SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
	WHERE id = NEW.id;
END;

-- Update existing records to use RFC3339 format
UPDATE user_payments
SET created_at = CASE
    WHEN created_at NOT LIKE '%Z' AND created_at NOT LIKE '%+%'
    THEN created_at || 'Z'
    ELSE created_at
    END,
    updated_at = CASE
    WHEN updated_at NOT LIKE '%Z' AND updated_at NOT LIKE '%+%'
    THEN updated_at || 'Z'
    ELSE updated_at
    END,
    subscription_start_date = CASE
    WHEN subscription_start_date IS NOT NULL AND subscription_start_date NOT LIKE '%Z' AND subscription_start_date NOT LIKE '%+%'
    THEN subscription_start_date || 'Z'
    ELSE subscription_start_date
    END,
    subscription_end_date = CASE
    WHEN subscription_end_date IS NOT NULL AND subscription_end_date NOT LIKE '%Z' AND subscription_end_date NOT LIKE '%+%'
    THEN subscription_end_date || 'Z'
    ELSE subscription_end_date
    END,
    subscription_cancelled_at = CASE
    WHEN subscription_cancelled_at IS NOT NULL AND subscription_cancelled_at NOT LIKE '%Z' AND subscription_cancelled_at NOT LIKE '%+%'
    THEN subscription_cancelled_at || 'Z'
    ELSE subscription_cancelled_at
    END,
    last_payment_date = CASE
    WHEN last_payment_date IS NOT NULL AND last_payment_date NOT LIKE '%Z' AND last_payment_date NOT LIKE '%+%'
    THEN last_payment_date || 'Z'
    ELSE last_payment_date
    END;

-- migrate:down
DROP TRIGGER IF EXISTS update_user_payments_updated_at;

CREATE TRIGGER update_user_payments_updated_at
AFTER UPDATE ON user_payments
FOR EACH ROW
BEGIN
	UPDATE user_payments
	SET updated_at = datetime('now', 'utc')
	WHERE id = NEW.id;
END;
