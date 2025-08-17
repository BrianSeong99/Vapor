use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use crate::services::{
    matching_engine::MatchingEngine,
    batch_processor::BatchProcessor,
};
use crate::blockchain::BlockchainClient;

pub mod health;
pub mod orders;
pub mod batch;
pub mod proofs;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: SqlitePool,
    pub matching_engine: Arc<Mutex<MatchingEngine>>,
    pub batch_processor: Arc<Mutex<BatchProcessor>>,
    pub blockchain_client: Option<Arc<BlockchainClient>>,
}

impl AppState {
    pub fn new(config: Config, db: SqlitePool) -> Self {
        Self { 
            config, 
            db,
            matching_engine: Arc::new(Mutex::new(MatchingEngine::new())),
            batch_processor: Arc::new(Mutex::new(BatchProcessor::new())),
            blockchain_client: None, // Initialize later with proper config
        }
    }
    
    pub fn with_blockchain_client(mut self, client: BlockchainClient) -> Self {
        self.blockchain_client = Some(Arc::new(client));
        self
    }
}
