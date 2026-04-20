//! Billing and token quota management module

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::SubscriptionResult;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenQuota {
    pub user_id: String,
    pub monthly_limit: u32,
    pub tokens_used: u32,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub monthly_cost: f64,
    pub quota_exceeded: bool,
    pub last_reset: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AIUsageLog {
    pub id: String,
    pub user_id: String,
    pub tokens_used: u32,
    pub cost: f64,
    pub feature_type: String,  // summary, tags, search, etc
    pub model_used: String,
    pub workspace_id: Option<String>,
    pub document_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Billing service for tracking AI usage and token quotas
pub struct BillingService {
    // TODO: Add database pool
}

impl BillingService {
    pub fn new() -> Self {
        Self {}
    }

    /// Record AI usage
    pub async fn record_usage(
        &self,
        _user_id: &str,
        _tokens: u32,
        _feature_type: &str,
    ) -> SubscriptionResult<AIUsageLog> {
        // TODO: Implement usage recording
        todo!()
    }

    /// Get user's token quota
    pub async fn get_quota(&self, _user_id: &str) -> SubscriptionResult<TokenQuota> {
        // TODO: Implement quota retrieval
        todo!()
    }

    /// Check if user has enough tokens
    pub async fn check_quota(
        &self,
        _user_id: &str,
        _required_tokens: u32,
    ) -> SubscriptionResult<bool> {
        // TODO: Implement quota check
        todo!()
    }

    /// Reset monthly quota (called on renewal)
    pub async fn reset_monthly_quota(&self, _user_id: &str) -> SubscriptionResult<()> {
        // TODO: Implement quota reset
        todo!()
    }

    /// Get usage report for user
    pub async fn get_usage_report(
        &self,
        _user_id: &str,
    ) -> SubscriptionResult<UsageReport> {
        // TODO: Implement report generation
        todo!()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsageReport {
    pub user_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_tokens_used: u32,
    pub total_cost: f64,
    pub usage_by_feature: Vec<FeatureUsage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureUsage {
    pub feature_type: String,
    pub tokens_used: u32,
    pub cost: f64,
    pub count: u32,
}

/// Calculate cost in USD based on token count
pub fn calculate_token_cost(tokens: u32, cost_per_1m: f64) -> f64 {
    (tokens as f64 / 1_000_000.0) * cost_per_1m
}

/// Count tokens locally using heuristic algorithm
pub fn count_tokens_local(text: &str) -> u32 {
    let mut token_count = 0;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        // Chinese characters (CJK)
        if !ch.is_ascii() && (ch as u32) >= 0x4e00 && (ch as u32) <= 0x9fff {
            token_count += 1;
            i += 1;
        }
        // English words or numbers
        else if ch.is_alphabetic() || ch.is_numeric() {
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            token_count += 1;
        }
        // Other characters (spaces, punctuation)
        else {
            token_count += 1;
            i += 1;
        }
    }

    // Add compensation for subword tokens
    let bonus = (text.len() as f32 / 50.0).ceil() as u32;
    (token_count as f32 * 1.3) as u32 + bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_token_cost() {
        let cost = calculate_token_cost(1_000_000, 0.0001);
        assert_eq!(cost, 0.0001);

        let cost = calculate_token_cost(500_000, 0.0001);
        assert!((cost - 0.00005).abs() < 0.000001);
    }

    #[test]
    fn test_count_tokens_english() {
        let text = "Hello world test";
        let tokens = count_tokens_local(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_count_tokens_chinese() {
        let text = "你好世界";
        let tokens = count_tokens_local(text);
        assert_eq!(tokens, 4); // 4 Chinese characters
    }

    #[test]
    fn test_count_tokens_mixed() {
        let text = "Hello 世界";
        let tokens = count_tokens_local(text);
        assert!(tokens >= 3); // At least "Hello" + "世" + "界"
    }
}
