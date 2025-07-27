//! Common test helpers and utilities

use crate::{
    core::AppState,
    services::{
        AiDataService, AiService, AuthService, InviteService, PaymentService, UserServiceImpl,
    },
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test services container that provides both `AppState` and individual service access
pub struct TestServices {
    pub app_state: Arc<AppState>,
    pub user_service: Arc<UserServiceImpl>,
    pub auth_service: Arc<AuthService>,
    pub invite_service: Arc<InviteService>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub ai_data_service: Arc<AiDataService>,
    pub payment_service: Arc<PaymentService>,
}

/// Create test services with all dependencies initialized
///
/// This centralized function ensures all tests use the same service configuration
/// and makes it easy to add new services in one place.
///
/// # Panics
///
/// Panics if any of the services fail to initialize (auth, AI, or payment services)
#[must_use]
pub fn create_test_services(pool: &SqlitePool) -> TestServices {
    let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new().expect("Failed to create auth service"));
    let invite_service = Arc::new(InviteService::new(pool.clone()));
    let ai_service_inner = AiService::new().expect("Failed to create AI service");
    let ai_service = Arc::new(RwLock::new(ai_service_inner));
    let ai_data_service = Arc::new(AiDataService::new(pool.clone()));
    let payment_service =
        Arc::new(PaymentService::new(pool.clone()).expect("Failed to create payment service"));
    let app_state = Arc::new(AppState {
        user: user_service.clone(),
        auth: auth_service.clone(),
        invite: invite_service.clone(),
        ai: ai_service.clone(),
        ai_data: ai_data_service.clone(),
        payment: payment_service.clone(),
    });

    TestServices {
        app_state,
        user_service,
        auth_service,
        invite_service,
        ai_service,
        ai_data_service,
        payment_service,
    }
}

/// Create a test `AppState` with all services initialized
///
/// This is a convenience function for tests that only need the `AppState`
#[must_use]
pub fn create_test_app_state(pool: &SqlitePool) -> Arc<AppState> {
    create_test_services(pool).app_state
}
