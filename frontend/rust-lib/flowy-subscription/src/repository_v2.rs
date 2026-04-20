//! Database repository layer for subscription and payment data
//! Phase 2B SQL implementation with Diesel ORM

use crate::error::{SubscriptionError, SubscriptionResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use flowy_sqlite::{Database, DBConnection};
use std::sync::Arc;

// ============== Data Models ==============

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PaymentOrder {
    pub id: String,
    pub user_id: String,
    pub lemon_order_id: String,
    pub amount_cents: u32,
    pub currency: String,
    pub status: String,
    pub payment_method: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub webhook_verified_at: Option<DateTime<Utc>>,
}

impl PaymentOrder {
    pub fn new(user_id: String, lemon_order_id: String, amount_cents: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            lemon_order_id,
            amount_cents,
            currency: "USD".to_string(),
            status: "pending".to_string(),
            payment_method: "lemon_squeezy".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            webhook_verified_at: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SubscriptionRecord {
    pub id: String,
    pub user_id: String,
    pub plan_type: String,  // free, pro, team
    pub status: String,     // active, expired, cancelled
    pub lemon_subscription_id: Option<String>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub renewal_date: Option<DateTime<Utc>>,
    pub price_cents: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

impl SubscriptionRecord {
    pub fn new(user_id: String, plan_type: String, period_days: i64) -> Self {
        let now = Utc::now();
        let period_end = now + chrono::Duration::days(period_days);
        
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            plan_type,
            status: "active".to_string(),
            lemon_subscription_id: None,
            period_start: now,
            period_end,
            renewal_date: Some(period_end),
            price_cents: 0,
            created_at: now,
            updated_at: now,
            cancelled_at: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebhookEventLog {
    pub id: String,
    pub provider: String,  // lemon_squeezy, polygon_listener, paypal, paddle
    pub event_type: String,
    pub payload: String,   // JSON string
    pub signature: Option<String>,
    pub status: String,    // received, processed, failed
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

impl WebhookEventLog {
    pub fn new(provider: String, event_type: String, payload: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider,
            event_type,
            payload,
            signature: None,
            status: "received".to_string(),
            error_message: None,
            created_at: Utc::now(),
            processed_at: None,
        }
    }
}

// ============== Repository ==============

/// Repository for subscription and payment data
pub struct SubscriptionRepository {
    pub db: Arc<Database>,
}

impl SubscriptionRepository {
    pub fn new(db: Database) -> Self {
        Self {
            db: Arc::new(db),
        }
    }

    // ========== Orders ==========

    /// Save payment order to database
    pub async fn save_order(&self, order: PaymentOrder) -> SubscriptionResult<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()
                .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            let payload = format!(
                "INSERT INTO payment_orders (id, user_id, lemon_order_id, amount_cents, currency, status, payment_method, created_at, updated_at, webhook_verified_at) 
                 VALUES ('{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', {})",
                order.id, order.user_id, order.lemon_order_id, order.amount_cents,
                order.currency, order.status, order.payment_method,
                order.created_at.to_rfc3339(), order.updated_at.to_rfc3339(),
                order.webhook_verified_at.map(|t| format!("'{}'", t.to_rfc3339())).unwrap_or_else(|| "NULL".to_string())
            );
            
            diesel::RunQueryDsl::execute(
                diesel::sql_query(&payload),
                &mut conn
            ).map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            tracing::info!("Saved order: {}", order.id);
            Ok(())
        })
        .await
        .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?
    }

    /// Get order by ID
    pub async fn get_order(&self, order_id: &str) -> SubscriptionResult<Option<PaymentOrder>> {
        tracing::debug!("Getting order: {}", order_id);
        // TODO: SELECT * FROM payment_orders WHERE id = ?
        Ok(None)
    }

    /// Get order by Lemon Squeezy ID
    pub async fn get_order_by_lemon_id(&self, lemon_order_id: &str) -> SubscriptionResult<Option<PaymentOrder>> {
        tracing::debug!("Getting order by Lemon ID: {}", lemon_order_id);
        // TODO: SELECT * FROM payment_orders WHERE lemon_order_id = ?
        Ok(None)
    }

    /// List orders for user
    pub async fn list_user_orders(&self, user_id: &str) -> SubscriptionResult<Vec<PaymentOrder>> {
        tracing::info!("Listing orders for user: {}", user_id);
        // TODO: SELECT * FROM payment_orders WHERE user_id = ? ORDER BY created_at DESC
        Ok(Vec::new())
    }

    /// Update order status
    pub async fn update_order_status(
        &self,
        order_id: &str,
        status: &str,
    ) -> SubscriptionResult<()> {
        tracing::info!("Updating order {} status to {}", order_id, status);
        // TODO: UPDATE payment_orders SET status = ?, updated_at = ? WHERE id = ?
        Ok(())
    }

    // ========== Subscriptions ==========

    /// Save subscription record
    pub async fn save_subscription(&self, subscription: SubscriptionRecord) -> SubscriptionResult<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()
                .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            let payload = format!(
                "INSERT INTO user_subscription (id, user_id, plan_type, status, lemon_subscription_id, period_start, period_end, renewal_date, price_cents, created_at, updated_at) 
                 VALUES ('{}', '{}', '{}', '{}', {}, '{}', '{}', {}, {}, '{}', '{}')",
                subscription.id, subscription.user_id, subscription.plan_type, subscription.status,
                subscription.lemon_subscription_id.as_ref().map(|s| format!("'{}'", s)).unwrap_or_else(|| "NULL".to_string()),
                subscription.period_start.to_rfc3339(), subscription.period_end.to_rfc3339(),
                subscription.renewal_date.map(|t| format!("'{}'", t.to_rfc3339())).unwrap_or_else(|| "NULL".to_string()),
                subscription.price_cents,
                subscription.created_at.to_rfc3339(), subscription.updated_at.to_rfc3339()
            );
            
            diesel::RunQueryDsl::execute(
                diesel::sql_query(&payload),
                &mut conn
            ).map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            tracing::info!("Saved subscription: {}", subscription.id);
            Ok(())
        })
        .await
        .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?
    }

    /// Get user's current subscription
    pub async fn get_user_subscription(&self, user_id: &str) -> SubscriptionResult<Option<SubscriptionRecord>> {
        tracing::debug!("Getting subscription for user: {}", user_id);
        // TODO: SELECT * FROM user_subscription WHERE user_id = ? AND status = 'active' ORDER BY created_at DESC LIMIT 1
        Ok(None)
    }

    /// Update subscription status
    pub async fn update_subscription_status(
        &self,
        subscription_id: &str,
        status: &str,
    ) -> SubscriptionResult<()> {
        tracing::info!("Updating subscription {} status to {}", subscription_id, status);
        // TODO: UPDATE user_subscription SET status = ?, updated_at = ? WHERE id = ?
        Ok(())
    }

    /// Cancel subscription
    pub async fn cancel_subscription(&self, subscription_id: &str) -> SubscriptionResult<()> {
        tracing::info!("Cancelling subscription: {}", subscription_id);
        // TODO: UPDATE user_subscription SET status = 'cancelled', cancelled_at = ?, updated_at = ? WHERE id = ?
        Ok(())
    }

    /// List all active subscriptions
    pub async fn list_active_subscriptions(&self) -> SubscriptionResult<Vec<SubscriptionRecord>> {
        tracing::info!("Listing active subscriptions");
        // TODO: SELECT * FROM user_subscription WHERE status = 'active' ORDER BY created_at DESC
        Ok(Vec::new())
    }

    // ========== Webhook Events ==========

    /// Log webhook event
    pub async fn log_webhook_event(&self, event: WebhookEventLog) -> SubscriptionResult<()> {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = db.get_connection()
                .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            let payload = format!(
                "INSERT INTO payment_webhooks (id, provider, event_type, payload, signature, status, error_message, created_at) 
                 VALUES ('{}', '{}', '{}', '{}', {}, '{}', {}, '{}')",
                event.id, event.provider, event.event_type, 
                event.payload.replace('\'', "''"),  // Escape single quotes
                event.signature.as_ref().map(|s| format!("'{}'", s)).unwrap_or_else(|| "NULL".to_string()),
                event.status,
                event.error_message.as_ref().map(|s| format!("'{}'", s)).unwrap_or_else(|| "NULL".to_string()),
                event.created_at.to_rfc3339()
            );
            
            diesel::RunQueryDsl::execute(
                diesel::sql_query(&payload),
                &mut conn
            ).map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?;
            
            tracing::info!("Logged webhook event: {}", event.id);
            Ok(())
        })
        .await
        .map_err(|e| SubscriptionError::DatabaseError(format!("{:?}", e)))?
    }

    /// Update webhook event status
    pub async fn update_webhook_status(
        &self,
        webhook_id: &str,
        status: &str,
        _error_message: Option<&str>,
    ) -> SubscriptionResult<()> {
        tracing::info!("Updating webhook {} status to {}", webhook_id, status);
        // TODO: UPDATE payment_webhooks SET status = ?, error_message = ?, processed_at = ? WHERE id = ?
        Ok(())
    }

    /// Get webhook event by ID
    pub async fn get_webhook_event(&self, webhook_id: &str) -> SubscriptionResult<Option<WebhookEventLog>> {
        tracing::debug!("Getting webhook event: {}", webhook_id);
        // TODO: SELECT * FROM payment_webhooks WHERE id = ?
        Ok(None)
    }

    /// List unprocessed webhook events
    pub async fn list_unprocessed_webhooks(&self) -> SubscriptionResult<Vec<WebhookEventLog>> {
        tracing::info!("Listing unprocessed webhooks");
        // TODO: SELECT * FROM payment_webhooks WHERE status = 'received' ORDER BY created_at ASC
        Ok(Vec::new())
    }

    // ========== Transactions ==========

    /// Start transaction
    pub async fn begin_transaction(&self) -> SubscriptionResult<()> {
        // TODO: BEGIN TRANSACTION
        Ok(())
    }

    /// Commit transaction
    pub async fn commit_transaction(&self) -> SubscriptionResult<()> {
        // TODO: COMMIT
        Ok(())
    }

    /// Rollback transaction
    pub async fn rollback_transaction(&self) -> SubscriptionResult<()> {
        // TODO: ROLLBACK
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_order_creation() {
        let order = PaymentOrder::new(
            "user123".to_string(),
            "lemon-order-456".to_string(),
            9999,
        );

        assert_eq!(order.user_id, "user123");
        assert_eq!(order.lemon_order_id, "lemon-order-456");
        assert_eq!(order.amount_cents, 9999);
        assert_eq!(order.status, "pending");
    }

    #[test]
    fn test_subscription_creation() {
        let sub = SubscriptionRecord::new(
            "user123".to_string(),
            "pro".to_string(),
            30,
        );

        assert_eq!(sub.user_id, "user123");
        assert_eq!(sub.plan_type, "pro");
        assert_eq!(sub.status, "active");
    }

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEventLog::new(
            "lemon_squeezy".to_string(),
            "order.completed".to_string(),
            r#"{"order_id": "123"}"#.to_string(),
        );

        assert_eq!(event.provider, "lemon_squeezy");
        assert_eq!(event.event_type, "order.completed");
        assert_eq!(event.status, "received");
    }
}
