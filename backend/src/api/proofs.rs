use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, warn, error};
use sqlx::Row;

use super::AppState;

#[derive(Debug, Deserialize)]
pub struct ProofQuery {
    pub proof_type: Option<String>, // "order" or "account"
}

#[derive(Debug, Serialize)]
pub struct ProofResponse {
    pub batch_id: u32,
    pub order_id: String,
    pub leaf_hash: String,
    pub proof: Vec<String>,
    pub root: String,
    pub valid: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountProofResponse {
    pub address: String,
    pub leaf_hash: String,
    pub proof: Vec<String>,
    pub root: String,
    pub valid: bool,
}

/// Get Merkle proof for a specific order in a batch
pub async fn get_order_proof(
    State(app_state): State<AppState>,
    Path((batch_id, order_id)): Path<(u32, String)>,
) -> Result<Json<ProofResponse>, StatusCode> {
    info!("Getting Merkle proof for batch {} order {}", batch_id, order_id);
    
    // For MVP, we'll generate a mock proof since we don't have persistent batch storage
    // In production, you'd retrieve the actual batch and generate the real proof
    
    // Check if order exists in database
    let query = "SELECT id FROM orders WHERE id = ?";
    let order_exists = sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error checking order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if order_exists.is_none() {
        warn!("Order not found: {}", order_id);
        return Err(StatusCode::NOT_FOUND);
    }

    // Generate mock proof for MVP
    let mock_proof = ProofResponse {
        batch_id,
        order_id: order_id.clone(),
        leaf_hash: format!("0x{:064x}", batch_id as u64),
        proof: vec![
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string(),
        ],
        root: format!("0x{:064x}", (batch_id * 1000) as u64),
        valid: true,
    };

    info!("Generated proof for order {} in batch {}", order_id, batch_id);
    Ok(Json(mock_proof))
}

/// Get Merkle proof for an account state
pub async fn get_account_proof(
    State(app_state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<AccountProofResponse>, StatusCode> {
    info!("Getting account state proof for address: {}", address);
    
    // For MVP, generate a mock account proof
    let mock_proof = AccountProofResponse {
        address: address.clone(),
        leaf_hash: format!("0x{:064x}", address.len() as u64),
        proof: vec![
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            "0x0987654321fedcba0987654321fedcba0987654321fedcba0987654321fedcba".to_string(),
        ],
        root: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
        valid: true,
    };

    info!("Generated account proof for address: {}", address);
    Ok(Json(mock_proof))
}

/// Verify a Merkle proof
#[derive(Debug, Deserialize)]
pub struct VerifyProofRequest {
    pub leaf_hash: String,
    pub proof: Vec<String>,
    pub root: String,
    pub index: Option<u32>,
}

pub async fn verify_proof(
    State(_app_state): State<AppState>,
    Json(req): Json<VerifyProofRequest>,
) -> Result<Json<Value>, StatusCode> {
    info!("Verifying Merkle proof");
    
    // For MVP, we'll do a simple validation
    // In production, you'd implement proper Merkle proof verification
    
    let is_valid = !req.leaf_hash.is_empty() 
        && !req.proof.is_empty() 
        && !req.root.is_empty()
        && req.leaf_hash.starts_with("0x")
        && req.root.starts_with("0x");

    info!("Proof verification result: {}", is_valid);
    
    Ok(Json(json!({
        "valid": is_valid,
        "leaf_hash": req.leaf_hash,
        "root": req.root,
        "proof_length": req.proof.len()
    })))
}

/// Get all available proofs for a batch
pub async fn get_batch_proofs(
    State(app_state): State<AppState>,
    Path(batch_id): Path<u32>,
    Query(query): Query<ProofQuery>,
) -> Result<Json<Value>, StatusCode> {
    info!("Getting all proofs for batch {}", batch_id);
    
    // Get all orders for this batch (simplified - in production you'd store batch->order mapping)
    let order_query = "SELECT id FROM orders ORDER BY created_at DESC LIMIT 10";
    let orders = sqlx::query(order_query)
        .fetch_all(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching orders: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let proof_type = query.proof_type.as_deref().unwrap_or("order");
    
    match proof_type {
        "order" => {
            let proofs: Vec<Value> = orders.iter()
                .enumerate()
                .map(|(i, row)| {
                    let order_id: String = row.try_get("id").unwrap_or_default();
                    json!({
                        "order_id": order_id,
                        "leaf_hash": format!("0x{:064x}", (batch_id * 100 + i as u32) as u64),
                        "available": true
                    })
                })
                .collect();

            Ok(Json(json!({
                "batch_id": batch_id,
                "proof_type": "order",
                "proofs": proofs,
                "count": proofs.len()
            })))
        }
        "account" => {
            // Mock account proofs
            let account_proofs = vec![
                json!({
                    "address": "0x742d35Cc6634C0532925a3b8D5C0B5Cc0532C75e",
                    "leaf_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "available": true
                }),
                json!({
                    "address": "0x8ba1f109551bD432803012645Hac136c23ad80e5",
                    "leaf_hash": "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
                    "available": true
                })
            ];

            Ok(Json(json!({
                "batch_id": batch_id,
                "proof_type": "account",
                "proofs": account_proofs,
                "count": account_proofs.len()
            })))
        }
        _ => {
            Ok(Json(json!({
                "error": "Invalid proof_type. Use 'order' or 'account'"
            })))
        }
    }
}

/// Get proof statistics
pub async fn get_proof_stats(
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    info!("Getting proof statistics");
    
    // Get total orders count
    let orders_count_query = "SELECT COUNT(*) as count FROM orders";
    let orders_count: i64 = sqlx::query(orders_count_query)
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error getting orders count: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .try_get("count")
        .unwrap_or(0);

    // Get batch processor stats
    let processor = app_state.batch_processor.lock().await;
    let batch_stats = processor.get_stats();
    
    Ok(Json(json!({
        "total_orders": orders_count,
        "current_batch_id": batch_stats.next_batch_id - 1,
        "current_batch_orders": batch_stats.current_batch_orders,
        "total_accounts": batch_stats.total_accounts,
        "has_active_batch": batch_stats.has_active_batch,
        "proof_depth": {
            "account_tree": 160,
            "order_tree": 20
        }
    })))
}