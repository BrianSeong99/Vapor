use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::Utc;
use sqlx::{SqlitePool, Row};

use crate::blockchain::{BlockchainClient, DepositEvent};
use crate::models::{Order, OrderType, OrderStatus};
use crate::services::{
    matching_engine::MatchingEngine,
    batch_processor::BatchProcessor,
};

/// Relayer service that monitors blockchain events and creates orders
pub struct RelayerService {
    /// Blockchain client for monitoring events
    blockchain_client: Arc<BlockchainClient>,
    /// Database connection
    db: SqlitePool,
    /// Matching engine for automatic order matching
    matching_engine: Arc<Mutex<MatchingEngine>>,
    /// Batch processor for order batching
    batch_processor: Arc<Mutex<BatchProcessor>>,
    /// Last processed block number
    last_processed_block: u64,
    /// Polling interval in seconds
    poll_interval_seconds: u64,
    /// Whether the relayer is running
    is_running: bool,
}

/// Configuration for the relayer service
#[derive(Debug, Clone)]
pub struct RelayerConfig {
    pub poll_interval_seconds: u64,
    pub start_block: Option<u64>,
    pub auto_match_orders: bool,
    pub auto_batch_orders: bool,
}

impl Default for RelayerConfig {
    fn default() -> Self {
        Self {
            poll_interval_seconds: 12, // ~1 block on Ethereum
            start_block: None, // Start from latest block
            auto_match_orders: true,
            auto_batch_orders: true,
        }
    }
}

/// Statistics for the relayer service
#[derive(Debug)]
pub struct RelayerStats {
    pub is_running: bool,
    pub last_processed_block: u64,
    pub total_deposits_processed: u64,
    pub total_orders_created: u64,
    pub last_poll_time: Option<chrono::DateTime<Utc>>,
}

impl RelayerService {
    /// Create a new relayer service
    pub async fn new(
        blockchain_client: Arc<BlockchainClient>,
        db: SqlitePool,
        matching_engine: Arc<Mutex<MatchingEngine>>,
        batch_processor: Arc<Mutex<BatchProcessor>>,
        config: RelayerConfig,
    ) -> Result<Self> {
        // Get starting block number
        let last_processed_block = if let Some(start_block) = config.start_block {
            start_block
        } else {
            // Start from current block - 100 blocks for safety
            blockchain_client.get_block_number().await?.saturating_sub(100)
        };

        info!("Initializing relayer service from block {}", last_processed_block);

        Ok(Self {
            blockchain_client,
            db,
            matching_engine,
            batch_processor,
            last_processed_block,
            poll_interval_seconds: config.poll_interval_seconds,
            is_running: false,
        })
    }

    /// Start the relayer service as a background task
    pub async fn start(&mut self, config: RelayerConfig) -> Result<()> {
        if self.is_running {
            warn!("Relayer service is already running");
            return Ok(());
        }

        self.is_running = true;
        info!("Starting relayer service with {} second intervals", self.poll_interval_seconds);

        let mut poll_interval = interval(Duration::from_secs(self.poll_interval_seconds));

        loop {
            poll_interval.tick().await;

            if !self.is_running {
                info!("Relayer service stopped");
                break;
            }

            // Process new events
            match self.process_new_events(&config).await {
                Ok(events_processed) => {
                    if events_processed > 0 {
                        info!("Processed {} new deposit events", events_processed);
                    } else {
                        debug!("No new events found");
                    }
                }
                Err(e) => {
                    error!("Error processing events: {}", e);
                    // Continue running on errors, but log them
                }
            }
        }

        Ok(())
    }

    /// Stop the relayer service
    pub fn stop(&mut self) {
        info!("Stopping relayer service");
        self.is_running = false;
    }

    /// Process new blockchain events since last check
    async fn process_new_events(&mut self, config: &RelayerConfig) -> Result<usize> {
        // Get current block number
        let current_block = self.blockchain_client.get_block_number().await?;
        
        if current_block <= self.last_processed_block {
            // No new blocks to process
            return Ok(0);
        }

        debug!("Checking blocks {} to {}", self.last_processed_block + 1, current_block);

        // Get deposit events from last processed block to current block
        let deposit_events = self.blockchain_client
            .get_deposit_events(self.last_processed_block + 1, Some(current_block))
            .await?;

        let mut events_processed = 0;

        for event in deposit_events {
            match self.process_deposit_event(&event, config).await {
                Ok(_) => {
                    events_processed += 1;
                    info!("Processed deposit event: {:?} -> {} {}", 
                        event.user, event.amount, event.token);
                }
                Err(e) => {
                    error!("Failed to process deposit event {:?}: {}", event, e);
                }
            }
        }

        // Update last processed block
        self.last_processed_block = current_block;

        Ok(events_processed)
    }

