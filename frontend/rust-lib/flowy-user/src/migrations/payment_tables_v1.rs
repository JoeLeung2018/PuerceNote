/// Payment tables migration v1
/// 
/// Creates tables for Lemon Squeezy payment orders, Web3 transactions,
/// user subscriptions, AI usage logs, and monthly token quotas.
/// 
/// Timeline: 2026-04-20
/// Part of: PuerceNote Commercial Edition v1.0

use crate::services::user::sqlite::UserDatabase;
use collab_user::core::UserProfile;
use flowy_error::FlowyResult;
use std::sync::Arc;

pub struct PaymentTablesV1Migration;

impl PaymentTablesV1Migration {
  pub async fn run(db: &Arc<UserDatabase>, _user: &UserProfile) -> FlowyResult<()> {
    Self::create_payment_tables(db)?;
    Self::create_ai_quota_tables(db)?;
    Ok(())
  }

  /// 创建支付相关表
  fn create_payment_tables(db: &Arc<UserDatabase>) -> FlowyResult<()> {
    let sql = r#"
      -- Lemon Squeezy 订单表
      CREATE TABLE IF NOT EXISTS payment_orders (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        
        -- 订单基本信息
        lemon_order_id TEXT UNIQUE NOT NULL,
        product_id TEXT NOT NULL,
        variant_id TEXT NOT NULL,
        
        -- 金额
        amount_cents INTEGER NOT NULL,      -- 以美分计
        currency TEXT NOT NULL DEFAULT 'USD',
        
        -- 订单状态
        status TEXT NOT NULL,  -- pending, completed, failed, refunded
        payment_method TEXT,   -- credit_card, webhook_verified
        
        -- 计费周期
        billing_period_start DATETIME,
        billing_period_end DATETIME,
        
        -- 时间戳
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        webhook_verified_at DATETIME,
        
        FOREIGN KEY (user_id) REFERENCES user(id),
        
        INDEX idx_user_id (user_id),
        INDEX idx_status (status),
        INDEX idx_created_at (created_at)
      );

      -- 商品表（缓存Lemon Squeezy产品信息）
      CREATE TABLE IF NOT EXISTS payment_products (
        id TEXT PRIMARY KEY,
        lemon_product_id TEXT UNIQUE NOT NULL,
        lemon_variant_id TEXT UNIQUE NOT NULL,
        
        -- 产品信息
        name TEXT NOT NULL,
        description TEXT,
        plan_type TEXT NOT NULL,  -- pro, team
        
        -- 价格
        price_cents INTEGER NOT NULL,
        billing_cycle TEXT NOT NULL,  -- monthly, yearly
        
        -- 状态
        is_active BOOLEAN DEFAULT TRUE,
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        
        INDEX idx_plan_type (plan_type)
      );

      -- Web3 交易表（Polygon区块链）
      CREATE TABLE IF NOT EXISTS web3_transactions (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        
        -- 交易信息
        tx_hash TEXT UNIQUE NOT NULL,
        from_address TEXT NOT NULL,
        to_address TEXT NOT NULL,
        
        -- 代币信息
        token_contract TEXT NOT NULL,  -- USDC/USDT合约地址
        amount_wei TEXT NOT NULL,      -- Web3单位（Wei）
        amount_decimal DECIMAL(20,6),  -- 人类可读格式
        
        -- 链和状态
        chain_id INTEGER NOT NULL DEFAULT 137,  -- Polygon mainnet
        block_number INTEGER,
        confirmations INTEGER DEFAULT 0,
        status TEXT NOT NULL,  -- pending, confirmed, failed
        
        -- 时间戳
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        confirmed_at DATETIME,
        
        FOREIGN KEY (user_id) REFERENCES user(id),
        
        INDEX idx_user_id (user_id),
        INDEX idx_tx_hash (tx_hash),
        INDEX idx_status (status)
      );

      -- 用户订阅表
      CREATE TABLE IF NOT EXISTS user_subscription (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL UNIQUE,
        
        -- 订阅信息
        plan_id TEXT NOT NULL,  -- pro, team, free
        status TEXT NOT NULL,   -- active, expired, cancelled, suspended
        payment_method TEXT,    -- lemon_squeezy, web3_polygon, manual
        
        -- 计费周期
        period_start DATETIME NOT NULL,
        period_end DATETIME NOT NULL,
        renewal_date DATETIME,
        
        -- 功能开关
        ai_enabled BOOLEAN DEFAULT FALSE,
        semantic_search_enabled BOOLEAN DEFAULT FALSE,
        collaboration_enabled BOOLEAN DEFAULT FALSE,
        
        -- 价格和货币
        price_cents INTEGER,
        currency TEXT DEFAULT 'USD',
        
        -- 时间戳
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        cancelled_at DATETIME,
        
        FOREIGN KEY (user_id) REFERENCES user(id),
        
        INDEX idx_user_id (user_id),
        INDEX idx_status (status),
        INDEX idx_period_end (period_end)
      );

      -- Webhook事件日志（用于调试和审计）
      CREATE TABLE IF NOT EXISTS payment_webhooks (
        id TEXT PRIMARY KEY,
        provider TEXT NOT NULL,  -- lemon_squeezy, polygon_listener
        event_type TEXT NOT NULL,
        
        -- 事件内容
        payload TEXT NOT NULL,  -- JSON格式
        signature TEXT,         -- 签名验证
        signature_verified BOOLEAN DEFAULT FALSE,
        
        -- 处理状态
        processed BOOLEAN DEFAULT FALSE,
        error_message TEXT,
        
        -- 时间戳
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        processed_at DATETIME,
        
        INDEX idx_provider (provider),
        INDEX idx_event_type (event_type),
        INDEX idx_processed (processed)
      );
    "#;

    db.exec_sql(sql)?;
    Ok(())
  }

