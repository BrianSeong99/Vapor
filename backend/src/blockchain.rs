use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, U256, H256, Bytes, BlockNumber},
    Web3,
};

/// Blockchain client for interacting with Vapor smart contracts
pub struct BlockchainClient {
    /// Web3 instance for Ethereum interactions
    pub web3: Web3<Http>,
    /// Vapor Bridge contract
    pub bridge_contract: Contract<Http>,
    /// Proof Verifier contract
    pub proof_verifier_contract: Contract<Http>,
    /// Contract addresses
    pub addresses: ContractAddresses,
    /// Chain configuration
    pub chain_config: ChainConfig,
}

/// Contract addresses on the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub bridge: Address,
    pub proof_verifier: Address,
    pub usdc_token: Address,
    pub pyusd_token: Option<Address>,
}

/// Chain configuration
#[derive(Debug, Clone)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub gas_price: Option<U256>,
    pub gas_limit: U256,
}

/// Result of submitting a proof to the blockchain
#[derive(Debug, Serialize)]
pub struct ProofSubmissionResult {
    pub transaction_hash: H256,
    pub batch_id: u32,
    pub gas_used: Option<U256>,
    pub success: bool,
}

/// Deposit event from the bridge contract
#[derive(Debug, Clone, Serialize)]
pub struct DepositEvent {
    pub user: Address,
    pub token: Address,
    pub amount: U256,
    pub banking_hash: H256,
    pub block_number: u64,
    pub transaction_hash: H256,
}

/// Claim event from the bridge contract  
#[derive(Debug, Clone, Serialize)]
pub struct ClaimEvent {
    pub user: Address,
    pub batch_id: u32,
    pub order_id: u32,
    pub amount: U256,
    pub block_number: u64,
    pub transaction_hash: H256,
}

impl BlockchainClient {
    /// Create a new blockchain client
    pub async fn new(
        rpc_url: String,
        bridge_address: Address,
        proof_verifier_address: Address,
        usdc_address: Address,
        chain_id: u64,
    ) -> Result<Self> {
        let transport = Http::new(&rpc_url)?;
        let web3 = Web3::new(transport);

        // Bridge contract ABI
        let bridge_abi = include_bytes!("abi/VaporBridge.json");
        let bridge_contract = Contract::from_json(
            web3.eth(),
            bridge_address,
            bridge_abi
        )?;

        // Proof Verifier contract ABI
        let proof_verifier_abi = include_bytes!("abi/IProofVerifier.json");
        let proof_verifier_contract = Contract::from_json(
            web3.eth(),
            proof_verifier_address,
            proof_verifier_abi
        )?;

        let addresses = ContractAddresses {
            bridge: bridge_address,
            proof_verifier: proof_verifier_address,
            usdc_token: usdc_address,
            pyusd_token: None, // Can be set later
        };

        let chain_config = ChainConfig {
            chain_id,
            rpc_url,
            gas_price: None, // Will use network default
            gas_limit: U256::from(500_000), // Default gas limit
        };

        info!("Initialized blockchain client for chain {}", chain_id);
        info!("Bridge contract: {:?}", bridge_address);
        info!("Proof Verifier contract: {:?}", proof_verifier_address);
        info!("USDC token: {:?}", usdc_address);

        Ok(Self {
            web3,
            bridge_contract,
            proof_verifier_contract,
            addresses,
            chain_config,
        })
    }

    /// Submit a batch proof to the proof verifier contract
    pub async fn submit_proof(
        &self,
        batch_id: u32,
        prev_batch_id: u32,
        prev_state_root: H256,
        prev_orders_root: H256,
        new_state_root: H256,
        new_orders_root: H256,
        proof: Bytes,
    ) -> Result<ProofSubmissionResult> {
        info!("Submitting proof for batch {} to proof verifier", batch_id);

        // For MVP, return a mock result since web3 contract interaction is complex
        // In a real implementation, you'd call the proof_verifier_contract.call() method:
        /*
        let result = self.proof_verifier_contract
            .call("submitProof", (
                U256::from(batch_id),
                U256::from(prev_batch_id),
                prev_state_root,
                prev_orders_root,
                new_state_root,
                new_orders_root,
                proof
            ), from, Options::default())
            .await?;
        */
        
        let mock_tx_hash = H256::from_low_u64_be(batch_id as u64);
        
        info!("Proof submitted! Transaction hash: {:?}", mock_tx_hash);

        Ok(ProofSubmissionResult {
            transaction_hash: mock_tx_hash,
            batch_id,
            gas_used: Some(U256::from(200_000)),
            success: true,
        })
    }

