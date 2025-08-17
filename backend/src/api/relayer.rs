use axum::{
    extract::{State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct ProcessEventsQuery {
    pub from_block: Option<u64>,
    pub to_block: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct RelayerStatsResponse {
    pub is_running: bool,
    pub last_processed_block: u64,
    pub total_deposits_processed: u64,
    pub total_orders_created: u64,
    pub last_poll_time: Option<String>,
    pub current_block: Option<u64>,
}

/// Get relayer service status and statistics
pub async fn get_relayer_status(
    State(app_state): State<AppState>,
) -> Result<Json<RelayerStatsResponse>, StatusCode> {
    info!("Getting relayer status");

    if let Some(relayer_service) = &app_state.relayer_service {
        let relayer = relayer_service.lock().await;
        let stats = relayer.get_stats();
        
        // Try to get current block number
        let current_block = if let Some(blockchain_client) = &app_state.blockchain_client {
            blockchain_client.get_block_number().await.ok()
        } else {
            None
        };

        let response = RelayerStatsResponse {
            is_running: stats.is_running,
            last_processed_block: stats.last_processed_block,
            total_deposits_processed: stats.total_deposits_processed,
            total_orders_created: stats.total_orders_created,
            last_poll_time: stats.last_poll_time.map(|t| t.to_rfc3339()),
            current_block,
        };

        Ok(Json(response))
    } else {
        warn!("Relayer service not initialized");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Manually process events from blockchain (useful for testing)
pub async fn process_events_manually(
    State(app_state): State<AppState>,
    Query(params): Query<ProcessEventsQuery>,
) -> Result<Json<Value>, StatusCode> {
    info!("Manual event processing requested: {:?}", params);

    if let Some(relayer_service) = &app_state.relayer_service {
        let mut relayer = relayer_service.lock().await;
        
        match relayer.process_events_manually(params.from_block, params.to_block).await {
            Ok(events_processed) => {
                info!("Manually processed {} events", events_processed);
                Ok(Json(json!({
                    "status": "success",
                    "events_processed": events_processed,
                    "from_block": params.from_block,
                    "to_block": params.to_block,
                    "message": format!("Processed {} deposit events", events_processed)
                })))
            }
            Err(e) => {
                error!("Failed to process events manually: {}", e);
                Ok(Json(json!({
                    "status": "error",
                    "message": format!("Failed to process events: {}", e)
                })))
            }
        }
    } else {
        warn!("Relayer service not initialized");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Update relayer configuration
#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub poll_interval_seconds: Option<u64>,
}

pub async fn update_relayer_config(
    State(app_state): State<AppState>,
    Json(req): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Updating relayer config: {:?}", req);

    if let Some(relayer_service) = &app_state.relayer_service {
        let mut relayer = relayer_service.lock().await;
        
        if let Some(poll_interval) = req.poll_interval_seconds {
            relayer.update_config(poll_interval);
            info!("Updated relayer poll interval to {} seconds", poll_interval);
        }

        Ok(Json(json!({
            "status": "success",
            "message": "Relayer configuration updated",
            "poll_interval_seconds": req.poll_interval_seconds
        })))
    } else {
        warn!("Relayer service not initialized");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Get current blockchain status as seen by relayer
pub async fn get_blockchain_status(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Getting blockchain status from relayer");

    if let Some(relayer_service) = &app_state.relayer_service {
        let relayer = relayer_service.lock().await;
        
        match relayer.get_current_block().await {
            Ok(current_block) => {
                let stats = relayer.get_stats();
                let blocks_behind = current_block.saturating_sub(stats.last_processed_block);

                Ok(Json(json!({
                    "status": "connected",
                    "current_block": current_block,
                    "last_processed_block": stats.last_processed_block,
                    "blocks_behind": blocks_behind,
                    "is_synced": blocks_behind <= 5, // Consider synced if within 5 blocks
                    "relayer_running": stats.is_running
                })))
            }
            Err(e) => {
                error!("Failed to get blockchain status: {}", e);
                Ok(Json(json!({
                    "status": "error",
                    "message": format!("Failed to get blockchain status: {}", e),
                    "relayer_running": relayer.get_stats().is_running
                })))
            }
        }
    } else {
        warn!("Relayer service not initialized");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
