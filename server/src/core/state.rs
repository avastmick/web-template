//! Application state management

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::{
    AiDataService, AiService, AuthService, InviteService, PaymentService, UserServiceImpl,
};

/// Application state for handlers that need all services
pub struct AppState {
    pub user: Arc<UserServiceImpl>,
    pub auth: Arc<AuthService>,
    pub invite: Arc<InviteService>,
    pub ai: Arc<RwLock<AiService>>,
    pub ai_data: Arc<AiDataService>,
    pub payment: Arc<PaymentService>,
}
