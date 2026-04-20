//! Webhook event handler for payment processing
//! Phase 2B.Webhook: Handles payment events from Lemon Squeezy, PayPal, Paddle, and Polygon

use crate::error::{SubscriptionError, SubscriptionResult};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::sync::Arc;

type HmacSha256 = Hmac<Sha256>;

// ============== Event Models ==============

#[derive(Debug, Clone)]
pub enum WebhookProvider {
    LemonSqueezy,
    PayPal,
    Paddle,
    PolygonListener,
}

#[derive(Debug, Clone)]
pub enum WebhookEventType {
    // Lemon Squeezy events
    OrderCreated,
    OrderCompleted,
    SubscriptionCreated,
    SubscriptionUpdated,
    SubscriptionCancelled,
    
    // PayPal events
    PaymentCaptureCompleted,
    BillingSubscriptionCreated,
    BillingSubscriptionCancelled,
    
    // Paddle events
    TransactionCompleted,
    SubscriptionPaused,
    SubscriptionResumed,
    
    // Polygon events (Web3)
    TransactionConfirmed,
    TransactionFailed,
}

#[derive(Debug, Clone)]
pub struct WebhookEvent {
    pub id: String,
    pub provider: WebhookProvider,
    pub event_type: WebhookEventType,
    pub payload: Value,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
}

// ============== Webhook Processor ==============

pub struct WebhookProcessor {
    secret_keys: WebhookSecrets,
    idempotency_cache: Arc<tokio::sync::Mutex<IdempotencyCache>>,
}

pub struct WebhookSecrets {
    pub lemon_squeezy_secret: String,
    pub paypal_secret: String,
    pub paddle_secret: String,
    pub polygon_secret: String,
}

struct IdempotencyCache {
    // webhook_id -> processing_result
    processed: std::collections::HashMap<String, ProcessingResult>,
}

#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub webhook_id: String,
    pub success: bool,
    pub message: String,
    pub processed_at: DateTime<Utc>,
}

impl WebhookProcessor {
    pub fn new(
        secrets: WebhookSecrets,
    ) -> Self {
        Self {
            secret_keys: secrets,
            idempotency_cache: Arc::new(tokio::sync::Mutex::new(
                IdempotencyCache {
                    processed: std::collections::HashMap::new(),
                }
            )),
        }
    }
    
    // ═══════════════════════════════════════════════════════════
    // Main webhook processing entry point
    // ═══════════════════════════════════════════════════════════
    
    pub async fn process_webhook(
        &self,
        provider: WebhookProvider,
        raw_body: &str,
        signature: &str,
        webhook_id: &str,
    ) -> SubscriptionResult<ProcessingResult> {
        // Step 1: Verify signature
        self.verify_signature(&provider, raw_body, signature)?;
        
        // Step 2: Check idempotency (prevent duplicate processing)
        let cache = self.idempotency_cache.lock().await;
        if let Some(cached_result) = cache.processed.get(webhook_id) {
            tracing::warn!("Webhook already processed: {}", webhook_id);
            return Ok(cached_result.clone());
        }
        drop(cache);
        
        // Step 3: Parse payload
        let payload: Value = serde_json::from_str(raw_body)
            .map_err(|e| SubscriptionError::InvalidRequest(e.to_string()))?;
        
        // Step 4: Determine event type and route
        let event_type = self.extract_event_type(&provider, &payload)?;
        
        // Step 5: Process event (within transaction)
        let result = match provider {
            WebhookProvider::LemonSqueezy => {
                self.handle_lemon_squeezy_event(&payload, event_type.clone()).await
            }
            WebhookProvider::PayPal => {
                self.handle_paypal_event(&payload, event_type.clone()).await
            }
            WebhookProvider::Paddle => {
                self.handle_paddle_event(&payload, event_type.clone()).await
            }
            WebhookProvider::PolygonListener => {
                self.handle_polygon_event(&payload, event_type.clone()).await
            }
        }?;
        
        // Step 6: Cache result for idempotency
        let processing_result = ProcessingResult {
            webhook_id: webhook_id.to_string(),
            success: true,
            message: result,
            processed_at: Utc::now(),
        };
        
        let mut cache = self.idempotency_cache.lock().await;
        cache.processed.insert(
            webhook_id.to_string(),
            processing_result.clone(),
        );
        
        // Step 7: Log webhook event
        self.log_webhook_event(
            provider,
            event_type,
            &payload,
            signature,
            true,
        ).await?;
        
        Ok(processing_result)
    }
    
    // ═══════════════════════════════════════════════════════════
    // Signature verification (different per provider)
    // ═══════════════════════════════════════════════════════════
    
