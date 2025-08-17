use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub api: ApiConfig,
    pub database: DatabaseConfig,
    pub blockchain: BlockchainConfig,
    pub batch: BatchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub rpc_url: String,
    pub contract_address: String,
    pub private_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    pub interval_seconds: u64,
    pub max_orders_per_batch: usize,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Config {
            api: ApiConfig {
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:cashlink.db".to_string()),
            },
            blockchain: BlockchainConfig {
                rpc_url: env::var("RPC_URL")
                    .unwrap_or_else(|_| "http://localhost:8545".to_string()),
                contract_address: env::var("CONTRACT_ADDRESS")
                    .map_err(|_| anyhow::anyhow!("CONTRACT_ADDRESS environment variable required"))?,
                private_key: env::var("PRIVATE_KEY")
                    .map_err(|_| anyhow::anyhow!("PRIVATE_KEY environment variable required"))?,
            },
            batch: BatchConfig {
                interval_seconds: env::var("BATCH_INTERVAL_SECONDS")
                    .unwrap_or_else(|_| "60".to_string())
                    .parse()
                    .unwrap_or(60),
                max_orders_per_batch: env::var("MAX_ORDERS_PER_BATCH")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
            },
        })
    }
}