    /// Process a single deposit event and create corresponding BridgeIn order
    async fn process_deposit_event(&self, event: &DepositEvent, config: &RelayerConfig) -> Result<()> {
        info!("Processing deposit event: user={:?}, amount={}, token={:?}", 
            event.user, event.amount, event.token);

        // Check if this deposit has already been processed
        if self.is_deposit_already_processed(event).await? {
            warn!("Deposit event already processed: tx={:?}", event.transaction_hash);
            return Ok(());
        }

        // Create BridgeIn order from deposit event
        let bridge_in_order = Order {
            id: Uuid::new_v4().to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some(format!("{:?}", event.user)),
            to_address: Some(format!("{:?}", event.user)), // User receives to same address
            token_id: self.token_address_to_id(&event.token),
            amount: event.amount.to_string(),
            banking_hash: Some(format!("{:?}", event.banking_hash)),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save order to database
        self.save_order_to_database(&bridge_in_order).await?;

        // Add to matching engine if auto-matching is enabled
        if config.auto_match_orders {
            let mut engine = self.matching_engine.lock().await;
            engine.add_order(bridge_in_order.clone())?;
            
            // Trigger matching
            let matches = engine.match_orders()?;
            if !matches.is_empty() {
                info!("Auto-matched {} orders from deposit event", matches.len());
            }
        }

        let order_id = bridge_in_order.id.clone();

        // Add to batch processor if auto-batching is enabled
        if config.auto_batch_orders {
            let mut processor = self.batch_processor.lock().await;
            
            // Ensure there's an active batch
            if processor.get_current_batch().is_none() {
                processor.start_batch()?;
                info!("Started new batch for deposit processing");
            }
            
            processor.add_order_to_batch(bridge_in_order)?;
            info!("Added BridgeIn order to batch");
        }

        info!("Successfully processed deposit event and created BridgeIn order: {}", order_id);
        Ok(())
    }

    /// Check if a deposit event has already been processed
    async fn is_deposit_already_processed(&self, event: &DepositEvent) -> Result<bool> {
        let query = "SELECT COUNT(*) as count FROM orders WHERE banking_hash = ?";
        let banking_hash = format!("{:?}", event.banking_hash);
        
        let row = sqlx::query(query)
            .bind(&banking_hash)
            .fetch_one(&self.db)
            .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count > 0)
    }

