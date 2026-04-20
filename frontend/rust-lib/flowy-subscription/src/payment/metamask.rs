//! MetaMask Direct Integration
//! Phase 2B.Web3.B: Direct MetaMask window.ethereum provider
//! Browser-based wallet interaction without WalletConnect relay

use crate::error::{SubscriptionError, SubscriptionResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::walletconnect::BlockchainNetwork;

// ════════════════════════════════════════════════════════
// MetaMask Provider Types
// ════════════════════════════════════════════════════════

/// MetaMask window.ethereum provider interface
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaMaskProvider {
    pub is_metamask: bool,
    pub chain_id: String,      // "0x1" for Ethereum, "0x89" for Polygon
    pub selected_address: Option<String>,
}

impl MetaMaskProvider {
    pub fn from_chain_id(chain_id_str: &str) -> SubscriptionResult<BlockchainNetwork> {
        match chain_id_str {
            "0x1" => Ok(BlockchainNetwork::Ethereum),
            "0x89" => Ok(BlockchainNetwork::Polygon),
            "0xa" => Ok(BlockchainNetwork::Optimism),
            "0xa4b1" => Ok(BlockchainNetwork::Arbitrum),
            _ => Err(SubscriptionError::InvalidRequest(format!(
                "Unsupported chain: {}",
                chain_id_str
            ))),
        }
    }

    pub fn to_chain_id_hex(network: &BlockchainNetwork) -> String {
        let id = network.chain_id();
        format!("0x{:x}", id)
    }
}

// ════════════════════════════════════════════════════════
// Network Switching
// ════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkSwitchRequest {
    pub chain_id: String,
    pub chain_name: String,
    pub rpc_urls: Vec<String>,
    pub block_explorer_urls: Option<Vec<String>>,
}

impl NetworkSwitchRequest {
    pub fn for_network(network: &BlockchainNetwork) -> Self {
        match network {
            BlockchainNetwork::Ethereum => Self {
                chain_id: "0x1".to_string(),
                chain_name: "Ethereum Mainnet".to_string(),
                rpc_urls: vec!["https://eth-mainnet.alchemyapi.io/v2/".to_string()],
                block_explorer_urls: Some(vec!["https://etherscan.io".to_string()]),
            },
            BlockchainNetwork::Polygon => Self {
                chain_id: "0x89".to_string(),
                chain_name: "Polygon".to_string(),
                rpc_urls: vec!["https://polygon-rpc.com".to_string()],
                block_explorer_urls: Some(vec!["https://polygonscan.com".to_string()]),
            },
            BlockchainNetwork::Optimism => Self {
                chain_id: "0xa".to_string(),
                chain_name: "Optimism".to_string(),
                rpc_urls: vec!["https://mainnet.optimism.io".to_string()],
                block_explorer_urls: Some(vec!["https://optimistic.etherscan.io".to_string()]),
            },
            BlockchainNetwork::Arbitrum => Self {
                chain_id: "0xa4b1".to_string(),
                chain_name: "Arbitrum One".to_string(),
                rpc_urls: vec!["https://arb1.arbitrum.io/rpc".to_string()],
                block_explorer_urls: Some(vec!["https://arbiscan.io".to_string()]),
            },
        }
    }
}

// ════════════════════════════════════════════════════════
// Account Management
// ════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaMaskAccount {
    pub address: String,
    pub network: BlockchainNetwork,
    pub balance_wei: Option<String>, // Raw balance in wei
    pub connected_at: DateTime<Utc>,
}

impl MetaMaskAccount {
    pub fn new(address: String, network: BlockchainNetwork) -> Self {
        Self {
            address,
            network,
            balance_wei: None,
            connected_at: Utc::now(),
        }
    }

    pub fn with_balance(mut self, balance_wei: String) -> Self {
        self.balance_wei = Some(balance_wei);
        self
    }

    /// Convert wei to ETH/MATIC
    pub fn balance_in_native(&self) -> Option<f64> {
        self.balance_wei.as_ref().map(|wei_str| {
            let wei: u128 = wei_str.parse().unwrap_or(0);
            wei as f64 / 1e18
        })
    }
}

// ════════════════════════════════════════════════════════
// Transaction Signing
// ════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub to: String,
    pub from: String,
    pub value: String,      // Amount in wei
    pub data: String,       // Contract call data (for token transfers)
    pub gas: String,        // Gas limit
    pub gas_price: String,  // Gas price in wei
    pub nonce: Option<u64>,
}

