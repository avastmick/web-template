//! Common test utilities and database setup
//!
//! This module provides shared functionality for integration tests,
//! including database setup that mirrors the production schema.
//!
//! ## Maintainability Note
//!
//! While this approach uses manual table creation instead of running migrations,
//! it provides several benefits for testing:
//!
//! 1. **Performance**: In-memory databases with manual schema creation are much faster
//! 2. **Isolation**: Each test gets a completely fresh database without migration state
//! 3. **Reliability**: Avoids `SQLx` migration tracking issues in concurrent tests
//! 4. **Single Source of Truth**: This schema represents the final state after all migrations
//!
//! **IMPORTANT**: When adding new migrations, you MUST update this schema to match
//! the final result. The schema here should always represent what the database
//! looks like after applying ALL migrations in production.
//!
//! To verify schema consistency:
//! 1. Run `just db-setup` to apply all migrations to a fresh database
//! 2. Run `just db-status` to see the final schema
//! 3. Update this file to match the production schema
//! 4. Run `just test` to ensure all tests pass

use sqlx::{Pool, Sqlite, SqlitePool};

/// Create users table with OAuth support
async fn create_users_table(pool: &Pool<Sqlite>) {
    sqlx::query(
        r"
        CREATE TABLE users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            hashed_password TEXT NOT NULL,
            provider TEXT NOT NULL DEFAULT 'local',
            provider_user_id TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_users_email ON users(email);
        CREATE INDEX idx_users_provider_oauth ON users(provider, provider_user_id) WHERE provider != 'local';
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create users table in test database");
}

/// Create `user_invites` table
async fn create_user_invites_table(pool: &Pool<Sqlite>) {
    sqlx::query(
        r"
        CREATE TABLE user_invites (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL COLLATE NOCASE,
            invited_by TEXT,
            invited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            used_at DATETIME,
            expires_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_user_invites_email ON user_invites(email);
        CREATE INDEX idx_user_invites_used_at ON user_invites(used_at) WHERE used_at IS NULL;
        CREATE INDEX idx_user_invites_expires_at ON user_invites(expires_at) WHERE expires_at IS NOT NULL;
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create user_invites table in test database");
}

/// Create AI-related tables
async fn create_ai_tables(pool: &Pool<Sqlite>) {
    sqlx::query(
        r"
        CREATE TABLE ai_conversations (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            title TEXT,
            model TEXT NOT NULL,
            system_prompt TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            archived_at TEXT,
            metadata TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE ai_messages (
            id TEXT PRIMARY KEY NOT NULL,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            token_count INTEGER,
            created_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
        );

        CREATE TABLE ai_usage (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            conversation_id TEXT,
            model TEXT NOT NULL,
            prompt_tokens INTEGER NOT NULL,
            completion_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            cost_cents INTEGER,
            request_id TEXT,
            duration_ms INTEGER,
            created_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE SET NULL
        );

        CREATE INDEX idx_ai_conversations_user_id ON ai_conversations(user_id);
        CREATE INDEX idx_ai_conversations_created_at ON ai_conversations(created_at);
        CREATE INDEX idx_ai_conversations_archived ON ai_conversations(archived_at) WHERE archived_at IS NOT NULL;

        CREATE INDEX idx_ai_messages_conversation_id ON ai_messages(conversation_id);
        CREATE INDEX idx_ai_messages_created_at ON ai_messages(created_at);

        CREATE INDEX idx_ai_usage_user_id ON ai_usage(user_id);
        CREATE INDEX idx_ai_usage_model ON ai_usage(model);
        CREATE INDEX idx_ai_usage_created_at ON ai_usage(created_at);
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create AI tables in test database");
}

/// Create OAuth states table
async fn create_oauth_states_table(pool: &Pool<Sqlite>) {
    sqlx::query(
        r"
        CREATE TABLE oauth_states (
            state TEXT PRIMARY KEY NOT NULL,
            provider TEXT NOT NULL,
            redirect_uri TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            expires_at DATETIME NOT NULL
        );
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create oauth_states table in test database");
}

/// Create `user_payments` table
async fn create_user_payments_table(pool: &Pool<Sqlite>) {
    sqlx::query(
        r"
        CREATE TABLE user_payments (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            stripe_customer_id TEXT UNIQUE,
            stripe_subscription_id TEXT,
            stripe_payment_intent_id TEXT,
            payment_status TEXT NOT NULL DEFAULT 'pending',
            payment_type TEXT NOT NULL DEFAULT 'subscription',
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
        CREATE INDEX idx_user_payments_user_id ON user_payments(user_id);
        CREATE INDEX idx_user_payments_stripe_customer_id ON user_payments(stripe_customer_id);
        CREATE INDEX idx_user_payments_payment_status ON user_payments(payment_status);
        CREATE INDEX idx_user_payments_subscription_end_date ON user_payments(subscription_end_date) WHERE subscription_end_date IS NOT NULL;
        ",
    )
    .execute(pool)
    .await
    .expect("Failed to create user_payments table in test database");
}

/// Create a test database with all tables and indexes
///
/// This function creates all tables with their final schema as they would appear
/// after all migrations have been applied. This ensures test databases match
/// production while avoiding migration complexity in tests.
pub async fn setup_test_database() -> Pool<Sqlite> {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    create_users_table(&pool).await;
    create_user_invites_table(&pool).await;
    create_ai_tables(&pool).await;
    create_oauth_states_table(&pool).await;
    create_user_payments_table(&pool).await;

    pool
}
