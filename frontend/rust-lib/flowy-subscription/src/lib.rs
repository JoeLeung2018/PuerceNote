//! # Flowy Subscription
//!
//! PuerceNote subscription, payment, and billing system
//!
//! This crate provides:
//! - Lemon Squeezy payment processing
//! - Web3/Polygon cryptocurrency payments
//! - User subscription management
//! - AI token quota tracking
//! - Webhook handling and audit logging

pub mod payment;
pub mod subscription;
pub mod billing;
pub mod event_handler;
pub mod error;
pub mod config;
pub mod repository;

// Re-export key types
pub use error::{SubscriptionError, SubscriptionResult};
pub use payment::{PaymentClient, PaymentProvider};
pub use subscription::{Subscription, SubscriptionPlan, SubscriptionStatus, SubscriptionService};
pub use billing::{BillingService, TokenQuota, calculate_token_cost, count_tokens_local};
pub use config::SubscriptionConfig;
pub use repository::{SubscriptionRepository, PaymentOrder, SubscriptionRecord, WebhookEventLog};

#[macro_use]
extern crate async_trait;

use tracing::info;

/// Initialize the subscription system
pub async fn init() -> SubscriptionResult<()> {
    info!("Initializing Flowy Subscription system");
    
    // Initialize payment clients
    payment::init().await?;
    
    // Initialize event handlers
    event_handler::init().await?;
    
    info!("Flowy Subscription initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subscription_init() {
        // Test subscription initialization
    }
}
