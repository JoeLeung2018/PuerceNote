//! Subscription management module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::SubscriptionResult;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionPlan {
    Free,
    Pro,
    Team,
}

impl SubscriptionPlan {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionPlan::Free => "free",
            SubscriptionPlan::Pro => "pro",
            SubscriptionPlan::Team => "team",
        }
    }

    pub fn monthly_price_cents(&self) -> u32 {
        match self {
            SubscriptionPlan::Free => 0,
            SubscriptionPlan::Pro => 999,    // $9.99
            SubscriptionPlan::Team => 1999,  // $19.99
        }
    }

    pub fn token_limit(&self) -> u32 {
        match self {
            SubscriptionPlan::Free => 0,
            SubscriptionPlan::Pro => 100_000,
            SubscriptionPlan::Team => 500_000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
    Suspended,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Expired => "expired",
            SubscriptionStatus::Cancelled => "cancelled",
            SubscriptionStatus::Suspended => "suspended",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_id: String,
    pub plan: SubscriptionPlan,
    pub status: SubscriptionStatus,
    pub payment_method: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub renewal_date: Option<DateTime<Utc>>,
    pub ai_enabled: bool,
    pub semantic_search_enabled: bool,
    pub collaboration_enabled: bool,
    pub price_cents: u32,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub user_id: String,
    pub plan: String,
    pub payment_method: String,
    pub payment_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub status: Option<String>,
    pub plan: Option<String>,
}

/// Subscription service
pub struct SubscriptionService {
    // TODO: Add database pool
}

impl SubscriptionService {
    pub fn new() -> Self {
        Self {}
    }

    /// Create new subscription
    pub async fn create_subscription(
        &self,
        _request: CreateSubscriptionRequest,
    ) -> SubscriptionResult<Subscription> {
        // TODO: Implement subscription creation
        todo!()
    }

    /// Get user subscription
    pub async fn get_subscription(&self, _user_id: &str) -> SubscriptionResult<Option<Subscription>> {
        // TODO: Implement subscription retrieval
        todo!()
    }

    /// Update subscription
    pub async fn update_subscription(
        &self,
        _user_id: &str,
        _request: UpdateSubscriptionRequest,
    ) -> SubscriptionResult<Subscription> {
        // TODO: Implement subscription update
        todo!()
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, _user_id: &str) -> SubscriptionResult<()> {
        // TODO: Implement subscription cancellation
        todo!()
    }

    /// Check if subscription is active
    pub async fn is_subscription_active(&self, _user_id: &str) -> SubscriptionResult<bool> {
        // TODO: Implement status check
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_properties() {
        assert_eq!(SubscriptionPlan::Pro.as_str(), "pro");
        assert_eq!(SubscriptionPlan::Pro.monthly_price_cents(), 999);
        assert_eq!(SubscriptionPlan::Pro.token_limit(), 100_000);

        assert_eq!(SubscriptionPlan::Team.as_str(), "team");
        assert_eq!(SubscriptionPlan::Team.monthly_price_cents(), 1999);
        assert_eq!(SubscriptionPlan::Team.token_limit(), 500_000);
    }
}
