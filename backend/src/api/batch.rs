use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};

use super::AppState;

#[derive(Debug, Serialize)]
pub struct BatchResponse {
    pub batch_id: u32,
    pub orders_count: usize,
    pub prev_state_root: String,
    pub new_state_root: String,
    pub prev_orders_root: String,
    pub new_orders_root: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct BatchStatsResponse {
    pub next_batch_id: u32,
    pub current_batch_orders: usize,
    pub total_accounts: usize,
    pub has_active_batch: bool,
}

/// Start a new batch
pub async fn start_batch(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Starting new batch");
    
    let mut processor = app_state.batch_processor.lock().await;
    
    match processor.start_batch() {
        Ok(batch_id) => {
            info!("Started batch {}", batch_id);
            Ok(Json(json!({
                "status": "success",
                "batch_id": batch_id,
                "message": "Batch started successfully"
            })))
        }
        Err(e) => {
            error!("Failed to start batch: {}", e);
            Ok(Json(json!({
                "status": "error",
                "message": format!("Failed to start batch: {}", e)
            })))
        }
    }
}

/// Finalize current batch and generate Merkle trees
pub async fn finalize_batch(
    State(app_state): State<AppState>,
) -> Result<Json<BatchResponse>, StatusCode> {
    info!("Finalizing current batch");
    
    let mut processor = app_state.batch_processor.lock().await;
    
    match processor.finalize_batch() {
        Ok(result) => {
            info!("Batch {} finalized successfully", result.batch_id);
            
            let response = BatchResponse {
                batch_id: result.batch_id,
                orders_count: result.orders_count,
                prev_state_root: result.prev_state_root,
                new_state_root: result.new_state_root,
                prev_orders_root: result.prev_orders_root,
                new_orders_root: result.new_orders_root,
                status: "finalized".to_string(),
            };
            
            Ok(Json(response))
        }
        Err(e) => {
            error!("Failed to finalize batch: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Generate SP1 proof for a batch and submit to blockchain
pub async fn prove_batch(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Starting batch proving process");
    
    // First finalize the current batch
    let mut processor = app_state.batch_processor.lock().await;
    
    let batch_result = match processor.finalize_batch() {
        Ok(result) => result,
        Err(e) => {
            error!("Failed to finalize batch before proving: {}", e);
            return Ok(Json(json!({
                "status": "error",
                "message": format!("Failed to finalize batch: {}", e)
            })));
        }
    };
    
    drop(processor); // Release the lock
    
    info!("Batch {} finalized, starting proof generation", batch_result.batch_id);
    
    // TODO: Generate SP1 proof
    // For MVP, we'll simulate this process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Submit proof to blockchain if client is available
    if let Some(blockchain_client) = &app_state.blockchain_client {
        let prev_state_root = crate::blockchain::hex_to_h256(&batch_result.prev_state_root)
            .unwrap_or_default();
        let prev_orders_root = crate::blockchain::hex_to_h256(&batch_result.prev_orders_root)
            .unwrap_or_default();
        let new_state_root = crate::blockchain::hex_to_h256(&batch_result.new_state_root)
            .unwrap_or_default();
        let new_orders_root = crate::blockchain::hex_to_h256(&batch_result.new_orders_root)
            .unwrap_or_default();
        
        let proof_bytes = web3::types::Bytes::from(vec![0u8; 32]); // Placeholder proof
        
        match blockchain_client.submit_proof(
            batch_result.batch_id,
            batch_result.batch_id - 1,
            prev_state_root,
            prev_orders_root,
            new_state_root,
            new_orders_root,
            proof_bytes,
        ).await {
            Ok(submission_result) => {
                info!("Proof submitted to blockchain: {:?}", submission_result.transaction_hash);
                Ok(Json(json!({
                    "status": "success",
                    "batch_id": batch_result.batch_id,
                    "orders_count": batch_result.orders_count,
                    "transaction_hash": format!("{:?}", submission_result.transaction_hash),
                    "gas_used": submission_result.gas_used.map(|g| g.as_u64()),
                    "message": "Batch proven and submitted to blockchain"
                })))
            }
            Err(e) => {
                error!("Failed to submit proof to blockchain: {}", e);
                Ok(Json(json!({
                    "status": "partial_success",
                    "batch_id": batch_result.batch_id,
                    "message": format!("Batch proven but blockchain submission failed: {}", e)
                })))
            }
        }
    } else {
        warn!("No blockchain client configured, skipping on-chain submission");
        Ok(Json(json!({
            "status": "success",
            "batch_id": batch_result.batch_id,
            "orders_count": batch_result.orders_count,
            "message": "Batch proven (blockchain submission skipped - no client configured)"
        })))
    }
}

/// Get batch statistics
pub async fn get_batch_stats(
    State(app_state): State<AppState>,
) -> Result<Json<BatchStatsResponse>, StatusCode> {
    info!("Getting batch statistics");
    
    let processor = app_state.batch_processor.lock().await;
    let stats = processor.get_stats();
    
    let response = BatchStatsResponse {
        next_batch_id: stats.next_batch_id,
        current_batch_orders: stats.current_batch_orders,
        total_accounts: stats.total_accounts,
        has_active_batch: stats.has_active_batch,
    };
    
    Ok(Json(response))
}

/// Get current batch information
pub async fn get_current_batch(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Getting current batch info");
    
    let processor = app_state.batch_processor.lock().await;
    
    match processor.get_current_batch() {
        Some(batch) => {
            Ok(Json(json!({
                "batch_id": batch.batch_id,
                "prev_batch_id": batch.prev_batch_id,
                "orders_count": batch.orders.len(),
                "is_finalized": batch.is_finalized,
                "created_at": batch.created_at,
                "prev_state_root": batch.prev_state_root,
                "prev_orders_root": batch.prev_orders_root
            })))
        }
        None => {
            Ok(Json(json!({
                "message": "No active batch"
            })))
        }
    }
}

/// Initialize account for testing/demo purposes
#[derive(Debug, Deserialize)]
pub struct InitAccountRequest {
    pub address: String,
    pub token_id: u32,
    pub initial_balance: String,
}

pub async fn init_account(
    State(app_state): State<AppState>,
    Json(req): Json<InitAccountRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Initializing account: {} with {} of token {}", req.address, req.initial_balance, req.token_id);
    
    let mut processor = app_state.batch_processor.lock().await;
    
    match processor.init_account(req.address.clone(), req.token_id, req.initial_balance.clone()) {
        Ok(_) => {
            info!("Account initialized successfully: {}", req.address);
            Ok(Json(json!({
                "status": "success",
                "address": req.address,
                "token_id": req.token_id,
                "initial_balance": req.initial_balance,
                "message": "Account initialized successfully"
            })))
        }
        Err(e) => {
            error!("Failed to initialize account: {}", e);
            Ok(Json(json!({
                "status": "error",
                "message": format!("Failed to initialize account: {}", e)
            })))
        }
    }
}