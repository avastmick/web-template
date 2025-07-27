//! Test context for integration tests
//!
//! Provides a unified interface for creating test data using application services
//! instead of direct SQL manipulation.

use std::sync::Arc;

use server::{
    core::AppState,
    handlers::auth_handler::RegisterUserPayload,
    models::User,
    services::{AuthService, InviteService, OAuthService, PaymentService, UserServiceImpl},
};
use sqlx::SqlitePool;

/// Test context that encapsulates all services
#[allow(dead_code)]
pub struct TestContext {
    pub user_service: Arc<UserServiceImpl>,
    pub oauth_service: Arc<OAuthService>,
    pub invite_service: Arc<InviteService>,
    pub payment_service: Arc<PaymentService>,
    pub auth_service: Arc<AuthService>,
    pub pool: SqlitePool,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new test context with all services initialized
    pub async fn new() -> Self {
        let pool = super::setup_test_database().await;

        // Set up required environment variables for services
        setup_test_env();

        // Initialize services
        let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
        let oauth_service =
            Arc::new(OAuthService::new(pool.clone()).expect("Failed to create OAuth service"));
        let invite_service = Arc::new(InviteService::new(pool.clone()));
        let payment_service =
            Arc::new(PaymentService::new(pool.clone()).expect("Failed to create Payment service"));
        let auth_service = Arc::new(AuthService::new().expect("Failed to create Auth service"));

        Self {
            user_service,
            oauth_service,
            invite_service,
            payment_service,
            auth_service,
            pool,
        }
    }

    /// Create a test user using the application service
    pub async fn create_test_user(&self, email: &str) -> User {
        self.user_service
            .create_user(&RegisterUserPayload {
                email: email.to_string(),
                password: "test_password123".to_string(),
            })
            .await
            .expect("Failed to create test user")
    }

    pub async fn create_oauth_state(
        &self,
        state: &str,
        provider: server::models::oauth::OAuthProvider,
    ) {
        self.oauth_service
            .store_oauth_state(state, provider, None)
            .await
            .expect("Failed to store OAuth state");
    }

    /// Create an invite for testing
    pub async fn create_test_invite(&self, email: &str, invited_by: Option<String>) {
        self.invite_service
            .create_invite(email, invited_by, None)
            .await
            .expect("Failed to create test invite");
    }

    /// Create a new test context with an existing pool
    pub fn new_with_pool(pool: SqlitePool) -> Self {
        // Set up required environment variables for services
        setup_test_env();

        // Initialize services
        let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
        let oauth_service =
            Arc::new(OAuthService::new(pool.clone()).expect("Failed to create OAuth service"));
        let invite_service = Arc::new(InviteService::new(pool.clone()));
        let payment_service =
            Arc::new(PaymentService::new(pool.clone()).expect("Failed to create Payment service"));
        let auth_service = Arc::new(AuthService::new().expect("Failed to create Auth service"));

        Self {
            user_service,
            oauth_service,
            invite_service,
            payment_service,
            auth_service,
            pool,
        }
    }

    /// Get or create the `AppState` for handler tests
    pub fn create_app_state(&self) -> Arc<AppState> {
        // For integration tests, we need to create AppState manually
        // since test_helpers is only available in unit tests
        Arc::new(AppState {
            user: self.user_service.clone(),
            auth: self.auth_service.clone(),
            invite: self.invite_service.clone(),
            ai: Arc::new(tokio::sync::RwLock::new(
                server::services::AiService::new().expect("Failed to create AI service"),
            )),
            ai_data: Arc::new(server::services::AiDataService::new(self.pool.clone())),
            payment: self.payment_service.clone(),
        })
    }
}

#[allow(dead_code)]
fn setup_test_env() {
    use std::env;

    #[allow(unsafe_code)]
    unsafe {
        // Auth service requirements
        env::set_var(
            "JWT_SECRET",
            "test_secret_key_that_is_long_enough_for_testing",
        );

        // OAuth service requirements
        env::set_var("CLIENT_URL", "http://localhost:8080");
        env::set_var("GOOGLE_CLIENT_ID", "test_google_client_id");
        env::set_var("GOOGLE_CLIENT_SECRET", "test_google_client_secret");
        env::set_var("GITHUB_CLIENT_ID", "test_github_client_id");
        env::set_var("GITHUB_CLIENT_SECRET", "test_github_client_secret");
        env::set_var(
            "GOOGLE_REDIRECT_URI",
            "http://localhost:8000/api/auth/google/callback",
        );
        env::set_var(
            "GITHUB_REDIRECT_URI",
            "http://localhost:8000/api/auth/github/callback",
        );

        // Payment service requirements
        env::set_var("STRIPE_SECRET_KEY", "test_stripe_key");
        env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "test_webhook_secret");
    }
}
