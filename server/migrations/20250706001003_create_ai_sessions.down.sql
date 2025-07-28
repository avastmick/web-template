-- Drop trigger
DROP TRIGGER IF EXISTS update_ai_sessions_updated_at;

-- Drop indexes
DROP INDEX IF EXISTS idx_ai_session_assets_session_id;
DROP INDEX IF EXISTS idx_ai_session_messages_session_id;
DROP INDEX IF EXISTS idx_ai_sessions_status;
DROP INDEX IF EXISTS idx_ai_sessions_expires_at;
DROP INDEX IF EXISTS idx_ai_sessions_user_id;

-- Drop tables in reverse order due to foreign key constraints
DROP TABLE IF EXISTS ai_session_assets;
DROP TABLE IF EXISTS ai_session_messages;
DROP TABLE IF EXISTS ai_sessions;
