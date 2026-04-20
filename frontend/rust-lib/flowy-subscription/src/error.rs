//! Error types for subscription system

use thiserror::Error;

pub type SubscriptionResult<T> = Result<T, SubscriptionError>;

#[derive(Error, Debug)]
pub enum SubscriptionError {
    #[error("Payment error: {0}")]
    PaymentError(String),

    #[error("Lemon Squeezy API error: {0}")]
    LemonSqueezyError(String),

    #[error("Web3 error: {0}")]
    Web3Error(String),

    #[error("Webhook signature verification failed")]
    WebhookSignatureInvalid,

    #[error("Invalid subscription: {0}")]
    InvalidSubscription(String),

    #[error("Subscription not found")]
    SubscriptionNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Token quota exceeded")]
    TokenQuotaExceeded,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Authorization failed: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Contract interaction error: {0}")]
    ContractError(String),
}

impl From<reqwest::Error> for SubscriptionError {
    fn from(err: reqwest::Error) -> Self {
        SubscriptionError::PaymentError(err.to_string())
    }
}

impl From<serde_json::Error> for SubscriptionError {
    fn from(err: serde_json::Error) -> Self {
        SubscriptionError::InvalidRequest(err.to_string())
    }
}

impl From<tokio::task::JoinError> for SubscriptionError {
    fn from(err: tokio::task::JoinError) -> Self {
        SubscriptionError::InternalError(err.to_string())
    }
}