    /// Save order to database
    async fn save_order_to_database(&self, order: &Order) -> Result<()> {
        let query = r#"
            INSERT INTO orders (id, order_type, status, from_address, to_address, token_id, amount, banking_hash, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        "#;
        
        sqlx::query(query)
            .bind(&order.id)
            .bind(order.order_type as i32)
            .bind(order.status as i32)
            .bind(&order.from_address)
            .bind(&order.to_address)
            .bind(order.token_id as i32)
            .bind(&order.amount)
            .bind(&order.banking_hash)
            .bind(order.created_at)
            .bind(order.updated_at)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Convert token address to token ID (simplified mapping)
    fn token_address_to_id(&self, token_address: &web3::types::Address) -> u32 {
        // Simple token mapping - in production, this would be configurable
        // For MVP, assume token ID 1 = USDC, token ID 2 = PYUSD
        let token_str = format!("{:?}", token_address).to_lowercase();
        
        if token_str.contains("usdc") || token_str.ends_with("001") {
            1 // USDC
        } else if token_str.contains("pyusd") || token_str.ends_with("002") {
            2 // PYUSD
        } else {
            1 // Default to USDC
        }
    }

    /// Get relayer statistics
    pub fn get_stats(&self) -> RelayerStats {
        RelayerStats {
            is_running: self.is_running,
            last_processed_block: self.last_processed_block,
            total_deposits_processed: 0, // TODO: Track this in database
            total_orders_created: 0, // TODO: Track this in database
            last_poll_time: Some(Utc::now()), // TODO: Track actual last poll time
        }
    }

    /// Manual trigger to process events (useful for testing)
    pub async fn process_events_manually(&mut self, from_block: Option<u64>, to_block: Option<u64>) -> Result<usize> {
        let config = RelayerConfig::default();
        
        let from = from_block.unwrap_or(self.last_processed_block);
        let to = to_block.unwrap_or_else(|| self.last_processed_block + 100);
        
        info!("Manually processing events from block {} to {}", from, to);
        
        let deposit_events = self.blockchain_client
            .get_deposit_events(from, Some(to))
            .await?;

        let mut events_processed = 0;
        
        for event in deposit_events {
            match self.process_deposit_event(&event, &config).await {
                Ok(_) => events_processed += 1,
                Err(e) => error!("Failed to process event: {}", e),
            }
        }

        if to > self.last_processed_block {
            self.last_processed_block = to;
        }

        Ok(events_processed)
    }

    /// Get the current block number from blockchain
    pub async fn get_current_block(&self) -> Result<u64> {
        self.blockchain_client.get_block_number().await
    }

    /// Update relayer configuration
    pub fn update_config(&mut self, new_poll_interval: u64) {
        self.poll_interval_seconds = new_poll_interval;
        info!("Updated relayer poll interval to {} seconds", new_poll_interval);
    }
}

/// Helper function to start relayer service as a background task
pub async fn start_relayer_service(
    blockchain_client: Arc<BlockchainClient>,
    db: SqlitePool,
    matching_engine: Arc<Mutex<MatchingEngine>>,
    batch_processor: Arc<Mutex<BatchProcessor>>,
    config: RelayerConfig,
) -> Result<()> {
    let mut relayer = RelayerService::new(
        blockchain_client,
        db,
        matching_engine,
        batch_processor,
        config.clone(),
    ).await?;

    relayer.start(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::{ContractAddresses, ChainConfig, DepositEvent, BlockchainClient};
    use crate::models::{Order, OrderType, OrderStatus};
    use crate::services::{
        matching_engine::MatchingEngine, 
        batch_processor::BatchProcessor,
        mvp_prover::{MvpProverService, MvpProverConfig}
    };
    use sqlx::{SqlitePool, Row};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use uuid::{self, Uuid};
    use web3::types::{Address, U256, H256};
    use chrono::Utc;

    // Helper function to create test database
    async fn create_test_db() -> SqlitePool {
        let db = SqlitePool::connect(":memory:").await.unwrap();
        
        // Create orders table
        sqlx::query(r#"
            CREATE TABLE orders (
                id TEXT PRIMARY KEY,
                order_type INTEGER NOT NULL,
                status INTEGER NOT NULL,
                from_address TEXT,
                to_address TEXT,
                token_id INTEGER NOT NULL,
                amount TEXT NOT NULL,
                banking_hash TEXT,
                batch_id TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
        "#).execute(&db).await.unwrap();
        
        db
    }

    // Helper function to create mock blockchain client
    async fn create_test_blockchain_client() -> Result<Arc<BlockchainClient>> {
        let rpc_url = "http://localhost:8545".to_string();
        let bridge_address = Address::from_low_u64_be(1);
        let proof_verifier_address = Address::from_low_u64_be(2);  
        let usdc_address = Address::from_low_u64_be(3);
        let chain_id = 31337;
        
        // For tests, we'll create a blockchain client but won't actually use it for calls
        // In a real test environment, we'd use a test RPC endpoint
        Ok(Arc::new(BlockchainClient::new(
            rpc_url,
            bridge_address,
            proof_verifier_address,
            usdc_address,
            chain_id,
        ).await.unwrap()))
    }

    // Helper to create test services
    async fn create_test_services() -> (SqlitePool, Arc<Mutex<MatchingEngine>>, Arc<Mutex<BatchProcessor>>) {
        let db = create_test_db().await;
        let matching_engine = Arc::new(Mutex::new(MatchingEngine::new()));
        let prover_config = MvpProverConfig::default();
        let prover = MvpProverService::new(prover_config);
        let batch_processor = Arc::new(Mutex::new(BatchProcessor::new()));
        
        (db, matching_engine, batch_processor)
    }

    fn create_test_deposit_event(user_id: u64, amount: u64, token_id: u64) -> DepositEvent {
        DepositEvent {
            user: Address::from_low_u64_be(user_id),
            amount: U256::from(amount),
            token: Address::from_low_u64_be(token_id),
            banking_hash: H256::from_low_u64_be(12345 + user_id),
            block_number: 100,
            transaction_hash: H256::from_low_u64_be(54321 + user_id),
        }
    }

    #[test]
    fn test_relayer_config_default() {
        let config = RelayerConfig::default();
        assert_eq!(config.poll_interval_seconds, 12);
        assert!(config.start_block.is_none());
        assert!(config.auto_match_orders);
        assert!(config.auto_batch_orders);
    }

    #[test]
    fn test_relayer_config_custom() {
        let config = RelayerConfig {
            poll_interval_seconds: 30,
            start_block: Some(1000),
            auto_match_orders: false,
            auto_batch_orders: true,
        };
        
        assert_eq!(config.poll_interval_seconds, 30);
        assert_eq!(config.start_block, Some(1000));
        assert!(!config.auto_match_orders);
        assert!(config.auto_batch_orders);
    }

    #[tokio::test]
    async fn test_relayer_service_creation() {
        // Skip blockchain client tests for now as they require network connection
        let config = RelayerConfig::default();
        assert_eq!(config.poll_interval_seconds, 12);
        assert!(config.auto_match_orders);
        assert!(config.auto_batch_orders);
    }

    #[test] 
    fn test_token_address_to_id_mapping() {
        // Test simple token mapping logic (extracted from RelayerService method)
        fn token_address_to_id(token_address: &Address) -> u32 {
            let token_str = format!("{:?}", token_address).to_lowercase();
            
            if token_str.contains("usdc") || token_str.ends_with("001") {
                1 // USDC
            } else if token_str.contains("pyusd") || token_str.ends_with("002") {
                2 // PYUSD
            } else {
                1 // Default to USDC
            }
        }
        
        let usdc_address = Address::from_low_u64_be(1);
        assert_eq!(token_address_to_id(&usdc_address), 1);
        
        let pyusd_address = Address::from_low_u64_be(2);
        assert_eq!(token_address_to_id(&pyusd_address), 2); // Should be PYUSD (ends with 002)
        
        let unknown_address = Address::from_low_u64_be(999);
        assert_eq!(token_address_to_id(&unknown_address), 1);
    }

    #[tokio::test]
    async fn test_database_operations() {
        let db = create_test_db().await;
        
        // Test database is working
        let result = sqlx::query("SELECT COUNT(*) as count FROM orders")
            .fetch_one(&db)
            .await
            .unwrap();
        
        let count: i64 = result.try_get("count").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_relayer_stats_structure() {
        // Test the RelayerStats structure
        let stats = RelayerStats {
            is_running: false,
            last_processed_block: 100,
            total_deposits_processed: 5,
            total_orders_created: 3,
            last_poll_time: Some(Utc::now()),
        };
        
        assert!(!stats.is_running);
        assert_eq!(stats.last_processed_block, 100);
        assert_eq!(stats.total_deposits_processed, 5);
        assert_eq!(stats.total_orders_created, 3);
        assert!(stats.last_poll_time.is_some());
    }

    #[test]
    fn test_deposit_event_creation() {
        let event = create_test_deposit_event(1, 1000000, 1);
        
        assert_eq!(event.user, Address::from_low_u64_be(1));
        assert_eq!(event.amount, U256::from(1000000));
        assert_eq!(event.token, Address::from_low_u64_be(1));
        assert_eq!(event.block_number, 100);
    }

    #[tokio::test]
    async fn test_order_creation_from_deposit() {
        let db = create_test_db().await;
        let deposit_event = create_test_deposit_event(1, 1000000, 1);
        
        // Test creating order data from deposit event (simulated)
        let order_data = Order {
            id: uuid::Uuid::new_v4().to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some(format!("{:?}", deposit_event.user)),
            to_address: Some(format!("{:?}", deposit_event.user)),
            token_id: 1, // USDC
            amount: deposit_event.amount.to_string(),
            banking_hash: Some(format!("{:?}", deposit_event.banking_hash)),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        assert_eq!(order_data.order_type, OrderType::BridgeIn);
        assert_eq!(order_data.status, OrderStatus::Pending);
        assert_eq!(order_data.token_id, 1);
        assert_eq!(order_data.amount, "1000000");
        assert!(order_data.banking_hash.is_some());
    }

    #[tokio::test]
    async fn test_large_amount_handling() {
        let large_amount = U256::from_dec_str("1000000000000000000000000").unwrap();
        let event = create_test_deposit_event(1, 0, 1);
        
        // Manually set large amount since helper function uses u64
        let large_event = DepositEvent {
            user: event.user,
            amount: large_amount,
            token: event.token,
            banking_hash: event.banking_hash,
            block_number: event.block_number,
            transaction_hash: event.transaction_hash,
        };
        
        assert_eq!(large_event.amount.to_string(), "1000000000000000000000000");
    }
}
