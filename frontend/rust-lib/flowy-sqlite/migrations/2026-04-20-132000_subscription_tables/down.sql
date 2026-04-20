-- Rollback: Drop subscription and payment tables
DROP TABLE IF EXISTS ai_usage_log;
DROP TABLE IF EXISTS payment_webhooks;
DROP TABLE IF EXISTS user_subscription;
DROP TABLE IF EXISTS payment_orders;
