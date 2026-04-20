//! Payment system module
//!
//! Supports multiple payment providers:
//! - Lemon Squeezy (credit cards)
//! - Web3/Polygon (USDC/USDT)

pub mod lemon_squeezy;
pub mod web3_payment;

use crate::error::SubscriptionResult;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PaymentProvider {
    LemonSqueezy,
    Web3Polygon,
    Manual,
}

impl PaymentProvider {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentProvider::LemonSqueezy => "lemon_squeezy",
            PaymentProvider::Web3Polygon => "web3_polygon",
            PaymentProvider::Manual => "manual",
        }
    }
}

#[async_trait::async_trait]
pub trait PaymentClient: Send + Sync {
    /// Process a payment
    async fn process_payment(&self, amount: u32, currency: &str) -> SubscriptionResult<String>;

    /// Verify payment
    async fn verify_payment(&self, payment_id: &str) -> SubscriptionResult<bool>;

    /// Get payment status
    async fn get_payment_status(&self, payment_id: &str) -> SubscriptionResult<PaymentStatus>;

    /// Refund payment
    async fn refund_payment(&self, payment_id: &str) -> SubscriptionResult<String>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentStatus {
    pub payment_id: String,
    pub status: String,  // pending, completed, failed, refunded
    pub amount: u32,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Initialize payment systems
pub async fn init() -> SubscriptionResult<()> {
    tracing::info!("Initializing payment systems");
    
    // Initialize Lemon Squeezy client
    lemon_squeezy::init().await?;
    
    tracing::info!("Payment systems initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_provider_as_str() {
        assert_eq!(PaymentProvider::LemonSqueezy.as_str(), "lemon_squeezy");
        assert_eq!(PaymentProvider::Web3Polygon.as_str(), "web3_polygon");
        assert_eq!(PaymentProvider::Manual.as_str(), "manual");
    }
}
