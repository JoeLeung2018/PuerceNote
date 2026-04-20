//! Vector usage billing module for AI features
//! Phase 2B.Vector: Tracks semantic search, embeddings, and vector operations

use crate::error::{SubscriptionError, SubscriptionResult};
use chrono::{DateTime, Utc, Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============== Vector Billing Models ==============

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum VectorOperation {
    /// Semantic search across document embeddings
    SemanticSearch = 1,
    /// Embedding generation for new content
    EmbeddingGeneration = 2,
    /// Batch indexing of documents
    BatchIndexing = 3,
    /// Vector similarity recommendation
    SimilarityRecommendation = 4,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorUsageRecord {
    pub id: String,
    pub user_id: String,
    pub workspace_id: String,
    pub operation: VectorOperation,
    pub tokens_used: u32,           // tokens consumed
    pub embedding_dim: u16,         // 768 by default
    pub collection_size: u32,       // size of vector collection
    pub query_latency_ms: u64,      // performance metric
    pub cost_usd: f64,              // calculated cost
    pub created_at: DateTime<Utc>,
}

impl VectorUsageRecord {
    pub fn new(
        user_id: String,
        workspace_id: String,
        operation: VectorOperation,
        tokens_used: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            workspace_id,
            operation,
            tokens_used,
            embedding_dim: 768,
            collection_size: 0,
            query_latency_ms: 0,
            cost_usd: 0.0,
            created_at: Utc::now(),
        }
    }

    /// Calculate cost based on operation type and tokens
    /// Pricing: $0.01 per 1K tokens (same as Phase 2B token pricing)
    pub fn calculate_cost(&mut self) -> f64 {
        self.cost_usd = match self.operation {
            VectorOperation::SemanticSearch => {
                // $0.01 per 1K tokens
                (self.tokens_used as f64 / 1_000.0) * 0.01
            }
            VectorOperation::EmbeddingGeneration => {
                // $0.01 per 1K tokens (based on output embedding tokens)
                (self.tokens_used as f64 / 1_000.0) * 0.01
            }
            VectorOperation::BatchIndexing => {
                // Batch indexing is free (background process)
                0.0
            }
            VectorOperation::SimilarityRecommendation => {
                // $0.005 per 1K tokens (50% discount vs search)
                (self.tokens_used as f64 / 1_000.0) * 0.005
            }
        };
        self.cost_usd
    }
}

// ============== Vector Quota Management ==============

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorQuotaStatus {
    pub user_id: String,
    pub workspace_id: String,
    pub plan_type: String,  // free, pro, team
    pub monthly_vector_limit: u32,  // e.g., 100K searches for Pro
    pub vector_operations_used: u32,  // count of operations
    pub tokens_used: u32,   // total tokens consumed
    pub cost_to_date: f64,  // cost this month
    pub last_reset: DateTime<Utc>,
    pub next_reset: DateTime<Utc>,
}

impl VectorQuotaStatus {
    pub fn new(user_id: String, workspace_id: String, plan_type: String) -> Self {
        let now = Utc::now();
        
        // Calculate next reset date (first of next month)
        let (next_year, next_month) = if now.month() == 12 {
            (now.year() + 1, 1)
        } else {
            (now.year(), now.month() + 1)
        };
        
        let next_reset = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(next_year, next_month, 1)
                .unwrap_or(NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap()),
            NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        );
        let next_reset = DateTime::<Utc>::from_naive_utc_and_offset(next_reset, Utc);
        
        let monthly_vector_limit = Self::get_limit_for_plan(&plan_type);
        
        Self {
            user_id,
            workspace_id,
            plan_type,
            monthly_vector_limit,
            vector_operations_used: 0,
            tokens_used: 0,
            cost_to_date: 0.0,
            last_reset: now,
            next_reset,
        }
    }

    /// Get vector operation limits by plan type
    pub fn get_limit_for_plan(plan_type: &str) -> u32 {
        match plan_type {
            "free" => 100,          // 100 searches/month
            "pro" => 10_000,        // 10K searches/month
            "team" => 100_000,      // 100K searches/month
            _ => 100,               // default to free
        }
    }

    /// Check if user can perform vector operation
    pub fn can_perform_operation(&self, operation: &VectorOperation) -> bool {
        match operation {
            VectorOperation::BatchIndexing => {
                // Batch indexing always allowed (free)
                true
            }
            _ => {
                // Other operations count against quota
                self.vector_operations_used < self.monthly_vector_limit
            }
        }
    }

    /// Add usage and check for overage
    pub fn add_usage(&mut self, tokens: u32) -> SubscriptionResult<()> {
        self.vector_operations_used += 1;
        self.tokens_used += tokens;
        
        if self.vector_operations_used > self.monthly_vector_limit {
            tracing::warn!(
                "User {} exceeded vector quota: {}/{}",
                self.user_id,
                self.vector_operations_used,
                self.monthly_vector_limit
            );
        }
        
        Ok(())
    }
}