    fn verify_signature(
        &self,
        provider: &WebhookProvider,
        raw_body: &str,
        signature: &str,
    ) -> SubscriptionResult<()> {
        match provider {
            WebhookProvider::LemonSqueezy => {
                // Lemon Squeezy: X-Signature header (SHA-256)
                let expected = self.compute_hmac_sha256(
                    &self.secret_keys.lemon_squeezy_secret,
                    raw_body,
                )?;
                
                if !constant_time_compare(signature, &expected) {
                    return Err(SubscriptionError::WebhookSignatureInvalid);
                }
            }
            WebhookProvider::PayPal => {
                // PayPal: POST /simulation/signature-verify
                // (More complex, involves certificate validation)
                tracing::debug!("PayPal signature verification skipped (cert-based)");
            }
            WebhookProvider::Paddle => {
                // Paddle: X-Paddle-Signature (SHA-256 with prefix)
                let expected = format!(
                    "sha256={}",
                    hex::encode(
                        self.compute_hmac_sha256(
                            &self.secret_keys.paddle_secret,
                            raw_body,
                        )?
                    )
                );
                
                if !constant_time_compare(signature, &expected) {
                    return Err(SubscriptionError::WebhookSignatureInvalid);
                }
            }
            WebhookProvider::PolygonListener => {
                // Polygon: Uses webhook API service (no HMAC needed)
                tracing::debug!("Polygon webhook from trusted service");
            }
        }
        
        Ok(())
    }
    