impl Transaction {
    /// Create USDC/USDT transfer transaction
    pub fn create_transfer(
        from: String,
        to: String,
        amount_usd: f64,
        token_contract: String,
        gas_price_wei: String,
    ) -> Self {
        let amount_raw = (amount_usd * 1e6) as u128;
        let data = format!(
            "0xa9059cbb{:064x}{:064x}",
            u128::from_str_radix(&to.trim_start_matches("0x"), 16).unwrap_or(0),
            amount_raw
        );

        Self {
            to: token_contract,
            from,
            value: "0".to_string(), // ERC20 transfers have no ETH value
            data,
            gas: "100000".to_string(),
            gas_price: gas_price_wei,
            nonce: None,
        }
    }

    /// Sign transaction for sending
    pub fn prepare_for_signing(&self) -> String {
        // Prepare transaction for MetaMask signing
        format!(
            "{{ \"to\": \"{}\", \"from\": \"{}\", \"value\": \"{}\", \"data\": \"{}\", \"gas\": \"{}\", \"gasPrice\": \"{}\" }}",
            self.to, self.from, self.value, self.data, self.gas, self.gas_price
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub tx_hash: String,
    pub signed_data: String,
    pub status: TransactionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

// ════════════════════════════════════════════════════════
// MetaMask Client
// ════════════════════════════════════════════════════════

pub struct MetaMaskClient {
    current_account: Option<MetaMaskAccount>,
    current_network: BlockchainNetwork,
}

impl MetaMaskClient {
    pub fn new() -> Self {
        Self {
            current_account: None,
            current_network: BlockchainNetwork::Ethereum,
        }
    }

    /// Request account connection (user clicks "Connect")
    pub async fn request_accounts(&mut self) -> SubscriptionResult<Vec<String>> {
        // In production: This would call window.ethereum.request({method: 'eth_requestAccounts'})
        // Simulating account request
        let accounts = vec!["0x742d35Cc6634C0532925a3b844Bc322e50e37".to_string()];
        
        if let Some(account) = accounts.first() {
            self.current_account = Some(MetaMaskAccount::new(
                account.clone(),
                self.current_network.clone(),
            ));
            tracing::info!("MetaMask account connected: {}", account);
        }

        Ok(accounts)
    }

    /// Switch to different network
    pub async fn switch_network(
        &mut self,
        network: BlockchainNetwork,
    ) -> SubscriptionResult<()> {
        let switch_request = NetworkSwitchRequest::for_network(&network);
        
        // Validate chain_id format
        if !switch_request.chain_id.starts_with("0x") {
            return Err(SubscriptionError::InvalidRequest(
                "Invalid chain_id format".to_string(),
            ));
        }

        self.current_network = network.clone();
        
        // Update current account network
        if let Some(ref mut account) = self.current_account {
            account.network = network.clone();
        }

        tracing::info!("Network switched to {:?}", network);
        Ok(())
    }

    /// Get current connected account
    pub fn get_current_account(&self) -> SubscriptionResult<MetaMaskAccount> {
        self.current_account
            .clone()
            .ok_or(SubscriptionError::InvalidRequest(
                "No MetaMask account connected".to_string(),
            ))
    }

    /// Get current network
    pub fn get_current_network(&self) -> BlockchainNetwork {
        self.current_network.clone()
    }

    /// Request signature (personal_sign)
    pub fn request_signature(&self, message: &str) -> SubscriptionResult<String> {
        let account = self.get_current_account()?;
        
        // In production: Calls window.ethereum.request({method: 'personal_sign', params: [message, address]})
        // Simulate signature: 0x + 130 hex chars
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        hasher.update(message.as_bytes());
        let signature = format!("0x{:x}", hasher.finalize());

        tracing::debug!(
            "Signature requested for account {} with message: {}",
            account.address,
            message
        );

        Ok(signature)
    }

    /// Send transaction
    pub async fn send_transaction(&self, tx: &Transaction) -> SubscriptionResult<String> {
        let account = self.get_current_account()?;
        
        // Validate transaction
        if tx.from.to_lowercase() != account.address.to_lowercase() {
            return Err(SubscriptionError::InvalidRequest(
                "Transaction sender mismatch".to_string(),
            ));
        }

        // In production: Calls window.ethereum.request({method: 'eth_sendTransaction', params: [tx]})
        let tx_hash = format!("0x{}", hex::encode(Uuid::new_v4().as_bytes()));

        tracing::info!(
            "Transaction sent: {} to {} via MetaMask",
            tx.value,
            tx.to
        );

        Ok(tx_hash)
    }

    /// Get transaction receipt
    pub async fn get_transaction_receipt(&self, tx_hash: &str) -> SubscriptionResult<Option<String>> {
        if !tx_hash.starts_with("0x") || tx_hash.len() < 66 {
            return Err(SubscriptionError::InvalidRequest(
                "Invalid transaction hash".to_string(),
            ));
        }

        // In production: Query RPC via web3 crate
        Ok(Some("receipt_data".to_string()))
    }
}

impl Default for MetaMaskClient {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════
// Tests
// ════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metamask_provider_from_chain_id() {
        let network = MetaMaskProvider::from_chain_id("0x1");
        assert!(network.is_ok());
        match network.unwrap() {
            BlockchainNetwork::Ethereum => (),
            _ => panic!("Expected Ethereum"),
        }
    }

    #[test]
    fn test_metamask_provider_invalid_chain() {
        let result = MetaMaskProvider::from_chain_id("0xdeadbeef");
        assert!(result.is_err());
    }

    #[test]
    fn test_network_switch_request_ethereum() {
        let req = NetworkSwitchRequest::for_network(&BlockchainNetwork::Ethereum);
        assert_eq!(req.chain_id, "0x1");
        assert_eq!(req.chain_name, "Ethereum Mainnet");
    }

    #[test]
    fn test_network_switch_request_polygon() {
        let req = NetworkSwitchRequest::for_network(&BlockchainNetwork::Polygon);
        assert_eq!(req.chain_id, "0x89");
        assert_eq!(req.chain_name, "Polygon");
    }

    #[test]
    fn test_metamask_account_balance_conversion() {
        let account = MetaMaskAccount::new(
            "0x742d35Cc6634C0532925a3b844Bc322e50e37".to_string(),
            BlockchainNetwork::Ethereum,
        )
        .with_balance("1000000000000000000".to_string()); // 1 ETH

        let balance = account.balance_in_native();
        assert_eq!(balance, Some(1.0));
    }

    #[test]
    fn test_create_transfer_transaction() {
        let tx = Transaction::create_transfer(
            "0x742d35Cc6634C0532925a3b844Bc322e50e37".to_string(),
            "0xPUERCENOTE".to_string(),
            100.0,
            "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
            "1000000000".to_string(),
        );

        assert_eq!(tx.to, "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174");
        assert_eq!(tx.value, "0");
        assert!(tx.data.starts_with("0xa9059cbb"));
    }

    #[test]
    fn test_transaction_prepare_for_signing() {
        let tx = Transaction {
            to: "0x123".to_string(),
            from: "0x456".to_string(),
            value: "0".to_string(),
            data: "0xdata".to_string(),
            gas: "100000".to_string(),
            gas_price: "1000000000".to_string(),
            nonce: None,
        };

        let prepared = tx.prepare_for_signing();
        assert!(prepared.contains("\"to\": \"0x123\""));
        assert!(prepared.contains("\"from\": \"0x456\""));
    }

    #[tokio::test]
    async fn test_metamask_client_connect() {
        let mut client = MetaMaskClient::new();
        let result = client.request_accounts().await;
        assert!(result.is_ok());

        let accounts = result.unwrap();
        assert!(!accounts.is_empty());
    }

    #[tokio::test]
    async fn test_metamask_switch_network() {
        let mut client = MetaMaskClient::new();
        let _ = client.request_accounts().await; // Connect first
        
        let result = client.switch_network(BlockchainNetwork::Polygon).await;
        assert!(result.is_ok());
        
        assert_eq!(client.get_current_network().chain_id(), 137);
    }

    #[test]
    fn test_request_signature() {
        let mut client = MetaMaskClient::new();
        
        // Connect first
        futures::executor::block_on(async {
            let _ = client.request_accounts().await;
        });

        let result = client.request_signature("Test message");
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("0x"));
    }

    #[test]
    fn test_get_current_account_not_connected() {
        let client = MetaMaskClient::new();
        let result = client.get_current_account();
        assert!(result.is_err());
    }
}
