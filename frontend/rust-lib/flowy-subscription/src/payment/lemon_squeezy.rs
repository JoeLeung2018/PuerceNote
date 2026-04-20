//! Lemon Squeezy payment provider
//!
//! Implements Lemon Squeezy API integration for credit card payments

use crate::config::SubscriptionConfig;
use crate::error::{SubscriptionError, SubscriptionResult};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::Sha256;
use std::sync::Arc;
use tracing::{debug, error, info};

type HmacSha256 = Hmac<Sha256>;

static LEMON_SQUEEZY_CLIENT: std::sync::OnceLock<Arc<LemonSqueezyClient>> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct LemonSqueezyClient {
    config: SubscriptionConfig,
    http_client: Client,
}

impl LemonSqueezyClient {
    pub fn new(config: SubscriptionConfig) -> Self {
        Self {
            config,
            http_client: Client::new(),
        }
    }

    /// Initialize global Lemon Squeezy client
    pub async fn init(config: SubscriptionConfig) -> SubscriptionResult<()> {
        let client = Self::new(config);
        
        // Test API connection
        client.test_connection().await?;
        
        let _ = LEMON_SQUEEZY_CLIENT.set(Arc::new(client));
        info!("Lemon Squeezy client initialized");
        Ok(())
    }

    /// Get global Lemon Squeezy client
    pub fn get() -> SubscriptionResult<Arc<LemonSqueezyClient>> {
        LEMON_SQUEEZY_CLIENT
            .get()
            .cloned()
            .ok_or_else(|| SubscriptionError::LemonSqueezyError("Client not initialized".to_string()))
    }

    /// Test API connection
    async fn test_connection(&self) -> SubscriptionResult<()> {
        debug!("Testing Lemon Squeezy API connection");
        
        let response = self
            .http_client
            .get(format!("{}/v1/stores/{}", self.config.lemon_squeezy_api_url, self.config.lemon_squeezy_store_id))
            .bearer_auth(&self.config.lemon_squeezy_api_key)
            .send()
            .await?;

        if response.status().is_success() {
            info!("Lemon Squeezy API connection successful");
            Ok(())
        } else {
            error!("Lemon Squeezy API connection failed: {}", response.status());
            Err(SubscriptionError::LemonSqueezyError(format!(
                "Connection failed: {}",
                response.status()
            )))
        }
    }

