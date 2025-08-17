use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use crate::services::{
    matching_engine::MatchingEngine,
    batch_processor::BatchProcessor,
    relayer::{RelayerService, RelayerConfig},
};
use crate::blockchain::BlockchainClient;

pub mod health;
pub mod orders;
pub mod batch;
pub mod proofs;
pub mod relayer;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: SqlitePool,
    pub matching_engine: Arc<Mutex<MatchingEngine>>,
    pub batch_processor: Arc<Mutex<BatchProcessor>>,
    pub blockchain_client: Option<Arc<BlockchainClient>>,
    pub relayer_service: Option<Arc<Mutex<RelayerService>>>,
}

impl AppState {
    pub fn new(config: Config, db: SqlitePool) -> Self {
        Self { 
            config, 
            db,
            matching_engine: Arc::new(Mutex::new(MatchingEngine::new())),
            batch_processor: Arc::new(Mutex::new(BatchProcessor::new())),
            blockchain_client: None, // Initialize later with proper config
            relayer_service: None, // Initialize later with blockchain client
        }
    }
    
    pub fn with_blockchain_client(mut self, client: BlockchainClient) -> Self {
        self.blockchain_client = Some(Arc::new(client));
        self
    }
    
    pub async fn with_relayer_service(mut self, relayer: RelayerService) -> Self {
        self.relayer_service = Some(Arc::new(Mutex::new(relayer)));
        self
    }
}
