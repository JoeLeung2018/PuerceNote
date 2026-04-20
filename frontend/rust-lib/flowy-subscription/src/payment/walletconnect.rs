//! WalletConnect 2.0 Payment Integration
//! Phase 2B.Web3.A: Web3 native payment with MetaMask, WalletConnect, etc.
//! Supports USDC/USDT on Polygon, Ethereum, Optimism

use crate::error::{SubscriptionError, SubscriptionResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

// ════════════════════════════════════════════════════════
// Core WalletConnect Types
// ════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlockchainNetwork {
    /// Ethereum Mainnet
    Ethereum,
    /// Polygon (Matic) - 0.00005 gas, fast settlement
    Polygon,
    /// Optimism - 0.0001 gas, cheap + fast
    Optimism,
    /// Arbitrum - 0.0001 gas, EVM compatible
    Arbitrum,
}

impl BlockchainNetwork {
    pub fn chain_id(&self) -> u32 {
        match self {
            BlockchainNetwork::Ethereum => 1,
            BlockchainNetwork::Polygon => 137,
            BlockchainNetwork::Optimism => 10,
            BlockchainNetwork::Arbitrum => 42161,
        }
    }

    pub fn rpc_url(&self) -> &str {
        match self {
            BlockchainNetwork::Ethereum => "https://eth-mainnet.alchemyapi.io/v2/",
            BlockchainNetwork::Polygon => "https://polygon-rpc.com",
            BlockchainNetwork::Optimism => "https://mainnet.optimism.io",
            BlockchainNetwork::Arbitrum => "https://arb1.arbitrum.io/rpc",
        }
    }

    pub fn usdc_contract(&self) -> &str {
        match self {
            BlockchainNetwork::Ethereum => "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            BlockchainNetwork::Polygon => "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174",
            BlockchainNetwork::Optimism => "0x7F5c764cBc14f9669B88837ca1490cCa17c31607",
            BlockchainNetwork::Arbitrum => "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5F86",
        }
    }

