use axum::{extract::State, Json};
use serde::Serialize;
use sqlx::Row;
use tracing::info;
use chrono::Utc;

use super::AppState;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub timestamp: String,
    pub database: DatabaseHealth,
    pub services: ServicesHealth,
    pub blockchain: Option<BlockchainHealth>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub total_orders: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ServicesHealth {
    pub matching_engine: ServiceStatus,
    pub batch_processor: ServiceStatus,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub status: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BlockchainHealth {
    pub connected: bool,
    pub chain_id: Option<u64>,
    pub latest_block: Option<u64>,
}

/// Health check endpoint with comprehensive system status
pub async fn health_check(State(app_state): State<AppState>) -> Json<HealthResponse> {
    info!("Health check requested");
    
    // Check database connectivity
    let database_health = check_database_health(&app_state).await;
    
    // Check services status
    let services_health = check_services_health(&app_state).await;
    
    // Check blockchain connectivity if available
    let blockchain_health = check_blockchain_health(&app_state).await;
    
    // Determine overall status
    let overall_status = if database_health.connected {
        "healthy"
    } else {
        "degraded"
    };

    let response = HealthResponse {
        status: overall_status.to_string(),
        service: "cashlink-backend".to_string(),
        version: "0.1.0".to_string(),
        timestamp: Utc::now().to_rfc3339(),
        database: database_health,
        services: services_health,
        blockchain: blockchain_health,
    };

    Json(response)
}

/// Simple health check for load balancers
pub async fn health_simple() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

async fn check_database_health(app_state: &AppState) -> DatabaseHealth {
    // Test database connection with a simple query
    let connected = match sqlx::query("SELECT 1").fetch_optional(&app_state.db).await {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            false
        }
    };

    let total_orders = if connected {
        sqlx::query("SELECT COUNT(*) as count FROM orders")
            .fetch_optional(&app_state.db)
            .await
            .ok()
            .flatten()
            .and_then(|row| row.try_get::<i64, _>("count").ok())
    } else {
        None
    };

    DatabaseHealth {
        connected,
        total_orders,
    }
}

async fn check_services_health(app_state: &AppState) -> ServicesHealth {
    // Check matching engine
    let matching_engine_status = {
        if let Ok(engine) = app_state.matching_engine.try_lock() {
            let stats = engine.get_stats();
            ServiceStatus {
                status: "healthy".to_string(),
                details: Some(format!("Fillers: {}, Pending orders: {}", 
                    stats.active_fillers, stats.pending_orders)),
            }
        } else {
            ServiceStatus {
                status: "busy".to_string(),
                details: Some("Engine is currently processing".to_string()),
            }
        }
    };

    // Check batch processor
    let batch_processor_status = {
        if let Ok(processor) = app_state.batch_processor.try_lock() {
            let stats = processor.get_stats();
            ServiceStatus {
                status: "healthy".to_string(),
                details: Some(format!("Next batch: {}, Accounts: {}, Active batch: {}", 
                    stats.next_batch_id, stats.total_accounts, stats.has_active_batch)),
            }
        } else {
            ServiceStatus {
                status: "busy".to_string(),
                details: Some("Processor is currently working".to_string()),
            }
        }
    };

    ServicesHealth {
        matching_engine: matching_engine_status,
        batch_processor: batch_processor_status,
    }
}

async fn check_blockchain_health(app_state: &AppState) -> Option<BlockchainHealth> {
    if let Some(blockchain_client) = &app_state.blockchain_client {
        match blockchain_client.get_network_stats().await {
            Ok(stats) => {
                Some(BlockchainHealth {
                    connected: true,
                    chain_id: Some(stats.chain_id),
                    latest_block: Some(stats.block_number),
                })
            }
            Err(e) => {
                tracing::error!("Blockchain health check failed: {}", e);
                Some(BlockchainHealth {
                    connected: false,
                    chain_id: None,
                    latest_block: None,
                })
            }
        }
    } else {
        None
    }
}