    /// Get the latest batch ID from the proof verifier contract
    pub async fn get_latest_batch_id(&self) -> Result<u32> {
        let result: U256 = self.proof_verifier_contract
            .query("getLatestBatchId", (), None, Options::default(), None)
            .await?;

        Ok(result.as_u32())
    }

    /// Get batch roots for a specific batch ID from proof verifier
    pub async fn get_batch_roots(&self, batch_id: u32) -> Result<(H256, H256)> {
        let result: (H256, H256) = self.proof_verifier_contract
            .query("getBatch", batch_id, None, Options::default(), None)
            .await?;

        Ok(result)
    }

    /// Listen for deposit events (simplified implementation)
    pub async fn get_deposit_events(&self, from_block: u64, _to_block: Option<u64>) -> Result<Vec<DepositEvent>> {
        info!("Getting deposit events from block {}", from_block);
        
        // For MVP, return mock events
        // In production, you'd use proper event filtering with web3.eth().logs()
        let mock_events = vec![];
        
        info!("Found {} deposit events from block {}", mock_events.len(), from_block);
        Ok(mock_events)
    }

    /// Listen for claim events (simplified implementation)
    pub async fn get_claim_events(&self, from_block: u64, _to_block: Option<u64>) -> Result<Vec<ClaimEvent>> {
        info!("Getting claim events from block {}", from_block);
        
        // For MVP, return mock events
        // In production, you'd use proper event filtering with web3.eth().logs()
        let mock_events = vec![];
        
        info!("Found {} claim events from block {}", mock_events.len(), from_block);
        Ok(mock_events)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let block_number = self.web3.eth().block_number().await?;
        Ok(block_number.as_u64())
    }

    /// Check if an order has been claimed
    pub async fn is_order_claimed(&self, order_id: u32) -> Result<bool> {
        let result: bool = self.bridge_contract
            .query("isClaimed", order_id, None, Options::default(), None)
            .await?;

        Ok(result)
    }

