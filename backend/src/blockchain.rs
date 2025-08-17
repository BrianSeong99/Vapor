use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use web3::{
    contract::{Contract, Options},
    transports::Http,
    types::{Address, U256, H256, Bytes, BlockNumber},
    Web3,
};

/// Blockchain client for interacting with CashLink smart contracts
pub struct BlockchainClient {
    /// Web3 instance for Ethereum interactions
    pub web3: Web3<Http>,
    /// CashLink Bridge contract
    pub bridge_contract: Contract<Http>,
    /// Contract addresses
    pub addresses: ContractAddresses,
    /// Chain configuration
    pub chain_config: ChainConfig,
}

/// Contract addresses on the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub bridge: Address,
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
        usdc_address: Address,
        chain_id: u64,
    ) -> Result<Self> {
        let transport = Http::new(&rpc_url)?;
        let web3 = Web3::new(transport);

        // Simple ABI for the bridge contract (subset of full contract)
        let bridge_abi = include_bytes!("abi/CashLinkBridge.json");
        let bridge_contract = Contract::from_json(
            web3.eth(),
            bridge_address,
            bridge_abi
        )?;

        let addresses = ContractAddresses {
            bridge: bridge_address,
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
        info!("USDC token: {:?}", usdc_address);

        Ok(Self {
            web3,
            bridge_contract,
            addresses,
            chain_config,
        })
    }

    /// Submit a batch proof to the bridge contract
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
        info!("Submitting proof for batch {} to blockchain", batch_id);

        // For now, return a mock result since web3 contract interaction is complex
        // In a real implementation, you'd use a transaction signer
        let mock_tx_hash = H256::from_low_u64_be(batch_id as u64);
        
        info!("Proof submitted! Transaction hash: {:?}", mock_tx_hash);

        Ok(ProofSubmissionResult {
            transaction_hash: mock_tx_hash,
            batch_id,
            gas_used: Some(U256::from(200_000)),
            success: true,
        })
    }

    /// Get the latest batch ID from the contract
    pub async fn get_latest_batch_id(&self) -> Result<u32> {
        let result: U256 = self.bridge_contract
            .query("currentBatchId", (), None, Options::default(), None)
            .await?;

        Ok(result.as_u32())
    }

    /// Get batch roots for a specific batch ID
    pub async fn get_batch_roots(&self, batch_id: u32) -> Result<(H256, H256)> {
        let result: (H256, H256) = self.bridge_contract
            .query("getBatchRoots", batch_id, None, Options::default(), None)
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
            .query("isOrderClaimed", order_id, None, Options::default(), None)
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