// ============== Vector Billing Service ==============

pub struct VectorBillingService;

impl VectorBillingService {
    /// Log vector operation usage
    pub async fn log_vector_operation(
        user_id: &str,
        workspace_id: &str,
        operation: VectorOperation,
        tokens_used: u32,
        latency_ms: u64,
    ) -> SubscriptionResult<VectorUsageRecord> {
        let mut record = VectorUsageRecord::new(
            user_id.to_string(),
            workspace_id.to_string(),
            operation.clone(),
            tokens_used,
        );
        
        record.query_latency_ms = latency_ms;
        record.calculate_cost();
        
        tracing::info!(
            "Vector operation logged: user={}, op={:?}, tokens={}, cost=${:.4}",
            user_id,
            operation,
            tokens_used,
            record.cost_usd
        );
        
        Ok(record)
    }

    /// Track semantic search query
    pub async fn track_semantic_search(
        user_id: &str,
        workspace_id: &str,
        query_tokens: u32,
        search_duration: std::time::Duration,
        result_count: u32,
    ) -> SubscriptionResult<VectorUsageRecord> {
        let mut record = Self::log_vector_operation(
            user_id,
            workspace_id,
            VectorOperation::SemanticSearch,
            query_tokens,
            search_duration.as_millis() as u64,
        ).await?;
        
        record.collection_size = result_count;
        
        tracing::info!(
            "Semantic search: workspace={}, results={}, latency={}ms, cost=${:.4}",
            workspace_id,
            result_count,
            record.query_latency_ms,
            record.cost_usd
        );
        
        Ok(record)
    }

    /// Track embedding generation
    pub async fn track_embedding_generation(
        user_id: &str,
        workspace_id: &str,
        content_tokens: u32,
        embedding_duration: std::time::Duration,
    ) -> SubscriptionResult<VectorUsageRecord> {
        Self::log_vector_operation(
            user_id,
            workspace_id,
            VectorOperation::EmbeddingGeneration,
            content_tokens,
            embedding_duration.as_millis() as u64,
        ).await
    }

    /// Estimate cost for vector operation
    pub fn estimate_cost(operation: &VectorOperation, tokens_used: u32) -> f64 {
        let mut record = VectorUsageRecord::new(
            "estimate".to_string(),
            "estimate".to_string(),
            operation.clone(),
            tokens_used,
        );
        record.calculate_cost();
        record.cost_usd
    }

    /// Generate billing summary
    pub fn generate_monthly_summary(
        user_id: &str,
        operations_count: u32,
        tokens_used: u32,
        total_cost: f64,
    ) -> String {
        format!(
            "Vector Usage Summary for {}\n\
            Operations: {}\n\
            Tokens: {}\n\
            Cost: ${:.2}\n\
            Avg Cost/Op: ${:.4}",
            user_id,
            operations_count,
            tokens_used,
            total_cost,
            if operations_count > 0 { total_cost / operations_count as f64 } else { 0.0 }
        )
    }
}

