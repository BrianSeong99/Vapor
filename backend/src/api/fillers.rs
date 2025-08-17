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
    LockOrderRequest, SubmitPaymentProofRequest
};

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
    let order_query = "SELECT * FROM orders WHERE id = $1 AND status = $2";
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
    let updated_row = sqlx::query("SELECT * FROM orders WHERE id = $1")
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
    let updated_row = sqlx::query("SELECT * FROM orders WHERE id = $1")
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
