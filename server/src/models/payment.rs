use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::convert::TryFrom;
use std::str::FromStr;
use uuid::Uuid;

/// Payment status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Active,
    Cancelled,
    Expired,
    Failed,
}

impl PaymentStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Pending => "pending",
            PaymentStatus::Active => "active",
            PaymentStatus::Cancelled => "cancelled",
            PaymentStatus::Expired => "expired",
            PaymentStatus::Failed => "failed",
        }
    }
}

impl FromStr for PaymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(PaymentStatus::Pending),
            "active" => Ok(PaymentStatus::Active),
            "cancelled" => Ok(PaymentStatus::Cancelled),
            "expired" => Ok(PaymentStatus::Expired),
            "failed" => Ok(PaymentStatus::Failed),
            _ => Err(format!("Invalid payment status: {s}")),
        }
    }
}

/// Payment type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Subscription,
    OneTime,
}

impl PaymentType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentType::Subscription => "subscription",
            PaymentType::OneTime => "one_time",
        }
    }
}

impl FromStr for PaymentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "subscription" => Ok(PaymentType::Subscription),
            "one_time" => Ok(PaymentType::OneTime),
            _ => Err(format!("Invalid payment type: {s}")),
        }
    }
}

/// Represents a user payment record in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPayment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub payment_status: PaymentStatus,
    pub payment_type: PaymentType,
    pub amount_cents: Option<i32>,
    pub currency: String,
    pub subscription_start_date: Option<DateTime<Utc>>,
    pub subscription_end_date: Option<DateTime<Utc>>,
    pub subscription_cancelled_at: Option<DateTime<Utc>>,
    pub last_payment_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database row representation for user payments
#[derive(Debug, Clone, FromRow)]
pub struct UserPaymentFromDb {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub stripe_payment_intent_id: Option<String>,
    pub payment_status: Option<String>,
    pub payment_type: Option<String>,
    pub amount_cents: Option<i32>,
    pub currency: Option<String>,
    pub subscription_start_date: Option<String>,
    pub subscription_end_date: Option<String>,
    pub subscription_cancelled_at: Option<String>,
    pub last_payment_date: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Error type for conversion from `UserPaymentFromDb` to `UserPayment`
#[derive(Debug, thiserror::Error)]
pub enum PaymentConversionError {
    #[error("Missing expected database value for field: {0}")]
    MissingDatabaseValue(String),
    #[error("Failed to parse UUID from string '{value}': {source}")]
    UuidParseError { value: String, source: uuid::Error },
    #[error("Failed to parse DateTime from string '{value}': {source}")]
    DateTimeParseError {
        value: String,
        source: chrono::ParseError,
    },
    #[error("Failed to parse payment status: {0}")]
    PaymentStatusParseError(String),
    #[error("Failed to parse payment type: {0}")]
    PaymentTypeParseError(String),
}

impl TryFrom<UserPaymentFromDb> for UserPayment {
    type Error = PaymentConversionError;

    fn try_from(db_row: UserPaymentFromDb) -> Result<Self, Self::Error> {
        let id_str = db_row
            .id
            .ok_or_else(|| PaymentConversionError::MissingDatabaseValue("id".to_string()))?;
        let user_id_str = db_row
            .user_id
            .ok_or_else(|| PaymentConversionError::MissingDatabaseValue("user_id".to_string()))?;
        let payment_status_str = db_row.payment_status.ok_or_else(|| {
            PaymentConversionError::MissingDatabaseValue("payment_status".to_string())
        })?;
        let payment_type_str = db_row.payment_type.ok_or_else(|| {
            PaymentConversionError::MissingDatabaseValue("payment_type".to_string())
        })?;
        let currency = db_row.currency.unwrap_or_else(|| "usd".to_string());
        let created_at_str = db_row.created_at.ok_or_else(|| {
            PaymentConversionError::MissingDatabaseValue("created_at".to_string())
        })?;
        let updated_at_str = db_row.updated_at.ok_or_else(|| {
            PaymentConversionError::MissingDatabaseValue("updated_at".to_string())
        })?;

        let id =
            Uuid::from_str(&id_str).map_err(|source| PaymentConversionError::UuidParseError {
                value: id_str.clone(),
                source,
            })?;
        let user_id = Uuid::from_str(&user_id_str).map_err(|source| {
            PaymentConversionError::UuidParseError {
                value: user_id_str.clone(),
                source,
            }
        })?;
        let payment_status = PaymentStatus::from_str(&payment_status_str)
            .map_err(PaymentConversionError::PaymentStatusParseError)?;
        let payment_type = PaymentType::from_str(&payment_type_str)
            .map_err(PaymentConversionError::PaymentTypeParseError)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|source| PaymentConversionError::DateTimeParseError {
                value: created_at_str.clone(),
                source,
            })?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|source| PaymentConversionError::DateTimeParseError {
                value: updated_at_str.clone(),
                source,
            })?
            .with_timezone(&Utc);

        // Parse optional DateTime fields
        let parse_optional_datetime =
            |opt_str: Option<String>| -> Result<Option<DateTime<Utc>>, PaymentConversionError> {
                match opt_str {
                    Some(s) => {
                        let dt = DateTime::parse_from_rfc3339(&s)
                            .map_err(|source| PaymentConversionError::DateTimeParseError {
                                value: s.clone(),
                                source,
                            })?
                            .with_timezone(&Utc);
                        Ok(Some(dt))
                    }
                    None => Ok(None),
                }
            };

        let subscription_start_date = parse_optional_datetime(db_row.subscription_start_date)?;
        let subscription_end_date = parse_optional_datetime(db_row.subscription_end_date)?;
        let subscription_cancelled_at = parse_optional_datetime(db_row.subscription_cancelled_at)?;
        let last_payment_date = parse_optional_datetime(db_row.last_payment_date)?;

        Ok(UserPayment {
            id,
            user_id,
            stripe_customer_id: db_row.stripe_customer_id,
            stripe_subscription_id: db_row.stripe_subscription_id,
            stripe_payment_intent_id: db_row.stripe_payment_intent_id,
            payment_status,
            payment_type,
            amount_cents: db_row.amount_cents,
            currency,
            subscription_start_date,
            subscription_end_date,
            subscription_cancelled_at,
            last_payment_date,
            created_at,
            updated_at,
        })
    }
}

impl UserPayment {
    /// Check if the payment is currently active
    #[must_use]
    pub fn is_active(&self) -> bool {
        match self.payment_status {
            PaymentStatus::Active => {
                // Check if subscription has not expired
                if let Some(end_date) = self.subscription_end_date {
                    end_date > Utc::now()
                } else {
                    // If no end date, consider active status as active
                    true
                }
            }
            _ => false,
        }
    }
}

/// Stripe webhook event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeWebhookEvent {
    pub id: Uuid,
    pub stripe_event_id: String,
    pub event_type: String,
    pub processed: bool,
    pub processing_attempts: i32,
    pub last_error: Option<String>,
    pub event_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Request to create a payment intent
#[derive(Debug, Deserialize)]
pub struct CreatePaymentIntentRequest {
    pub amount_cents: i64,
    pub currency: String,
}

/// Response from creating a payment intent
#[derive(Debug, Serialize)]
pub struct CreatePaymentIntentResponse {
    pub client_secret: String,
    pub payment_intent_id: String,
}

/// User payment status response
#[derive(Debug, Serialize)]
pub struct UserPaymentStatusResponse {
    pub has_active_payment: bool,
    pub payment_status: Option<PaymentStatus>,
    pub payment_type: Option<PaymentType>,
    pub subscription_end_date: Option<DateTime<Utc>>,
}