    /// Create checkout session
    pub async fn create_checkout(
        &self,
        product_id: u32,
        variant_id: u32,
        email: Option<String>,
        user_id: Option<String>,
    ) -> SubscriptionResult<CheckoutResponse> {
        info!("Creating checkout for product: {}", product_id);

        let body = json!({
            "data": {
                "type": "checkouts",
                "attributes": {
                    "product_id": product_id,
                    "variant_id": variant_id,
                    "customer_email": email,
                    "metadata": {
                        "user_id": user_id,
                    }
                }
            }
        });

        let response = self
            .http_client
            .post(format!(
                "{}/v1/stores/{}/checkouts",
                self.config.lemon_squeezy_api_url, self.config.lemon_squeezy_store_id
            ))
            .bearer_auth(&self.config.lemon_squeezy_api_key)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let checkout: CheckoutResponse = response.json().await?;
            info!("Checkout created: {}", checkout.data.attributes.url);
            Ok(checkout)
        } else {
            let error_msg = response.text().await.unwrap_or_default();
            error!("Failed to create checkout: {}", error_msg);
            Err(SubscriptionError::LemonSqueezyError(format!(
                "Failed to create checkout: {}",
                error_msg
            )))
        }
    }

    /// Verify webhook signature
    pub fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> SubscriptionResult<bool> {
        let mut mac = HmacSha256::new_from_slice(self.config.lemon_squeezy_webhook_secret.as_bytes())
            .map_err(|e| SubscriptionError::LemonSqueezyError(format!("Invalid key: {}", e)))?;
        
        mac.update(payload);
        
        let computed = hex::encode(mac.finalize().into_bytes());
        
        if computed == signature {
            debug!("Webhook signature verified");
            Ok(true)
        } else {
            error!("Webhook signature mismatch");
            Err(SubscriptionError::WebhookSignatureInvalid)
        }
    }

    /// Process webhook event
    pub async fn process_webhook(
        &self,
        event_data: WebhookEvent,
    ) -> SubscriptionResult<ProcessedWebhookEvent> {
        info!("Processing webhook event: {}", event_data.event_type);

        let processed = match event_data.event_type.as_str() {
            "order.created" => ProcessedWebhookEvent::OrderCreated {
                order_id: event_data.data.get("order_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                user_id: event_data.data.get("user_id").and_then(|v| v.as_str()).map(|s| s.to_string()),
            },
            "order.updated" => ProcessedWebhookEvent::OrderUpdated {
                order_id: event_data.data.get("order_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                status: event_data.data.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            },
            "subscription.created" => ProcessedWebhookEvent::SubscriptionCreated {
                subscription_id: event_data.data.get("subscription_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            },
            "subscription.updated" => ProcessedWebhookEvent::SubscriptionUpdated {
                subscription_id: event_data.data.get("subscription_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            },
            "subscription.cancelled" => ProcessedWebhookEvent::SubscriptionCancelled {
                subscription_id: event_data.data.get("subscription_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            },
            _ => ProcessedWebhookEvent::Unknown {
                event_type: event_data.event_type,
            },
        };

        debug!("Webhook processed: {:?}", processed);
        Ok(processed)
    }

    /// Get order details
    pub async fn get_order(&self, order_id: u32) -> SubscriptionResult<OrderResponse> {
        debug!("Fetching order: {}", order_id);

        let response = self
            .http_client
            .get(format!(
                "{}/v1/stores/{}/orders/{}",
                self.config.lemon_squeezy_api_url, self.config.lemon_squeezy_store_id, order_id
            ))
            .bearer_auth(&self.config.lemon_squeezy_api_key)
            .send()
            .await?;

        if response.status().is_success() {
            let order = response.json().await?;
            Ok(order)
        } else {
            let error_msg = response.text().await.unwrap_or_default();
            Err(SubscriptionError::LemonSqueezyError(format!(
                "Failed to fetch order: {}",
                error_msg
            )))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckoutResponse {
    pub data: CheckoutData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckoutData {
    pub id: String,
    pub attributes: CheckoutAttributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckoutAttributes {
    pub url: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderResponse {
    pub data: OrderData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderData {
    pub id: String,
    pub attributes: OrderAttributes,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderAttributes {
    pub status: String,
    pub total: u32,
    pub currency: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event_type: String,
    pub data: serde_json::Value,
}

#[derive(Clone, Debug)]
pub enum ProcessedWebhookEvent {
    OrderCreated {
        order_id: String,
        user_id: Option<String>,
    },
    OrderUpdated {
        order_id: String,
        status: String,
    },
    SubscriptionCreated {
        subscription_id: String,
    },
    SubscriptionUpdated {
        subscription_id: String,
    },
    SubscriptionCancelled {
        subscription_id: String,
    },
    Unknown {
        event_type: String,
    },
}

/// Initialize Lemon Squeezy module
pub async fn init() -> SubscriptionResult<()> {
    let config = SubscriptionConfig::from_env()?;
    config.validate()?;
    LemonSqueezyClient::init(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_signature() {
        let config = SubscriptionConfig {
            lemon_squeezy_webhook_secret: "test-secret".to_string(),
            ..Default::default()
        };

        let client = LemonSqueezyClient::new(config);
        let payload = b"test payload";
        
        let mut mac = HmacSha256::new_from_slice(b"test-secret").unwrap();
        mac.update(payload);
        let signature = hex::encode(mac.finalize().into_bytes());

        assert!(client.verify_webhook_signature(payload, &signature).is_ok());
    }
}
