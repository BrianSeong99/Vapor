use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use sqlx::Row;

use super::AppState;
use crate::models::{
    Order, OrderResponse, OrderType, OrderStatus, 
    LockOrderRequest, SubmitPaymentProofRequest,
    FillerBalance, ClaimRequest, ClaimResponse, ProcessedClaim, WalletClaim,
};
// TODO: Fix database helpers import issue
// use crate::database::helpers::{get_filler_balance, upsert_filler_balance, add_filler_wallet, insert_claim};

#[derive(Debug, Deserialize)]
pub struct FillerQuery {
    pub status: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct DiscoveryOrdersResponse {
    pub orders: Vec<OrderResponse>,
    pub total: usize,
}

/// Get orders in discovery phase for fillers (GET /fillers/discovery)
pub async fn get_discovery_orders(
    Query(query): Query<FillerQuery>,
    State(app_state): State<AppState>,
) -> Result<Json<DiscoveryOrdersResponse>, StatusCode> {
    info!("Getting discovery orders for fillers");

    let mut sql_query = "SELECT * FROM orders WHERE status = $1".to_string();
    let mut params = vec![OrderStatus::Discovery as i32];
    
    if let Some(limit) = query.limit {
        sql_query.push_str(&format!(" LIMIT {}", limit.min(100))); // Cap at 100
    } else {
        sql_query.push_str(" LIMIT 20"); // Default limit
    }

    let rows = sqlx::query(&sql_query)
        .bind(params[0])
        .fetch_all(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching discovery orders: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let orders: Vec<OrderResponse> = rows.iter()
        .map(|row| OrderResponse {
            id: row.try_get("id").unwrap_or_default(),
            order_type: OrderType::from(row.try_get::<i32, _>("order_type").unwrap_or(0)),
            status: OrderStatus::from(row.try_get::<i32, _>("status").unwrap_or(0)),
            amount: row.try_get("amount").unwrap_or_default(),
            bank_account: row.try_get("bank_account").ok(),
            bank_service: row.try_get("bank_service").ok(),
            filler_id: row.try_get("filler_id").ok(),
            locked_amount: row.try_get("locked_amount").ok(),
            created_at: row.try_get("created_at").unwrap_or_default(),
        })
        .collect();

    let total = orders.len();
    
    info!("Found {} orders in discovery phase", total);
    Ok(Json(DiscoveryOrdersResponse { orders, total }))
}

/// Lock an order for filling (POST /fillers/orders/:id/lock)
pub async fn lock_order(
    Path(order_id): Path<String>,
    State(app_state): State<AppState>,
    Json(req): Json<LockOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    info!("Locking order {} for filler {}", order_id, req.filler_id);

    // Verify order exists and is in discovery phase
    let order_query = "SELECT id, order_type, status, from_address, to_address, token_id, amount, bank_account, bank_service, banking_hash, filler_id, locked_amount, batch_id, created_at, updated_at FROM orders WHERE id = $1 AND status = $2";
    let row = sqlx::query(order_query)
        .bind(&order_id)
        .bind(OrderStatus::Discovery as i32)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error checking order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let Some(row) = row else {
        warn!("Order not found or not available for locking: {}", order_id);
        return Err(StatusCode::NOT_FOUND);
    };

    // Parse order amount to validate lock amount
    let order_amount: u64 = row.try_get::<String, _>("amount")
        .unwrap_or_default()
        .parse()
        .map_err(|_| {
            error!("Invalid order amount format");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let lock_amount: u64 = req.amount.parse()
        .map_err(|_| {
            error!("Invalid lock amount format");
            StatusCode::BAD_REQUEST
        })?;

    if lock_amount > order_amount {
        warn!("Lock amount {} exceeds order amount {}", lock_amount, order_amount);
        return Err(StatusCode::BAD_REQUEST);
    }

    // Update order to locked status
    let update_query = r#"
        UPDATE orders 
        SET status = $1, filler_id = $2, locked_amount = $3, updated_at = $4
        WHERE id = $5 AND status = $6
    "#;
    
    let result = sqlx::query(update_query)
        .bind(OrderStatus::Locked as i32)
        .bind(&req.filler_id)
        .bind(&req.amount)
        .bind(chrono::Utc::now())
        .bind(&order_id)
        .bind(OrderStatus::Discovery as i32) // Ensure it's still in discovery
        .execute(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error locking order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;



    if result.rows_affected() == 0 {
        warn!("Order {} was already locked or changed status", order_id);
        return Err(StatusCode::CONFLICT);
    }

    // Fetch updated order
    let updated_row = sqlx::query("SELECT id, order_type, status, from_address, to_address, token_id, amount, bank_account, bank_service, banking_hash, filler_id, locked_amount, batch_id, created_at, updated_at FROM orders WHERE id = $1")
        .bind(&order_id)
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching updated order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let order_response = OrderResponse {
        id: updated_row.try_get("id").unwrap_or_default(),
        order_type: OrderType::from(updated_row.try_get::<i32, _>("order_type").unwrap_or(0)),
        status: OrderStatus::from(updated_row.try_get::<i32, _>("status").unwrap_or(0)),
        amount: updated_row.try_get("amount").unwrap_or_default(),
        bank_account: updated_row.try_get("bank_account").ok(),
        bank_service: updated_row.try_get("bank_service").ok(),
        filler_id: updated_row.try_get("filler_id").ok(),
        locked_amount: updated_row.try_get("locked_amount").ok(),
        created_at: updated_row.try_get("created_at").unwrap_or_default(),
    };

    info!("Order {} successfully locked for filler {}", order_id, req.filler_id);
    Ok(Json(order_response))
}

/// Submit payment proof (POST /fillers/orders/:id/payment-proof)
pub async fn submit_payment_proof(
    Path(order_id): Path<String>,
    State(app_state): State<AppState>,
    Json(req): Json<SubmitPaymentProofRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    info!("Submitting payment proof for order {}", order_id);

    // Update order with payment proof
    let update_query = r#"
        UPDATE orders 
        SET status = $1, banking_hash = $2, updated_at = $3
        WHERE id = $4 AND status = $5
    "#;
    
    let result = sqlx::query(update_query)
        .bind(OrderStatus::MarkPaid as i32)
        .bind(&req.banking_hash)
        .bind(chrono::Utc::now())
        .bind(&order_id)
        .bind(OrderStatus::Locked as i32) // Must be locked to submit proof
        .execute(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error submitting payment proof: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        warn!("Order {} not found or not in locked status", order_id);
        return Err(StatusCode::NOT_FOUND);
    }

    // Fetch updated order
    let updated_row = sqlx::query("SELECT id, order_type, status, from_address, to_address, token_id, amount, bank_account, bank_service, banking_hash, filler_id, locked_amount, batch_id, created_at, updated_at FROM orders WHERE id = $1")
        .bind(&order_id)
        .fetch_one(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching updated order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let order_response = OrderResponse {
        id: updated_row.try_get("id").unwrap_or_default(),
        order_type: OrderType::from(updated_row.try_get::<i32, _>("order_type").unwrap_or(0)),
        status: OrderStatus::from(updated_row.try_get::<i32, _>("status").unwrap_or(0)),
        amount: updated_row.try_get("amount").unwrap_or_default(),
        bank_account: updated_row.try_get("bank_account").ok(),
        bank_service: updated_row.try_get("bank_service").ok(),
        filler_id: updated_row.try_get("filler_id").ok(),
        locked_amount: updated_row.try_get("locked_amount").ok(),
        created_at: updated_row.try_get("created_at").unwrap_or_default(),
    };

    info!("Payment proof submitted for order {}", order_id);
    Ok(Json(order_response))
}

/// Get filler balance (GET /fillers/:filler_id/balance)
pub async fn get_filler_balance_api(
    Path(filler_id): Path<String>,
    State(_app_state): State<AppState>,
) -> Result<Json<FillerBalance>, StatusCode> {
    info!("Getting balance for filler {}", filler_id);

    // TODO: Implement actual database lookup once import issue is resolved
    // For now, return a mock balance with some realistic data
    let balance = FillerBalance {
        filler_id: filler_id.clone(),
        total_balance: "150000000000000000000000".to_string(), // 150k USDT
        available_balance: "120000000000000000000000".to_string(), // 120k USDT available
        locked_balance: "30000000000000000000000".to_string(), // 30k USDT locked
        completed_jobs: 2,
        wallets: vec![
            crate::models::FillerWallet {
                address: "0X8aj81j2gasjd81as...".to_string(),
                balance: "96000000000000000000000".to_string(), // 96k USDT (32% of total)
                percentage: 32.0,
            },
            crate::models::FillerWallet {
                address: "0X8aj81j2gasjd81as...".to_string(),
                balance: "54000000000000000000000".to_string(), // 54k USDT (68% of total)  
                percentage: 68.0,
            },
        ],
    };

    info!("Returning mock balance for filler {}: total={}, available={}", 
          filler_id, balance.total_balance, balance.available_balance);
    Ok(Json(balance))
}

/// Add wallet to filler (POST /fillers/:filler_id/wallets)
#[derive(Debug, Deserialize)]
pub struct AddWalletRequest {
    pub wallet_address: String,
    pub balance: Option<String>,
}

pub async fn add_wallet_to_filler(
    Path(filler_id): Path<String>,
    State(_app_state): State<AppState>,
    Json(req): Json<AddWalletRequest>,
) -> Result<Json<FillerBalance>, StatusCode> {
    info!("Adding wallet {} to filler {}", req.wallet_address, filler_id);

    // TODO: Implement actual database storage once import issue is resolved
    // For now, return a mock updated balance
    let updated_balance = FillerBalance {
        filler_id: filler_id.clone(),
        total_balance: "150000000000000000000000".to_string(),
        available_balance: "120000000000000000000000".to_string(),
        locked_balance: "30000000000000000000000".to_string(),
        completed_jobs: 2,
        wallets: vec![
            crate::models::FillerWallet {
                address: req.wallet_address.clone(),
                balance: req.balance.unwrap_or_else(|| "0".to_string()),
                percentage: 0.0,
            },
        ],
    };

    info!("Mock: Added wallet {} to filler {}", req.wallet_address, filler_id);
    Ok(Json(updated_balance))
}

/// Claim tokens from multiple wallets (POST /fillers/claim)
pub async fn claim_tokens(
    State(_app_state): State<AppState>,
    Json(req): Json<ClaimRequest>,
) -> Result<Json<ClaimResponse>, StatusCode> {
    info!("Processing claim request for filler {} with {} claims", 
          req.filler_id, req.claims.len());

    // TODO: Implement actual validation and database operations once import issue is resolved
    let mut processed_claims = Vec::new();
    let mut total_claimed = 0u64;

    for claim in &req.claims {
        let claim_amount: u64 = claim.amount.parse().map_err(|_| {
            error!("Invalid claim amount: {}", claim.amount);
            StatusCode::BAD_REQUEST
        })?;

        // Create bridge-out order for this claim
        let bridge_out_order = create_bridge_out_order(
            &claim.wallet_address,
            &claim.destination_address,
            &claim.amount,
        );

        // Generate merkle proof (this would integrate with the actual merkle tree)
        let merkle_proof = generate_mock_merkle_proof(&bridge_out_order);

        processed_claims.push(ProcessedClaim {
            wallet_address: claim.wallet_address.clone(),
            amount: claim.amount.clone(),
            destination_address: claim.destination_address.clone(),
            merkle_proof,
            success: true,
            error: None,
        });

        total_claimed += claim_amount;
    }

    // TODO: Submit batch claim to smart contract
    // This would involve calling the smart contract's batch claim function
    let transaction_hash = submit_batch_claim_to_contract(&processed_claims).await;

    let response = ClaimResponse {
        transaction_hash,
        batch_id: 1, // TODO: Use actual batch ID from blockchain
        total_claimed: total_claimed.to_string(),
        claims_processed: processed_claims,
    };

    info!("Mock: Processed {} claims for filler {}, total claimed: {}", 
          req.claims.len(), req.filler_id, total_claimed);

    Ok(Json(response))
}

/// Helper function to create a bridge-out order
fn create_bridge_out_order(
    from_address: &str, 
    to_address: &str, 
    amount: &str
) -> BridgeOutOrder {
    BridgeOutOrder {
        from_address: from_address.to_string(),
        to_address: to_address.to_string(),
        amount: amount.to_string(),
        token_id: 1, // USDC
    }
}

/// Temporary struct for bridge-out orders
#[derive(Debug)]
struct BridgeOutOrder {
    from_address: String,
    to_address: String,
    amount: String,
    token_id: u32,
}

/// Generate mock merkle proof for testing
fn generate_mock_merkle_proof(order: &BridgeOutOrder) -> Vec<String> {
    // This is a mock implementation
    // In reality, this would generate the actual merkle proof
    vec![
        format!("0x{:064x}", 1), // Mock proof element 1
        format!("0x{:064x}", 2), // Mock proof element 2
        format!("0x{:064x}", 3), // Mock proof element 3
    ]
}

/// Submit batch claim to smart contract (mock implementation)
async fn submit_batch_claim_to_contract(claims: &[ProcessedClaim]) -> Option<String> {
    // This is a mock implementation
    // In reality, this would:
    // 1. Create the batch claim transaction
    // 2. Generate the merkle root for all claims
    // 3. Submit to the smart contract
    // 4. Return the transaction hash
    
    info!("Mock: Submitting {} claims to smart contract", claims.len());
    Some("0x1234567890abcdef1234567890abcdef12345678".to_string())
}