// ============== Tests ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_usage_record_creation() {
        let record = VectorUsageRecord::new(
            "user123".to_string(),
            "ws_456".to_string(),
            VectorOperation::SemanticSearch,
            1000,
        );

        assert_eq!(record.user_id, "user123");
        assert_eq!(record.workspace_id, "ws_456");
        assert_eq!(record.tokens_used, 1000);
    }

    #[test]
    fn test_cost_calculation_semantic_search() {
        let mut record = VectorUsageRecord::new(
            "user123".to_string(),
            "ws_456".to_string(),
            VectorOperation::SemanticSearch,
            1000,  // 1K tokens
        );
        
        record.calculate_cost();
        
        // $0.01 per 1K tokens = $0.01
        assert!((record.cost_usd - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_embedding() {
        let mut record = VectorUsageRecord::new(
            "user123".to_string(),
            "ws_456".to_string(),
            VectorOperation::EmbeddingGeneration,
            500,   // 500 tokens
        );
        
        record.calculate_cost();
        
        // $0.01 per 1K tokens = $0.005
        assert!((record.cost_usd - 0.005).abs() < 0.0001);
    }

    #[test]
    fn test_batch_indexing_free() {
        let mut record = VectorUsageRecord::new(
            "user123".to_string(),
            "ws_456".to_string(),
            VectorOperation::BatchIndexing,
            10000,  // any amount
        );
        
        record.calculate_cost();
        
        // Batch indexing should be free
        assert_eq!(record.cost_usd, 0.0);
    }

    #[test]
    fn test_vector_quota_creation() {
        let quota = VectorQuotaStatus::new(
            "user123".to_string(),
            "ws_456".to_string(),
            "pro".to_string(),
        );

        assert_eq!(quota.user_id, "user123");
        assert_eq!(quota.plan_type, "pro");
        assert_eq!(quota.monthly_vector_limit, 10_000);
    }

    #[test]
    fn test_quota_limits_by_plan() {
        assert_eq!(VectorQuotaStatus::get_limit_for_plan("free"), 100);
        assert_eq!(VectorQuotaStatus::get_limit_for_plan("pro"), 10_000);
        assert_eq!(VectorQuotaStatus::get_limit_for_plan("team"), 100_000);
    }

    #[test]
    fn test_quota_check() {
        let quota = VectorQuotaStatus::new(
            "user123".to_string(),
            "ws_456".to_string(),
            "free".to_string(),
        );

        assert!(quota.can_perform_operation(&VectorOperation::SemanticSearch));
        assert!(quota.can_perform_operation(&VectorOperation::BatchIndexing));
    }

    #[test]
    fn test_batch_indexing_always_allowed() {
        let quota = VectorQuotaStatus {
            user_id: "user123".to_string(),
            workspace_id: "ws_456".to_string(),
            plan_type: "free".to_string(),
            monthly_vector_limit: 100,
            vector_operations_used: 100,  // at limit
            tokens_used: 0,
            cost_to_date: 0.0,
            last_reset: Utc::now(),
            next_reset: Utc::now(),
        };

        // Batch indexing should still be allowed even at limit
        assert!(quota.can_perform_operation(&VectorOperation::BatchIndexing));
    }

    #[test]
    fn test_cost_estimate() {
        let cost = VectorBillingService::estimate_cost(&VectorOperation::SemanticSearch, 5000);
        
        // $0.01 per 1K = $0.05 for 5K tokens
        assert!((cost - 0.05).abs() < 0.0001);
    }

    #[test]
    fn test_monthly_summary() {
        let summary = VectorBillingService::generate_monthly_summary(
            "user123",
            100,        // operations
            50_000,     // tokens
            5.00,       // total cost
        );

        assert!(summary.contains("user123"));
        assert!(summary.contains("100"));
        assert!(summary.contains("50000"));
        assert!(summary.contains("$5.00"));
    }
}
