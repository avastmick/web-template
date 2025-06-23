//! Service for managing AI conversations, messages, and usage data

use sqlx::SqlitePool;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AiConversation, AiMessage, AiUsage, ConversationResponse, ConversationWithMessages,
    CreateConversationRequest, CreateMessageRequest, MessageResponse, UsageStatsResponse,
};

#[derive(Debug)]
pub struct UsageRecord<'a> {
    pub conversation_id: Option<&'a str>,
    pub user_id: &'a str,
    pub model: &'a str,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub request_id: Option<&'a str>,
    pub duration_ms: Option<i64>,
}

pub struct AiDataService {
    db: SqlitePool,
}

impl AiDataService {
    #[must_use]
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Create a new conversation
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn create_conversation(
        &self,
        user_id: &str,
        request: CreateConversationRequest,
    ) -> AppResult<ConversationResponse> {
        let conversation = AiConversation::new(user_id.to_string(), request.model)
            .with_title(
                request
                    .title
                    .unwrap_or_else(|| "New Conversation".to_string()),
            )
            .with_system_prompt(request.system_prompt.unwrap_or_default());

        sqlx::query!(
            r#"
            INSERT INTO ai_conversations (id, user_id, title, model, system_prompt, created_at, updated_at, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            conversation.id,
            conversation.user_id,
            conversation.title,
            conversation.model,
            conversation.system_prompt,
            conversation.created_at,
            conversation.updated_at,
            conversation.metadata
        )
        .execute(&self.db)
        .await?;

        Ok(ConversationResponse {
            id: conversation.id,
            title: conversation.title,
            model: conversation.model,
            system_prompt: conversation.system_prompt,
            created_at: conversation.created_at,
            updated_at: conversation.updated_at,
            message_count: Some(0),
            last_message_at: None,
        })
    }

    /// Get conversations for a user
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_user_conversations(
        &self,
        user_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> AppResult<Vec<ConversationResponse>> {
        let limit = limit.unwrap_or(50);
        let offset = offset.unwrap_or(0);

        let conversations = sqlx::query_as!(
            AiConversation,
            r#"
            SELECT id, user_id, title, model, system_prompt, created_at, updated_at, archived_at, metadata
            FROM ai_conversations
            WHERE user_id = ?1 AND archived_at IS NULL
            ORDER BY updated_at DESC
            LIMIT ?2 OFFSET ?3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        let mut results = Vec::new();
        for conv in conversations {
            // Get message count and last message timestamp
            let stats = sqlx::query!(
                r#"
                SELECT
                    COUNT(*) as message_count,
                    COALESCE(MAX(created_at), '') as last_message_at
                FROM ai_messages
                WHERE conversation_id = ?1
                "#,
                conv.id
            )
            .fetch_one(&self.db)
            .await?;

            results.push(ConversationResponse {
                id: conv.id,
                title: conv.title,
                model: conv.model,
                system_prompt: conv.system_prompt,
                created_at: conv.created_at,
                updated_at: conv.updated_at,
                message_count: Some(stats.message_count),
                last_message_at: if stats.last_message_at.is_empty() {
                    None
                } else {
                    Some(stats.last_message_at)
                },
            });
        }

        Ok(results)
    }

    /// Get a conversation with its messages
    ///
    /// # Errors
    ///
    /// Returns an error if the conversation is not found or database operation fails
    pub async fn get_conversation_with_messages(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> AppResult<ConversationWithMessages> {
        // Get conversation
        let conversation = sqlx::query_as!(
            AiConversation,
            r#"
            SELECT id, user_id, title, model, system_prompt, created_at, updated_at, archived_at, metadata
            FROM ai_conversations
            WHERE id = ?1 AND user_id = ?2 AND archived_at IS NULL
            "#,
            conversation_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AppError::BadRequest("Conversation not found".to_string()))?;

        // Get messages
        let messages = sqlx::query_as!(
            AiMessage,
            r#"
            SELECT id, conversation_id, role, content, token_count, created_at, metadata
            FROM ai_messages
            WHERE conversation_id = ?1
            ORDER BY created_at ASC
            "#,
            conversation_id
        )
        .fetch_all(&self.db)
        .await?;

        let message_responses: Vec<MessageResponse> = messages
            .into_iter()
            .map(|msg| MessageResponse {
                id: msg.id,
                role: msg.role,
                content: msg.content,
                token_count: msg.token_count,
                created_at: msg.created_at,
            })
            .collect();

        Ok(ConversationWithMessages {
            conversation: ConversationResponse {
                id: conversation.id,
                title: conversation.title,
                model: conversation.model,
                system_prompt: conversation.system_prompt,
                created_at: conversation.created_at,
                updated_at: conversation.updated_at,
                message_count: Some(i64::try_from(message_responses.len()).unwrap_or(0)),
                last_message_at: message_responses.last().map(|m| m.created_at.clone()),
            },
            messages: message_responses,
        })
    }

    /// Add a message to a conversation
    ///
    /// # Errors
    ///
    /// Returns an error if the conversation is not found or database operation fails
    pub async fn add_message(
        &self,
        conversation_id: &str,
        user_id: &str,
        request: CreateMessageRequest,
    ) -> AppResult<MessageResponse> {
        // Verify conversation exists and belongs to user
        let conversation_exists = sqlx::query!(
            r#"
            SELECT id FROM ai_conversations
            WHERE id = ?1 AND user_id = ?2 AND archived_at IS NULL
            "#,
            conversation_id,
            user_id
        )
        .fetch_optional(&self.db)
        .await?
        .is_some();

        if !conversation_exists {
            return Err(AppError::BadRequest("Conversation not found".to_string()));
        }

        let message = AiMessage::new(conversation_id.to_string(), request.role, request.content);

        sqlx::query!(
            r#"
            INSERT INTO ai_messages (id, conversation_id, role, content, token_count, created_at, metadata)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            message.id,
            message.conversation_id,
            message.role,
            message.content,
            message.token_count,
            message.created_at,
            message.metadata
        )
        .execute(&self.db)
        .await?;

        // Update conversation timestamp
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query!(
            r#"
            UPDATE ai_conversations
            SET updated_at = ?1
            WHERE id = ?2
            "#,
            now,
            conversation_id
        )
        .execute(&self.db)
        .await?;

        Ok(MessageResponse {
            id: message.id,
            role: message.role,
            content: message.content,
            token_count: message.token_count,
            created_at: message.created_at,
        })
    }

    /// Record AI usage statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn record_usage(&self, usage_record: UsageRecord<'_>) -> AppResult<()> {
        let usage = AiUsage::new(
            usage_record.user_id.to_string(),
            usage_record.model.to_string(),
            usage_record.prompt_tokens,
            usage_record.completion_tokens,
        );

        let usage = if let Some(conv_id) = usage_record.conversation_id {
            usage.with_conversation(conv_id.to_string())
        } else {
            usage
        };

        let usage = if let Some(req_id) = usage_record.request_id {
            usage.with_request_id(req_id.to_string())
        } else {
            usage
        };

        let usage = if let Some(duration) = usage_record.duration_ms {
            usage.with_duration(duration)
        } else {
            usage
        };

        sqlx::query!(
            r#"
            INSERT INTO ai_usage (id, conversation_id, user_id, model, prompt_tokens, completion_tokens, total_tokens, cost_cents, created_at, request_id, duration_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            usage.id,
            usage.conversation_id,
            usage.user_id,
            usage.model,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens,
            usage.cost_cents,
            usage.created_at,
            usage.request_id,
            usage.duration_ms
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get usage statistics for a user
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails
    pub async fn get_user_usage_stats(&self, user_id: &str) -> AppResult<UsageStatsResponse> {
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT conversation_id) as total_conversations,
                COALESCE(SUM(prompt_tokens + completion_tokens), 0) as total_tokens,
                COALESCE(SUM(cost_cents), 0) as total_cost_cents
            FROM ai_usage
            WHERE user_id = ?1
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        let message_count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM ai_messages m
            JOIN ai_conversations c ON m.conversation_id = c.id
            WHERE c.user_id = ?1 AND c.archived_at IS NULL
            "#,
            user_id
        )
        .fetch_one(&self.db)
        .await?;

        let models = sqlx::query!(
            r#"
            SELECT DISTINCT model
            FROM ai_usage
            WHERE user_id = ?1
            ORDER BY model
            "#,
            user_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(UsageStatsResponse {
            total_conversations: stats.total_conversations,
            total_messages: message_count.count,
            total_tokens: stats.total_tokens,
            total_cost_cents: if stats.total_cost_cents == 0 {
                None
            } else {
                Some(stats.total_cost_cents)
            },
            models_used: models.into_iter().map(|m| m.model).collect(),
        })
    }

    /// Archive a conversation (soft delete)
    ///
    /// # Errors
    ///
    /// Returns an error if the conversation is not found or database operation fails
    pub async fn archive_conversation(
        &self,
        conversation_id: &str,
        user_id: &str,
    ) -> AppResult<()> {
        let now = chrono::Utc::now().to_rfc3339();

        let rows_affected = sqlx::query!(
            r#"
            UPDATE ai_conversations
            SET archived_at = ?1, updated_at = ?2
            WHERE id = ?3 AND user_id = ?4 AND archived_at IS NULL
            "#,
            now,
            now,
            conversation_id,
            user_id
        )
        .execute(&self.db)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::BadRequest("Conversation not found".to_string()));
        }

        Ok(())
    }
}