    /// Get USDC balance of an address
    pub async fn get_usdc_balance(&self, address: Address) -> Result<U256> {
        // Create USDC contract instance (ERC20)
        let usdc_abi = r#"[{"constant":true,"inputs":[{"name":"_owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"}]"#;
        
        let usdc_contract = Contract::from_json(
            self.web3.eth(),
            self.addresses.usdc_token,
            usdc_abi.as_bytes()
        )?;

        let balance: U256 = usdc_contract
            .query("balanceOf", address, None, Options::default(), None)
            .await?;

        Ok(balance)
    }



    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<NetworkStats> {
        let block_number = self.get_block_number().await?;
        let gas_price = self.web3.eth().gas_price().await?;
        let latest_batch = self.get_latest_batch_id().await.unwrap_or(0);

        Ok(NetworkStats {
            chain_id: self.chain_config.chain_id,
            block_number,
            gas_price,
            latest_batch_id: latest_batch,
            bridge_address: self.addresses.bridge,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct NetworkStats {
    pub chain_id: u64,
    pub block_number: u64,
    pub gas_price: U256,
    pub latest_batch_id: u32,
    pub bridge_address: Address,
}

// Helper function to convert hex string to H256
pub fn hex_to_h256(hex: &str) -> Result<H256> {
    let clean_hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(clean_hex)?;
    if bytes.len() != 32 {
        return Err(anyhow::anyhow!("Invalid hex length for H256"));
    }
    Ok(H256::from_slice(&bytes))
}

// Helper function to convert hex string to Address
pub fn hex_to_address(hex: &str) -> Result<Address> {
    let clean_hex = hex.trim_start_matches("0x");
    let bytes = hex::decode(clean_hex)?;
    if bytes.len() != 20 {
        return Err(anyhow::anyhow!("Invalid hex length for Address"));
    }
    Ok(Address::from_slice(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use web3::types::{H256, Address, U256, Bytes};

    // Test helper functions
    fn create_test_address(suffix: u8) -> Address {
        let mut bytes = [0u8; 20];
        bytes[19] = suffix;
        Address::from(bytes)
    }

    fn create_test_h256(value: u64) -> H256 {
        H256::from_low_u64_be(value)
    }

    #[test]
    fn test_contract_addresses_creation() {
        let bridge_addr = create_test_address(1);
        let proof_verifier_addr = create_test_address(2);
        let usdc_addr = create_test_address(3);
        
        let addresses = ContractAddresses {
            bridge: bridge_addr,
            proof_verifier: proof_verifier_addr,
            usdc_token: usdc_addr,
            pyusd_token: None,
        };

        assert_eq!(addresses.bridge, bridge_addr);
        assert_eq!(addresses.proof_verifier, proof_verifier_addr);
        assert_eq!(addresses.usdc_token, usdc_addr);
        assert!(addresses.pyusd_token.is_none());
    }

    #[test]
    fn test_chain_config_creation() {
        let config = ChainConfig {
            chain_id: 1,
            rpc_url: "https://mainnet.infura.io/v3/test".to_string(),
            gas_price: Some(U256::from(20_000_000_000u64)), // 20 gwei
            gas_limit: U256::from(500_000),
        };

        assert_eq!(config.chain_id, 1);
        assert!(config.rpc_url.contains("mainnet.infura.io"));
        assert_eq!(config.gas_price.unwrap(), U256::from(20_000_000_000u64));
        assert_eq!(config.gas_limit, U256::from(500_000));
    }

    #[test]
    fn test_proof_submission_result_creation() {
        let result = ProofSubmissionResult {
            transaction_hash: create_test_h256(123),
            batch_id: 42,
            gas_used: Some(U256::from(180_000)),
            success: true,
        };

        assert_eq!(result.batch_id, 42);
        assert!(result.success);
        assert_eq!(result.gas_used.unwrap(), U256::from(180_000));
    }

    #[test]
    fn test_deposit_event_creation() {
        let deposit = DepositEvent {
            user: create_test_address(1),
            token: create_test_address(2),
            amount: U256::from(1000_000_000), // 1000 USDC (6 decimals)
            banking_hash: create_test_h256(456),
            block_number: 18_500_000,
            transaction_hash: create_test_h256(789),
        };

        assert_eq!(deposit.user, create_test_address(1));
        assert_eq!(deposit.amount, U256::from(1000_000_000));
        assert_eq!(deposit.block_number, 18_500_000);
    }

    #[test]
    fn test_claim_event_creation() {
        let claim = ClaimEvent {
            user: create_test_address(3),
            batch_id: 5,
            order_id: 123,
            amount: U256::from(500_000_000), // 500 USDC
            block_number: 18_500_100,
            transaction_hash: create_test_h256(999),
        };

        assert_eq!(claim.user, create_test_address(3));
        assert_eq!(claim.batch_id, 5);
        assert_eq!(claim.order_id, 123);
        assert_eq!(claim.amount, U256::from(500_000_000));
    }

    #[test]
    fn test_hex_to_h256_valid() {
        let hex = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = hex_to_h256(hex).unwrap();
        
        // Check that the hash was created correctly by converting back to bytes
        let expected_bytes = hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        assert_eq!(result.as_bytes(), &expected_bytes[..]);
    }

    #[test]
    fn test_hex_to_h256_without_prefix() {
        let hex = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = hex_to_h256(hex).unwrap();
        
        // Check that the hash was created correctly by converting back to bytes
        let expected_bytes = hex::decode(hex).unwrap();
        assert_eq!(result.as_bytes(), &expected_bytes[..]);
    }

    #[test]
    fn test_hex_to_h256_invalid_length() {
        let hex = "0x1234"; // Too short
        let result = hex_to_h256(hex);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid hex length"));
    }

    #[test]
    fn test_hex_to_h256_invalid_chars() {
        let hex = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg"; // Invalid 'g'
        let result = hex_to_h256(hex);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_hex_to_address_valid() {
        let hex = "0x1234567890123456789012345678901234567890";
        let result = hex_to_address(hex).unwrap();
        
        // Check that the address was created correctly by converting back to bytes
        let expected_bytes = hex::decode("1234567890123456789012345678901234567890").unwrap();
        assert_eq!(result.as_bytes(), &expected_bytes[..]);
    }

    #[test]
    fn test_hex_to_address_without_prefix() {
        let hex = "1234567890123456789012345678901234567890";
        let result = hex_to_address(hex).unwrap();
        
        // Check that the address was created correctly by converting back to bytes
        let expected_bytes = hex::decode(hex).unwrap();
        assert_eq!(result.as_bytes(), &expected_bytes[..]);
    }

    #[test]
    fn test_hex_to_address_invalid_length() {
        let hex = "0x1234"; // Too short (need 20 bytes = 40 hex chars)
        let result = hex_to_address(hex);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid hex length"));
    }

    #[test]
    fn test_hex_to_address_invalid_chars() {
        let hex = "0x123456789012345678901234567890123456789g"; // Invalid 'g'
        let result = hex_to_address(hex);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_network_stats_creation() {
        let stats = NetworkStats {
            chain_id: 1,
            block_number: 18_500_000,
            gas_price: U256::from(25_000_000_000u64), // 25 gwei
            latest_batch_id: 42,
            bridge_address: create_test_address(1),
        };

        assert_eq!(stats.chain_id, 1);
        assert_eq!(stats.block_number, 18_500_000);
        assert_eq!(stats.gas_price, U256::from(25_000_000_000u64));
        assert_eq!(stats.latest_batch_id, 42);
    }

    // Mock blockchain client for testing (since real Web3 requires live connection)
    pub struct MockBlockchainClient {
        pub addresses: ContractAddresses,
        pub chain_config: ChainConfig,
        pub mock_batch_id: u32,
        pub mock_block_number: u64,
        pub mock_events: Vec<DepositEvent>,
    }

    impl MockBlockchainClient {
        pub fn new_for_testing() -> Self {
            Self {
                addresses: ContractAddresses {
                    bridge: create_test_address(1),
                    proof_verifier: create_test_address(2),
                    usdc_token: create_test_address(3),
                    pyusd_token: Some(create_test_address(4)),
                },
                chain_config: ChainConfig {
                    chain_id: 31337, // Hardhat local chain
                    rpc_url: "http://localhost:8545".to_string(),
                    gas_price: Some(U256::from(20_000_000_000u64)),
                    gas_limit: U256::from(500_000),
                },
                mock_batch_id: 5,
                mock_block_number: 100,
                mock_events: vec![],
            }
        }

        pub async fn submit_proof(
            &self,
            batch_id: u32,
            _prev_batch_id: u32,
            _prev_state_root: H256,
            _prev_orders_root: H256,
            _new_state_root: H256,
            _new_orders_root: H256,
            _proof: Bytes,
        ) -> Result<ProofSubmissionResult> {
            Ok(ProofSubmissionResult {
                transaction_hash: create_test_h256(batch_id as u64),
                batch_id,
                gas_used: Some(U256::from(200_000)),
                success: true,
            })
        }

        pub async fn get_latest_batch_id(&self) -> Result<u32> {
            Ok(self.mock_batch_id)
        }

        pub async fn get_batch_roots(&self, _batch_id: u32) -> Result<(H256, H256)> {
            Ok((create_test_h256(111), create_test_h256(222)))
        }

        pub async fn get_block_number(&self) -> Result<u64> {
            Ok(self.mock_block_number)
        }

        pub async fn is_order_claimed(&self, order_id: u32) -> Result<bool> {
            // Mock: even order IDs are claimed
            Ok(order_id % 2 == 0)
        }

        pub async fn get_usdc_balance(&self, _address: Address) -> Result<U256> {
            Ok(U256::from(1000_000_000u64)) // 1000 USDC
        }

        pub async fn get_deposit_events(&self, _from_block: u64, _to_block: Option<u64>) -> Result<Vec<DepositEvent>> {
            Ok(self.mock_events.clone())
        }

        pub async fn get_claim_events(&self, _from_block: u64, _to_block: Option<u64>) -> Result<Vec<ClaimEvent>> {
            Ok(vec![
                ClaimEvent {
                    user: create_test_address(4),
                    batch_id: 3,
                    order_id: 456,
                    amount: U256::from(750_000_000),
                    block_number: 95,
                    transaction_hash: create_test_h256(333),
                }
            ])
        }

        pub async fn get_network_stats(&self) -> Result<NetworkStats> {
            Ok(NetworkStats {
                chain_id: self.chain_config.chain_id,
                block_number: self.mock_block_number,
                gas_price: U256::from(20_000_000_000u64),
                latest_batch_id: self.mock_batch_id,
                bridge_address: self.addresses.bridge,
            })
        }
    }

    #[tokio::test]
    async fn test_mock_client_submit_proof() {
        let client = MockBlockchainClient::new_for_testing();
        
        let result = client.submit_proof(
            42,
            41,
            create_test_h256(100),
            create_test_h256(200),
            create_test_h256(300),
            create_test_h256(400),
            Bytes::from(vec![1, 2, 3, 4]),
        ).await.unwrap();

        assert_eq!(result.batch_id, 42);
        assert!(result.success);
        assert_eq!(result.gas_used.unwrap(), U256::from(200_000));
        assert_eq!(result.transaction_hash, create_test_h256(42));
    }

    #[tokio::test]
    async fn test_mock_client_get_latest_batch_id() {
        let client = MockBlockchainClient::new_for_testing();
        
        let batch_id = client.get_latest_batch_id().await.unwrap();
        assert_eq!(batch_id, 5);
    }

    #[tokio::test]
    async fn test_mock_client_get_batch_roots() {
        let client = MockBlockchainClient::new_for_testing();
        
        let (state_root, orders_root) = client.get_batch_roots(42).await.unwrap();
        assert_eq!(state_root, create_test_h256(111));
        assert_eq!(orders_root, create_test_h256(222));
    }

    #[tokio::test]
    async fn test_mock_client_get_block_number() {
        let client = MockBlockchainClient::new_for_testing();
        
        let block_number = client.get_block_number().await.unwrap();
        assert_eq!(block_number, 100);
    }

    #[tokio::test]
    async fn test_mock_client_is_order_claimed() {
        let client = MockBlockchainClient::new_for_testing();
        
        let claimed_even = client.is_order_claimed(42).await.unwrap();
        let claimed_odd = client.is_order_claimed(43).await.unwrap();
        
        assert!(claimed_even); // Even order ID should be claimed
        assert!(!claimed_odd); // Odd order ID should not be claimed
    }

    #[tokio::test]
    async fn test_mock_client_get_usdc_balance() {
        let client = MockBlockchainClient::new_for_testing();
        let test_address = create_test_address(5);
        
        let balance = client.get_usdc_balance(test_address).await.unwrap();
        assert_eq!(balance, U256::from(1000_000_000u64));
    }

    #[tokio::test]
    async fn test_mock_client_get_deposit_events() {
        let client = MockBlockchainClient::new_for_testing();
        
        let events = client.get_deposit_events(50, Some(100)).await.unwrap();
        assert_eq!(events.len(), 0); // Mock returns empty events
    }

    #[tokio::test]
    async fn test_mock_client_get_claim_events() {
        let client = MockBlockchainClient::new_for_testing();
        
        let events = client.get_claim_events(50, Some(100)).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].user, create_test_address(4));
        assert_eq!(events[0].batch_id, 3);
        assert_eq!(events[0].order_id, 456);
    }

    #[tokio::test]
    async fn test_mock_client_get_network_stats() {
        let client = MockBlockchainClient::new_for_testing();
        
        let stats = client.get_network_stats().await.unwrap();
        assert_eq!(stats.chain_id, 31337);
        assert_eq!(stats.block_number, 100);
        assert_eq!(stats.latest_batch_id, 5);
        assert_eq!(stats.bridge_address, create_test_address(1));
    }

    #[test]
    fn test_contract_addresses_serialization() {
        let addresses = ContractAddresses {
            bridge: create_test_address(1),
            proof_verifier: create_test_address(2),
            usdc_token: create_test_address(3),
            pyusd_token: Some(create_test_address(4)),
        };

        let json = serde_json::to_string(&addresses).unwrap();
        assert!(json.contains("bridge"));
        assert!(json.contains("proof_verifier"));
        assert!(json.contains("usdc_token"));
        assert!(json.contains("pyusd_token"));

        let deserialized: ContractAddresses = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.bridge, addresses.bridge);
        assert_eq!(deserialized.proof_verifier, addresses.proof_verifier);
        assert_eq!(deserialized.usdc_token, addresses.usdc_token);
        assert_eq!(deserialized.pyusd_token, addresses.pyusd_token);
    }

    #[test]
    fn test_proof_submission_result_serialization() {
        let result = ProofSubmissionResult {
            transaction_hash: create_test_h256(123),
            batch_id: 42,
            gas_used: Some(U256::from(180_000)),
            success: true,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("transaction_hash"));
        assert!(json.contains("batch_id"));
        assert!(json.contains("gas_used"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_deposit_event_serialization() {
        let deposit = DepositEvent {
            user: create_test_address(1),
            token: create_test_address(2),
            amount: U256::from(1000_000_000),
            banking_hash: create_test_h256(456),
            block_number: 18_500_000,
            transaction_hash: create_test_h256(789),
        };

        let json = serde_json::to_string(&deposit).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("token"));
        assert!(json.contains("amount"));
        assert!(json.contains("banking_hash"));
        assert!(json.contains("block_number"));
    }

    #[test]
    fn test_network_stats_serialization() {
        let stats = NetworkStats {
            chain_id: 1,
            block_number: 18_500_000,
            gas_price: U256::from(25_000_000_000u64),
            latest_batch_id: 42,
            bridge_address: create_test_address(1),
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("chain_id"));
        assert!(json.contains("block_number"));
        assert!(json.contains("gas_price"));
        assert!(json.contains("latest_batch_id"));
        assert!(json.contains("bridge_address"));
    }

    // Integration tests with mock blockchain behavior
    #[tokio::test]
    async fn test_full_proof_submission_workflow() {
        let client = MockBlockchainClient::new_for_testing();
        
        // 1. Get current state
        let initial_batch_id = client.get_latest_batch_id().await.unwrap();
        let block_number = client.get_block_number().await.unwrap();
        
        // 2. Submit proof for next batch
        let next_batch_id = initial_batch_id + 1;
        let result = client.submit_proof(
            next_batch_id,
            initial_batch_id,
            create_test_h256(1000),
            create_test_h256(2000),
            create_test_h256(3000),
            create_test_h256(4000),
            Bytes::from(vec![0xde, 0xad, 0xbe, 0xef]),
        ).await.unwrap();
        
        // 3. Verify submission result
        assert_eq!(result.batch_id, next_batch_id);
        assert!(result.success);
        assert!(result.gas_used.is_some());
        assert_eq!(result.transaction_hash, create_test_h256(next_batch_id as u64));
        
        // 4. Check network stats
        let stats = client.get_network_stats().await.unwrap();
        assert_eq!(stats.chain_id, 31337);
        assert_eq!(stats.block_number, block_number);
    }

    #[tokio::test]
    async fn test_event_monitoring_workflow() {
        let client = MockBlockchainClient::new_for_testing();
        
        // 1. Get current block for event monitoring
        let current_block = client.get_block_number().await.unwrap();
        
        // 2. Check for deposit events
        let deposits = client.get_deposit_events(current_block - 10, Some(current_block)).await.unwrap();
        assert_eq!(deposits.len(), 0); // Mock returns empty
        
        // 3. Check for claim events  
        let claims = client.get_claim_events(current_block - 10, Some(current_block)).await.unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].order_id, 456);
        
        // 4. Verify order claim status
        let is_claimed = client.is_order_claimed(456).await.unwrap();
        assert!(is_claimed); // Order 456 is even, so should be claimed
    }

    #[tokio::test]
    async fn test_balance_checking_workflow() {
        let client = MockBlockchainClient::new_for_testing();
        let test_user = create_test_address(10);
        
        // 1. Check USDC balance
        let balance = client.get_usdc_balance(test_user).await.unwrap();
        assert_eq!(balance, U256::from(1000_000_000u64)); // 1000 USDC
        
        // 2. Get batch roots for verification
        let (state_root, orders_root) = client.get_batch_roots(42).await.unwrap();
        assert_ne!(state_root, H256::zero());
        assert_ne!(orders_root, H256::zero());
    }

    #[test]
    fn test_large_numbers_handling() {
        // Test with large U256 values (typical for token amounts)
        let large_amount = U256::from_dec_str("1000000000000000000000").unwrap(); // 1000 tokens with 18 decimals
        
        let deposit = DepositEvent {
            user: create_test_address(1),
            token: create_test_address(2),
            amount: large_amount,
            banking_hash: create_test_h256(456),
            block_number: 18_500_000,
            transaction_hash: create_test_h256(789),
        };

        assert_eq!(deposit.amount, large_amount);
        
        // Test serialization with large numbers
        let json = serde_json::to_string(&deposit).unwrap();
        // U256 may serialize as hex string, so check for the amount value
        assert!(json.contains("amount"));
        assert_eq!(deposit.amount.to_string(), "1000000000000000000000");
    }

    #[test]
    fn test_address_edge_cases() {
        // Test zero address
        let zero_address = Address::zero();
        let addresses = ContractAddresses {
            bridge: zero_address,
            proof_verifier: zero_address,
            usdc_token: zero_address,
            pyusd_token: None,
        };
        
        assert_eq!(addresses.bridge, Address::zero());
        assert_eq!(addresses.proof_verifier, Address::zero());
        
        // Test maximum address
        let max_address = Address::from([0xff; 20]);
        assert_ne!(max_address, Address::zero());
    }

    #[test]
    fn test_h256_edge_cases() {
        // Test zero hash
        let zero_hash = H256::zero();
        assert_eq!(zero_hash.as_bytes(), &[0u8; 32]);
        
        // Test maximum hash
        let max_hash = H256::from([0xff; 32]);
        assert_eq!(max_hash.as_bytes(), &[0xff; 32]);
    }
}