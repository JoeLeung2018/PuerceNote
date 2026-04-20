//! Subscription system configuration

use crate::error::{SubscriptionError, SubscriptionResult};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubscriptionConfig {
    // Lemon Squeezy configuration
    pub lemon_squeezy_api_key: String,
    pub lemon_squeezy_webhook_secret: String,
    pub lemon_squeezy_store_id: u32,
    pub lemon_squeezy_api_url: String,

    // Products and pricing
    pub lemon_squeezy_pro_product_id: u32,
    pub lemon_squeezy_pro_variant_id: u32,
    pub lemon_squeezy_team_product_id: u32,
    pub lemon_squeezy_team_variant_id: u32,

    // Web3 configuration
    pub web3_enabled: bool,
    pub polygon_rpc_url: String,
    pub polygon_network: String,  // testnet or mainnet
    pub payment_contract_address: String,
    pub usdc_contract_address: String,
    pub usdt_contract_address: String,

    // WalletConnect
    pub walletconnect_project_id: String,

    // AI token configuration
    pub pro_monthly_token_limit: u32,
    pub team_monthly_token_limit: u32,
    pub token_cost_per_1m: f64,

    // vLLM configuration
    pub vllm_proxy_url: String,
    pub vllm_model: String,

    // Server configuration
    pub api_server_host: String,
    pub api_server_port: u16,
    pub webhook_server_port: u16,

    // Security
    pub jwt_secret: String,
    pub session_timeout_secs: u64,

    // Environment
    pub environment: String,
    pub debug_mode: bool,
}

impl SubscriptionConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> SubscriptionResult<Self> {
        let config = SubscriptionConfig {
            lemon_squeezy_api_key: env::var("LEMON_SQUEEZY_API_KEY")
                .map_err(|_| SubscriptionError::ConfigError("LEMON_SQUEEZY_API_KEY not set".to_string()))?,
            
            lemon_squeezy_webhook_secret: env::var("LEMON_SQUEEZY_WEBHOOK_SIGNATURE_SECRET")
                .map_err(|_| SubscriptionError::ConfigError("LEMON_SQUEEZY_WEBHOOK_SIGNATURE_SECRET not set".to_string()))?,
            
            lemon_squeezy_store_id: env::var("LEMON_SQUEEZY_STORE_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            lemon_squeezy_api_url: env::var("LEMON_SQUEEZY_API_URL")
                .unwrap_or_else(|_| "https://api.lemonsqueezy.com".to_string()),

            lemon_squeezy_pro_product_id: env::var("LEMON_SQUEEZY_PRO_PRODUCT_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            lemon_squeezy_pro_variant_id: env::var("LEMON_SQUEEZY_PRO_VARIANT_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            lemon_squeezy_team_product_id: env::var("LEMON_SQUEEZY_TEAM_PRODUCT_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            lemon_squeezy_team_variant_id: env::var("LEMON_SQUEEZY_TEAM_VARIANT_ID")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),

            web3_enabled: env::var("ENABLE_WEB3")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),

            polygon_rpc_url: env::var("POLYGON_RPC_TESTNET")
                .unwrap_or_else(|_| "https://rpc-mumbai.maticvigil.com".to_string()),

            polygon_network: env::var("POLYGON_NETWORK")
                .unwrap_or_else(|_| "testnet".to_string()),

            payment_contract_address: env::var("PAYMENT_CONTRACT_ADDRESS_TESTNET")
                .unwrap_or_default(),

            usdc_contract_address: env::var("USDC_CONTRACT_ADDRESS")
                .unwrap_or_default(),

            usdt_contract_address: env::var("USDT_CONTRACT_ADDRESS")
                .unwrap_or_default(),

            walletconnect_project_id: env::var("WALLETCONNECT_PROJECT_ID")
                .unwrap_or_default(),

            pro_monthly_token_limit: env::var("PRO_MONTHLY_TOKEN_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100_000),

            team_monthly_token_limit: env::var("TEAM_MONTHLY_TOKEN_LIMIT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500_000),

            token_cost_per_1m: env::var("TOKEN_COST_PER_1M")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.0001),

            vllm_proxy_url: env::var("VLLM_PROXY_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000/v1".to_string()),

            vllm_model: env::var("VLLM_MODEL")
                .unwrap_or_else(|_| "meta-llama/Llama-2-9b-hf".to_string()),

            api_server_host: env::var("API_SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),

            api_server_port: env::var("API_SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8080),

            webhook_server_port: env::var("WEBHOOK_SERVER_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8081),

            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key".to_string()),

            session_timeout_secs: env::var("SESSION_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(86400),

            environment: env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),

            debug_mode: env::var("ENABLE_DEBUG_MODE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
        };

        Ok(config)
    }

    /// Validate critical configuration
    pub fn validate(&self) -> SubscriptionResult<()> {
        if self.lemon_squeezy_api_key.is_empty() {
            return Err(SubscriptionError::ConfigError(
                "Lemon Squeezy API key is required".to_string(),
            ));
        }

        if self.lemon_squeezy_webhook_secret.is_empty() {
            return Err(SubscriptionError::ConfigError(
                "Lemon Squeezy webhook secret is required".to_string(),
            ));
        }

        if self.jwt_secret.is_empty() {
            return Err(SubscriptionError::ConfigError(
                "JWT secret is required".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // Test configuration loading
        // Note: Set environment variables before running tests
    }
}
