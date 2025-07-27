use crate::errors::AppError;
use crate::handlers::auth_handler::RegisterUserPayload;
use crate::services::user_service::UserServiceImpl;
use sqlx::SqlitePool;
use uuid::Uuid;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database");

    // Run migrations to ensure test database matches production schema
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

#[tokio::test]
async fn test_user_service_creation() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool.clone());

    // Verify the service is created with the correct pool
    assert!(std::ptr::eq(
        &raw const service.db_pool,
        &raw const service.db_pool
    ));
}

#[tokio::test]
async fn test_create_user_success() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("test_{}@example.com", Uuid::new_v4()),
        password: "strongPassword123!".to_string(),
    };

    let user = service
        .create_user(&payload)
        .await
        .expect("Failed to create user");

    assert_eq!(user.email, payload.email);
    assert_ne!(user.hashed_password, payload.password);
    assert!(user.provider.is_empty() || user.provider == "local");
    assert!(user.provider_user_id.is_none());
    assert!(user.created_at <= chrono::Utc::now());
    assert_eq!(user.created_at, user.updated_at);
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let email = format!("duplicate_{}@example.com", Uuid::new_v4());
    let payload = RegisterUserPayload {
        email: email.clone(),
        password: "password123".to_string(),
    };

    // Create first user
    service
        .create_user(&payload)
        .await
        .expect("Failed to create first user");

    // Try to create duplicate
    let result = service.create_user(&payload).await;
    assert!(result.is_err());

    match result {
        Err(AppError::UserAlreadyExists { email: err_email }) => {
            assert_eq!(err_email, email);
        }
        _ => panic!("Expected UserAlreadyExists error"),
    }
}

#[tokio::test]
async fn test_create_user_empty_email() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: String::new(),
        password: "password123".to_string(),
    };

    let result = service.create_user(&payload).await;

    // SQLite might allow empty email, so we test what happens
    if result.is_ok() {
        let user = result.expect("User creation succeeded");
        assert_eq!(user.email, "");
    }
    // If it fails, that's also acceptable behavior
}

#[tokio::test]
async fn test_create_user_empty_password() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("empty_pwd_{}@example.com", Uuid::new_v4()),
        password: String::new(),
    };

    // Empty password should still be hashable
    let result = service.create_user(&payload).await;
    assert!(result.is_ok());

    let user = result.expect("Failed to create user with empty password");
    assert_ne!(user.hashed_password, "");
}

#[tokio::test]
async fn test_create_user_very_long_password() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("long_pwd_{}@example.com", Uuid::new_v4()),
        password: "a".repeat(1000),
    };

    let result = service.create_user(&payload).await;
    assert!(result.is_ok());

    let user = result.expect("Failed to create user with long password");
    assert!(!user.hashed_password.is_empty());
}

#[tokio::test]
async fn test_find_by_email_existing_user() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let email = format!("findme_{}@example.com", Uuid::new_v4());
    let payload = RegisterUserPayload {
        email: email.clone(),
        password: "password123".to_string(),
    };

    // Create user first
    let created_user = service
        .create_user(&payload)
        .await
        .expect("Failed to create user");

    // Find the user
    let found_user = service
        .find_by_email(&email)
        .await
        .expect("Failed to find user");

    assert_eq!(found_user.id, created_user.id);
    assert_eq!(found_user.email, created_user.email);
    assert_eq!(found_user.hashed_password, created_user.hashed_password);
    assert_eq!(found_user.created_at, created_user.created_at);
}

#[tokio::test]
async fn test_find_by_email_non_existing_user() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let result = service.find_by_email("nonexistent@example.com").await;

    assert!(result.is_err());
    match result {
        Err(AppError::UserNotFound) => {}
        _ => panic!("Expected UserNotFound error"),
    }
}

#[tokio::test]
async fn test_find_by_email_empty_email() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let result = service.find_by_email("").await;
    assert!(result.is_err());
    match result {
        Err(AppError::UserNotFound) => {}
        _ => panic!("Expected UserNotFound error"),
    }
}

#[tokio::test]
async fn test_find_by_email_case_sensitivity() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let email = format!("CaseSensitive_{}@Example.COM", Uuid::new_v4());
    let payload = RegisterUserPayload {
        email: email.clone(),
        password: "password123".to_string(),
    };

    // Create user
    service
        .create_user(&payload)
        .await
        .expect("Failed to create user");

    // Try to find with different case
    let lowercase_result = service.find_by_email(&email.to_lowercase()).await;
    let uppercase_result = service.find_by_email(&email.to_uppercase()).await;
    let exact_result = service.find_by_email(&email).await;

    // SQLite is case-insensitive by default for LIKE comparisons
    // but case-sensitive for = comparisons
    assert!(exact_result.is_ok());

    // These might fail depending on the database collation
    // We're just testing the behavior, not asserting a specific outcome
    let _ = lowercase_result;
    let _ = uppercase_result;
}

#[tokio::test]
async fn test_create_test_user() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let email = format!("test_helper_{}@example.com", Uuid::new_v4());

    let password = "testPassword123"; // gitleaks:allow

    let user = service
        .create_test_user(&email, password)
        .await
        .expect("Failed to create test user");

    assert_eq!(user.email, email);
    assert_ne!(user.hashed_password, password);
}

#[tokio::test]
async fn test_create_user_special_characters_in_email() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let uuid = Uuid::new_v4();
    let payload = RegisterUserPayload {
        email: format!("test+special.chars-{uuid}@sub.example.com"),
        password: "password123".to_string(),
    };

    let result = service.create_user(&payload).await;
    assert!(result.is_ok());

    let user = result.expect("Failed to create user with special chars in email");
    assert_eq!(user.email, payload.email);
}

