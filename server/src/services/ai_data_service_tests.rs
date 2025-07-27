//! Tests for AI data service

#[cfg(test)]
mod tests {
    use crate::errors::AppError;
    use crate::models::{CreateConversationRequest, CreateMessageRequest};
    use crate::services::AiDataService;
    use crate::services::ai_data_service::UsageRecord;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    // Helper function to create a test user
    async fn create_test_user(pool: &SqlitePool) -> String {
        let user_id = Uuid::new_v4().to_string();
        let email = format!("test+{user_id}@example.com");
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, hashed_password, provider, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            user_id,
            email,
            "hashed_password",
            "local",
            now,
            now
        )
        .execute(pool)
        .await
        .expect("Failed to create test user");

        user_id
    }

    #[sqlx::test]
    async fn test_create_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("Test Conversation".to_string()),
            system_prompt: Some("You are a helpful assistant".to_string()),
        };

        let result = service
            .create_conversation(&user_id, request)
            .await
            .expect("Failed to create conversation");

        assert_eq!(result.title, Some("Test Conversation".to_string()));
        assert_eq!(result.model, "gpt-4");
        assert_eq!(
            result.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
        assert_eq!(result.message_count, Some(0));
        assert!(result.last_message_at.is_none());
    }

    #[sqlx::test]
    async fn test_create_conversation_with_defaults(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let request = CreateConversationRequest {
            model: "gpt-3.5".to_string(),
            title: None,
            system_prompt: None,
        };

        let result = service
            .create_conversation(&user_id, request)
            .await
            .expect("Failed to create conversation with defaults");

        assert_eq!(result.title, Some("New Conversation".to_string()));
        assert_eq!(result.model, "gpt-3.5");
        assert_eq!(result.system_prompt, Some(String::new()));
    }

    #[sqlx::test]
    async fn test_get_user_conversations_empty(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let conversations = service
            .get_user_conversations(&user_id, Some(10), Some(0))
            .await
            .expect("Failed to get empty conversations");

        assert!(conversations.is_empty());
    }

    #[sqlx::test]
    async fn test_get_user_conversations_with_data(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create multiple conversations
        for i in 0..3 {
            let request = CreateConversationRequest {
                model: "gpt-4".to_string(),
                title: Some(format!("Conversation {i}")),
                system_prompt: None,
            };
            service
                .create_conversation(&user_id, request)
                .await
                .expect("Failed to create conversation in loop");
        }

        let conversations = service
            .get_user_conversations(&user_id, Some(10), Some(0))
            .await
            .expect("Failed to get conversations with data");

        assert_eq!(conversations.len(), 3);
        // They should be ordered by updated_at DESC
        assert_eq!(conversations[0].title, Some("Conversation 2".to_string()));
    }

    #[sqlx::test]
    async fn test_get_user_conversations_pagination(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create 5 conversations
        for i in 0..5 {
            let request = CreateConversationRequest {
                model: "gpt-4".to_string(),
                title: Some(format!("Conversation {i}")),
                system_prompt: None,
            };
            service
                .create_conversation(&user_id, request)
                .await
                .expect("Failed to create conversation in loop");
        }

        // Test pagination
        let page1 = service
            .get_user_conversations(&user_id, Some(2), Some(0))
            .await
            .expect("Failed to get first page");
        assert_eq!(page1.len(), 2);

        let page2 = service
            .get_user_conversations(&user_id, Some(2), Some(2))
            .await
            .expect("Failed to get second page");
        assert_eq!(page2.len(), 2);

        let page3 = service
            .get_user_conversations(&user_id, Some(2), Some(4))
            .await
            .expect("Failed to get third page");
        assert_eq!(page3.len(), 1);
    }

    #[sqlx::test]
    async fn test_get_conversation_with_messages_not_found(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let result = service
            .get_conversation_with_messages("nonexistent", &user_id)
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, "Conversation not found"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_get_conversation_with_messages(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create conversation
        let conv_request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("Test Chat".to_string()),
            system_prompt: Some("Be helpful".to_string()),
        };
        let conversation = service
            .create_conversation(&user_id, conv_request)
            .await
            .expect("Failed to create conversation for messages test");

        // Add messages
        let msg1 = CreateMessageRequest {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        service
            .add_message(&conversation.id, &user_id, msg1)
            .await
            .expect("Failed to add first message");

        let msg2 = CreateMessageRequest {
            role: "assistant".to_string(),
            content: "Hi there!".to_string(),
        };
        service
            .add_message(&conversation.id, &user_id, msg2)
            .await
            .expect("Failed to add second message");

        // Get conversation with messages
        let result = service
            .get_conversation_with_messages(&conversation.id, &user_id)
            .await
            .expect("Failed to get conversation with messages");

        assert_eq!(result.conversation.title, Some("Test Chat".to_string()));
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].content, "Hello");
        assert_eq!(result.messages[1].content, "Hi there!");
        assert_eq!(result.conversation.message_count, Some(2));
        assert!(result.conversation.last_message_at.is_some());
    }

    #[sqlx::test]
    async fn test_add_message_to_nonexistent_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let msg_request = CreateMessageRequest {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };

        let result = service
            .add_message("nonexistent", &user_id, msg_request)
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, "Conversation not found"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_add_message_updates_conversation_timestamp(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create conversation
        let conv_request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("Test".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id, conv_request)
            .await
            .expect("Failed to create conversation for timestamp test");
        let original_updated_at = conversation.updated_at.clone();

        // Wait a bit to ensure timestamp difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Add message
        let msg_request = CreateMessageRequest {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };
        service
            .add_message(&conversation.id, &user_id, msg_request)
            .await
            .expect("Failed to add message for timestamp test");

        // Get conversation again
        let updated_conversations = service
            .get_user_conversations(&user_id, Some(1), Some(0))
            .await
            .expect("Failed to get updated conversations");

        assert_ne!(updated_conversations[0].updated_at, original_updated_at);
    }

    #[sqlx::test]
    async fn test_record_usage(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create a conversation first to get a valid conversation_id
        let conv_request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("Test Conversation".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id, conv_request)
            .await
            .expect("Failed to create conversation");

        let usage = UsageRecord {
            conversation_id: Some(&conversation.id),
            user_id: &user_id,
            model: "gpt-4",
            prompt_tokens: 100,
            completion_tokens: 50,
            request_id: Some("req123"),
            duration_ms: Some(1500),
        };

        service
            .record_usage(usage)
            .await
            .expect("Failed to record usage");

        // Verify it was recorded
        let stats = service
            .get_user_usage_stats(&user_id)
            .await
            .expect("Failed to get usage stats");
        assert_eq!(stats.total_tokens, 150);
        assert_eq!(stats.models_used, vec!["gpt-4"]);
    }

    #[sqlx::test]
    async fn test_record_usage_without_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let usage = UsageRecord {
            conversation_id: None,
            user_id: &user_id,
            model: "gpt-3.5",
            prompt_tokens: 50,
            completion_tokens: 25,
            request_id: None,
            duration_ms: None,
        };

        service
            .record_usage(usage)
            .await
            .expect("Failed to record usage without conversation");

        let stats = service
            .get_user_usage_stats(&user_id)
            .await
            .expect("Failed to get usage stats");
        assert_eq!(stats.total_tokens, 75);
    }

    #[sqlx::test]
    async fn test_get_user_usage_stats_empty(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let stats = service
            .get_user_usage_stats(&user_id)
            .await
            .expect("Failed to get empty usage stats");

        assert_eq!(stats.total_conversations, 0);
        assert_eq!(stats.total_messages, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.total_cost_cents, None);
        assert!(stats.models_used.is_empty());
    }

    #[sqlx::test]
    async fn test_get_user_usage_stats_with_data(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create conversation and add messages
        let conv_request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("Test".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id, conv_request)
            .await
            .expect("Failed to create conversation for usage stats test");

        // Add messages
        for i in 0..3 {
            let msg = CreateMessageRequest {
                role: if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                content: format!("Message {i}"),
            };
            service
                .add_message(&conversation.id, &user_id, msg)
                .await
                .expect("Failed to add message in loop");
        }

        // Record usage
        let usage1 = UsageRecord {
            conversation_id: Some(&conversation.id),
            user_id: &user_id,
            model: "gpt-4",
            prompt_tokens: 100,
            completion_tokens: 50,
            request_id: Some("req1"),
            duration_ms: Some(1000),
        };
        service
            .record_usage(usage1)
            .await
            .expect("Failed to record first usage");

        let usage2 = UsageRecord {
            conversation_id: Some(&conversation.id),
            user_id: &user_id,
            model: "gpt-3.5",
            prompt_tokens: 50,
            completion_tokens: 25,
            request_id: Some("req2"),
            duration_ms: Some(500),
        };
        service
            .record_usage(usage2)
            .await
            .expect("Failed to record second usage");

        let stats = service
            .get_user_usage_stats(&user_id)
            .await
            .expect("Failed to get usage stats with data");

        assert_eq!(stats.total_conversations, 1);
        assert_eq!(stats.total_messages, 3);
        assert_eq!(stats.total_tokens, 225); // 150 + 75
        assert_eq!(stats.models_used.len(), 2);
        assert!(stats.models_used.contains(&"gpt-3.5".to_string()));
        assert!(stats.models_used.contains(&"gpt-4".to_string()));
    }

    #[sqlx::test]
    async fn test_archive_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create conversation
        let request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("To Archive".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id, request)
            .await
            .expect("Failed to create conversation for archive test");

        // Archive it
        service
            .archive_conversation(&conversation.id, &user_id)
            .await
            .expect("Failed to archive conversation");

        // Should not appear in active conversations
        let conversations = service
            .get_user_conversations(&user_id, Some(10), Some(0))
            .await
            .expect("Failed to get conversations after archive");
        assert!(conversations.is_empty());

        // Should not be able to get it with messages
        let result = service
            .get_conversation_with_messages(&conversation.id, &user_id)
            .await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_archive_nonexistent_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        let result = service.archive_conversation("nonexistent", &user_id).await;

        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, "Conversation not found"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_archive_already_archived_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id = create_test_user(&pool).await;

        // Create and archive conversation
        let request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("To Archive".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id, request)
            .await
            .expect("Failed to create conversation for double archive test");
        service
            .archive_conversation(&conversation.id, &user_id)
            .await
            .expect("Failed to archive conversation first time");

        // Try to archive again
        let result = service
            .archive_conversation(&conversation.id, &user_id)
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, "Conversation not found"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_archive_other_users_conversation(pool: SqlitePool) {
        let service = AiDataService::new(pool.clone());
        let user_id1 = create_test_user(&pool).await;
        let user_id2 = create_test_user(&pool).await;

        // Create conversation for user1
        let request = CreateConversationRequest {
            model: "gpt-4".to_string(),
            title: Some("User1 Conversation".to_string()),
            system_prompt: None,
        };
        let conversation = service
            .create_conversation(&user_id1, request)
            .await
            .expect("Failed to create conversation for user1");

        // Try to archive as user2
        let result = service
            .archive_conversation(&conversation.id, &user_id2)
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::BadRequest(msg)) => assert_eq!(msg, "Conversation not found"),
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[test]
    fn test_usage_record_creation() {
        let usage = UsageRecord {
            conversation_id: Some("conv123"),
            user_id: "user123",
            model: "gpt-4",
            prompt_tokens: 100,
            completion_tokens: 50,
            request_id: Some("req123"),
            duration_ms: Some(1500),
        };

        assert_eq!(usage.user_id, "user123");
        assert_eq!(usage.model, "gpt-4");
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.conversation_id, Some("conv123"));
        assert_eq!(usage.request_id, Some("req123"));
        assert_eq!(usage.duration_ms, Some(1500));
    }
}
