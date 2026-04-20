-- Create subscription and payment tables for PuerceNote Phase 2B
-- These tables store all payment, subscription, and webhook data

-- Payment orders table (tracks all payment attempts across providers)
CREATE TABLE payment_orders (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    lemon_order_id TEXT NOT NULL,
    amount_cents INTEGER NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, completed, failed, refunded
    payment_method TEXT NOT NULL,  -- lemon_squeezy, paypal, paddle, web3, etc.
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    webhook_verified_at TIMESTAMP,
    
    UNIQUE(lemon_order_id),
    INDEX idx_user_id (user_id),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);

-- User subscription table (tracks active subscriptions)
CREATE TABLE user_subscription (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL UNIQUE,
    plan_type TEXT NOT NULL,  -- free, pro, team
    status TEXT NOT NULL DEFAULT 'active',  -- active, expired, cancelled
    lemon_subscription_id TEXT,
    period_start TIMESTAMP NOT NULL,
    period_end TIMESTAMP NOT NULL,
    renewal_date TIMESTAMP,
    price_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    cancelled_at TIMESTAMP,
    
    INDEX idx_user_id (user_id),
    INDEX idx_status (status),
    INDEX idx_renewal_date (renewal_date)
);

-- Payment webhook events table (audit log for all webhook events)
CREATE TABLE payment_webhooks (
    id TEXT PRIMARY KEY NOT NULL,
    provider TEXT NOT NULL,  -- lemon_squeezy, paypal, paddle, polygon_listener
    event_type TEXT NOT NULL,  -- order.completed, subscription.created, etc.
    payload TEXT NOT NULL,  -- JSON string of event data
    signature TEXT,  -- Webhook signature for verification
    status TEXT NOT NULL DEFAULT 'received',  -- received, processed, failed
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    processed_at TIMESTAMP,
    
    INDEX idx_provider (provider),
    INDEX idx_event_type (event_type),
    INDEX idx_status (status),
    INDEX idx_created_at (created_at)
);

-- AI usage log table (tracks token usage for billing)
CREATE TABLE ai_usage_log (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    document_id TEXT,
    feature_type TEXT NOT NULL,  -- summary, tags, search, etc.
    model_used TEXT NOT NULL,  -- gpt-4, gpt-3.5-turbo, etc.
    tokens_used INTEGER NOT NULL,
    cost_usd REAL NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_user_id (user_id),
    INDEX idx_workspace_id (workspace_id),
    INDEX idx_created_at (created_at)
);