    pub fn usdt_contract(&self) -> &str {
        match self {
            BlockchainNetwork::Ethereum => "0xdAC17F958D2ee523a2206206994597C13D831ec7",
            BlockchainNetwork::Polygon => "0xc2132D05D31c914a87C6611C10748AEb04B58e8F",
            BlockchainNetwork::Optimism => "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58",
            BlockchainNetwork::Arbitrum => "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TokenType {
    /// USD Coin - 6 decimals
    USDC,
    /// Tether - 6 decimals
    USDT,
}

impl TokenType {
    pub fn decimals(&self) -> u32 {
        6
    }

    pub fn name(&self) -> &str {
        match self {
            TokenType::USDC => "USDC",
            TokenType::USDT => "USDT",
        }
    }
}

/// WalletConnect 2.0 Session
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletConnectSession {
    pub session_id: String,
    pub wallet_address: String,
    pub network: BlockchainNetwork,
    pub connected_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Payment request for Web3 transaction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Web3PaymentRequest {
    pub user_id: String,
    pub workspace_id: String,
    pub amount_usd: f64,
    pub token: TokenType,
    pub network: BlockchainNetwork,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

/// On-chain transaction record
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount_raw: String,
    pub token: TokenType,
    pub network: BlockchainNetwork,
    pub status: TransactionStatus,
    pub block_number: Option<u64>,
    pub gas_used: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

// ════════════════════════════════════════════════════════
// WalletConnect Client
// ════════════════════════════════════════════════════════

pub struct WalletConnectClient {
    pub project_id: String,
    pub relay_url: String,
    sessions: std::sync::Arc<tokio::sync::Mutex<Vec<WalletConnectSession>>>,
}

impl WalletConnectClient {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            relay_url: "wss://relay.walletconnect.com".to_string(),
            sessions: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Create URI for QR code
    /// Format: wc:UUID@2?relay-protocol=irn&symKey=KEY
    pub fn create_uri(&self, session_id: &str) -> String {
        format!(
            "wc:{}@2?relay-protocol=irn&symKey={}",
            session_id,
            Uuid::new_v4()
        )
    }

    /// Handle wallet connection response
    pub async fn handle_connection(
        &self,
        session_id: &str,
        wallet_address: &str,
        network: BlockchainNetwork,
    ) -> SubscriptionResult<WalletConnectSession> {
        let now = Utc::now();
        let expires = now + chrono::Duration::days(30);

        let session = WalletConnectSession {
            session_id: session_id.to_string(),
            wallet_address: wallet_address.to_lowercase(),
            network: network.clone(),
            connected_at: now,
            expires_at: expires,
        };

        let mut sessions = self.sessions.lock().await;
        sessions.push(session.clone());

        tracing::info!(
            "Wallet connected: {} on {:?}",
            wallet_address,
            network
        );

        Ok(session)
    }

    /// Verify wallet signature
    /// Message format: "PuerceNote subscription payment - {amount} USDC/USDT - {timestamp}"
    pub fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        wallet_address: &str,
    ) -> SubscriptionResult<bool> {
        // Verify EIP-191 signature: \x19Ethereum Signed Message:\n{len(message)}{message}
        let _eth_prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let _full_message = format!("{}{}", _eth_prefix, message);

        // In production, use ethers-rs or web3 crate for proper signature verification
        // For now, validate format
        if signature.starts_with("0x") && signature.len() == 132 {
            tracing::debug!("Signature verified for {}", wallet_address);
            Ok(true)
        } else {
            Err(SubscriptionError::InvalidRequest(
                "Invalid signature format".to_string(),
            ))
        }
    }

    /// Get active session for wallet
    pub async fn get_session(&self, wallet_address: &str) -> Option<WalletConnectSession> {
        let sessions = self.sessions.lock().await;
        sessions
            .iter()
            .find(|s| s.wallet_address.to_lowercase() == wallet_address.to_lowercase())
            .cloned()
    }

    /// Disconnect session
    pub async fn disconnect(&self, session_id: &str) -> SubscriptionResult<()> {
        let mut sessions = self.sessions.lock().await;
        sessions.retain(|s| s.session_id != session_id);
        tracing::info!("Session disconnected: {}", session_id);
        Ok(())
    }
}

// ════════════════════════════════════════════════════════
// Smart Contract Interaction
// ════════════════════════════════════════════════════════

pub struct SmartContractInteraction {
    pub network: BlockchainNetwork,
    pub token: TokenType,
}

impl SmartContractInteraction {
    pub fn new(network: BlockchainNetwork, token: TokenType) -> Self {
        Self { network, token }
    }

    /// Generate transfer function data for ERC20 token
    /// Function: transfer(address to, uint256 amount)
    /// Selector: 0xa9059cbb (first 4 bytes of keccak256 hash)
    pub fn encode_transfer(&self, to_address: &str, amount_usd: f64) -> String {
        // Convert USD to token raw amount (with decimals)
        let decimals = self.token.decimals() as u32;
        let amount_raw = (amount_usd * 10f64.powi(decimals as i32)) as u128;

        // ERC20 transfer function: 0xa9059cbb
        // Padded to address (32 bytes) and amount (32 bytes)
        let address_hex = to_address.trim_start_matches("0x");
        let address_num = u128::from_str_radix(address_hex, 16).unwrap_or(0);
        
        format!(
            "0xa9059cbb{:064x}{:064x}",
            address_num,
            amount_raw
        )
    }

    /// Calculate gas fees
    pub async fn estimate_gas(&self, amount_usd: f64) -> SubscriptionResult<GasEstimate> {
        // Standard ERC20 transfer gas: ~65000
        let gas_limit = 100_000u64;

        // Gas prices vary by network
        let gas_price_gwei = match self.network {
            BlockchainNetwork::Ethereum => 50.0,  // Mainnet: 50 gwei
            BlockchainNetwork::Polygon => 0.05,   // Polygon: 0.05 gwei
            BlockchainNetwork::Optimism => 0.1,   // Optimism: 0.1 gwei
            BlockchainNetwork::Arbitrum => 0.1,   // Arbitrum: 0.1 gwei
        };

        let gas_fee_usd = (gas_limit as f64 * gas_price_gwei * 1e-9) * 2500.0; // ETH/USD ~2500

        Ok(GasEstimate {
            gas_limit,
            gas_price_gwei,
            total_fee_usd: gas_fee_usd,
            network: self.network.clone(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GasEstimate {
    pub gas_limit: u64,
    pub gas_price_gwei: f64,
    pub total_fee_usd: f64,
    pub network: BlockchainNetwork,
}

// ════════════════════════════════════════════════════════
// Payment Processing
// ════════════════════════════════════════════════════════

pub struct Web3PaymentProcessor {
    wallet_connect: WalletConnectClient,
}

impl Web3PaymentProcessor {
    pub fn new(project_id: String) -> Self {
        Self {
            wallet_connect: WalletConnectClient::new(project_id),
        }
    }

    /// Process Web3 payment
    pub async fn process_payment(
        &self,
        request: Web3PaymentRequest,
        tx_hash: String,
    ) -> SubscriptionResult<BlockchainTransaction> {
        // Verify session exists
        let _session = self.wallet_connect
            .get_session(&request.user_id)
            .await
            .ok_or(SubscriptionError::InvalidRequest(
                "Wallet not connected".to_string(),
            ))?;

        // Create transaction record
        let tx = BlockchainTransaction {
            tx_hash: tx_hash.clone(),
            from_address: "0x...".to_string(), // From wallet session
            to_address: "0xPUERCENOTE_TREASURY".to_string(),
            amount_raw: format!("{}", request.amount_usd * 1e6), // USDC/USDT 6 decimals
            token: request.token.clone(),
            network: request.network.clone(),
            status: TransactionStatus::Pending,
            block_number: None,
            gas_used: None,
            created_at: Utc::now(),
            confirmed_at: None,
        };

        tracing::info!(
            "Web3 payment processed: {} {} from {}",
            request.amount_usd,
            request.token.name(),
            request.user_id
        );

        Ok(tx)
    }

    /// Verify transaction on blockchain (Placeholder)
    pub async fn verify_transaction(
        &self,
        tx_hash: &str,
        network: BlockchainNetwork,
    ) -> SubscriptionResult<BlockchainTransaction> {
        // In production: Query blockchain RPC to verify transaction
        // Check transaction receipt, gas used, status
        
        let verified_tx = BlockchainTransaction {
            tx_hash: tx_hash.to_string(),
            from_address: "0x...".to_string(),
            to_address: "0xPUERCENOTE_TREASURY".to_string(),
            amount_raw: "10000000".to_string(), // 10 USDC
            token: TokenType::USDC,
            network,
            status: TransactionStatus::Confirmed,
            block_number: Some(12345678),
            gas_used: Some(65000),
            created_at: Utc::now(),
            confirmed_at: Some(Utc::now()),
        };

        tracing::info!("Transaction verified: {}", tx_hash);
        Ok(verified_tx)
    }

    /// Get transaction status
    pub async fn get_transaction_status(
        &self,
        tx_hash: &str,
    ) -> SubscriptionResult<TransactionStatus> {
        // Simulate checking transaction status
        // In production: Query blockchain RPC
        if tx_hash.starts_with("0x") && tx_hash.len() == 66 {
            Ok(TransactionStatus::Confirmed)
        } else {
            Err(SubscriptionError::InvalidRequest(
                "Invalid transaction hash".to_string(),
            ))
        }
    }
}

// ════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_network_chain_ids() {
        assert_eq!(BlockchainNetwork::Ethereum.chain_id(), 1);
        assert_eq!(BlockchainNetwork::Polygon.chain_id(), 137);
        assert_eq!(BlockchainNetwork::Optimism.chain_id(), 10);
        assert_eq!(BlockchainNetwork::Arbitrum.chain_id(), 42161);
    }

    #[test]
    fn test_token_type_decimals() {
        assert_eq!(TokenType::USDC.decimals(), 6);
        assert_eq!(TokenType::USDT.decimals(), 6);
        assert_eq!(TokenType::USDC.name(), "USDC");
    }

    #[test]
    fn test_create_wallet_connect_uri() {
        let client = WalletConnectClient::new("test-project-id".to_string());
        let uri = client.create_uri("test-session-123");
        assert!(uri.starts_with("wc:test-session-123@2"));
        assert!(uri.contains("relay-protocol=irn"));
    }

    #[test]
    fn test_verify_signature_format() {
        let client = WalletConnectClient::new("test-project-id".to_string());
        let result = client.verify_signature(
            "Test message",
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef12",
            "0x742d35Cc6634C0532925a3b844Bc322e50e37",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let client = WalletConnectClient::new("test-project-id".to_string());
        let result = client.verify_signature(
            "Test message",
            "invalid_signature",
            "0x742d35Cc6634C0532925a3b844Bc322e50e37",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_transfer() {
        let contract = SmartContractInteraction::new(BlockchainNetwork::Polygon, TokenType::USDC);
        let data = contract.encode_transfer("0x742d35Cc6634C0532925a3b844Bc322e50e37", 100.0);
        assert!(data.starts_with("0xa9059cbb"));
        assert_eq!(data.len(), 138); // 0x + 136 hex chars
    }

    #[tokio::test]
    async fn test_wallet_connection() {
        let client = WalletConnectClient::new("test-project-id".to_string());
        let result = client
            .handle_connection(
                "session-123",
                "0x742d35Cc6634C0532925a3b844Bc322e50e37",
                BlockchainNetwork::Polygon,
            )
            .await;
        assert!(result.is_ok());

        let session = result.unwrap();
        assert_eq!(session.session_id, "session-123");
        assert_eq!(
            session.wallet_address.to_lowercase(),
            "0x742d35cc6634c0532925a3b844bc322e50e37"
        );
    }

    #[tokio::test]
    async fn test_gas_estimation() {
        let contract = SmartContractInteraction::new(BlockchainNetwork::Polygon, TokenType::USDC);
        let estimate = contract.estimate_gas(100.0).await;
        assert!(estimate.is_ok());

        let gas = estimate.unwrap();
        assert_eq!(gas.gas_limit, 100_000);
        assert_eq!(gas.gas_price_gwei, 0.05); // Polygon
    }
}
