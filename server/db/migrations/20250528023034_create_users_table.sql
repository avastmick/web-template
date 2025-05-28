-- web-template/db/migrations/20250528023034_create_users_table.sql

-- migrate:up
CREATE TABLE users (
    id TEXT PRIMARY KEY, -- UUID stored as TEXT, ensure application provides this
    email TEXT UNIQUE NOT NULL,
    hashed_password TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, -- Stores as ISO8601 string
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP  -- Stores as ISO8601 string
);

-- Index on email for faster lookups
CREATE INDEX idx_users_email ON users(email);

-- For SQLite, updated_at trigger is usually handled by the application.
-- If using PostgreSQL later, you can add a trigger:
-- CREATE OR REPLACE FUNCTION trigger_set_timestamp()
-- RETURNS TRIGGER AS $$
-- BEGIN
--   NEW.updated_at = NOW();
--   RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;
-- CREATE TRIGGER set_users_updated_at
-- BEFORE UPDATE ON users
-- FOR EACH ROW
-- EXECUTE FUNCTION trigger_set_timestamp();

-- migrate:down
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users;

-- If using PostgreSQL and added the trigger:
-- DROP TRIGGER IF EXISTS set_users_updated_at ON users;
-- DROP FUNCTION IF EXISTS trigger_set_timestamp();
