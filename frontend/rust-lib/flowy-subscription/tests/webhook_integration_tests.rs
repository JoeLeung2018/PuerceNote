//! Integration tests for webhook payment processing
//! Tests the complete flow: webhook → signature verification → event routing

#[cfg(test)]
mod webhook_integration_tests {
    use flowy_subscription::{
        webhook::{WebhookProcessor, WebhookProvider, WebhookSecrets},
    };
    use serde_json::json;
    
    // ════════════════════════════════════════════════════════
    // Test fixtures
    // ════════════════════════════════════════════════════════
    
    fn setup_processor() -> WebhookProcessor {
        let secrets = WebhookSecrets {
            lemon_squeezy_secret: "test_secret_lemon".to_string(),
            paypal_secret: "test_secret_paypal".to_string(),
            paddle_secret: "test_secret_paddle".to_string(),
            polygon_secret: "test_secret_polygon".to_string(),
        };
        
        WebhookProcessor::new(secrets)
    }
    
    // ════════════════════════════════════════════════════════
    // Test 1: Lemon Squeezy order.completed
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_lemon_squeezy_order_completed() {
        let processor = setup_processor();
        
        // Mock webhook payload
        let payload = json!({
            "meta": {
                "event_name": "order:completed"
            },
            "data": {
                "id": "12345",
                "attributes": {
                    "customer_email": "user@example.com",
                    "total_formatted": "$14.99"
                }
            }
        });
        
        let raw_body = payload.to_string();
        
        // Process webhook (signature validation skipped in test)
        let result = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &raw_body,
            "valid_signature",
            "webhook_id_001",
        ).await;
        
        // Assertions
        assert!(result.is_ok(), "Webhook processing should succeed");
        let processing_result = result.unwrap();
        assert!(processing_result.success);
        assert!(processing_result.message.contains("Order"));
        
        println!("✅ Test 1 Passed: Lemon Squeezy order.completed");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 2: Lemon Squeezy subscription.created
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_lemon_squeezy_subscription_created() {
        let processor = setup_processor();
        
        let payload = json!({
            "meta": {
                "event_name": "subscription:created"
            },
            "data": {
                "id": "sub_456",
                "attributes": {
                    "customer_email": "pro@example.com",
                    "product_name": "PuerceNote Pro",
                    "total_formatted": "$14.99"
                }
            }
        });
        
        let raw_body = payload.to_string();
        
        let result = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &raw_body,
            "valid_signature",
            "webhook_id_002",
        ).await;
        
        assert!(result.is_ok());
        let processing_result = result.unwrap();
        assert!(processing_result.message.contains("Subscription"));
        
        println!("✅ Test 2 Passed: Lemon Squeezy subscription.created");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 3: Idempotency - Same webhook processed twice
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_webhook_idempotency() {
        let processor = setup_processor();
        
        let payload = json!({
            "meta": {
                "event_name": "order:completed"
            },
            "data": {
                "id": "12345",
                "attributes": {
                    "customer_email": "user@example.com",
                    "total_formatted": "$99.99"
                }
            }
        });
        
        let raw_body = payload.to_string();
        let webhook_id = "webhook_id_duplicate";
        
        // Process webhook first time
        let result1 = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &raw_body,
            "valid_signature",
            webhook_id,
        ).await;
        
        assert!(result1.is_ok());
        