    fn compute_hmac_sha256(&self, secret: &str, message: &str) -> SubscriptionResult<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| SubscriptionError::InvalidRequest("Invalid secret".to_string()))?;
        mac.update(message.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
    
    // ═══════════════════════════════════════════════════════════
    // Event type extraction
    // ═══════════════════════════════════════════════════════════
    
    fn extract_event_type(
        &self,
        provider: &WebhookProvider,
        payload: &Value,
    ) -> SubscriptionResult<WebhookEventType> {
        match provider {
            WebhookProvider::LemonSqueezy => {
                // Lemon Squeezy: payload.meta.event_name
                let event_name = payload
                    .get("meta")
                    .and_then(|m| m.get("event_name"))
                    .and_then(|n| n.as_str())
                    .ok_or(SubscriptionError::InvalidRequest("Missing event_name".to_string()))?;
                
                Ok(match event_name {
                    "order:created" => WebhookEventType::OrderCreated,
                    "order:completed" => WebhookEventType::OrderCompleted,
                    "subscription:created" => WebhookEventType::SubscriptionCreated,
                    "subscription:updated" => WebhookEventType::SubscriptionUpdated,
                    "subscription:cancelled" => WebhookEventType::SubscriptionCancelled,
                    _ => return Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
                })
            }
            WebhookProvider::PayPal => {
                // PayPal: event_type field
                let event_type = payload
                    .get("event_type")
                    .and_then(|t| t.as_str())
                    .ok_or(SubscriptionError::InvalidRequest("Missing event_type".to_string()))?;
                
                Ok(match event_type {
                    "PAYMENT.CAPTURE.COMPLETED" => WebhookEventType::PaymentCaptureCompleted,
                    "BILLING.SUBSCRIPTION.CREATED" => WebhookEventType::BillingSubscriptionCreated,
                    "BILLING.SUBSCRIPTION.CANCELLED" => WebhookEventType::BillingSubscriptionCancelled,
                    _ => return Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
                })
            }
            WebhookProvider::Paddle => {
                // Paddle: event.type field
                let event_type = payload
                    .get("event")
                    .and_then(|e| e.get("type"))
                    .and_then(|t| t.as_str())
                    .ok_or(SubscriptionError::InvalidRequest("Missing event.type".to_string()))?;
                
                Ok(match event_type {
                    "transaction.completed" => WebhookEventType::TransactionCompleted,
                    "subscription.paused" => WebhookEventType::SubscriptionPaused,
                    "subscription.resumed" => WebhookEventType::SubscriptionResumed,
                    _ => return Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
                })
            }
            WebhookProvider::PolygonListener => {
                // Polygon: transaction status
                let status = payload
                    .get("status")
                    .and_then(|s| s.as_str())
                    .ok_or(SubscriptionError::InvalidRequest("Missing status".to_string()))?;
                
                Ok(match status {
                    "confirmed" => WebhookEventType::TransactionConfirmed,
                    "failed" => WebhookEventType::TransactionFailed,
                    _ => return Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
                })
            }
        }
    }
    
    // ═══════════════════════════════════════════════════════════
    // Lemon Squeezy event handlers
    // ═══════════════════════════════════════════════════════════
    
    async fn handle_lemon_squeezy_event(
        &self,
        payload: &Value,
        event_type: WebhookEventType,
    ) -> SubscriptionResult<String> {
        match event_type {
            WebhookEventType::OrderCompleted => {
                self.handle_lemon_order_completed(payload).await
            }
            WebhookEventType::SubscriptionCreated => {
                self.handle_lemon_subscription_created(payload).await
            }
            WebhookEventType::SubscriptionUpdated => {
                self.handle_lemon_subscription_updated(payload).await
            }
            WebhookEventType::SubscriptionCancelled => {
                self.handle_lemon_subscription_cancelled(payload).await
            }
            _ => Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
        }
    }
    
    async fn handle_lemon_order_completed(&self, payload: &Value) -> SubscriptionResult<String> {
        // Extract order details
        let lemon_order_id = payload
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing order ID".to_string()))?;
        
        let user_id = payload
            .get("data")
            .and_then(|d| d.get("attributes"))
            .and_then(|a| a.get("customer_email"))
            .and_then(|e| e.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing customer email".to_string()))?;
        
        let amount_cents = payload
            .get("data")
            .and_then(|d| d.get("attributes"))
            .and_then(|a| a.get("total_formatted"))
            .and_then(|t| t.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing amount".to_string()))?;
        
        // Log order (database save will be implemented when repository_v2 is ready)
        tracing::info!(
            "✅ Lemon Squeezy order completed: id={}, user={}, amount={}",
            lemon_order_id,
            user_id,
            amount_cents
        );
        
        // TODO: Save to database via repository_v2 when ready
        // self.repo.save_order(...).await?;
        
        Ok(format!("Order {} processed", lemon_order_id))
    }
    
    async fn handle_lemon_subscription_created(&self, payload: &Value) -> SubscriptionResult<String> {
        let sub_id = payload
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing subscription ID".to_string()))?;
        
        let user_id = payload
            .get("data")
            .and_then(|d| d.get("attributes"))
            .and_then(|a| a.get("customer_email"))
            .and_then(|e| e.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing customer email".to_string()))?;
        
        let plan_type = extract_plan_from_payload(payload)?;
        
        // Log subscription (database save will be implemented when repository_v2 is ready)
        tracing::info!(
            "✅ Lemon Squeezy subscription created: id={}, user={}, plan={}",
            sub_id,
            user_id,
            plan_type
        );
        
        // TODO: Save to database via repository_v2 when ready
        // self.repo.save_subscription(...).await?;
        
        Ok(format!("Subscription {} activated", sub_id))
    }
    
    async fn handle_lemon_subscription_updated(&self, payload: &Value) -> SubscriptionResult<String> {
        let sub_id = payload
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing subscription ID".to_string()))?;
        
        tracing::info!("✅ Lemon Squeezy subscription updated: {}", sub_id);
        Ok(format!("Subscription {} updated", sub_id))
    }
    
    async fn handle_lemon_subscription_cancelled(&self, payload: &Value) -> SubscriptionResult<String> {
        let sub_id = payload
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing subscription ID".to_string()))?;
        
        tracing::info!("⚠️ Lemon Squeezy subscription cancelled: {}", sub_id);
        Ok(format!("Subscription {} cancelled", sub_id))
    }
    
    // ═══════════════════════════════════════════════════════════
    // PayPal event handlers (simplified)
    // ═══════════════════════════════════════════════════════════
    
    async fn handle_paypal_event(
        &self,
        payload: &Value,
        event_type: WebhookEventType,
    ) -> SubscriptionResult<String> {
        match event_type {
            WebhookEventType::PaymentCaptureCompleted => {
                self.handle_paypal_payment_completed(payload).await
            }
            _ => Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
        }
    }
    
    async fn handle_paypal_payment_completed(&self, payload: &Value) -> SubscriptionResult<String> {
        let order_id = payload
            .get("resource")
            .and_then(|r| r.get("supplementary_data"))
            .and_then(|s| s.get("related_ids"))
            .and_then(|ri| ri.get("order_id"))
            .and_then(|o| o.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing order ID".to_string()))?;
        
        tracing::info!("✅ PayPal payment completed: {}", order_id);
        Ok(format!("PayPal order {} processed", order_id))
    }
    
    // ═══════════════════════════════════════════════════════════
    // Paddle event handlers (simplified)
    // ═══════════════════════════════════════════════════════════
    
    async fn handle_paddle_event(
        &self,
        payload: &Value,
        event_type: WebhookEventType,
    ) -> SubscriptionResult<String> {
        match event_type {
            WebhookEventType::TransactionCompleted => {
                self.handle_paddle_transaction_completed(payload).await
            }
            _ => Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
        }
    }
    
    async fn handle_paddle_transaction_completed(&self, payload: &Value) -> SubscriptionResult<String> {
        let transaction_id = payload
            .get("event")
            .and_then(|e| e.get("data"))
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing transaction ID".to_string()))?;
        
        tracing::info!("✅ Paddle transaction completed: {}", transaction_id);
        Ok(format!("Paddle transaction {} processed", transaction_id))
    }
    
    // ═══════════════════════════════════════════════════════════
    // Polygon/Web3 event handlers
    // ═══════════════════════════════════════════════════════════
    
    async fn handle_polygon_event(
        &self,
        payload: &Value,
        event_type: WebhookEventType,
    ) -> SubscriptionResult<String> {
        match event_type {
            WebhookEventType::TransactionConfirmed => {
                self.handle_polygon_tx_confirmed(payload).await
            }
            WebhookEventType::TransactionFailed => {
                self.handle_polygon_tx_failed(payload).await
            }
            _ => Err(SubscriptionError::InvalidRequest("Unknown event type".to_string())),
        }
    }
    
    async fn handle_polygon_tx_confirmed(&self, payload: &Value) -> SubscriptionResult<String> {
        let tx_hash = payload
            .get("tx_hash")
            .and_then(|h| h.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing tx_hash".to_string()))?;
        
        tracing::info!("✅ Polygon transaction confirmed: {}", tx_hash);
        Ok(format!("Polygon TX {} confirmed", tx_hash))
    }
    
    async fn handle_polygon_tx_failed(&self, payload: &Value) -> SubscriptionResult<String> {
        let tx_hash = payload
            .get("tx_hash")
            .and_then(|h| h.as_str())
            .ok_or(SubscriptionError::InvalidRequest("Missing tx_hash".to_string()))?;
        
        tracing::warn!("❌ Polygon transaction failed: {}", tx_hash);
        Ok(format!("Polygon TX {} failed", tx_hash))
    }
    
    async fn log_webhook_event(
        &self,
        _provider: WebhookProvider,
        _event_type: WebhookEventType,
        _payload: &Value,
        _signature: &str,
        _success: bool,
    ) -> SubscriptionResult<()> {
        // TODO: Implement logging to database when repository_v2 is ready
        // self.repo.log_webhook_event(...).await?;
        
        tracing::info!("Webhook event logged (database logging to be implemented)");
        Ok(())
    }
}

// ============== Helper Functions ==============

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    
    result == 0
}

fn parse_amount(formatted: &str) -> SubscriptionResult<i32> {
    formatted
        .replace("$", "")
        .parse::<f64>()
        .map(|v| (v * 100.0) as i32)
        .map_err(|_| SubscriptionError::InvalidRequest("Invalid amount".to_string()))
}

fn extract_plan_from_payload(payload: &Value) -> SubscriptionResult<String> {
    payload
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("product_name"))
        .and_then(|n| n.as_str())
        .map(|s| {
            if s.contains("Pro") {
                "pro"
            } else if s.contains("Team") {
                "team"
            } else {
                "free"
            }
            .to_string()
        })
        .ok_or(SubscriptionError::InvalidRequest("Missing plan info".to_string()))
}

fn extract_price_from_payload(payload: &Value) -> SubscriptionResult<i32> {
    payload
        .get("data")
        .and_then(|d| d.get("attributes"))
        .and_then(|a| a.get("total_formatted"))
        .and_then(|t| t.as_str())
        .ok_or(SubscriptionError::InvalidRequest("Missing price".to_string()))
        .and_then(parse_amount)
}

// ============== Tests ==============

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constant_time_compare_equal() {
        assert!(constant_time_compare("abc123", "abc123"));
    }
    
    #[test]
    fn test_constant_time_compare_not_equal() {
        assert!(!constant_time_compare("abc123", "abc124"));
    }
    
    #[test]
    fn test_constant_time_compare_different_length() {
        assert!(!constant_time_compare("abc", "abcd"));
    }
    
    #[test]
    fn test_parse_amount() {
        assert_eq!(parse_amount("$14.99").unwrap(), 1499);
        assert_eq!(parse_amount("$99.99").unwrap(), 9999);
    }
    
    #[test]
    fn test_extract_plan_pro() {
        let payload = json!({
            "data": {
                "attributes": {
                    "product_name": "PuerceNote Pro"
                }
            }
        });
        assert_eq!(extract_plan_from_payload(&payload).unwrap(), "pro");
    }
    
    #[test]
    fn test_extract_plan_team() {
        let payload = json!({
            "data": {
                "attributes": {
                    "product_name": "PuerceNote Team"
                }
            }
        });
        assert_eq!(extract_plan_from_payload(&payload).unwrap(), "team");
    }
}
