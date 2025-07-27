-- Drop cli_devices table and related objects
DROP INDEX IF EXISTS idx_cli_devices_active;
DROP INDEX IF EXISTS idx_cli_devices_fingerprint;
DROP INDEX IF EXISTS idx_cli_devices_last_used_at;
DROP INDEX IF EXISTS idx_cli_devices_user_id;
DROP TABLE IF EXISTS cli_devices;
