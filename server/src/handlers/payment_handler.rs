use crate::core::AppState;
use crate::errors::AppError;
use crate::middleware::JwtAuth;
use crate::models::payment::CreatePaymentIntentRequest;
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;

/// Get user payment status
///
/// # Errors
///
/// Returns an error if the payment service fails to retrieve status
pub async fn get_payment_status_handler(
    auth: JwtAuth,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let status = app_state
        .payment
        .get_user_payment_status(auth.user.user_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(status))
}

/// Create Stripe payment intent
///
/// # Errors
///
/// Returns an error if the payment service fails to create payment intent
pub async fn create_payment_intent_handler(
    auth: JwtAuth,
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CreatePaymentIntentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let response = app_state
        .payment
        .create_payment_intent(auth.user.user_id, &auth.user.email, request)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    Ok(Json(response))
}

/// Handle Stripe webhook
///
/// # Errors
///
/// Returns an error if the webhook signature is missing or invalid
pub async fn stripe_webhook_handler(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, AppError> {
    // Get Stripe signature header
    let stripe_signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Missing Stripe-Signature header".to_string()))?;

    // Process webhook event
    Box::pin(
        app_state
            .payment
            .process_webhook_event(&body, stripe_signature),
    )
    .await
    .map_err(|e| {
        tracing::error!("Webhook processing error: {}", e);
        AppError::BadRequest("Invalid webhook signature or payload".to_string())
    })?;

    Ok(StatusCode::OK)
}
