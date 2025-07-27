-- Drop cli_auth_flows table and related objects
DROP INDEX IF EXISTS idx_cli_auth_flows_status;
DROP INDEX IF EXISTS idx_cli_auth_flows_expires_at;
DROP INDEX IF EXISTS idx_cli_auth_flows_device_id;
DROP INDEX IF EXISTS idx_cli_auth_flows_state;
DROP TABLE IF EXISTS cli_auth_flows;