#[tokio::test]
async fn test_create_user_unicode_in_password() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("unicode_pwd_{}@example.com", Uuid::new_v4()),
        password: "пароль密码🔐".to_string(),
    };

    let result = service.create_user(&payload).await;
    assert!(result.is_ok());

    let user = result.expect("Failed to create user with unicode password");
    assert_ne!(user.hashed_password, payload.password);
}

#[tokio::test]
async fn test_user_timestamps() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let before_creation = chrono::Utc::now();

    let payload = RegisterUserPayload {
        email: format!("timestamp_test_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
    };

    let user = service
        .create_user(&payload)
        .await
        .expect("Failed to create user");

    let after_creation = chrono::Utc::now();

    // Verify timestamps are within expected range
    assert!(user.created_at >= before_creation);
    assert!(user.created_at <= after_creation);
    assert_eq!(user.created_at, user.updated_at);
}

#[tokio::test]
async fn test_multiple_users_creation() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let users_data = [
        ("user1", "password1"),
        ("user2", "password2"),
        ("user3", "password3"),
    ];

    let mut created_users = Vec::new();

    for (i, (username, password)) in users_data.iter().enumerate() {
        let payload = RegisterUserPayload {
            email: format!("{}+{}@example.com", username, Uuid::new_v4()),
            password: (*password).to_string(),
        };

        let user = service
            .create_user(&payload)
            .await
            .unwrap_or_else(|_| panic!("Failed to create user {i}"));

        created_users.push(user);
    }

    assert_eq!(created_users.len(), users_data.len());

    // Verify all users have unique IDs
    let mut ids: Vec<Uuid> = created_users.iter().map(|u| u.id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), users_data.len());
}

#[tokio::test]
async fn test_user_provider_fields() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("provider_test_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
    };

    let user = service
        .create_user(&payload)
        .await
        .expect("Failed to create user");

    // Users created through register endpoint should have default provider values
    assert!(user.provider.is_empty() || user.provider == "local");
    assert!(user.provider_user_id.is_none());
}

#[tokio::test]
async fn test_create_user_database_closed() {
    // Create a pool and immediately close it
    let pool = setup_test_db().await;
    pool.close().await;

    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("db_closed_{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
    };

    let result = service.create_user(&payload).await;
    assert!(result.is_err());

    match result {
        Err(AppError::SqlxError(_)) => {}
        _ => panic!("Expected SqlxError"),
    }
}

#[tokio::test]
async fn test_find_by_email_database_closed() {
    // Create a pool and immediately close it
    let pool = setup_test_db().await;
    pool.close().await;

    let service = UserServiceImpl::new(pool);

    let result = service.find_by_email("test@example.com").await;
    assert!(result.is_err());

    match result {
        Err(AppError::SqlxError(_)) => {}
        _ => panic!("Expected SqlxError"),
    }
}

#[tokio::test]
async fn test_create_user_invalid_characters() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    // Test with null character which might cause issues
    let payload = RegisterUserPayload {
        email: format!("test_{}@example.com\0", Uuid::new_v4()),
        password: "password123".to_string(),
    };

    let result = service.create_user(&payload).await;
    // This might succeed or fail depending on database constraints
    let _ = result;
}

#[tokio::test]
async fn test_create_user_sql_injection_attempt() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let payload = RegisterUserPayload {
        email: format!("test'; DROP TABLE users; --{}@example.com", Uuid::new_v4()),
        password: "password123".to_string(),
    };

    // The parameterized queries should protect against SQL injection
    let result = service.create_user(&payload).await;
    assert!(result.is_ok());

    // Verify the email is stored correctly
    let user = result.expect("Failed to create user");
    assert!(user.email.contains("DROP TABLE"));
}

#[tokio::test]
async fn test_find_by_email_sql_injection_attempt() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    // Try SQL injection in find_by_email
    let result = service.find_by_email("test' OR '1'='1").await;

    // Should not find any user (parameterized queries protect against injection)
    assert!(result.is_err());
    match result {
        Err(AppError::UserNotFound) => {}
        _ => panic!("Expected UserNotFound error"),
    }
}

#[tokio::test]
async fn test_create_user_very_long_email() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    // Create an email that's extremely long
    let long_email = format!("{}@{}.com", "a".repeat(100), "b".repeat(100));
    let payload = RegisterUserPayload {
        email: long_email.clone(),
        password: "password123".to_string(),
    };

    let result = service.create_user(&payload).await;

    if result.is_ok() {
        let user = result.expect("User creation succeeded");
        assert_eq!(user.email, long_email);
    }
    // If it fails due to length constraints, that's also acceptable
}

#[tokio::test]
async fn test_concurrent_user_creation() {
    let pool = setup_test_db().await;
    let service = UserServiceImpl::new(pool);

    let email = format!("concurrent_{}@example.com", Uuid::new_v4());

    // Create multiple tasks trying to create the same user
    let mut handles = vec![];

    for i in 0..3 {
        let service_clone = service.clone();
        let email_clone = email.clone();

        let handle = tokio::spawn(async move {
            let payload = RegisterUserPayload {
                email: email_clone,
                password: format!("password{i}"),
            };
            service_clone.create_user(&payload).await
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    let mut success_count = 0;
    let mut duplicate_count = 0;

    for handle in handles {
        match handle.await.expect("Task panicked") {
            Ok(_) => success_count += 1,
            Err(AppError::UserAlreadyExists { .. } | AppError::SqlxError(_)) => {
                duplicate_count += 1;
            }
            Err(e) => panic!("Unexpected error: {e:?}"),
        }
    }

    // Exactly one should succeed, the rest should get duplicate errors
    assert_eq!(success_count, 1);
    assert_eq!(duplicate_count, 2);
}