  /// 创建AI使用和配额表
  fn create_ai_quota_tables(db: &Arc<UserDatabase>) -> FlowyResult<()> {
    let sql = r#"
      -- AI使用日志（本地计费记录）
      CREATE TABLE IF NOT EXISTS ai_usage_logs (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        
        -- 使用统计
        tokens_used INTEGER NOT NULL,
        cost DECIMAL(10, 6) NOT NULL,  -- 成本（美元），精确到微美分
        
        -- 功能类型
        feature_type TEXT NOT NULL,  -- summary, tags, search, etc
        model_used TEXT NOT NULL DEFAULT 'llama-2-9b',
        
        -- 来源
        workspace_id TEXT,
        document_id TEXT,
        
        -- 时间
        created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        
        FOREIGN KEY (user_id) REFERENCES user(id),
        
        INDEX idx_user_id (user_id),
        INDEX idx_created_at (created_at),
        INDEX idx_feature_type (feature_type)
      );

      -- 用户AI配额表（每月重置）
      CREATE TABLE IF NOT EXISTS user_ai_quota (
        id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL UNIQUE,
        
        -- 配额信息
        monthly_limit INTEGER NOT NULL,  -- 每月Token限额
        tokens_used INTEGER DEFAULT 0,   -- 本月已使用
        
        -- 计费周期
        period_start DATETIME NOT NULL,
        period_end DATETIME NOT NULL,
        
        -- 成本追踪
        monthly_cost DECIMAL(10, 6) DEFAULT 0.0,  -- 本月成本（美元）
        
        -- 状态
        quota_exceeded BOOLEAN DEFAULT FALSE,
        
        -- 更新时间
        last_reset DATETIME,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        
        FOREIGN KEY (user_id) REFERENCES user(id),
        
        INDEX idx_user_id (user_id),
        INDEX idx_period_end (period_end)
      );

      -- Trigger: 创建用户订阅时自动初始化配额
      CREATE TRIGGER IF NOT EXISTS init_ai_quota_on_subscription
      AFTER INSERT ON user_subscription
      FOR EACH ROW
      WHEN NEW.plan_id IN ('pro', 'team')
      BEGIN
        INSERT OR IGNORE INTO user_ai_quota (
          id, user_id, monthly_limit, tokens_used, period_start, period_end, monthly_cost, last_reset
        ) VALUES (
          NEW.id || '-quota',
          NEW.user_id,
          CASE WHEN NEW.plan_id = 'pro' THEN 100000
               WHEN NEW.plan_id = 'team' THEN 500000
               ELSE 0
          END,
          0,
          datetime('now'),
          datetime('now', '+30 days'),
          0,
          datetime('now')
        );
      END;

      -- Trigger: 月度配额自动重置（当使用日志超过period_end时）
      CREATE TRIGGER IF NOT EXISTS reset_monthly_quota_on_usage
      AFTER INSERT ON ai_usage_logs
      FOR EACH ROW
      WHEN (SELECT period_end FROM user_ai_quota WHERE user_id = NEW.user_id) < datetime('now')
      BEGIN
        UPDATE user_ai_quota
        SET tokens_used = 0,
            monthly_cost = 0,
            period_start = datetime('now'),
            period_end = datetime('now', '+30 days'),
            quota_exceeded = FALSE,
            last_reset = datetime('now')
        WHERE user_id = NEW.user_id;
      END;
    "#;

    db.exec_sql(sql)?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_payment_table_creation() {
    // 测试表创建逻辑
    // TODO: 实现单元测试
  }
}
