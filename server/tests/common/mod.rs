//! Common test utilities and database setup
//!
//! This module provides shared functionality for integration tests,
//! using `SQLx` migrations to ensure test databases match production schema.

use sqlx::{Pool, Sqlite, SqlitePool, migrate::MigrateDatabase};

/// Create a test database with all migrations applied
///
/// This function creates an in-memory `SQLite` database and runs all migrations
/// to ensure the test schema exactly matches production.
pub async fn setup_test_database() -> Pool<Sqlite> {
    // Create a unique in-memory database for this test
    let url = "sqlite::memory:";

    // Create the database
    if !Sqlite::database_exists(url).await.unwrap_or(false) {
        Sqlite::create_database(url)
            .await
            .expect("Failed to create test database");
    }

    // Connect to the database
    let pool = SqlitePool::connect(url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}

pub mod test_context;
#[allow(unused_imports)]
pub use test_context::TestContext;
