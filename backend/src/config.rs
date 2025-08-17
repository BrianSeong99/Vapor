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
    pub proof_verifier_address: String,
    pub usdc_address: String,
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
                port: env::var("SERVER_PORT")
                    .or_else(|_| env::var("PORT"))
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "sqlite:vapor.db".to_string()),
            },
            blockchain: BlockchainConfig {
                rpc_url: env::var("CHAIN_RPC_URL")
                    .or_else(|_| env::var("RPC_URL"))
                    .unwrap_or_else(|_| "http://localhost:8545".to_string()),
                contract_address: env::var("VAPOR_BRIDGE_CONTRACT")
                    .or_else(|_| env::var("CONTRACT_ADDRESS"))
                    .map_err(|_| anyhow::anyhow!("VAPOR_BRIDGE_CONTRACT or CONTRACT_ADDRESS environment variable required"))?,
                proof_verifier_address: env::var("PROOF_VERIFIER_CONTRACT")
                    .map_err(|_| anyhow::anyhow!("PROOF_VERIFIER_CONTRACT environment variable required"))?,
                usdc_address: env::var("USDC_CONTRACT")
                    .map_err(|_| anyhow::anyhow!("USDC_CONTRACT environment variable required"))?,
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

impl Default for Config {
    fn default() -> Self {
        Config {
            api: ApiConfig { port: 8080 },
            database: DatabaseConfig { 
                url: ":memory:".to_string() 
            },
            blockchain: BlockchainConfig {
                rpc_url: "http://localhost:8545".to_string(),
                contract_address: "0x0000000000000000000000000000000000000000".to_string(),
                proof_verifier_address: "0x0000000000000000000000000000000000000001".to_string(),
                usdc_address: "0x0000000000000000000000000000000000000002".to_string(),
                private_key: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            },
            batch: BatchConfig {
                interval_seconds: 60,
                max_orders_per_batch: 100,
            },
        }
    }
}
