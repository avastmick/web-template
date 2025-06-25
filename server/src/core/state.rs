//! Application state management

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::services::{
    AiDataService, AiService, AuthService, InviteService, PaymentService, UserServiceImpl,
};

/// Application state for handlers that need all services
#[allow(clippy::struct_field_names)]
pub struct AppState {
    pub user_service: Arc<UserServiceImpl>,
    pub auth_service: Arc<AuthService>,
    pub invite_service: Arc<InviteService>,
    pub ai_service: Arc<RwLock<AiService>>,
    pub ai_data_service: Arc<AiDataService>,
    pub payment_service: Arc<PaymentService>,
}