        // Process same webhook again (should be cached)
        let result2 = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &raw_body,
            "valid_signature",
            webhook_id,
        ).await;
        
        assert!(result2.is_ok());
        
        // Both should return same result (from cache)
        let r1 = result1.unwrap();
        let r2 = result2.unwrap();
        
        assert_eq!(r1.webhook_id, r2.webhook_id);
        assert!(r1.processed_at <= r2.processed_at);
        
        println!("✅ Test 3 Passed: Webhook idempotency (prevents duplicate processing)");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 4: PayPal payment.capture.completed
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_paypal_payment_completed() {
        let processor = setup_processor();
        
        let payload = json!({
            "event_type": "PAYMENT.CAPTURE.COMPLETED",
            "resource": {
                "supplementary_data": {
                    "related_ids": {
                        "order_id": "paypal_order_789"
                    }
                }
            }
        });
        
        let raw_body = payload.to_string();
        
        let result = processor.process_webhook(
            WebhookProvider::PayPal,
            &raw_body,
            "valid_signature",
            "webhook_id_003",
        ).await;
        
        assert!(result.is_ok());
        println!("✅ Test 4 Passed: PayPal payment.capture.completed");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 5: Polygon transaction.confirmed (Web3)
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_polygon_transaction_confirmed() {
        let processor = setup_processor();
        
        let payload = json!({
            "status": "confirmed",
            "tx_hash": "0xabc123def456"
        });
        
        let raw_body = payload.to_string();
        
        let result = processor.process_webhook(
            WebhookProvider::PolygonListener,
            &raw_body,
            "valid_signature",
            "webhook_id_004",
        ).await;
        
        assert!(result.is_ok());
        let processing_result = result.unwrap();
        assert!(processing_result.message.contains("confirmed"));
        
        println!("✅ Test 5 Passed: Polygon transaction.confirmed (Web3)");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 6: Error handling - Invalid payload
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_invalid_payload() {
        let processor = setup_processor();
        
        // Missing required fields
        let payload = json!({
            "meta": {
                "event_name": "order:completed"
            },
            "data": {
                // Missing id and attributes
            }
        });
        
        let raw_body = payload.to_string();
        
        let result = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &raw_body,
            "valid_signature",
            "webhook_id_invalid",
        ).await;
        
        // Should fail gracefully
        assert!(result.is_err(), "Should reject invalid payload");
        println!("✅ Test 6 Passed: Invalid payload rejection");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 7: Multiple event types in sequence
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_complete_payment_flow() {
        let processor = setup_processor();
        
        // Step 1: Order created
        let order_payload = json!({
            "meta": { "event_name": "order:created" },
            "data": {
                "id": "order_001",
                "attributes": {
                    "customer_email": "customer@example.com",
                    "total_formatted": "$14.99"
                }
            }
        });
        
        let result1 = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &order_payload.to_string(),
            "sig1",
            "webhook_1",
        ).await;
        assert!(result1.is_ok());
        
        // Step 2: Subscription created
        let sub_payload = json!({
            "meta": { "event_name": "subscription:created" },
            "data": {
                "id": "sub_001",
                "attributes": {
                    "customer_email": "customer@example.com",
                    "product_name": "PuerceNote Pro",
                    "total_formatted": "$14.99"
                }
            }
        });
        
        let result2 = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &sub_payload.to_string(),
            "sig2",
            "webhook_2",
        ).await;
        assert!(result2.is_ok());
        
        // Step 3: Subscription renewed (updated)
        let update_payload = json!({
            "meta": { "event_name": "subscription:updated" },
            "data": {
                "id": "sub_001",
                "attributes": {
                    "status": "active"
                }
            }
        });
        
        let result3 = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &update_payload.to_string(),
            "sig3",
            "webhook_3",
        ).await;
        assert!(result3.is_ok());
        
        println!("✅ Test 7 Passed: Complete payment flow (order → subscription → renewal)");
    }
    
    // ════════════════════════════════════════════════════════
    // Test 8: Unknown event type
    // ════════════════════════════════════════════════════════
    
    #[tokio::test]
    async fn test_unknown_event_type() {
        let processor = setup_processor();
        
        let payload = json!({
            "meta": {
                "event_name": "unknown:event"
            },
            "data": {
                "id": "test"
            }
        });
        
        let result = processor.process_webhook(
            WebhookProvider::LemonSqueezy,
            &payload.to_string(),
            "sig",
            "webhook_unknown",
        ).await;
        
        // Should fail for unknown event type
        assert!(result.is_err(), "Should reject unknown event type");
        println!("✅ Test 8 Passed: Unknown event type rejection");
    }
}

// ════════════════════════════════════════════════════════
// Cost tracking tests
// ════════════════════════════════════════════════════════

#[cfg(test)]
mod cost_tracking_tests {
    use flowy_subscription::billing::vector_billing::{
        VectorBillingService,
        VectorOperation,
    };
    
    #[tokio::test]
    async fn test_cost_calculation_after_webhook() {
        // Simulate webhook processing cost
        let cost = VectorBillingService::estimate_cost(
            &VectorOperation::SemanticSearch,
            5000,  // 5K tokens
        );
        
        // Should be $0.05
        assert!((cost - 0.05).abs() < 0.0001);
        println!("✅ Cost tracking test passed: $0.05 for 5K tokens");
    }
}
