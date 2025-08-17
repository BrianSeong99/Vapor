use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::Utc;
use sqlx::Row;

use super::AppState;
use crate::models::{CreateOrderRequest, OrderResponse, OrderStatusResponse, Order, OrderType, OrderStatus};

#[derive(Debug, Deserialize)]
pub struct OrderQuery {
    pub status: Option<String>,
    pub order_type: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct OrdersListResponse {
    pub orders: Vec<OrderResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct MatchResponse {
    pub order_id: String,
    pub filler_id: String,
    pub amount_usd: u64,
    pub locked_until: String,
}

/// Create a new order (BridgeIn/Transfer/BridgeOut)
pub async fn create_order(
    State(app_state): State<AppState>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    info!("Creating order: {:?}", req);
    
    // Create new order
    let order = Order::new(req);
    
    // Save to database (simplified for MVP)
    let query = r#"
        INSERT INTO orders (id, order_type, status, from_address, to_address, token_id, amount, banking_hash, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
    "#;
    
    let result = sqlx::query(query)
        .bind(&order.id)
        .bind(order.order_type as i32)
        .bind(order.status as i32)
        .bind(&order.from_address)
        .bind(&order.to_address)
        .bind(order.token_id as i32)
        .bind(&order.amount)
        .bind(&order.banking_hash)
        .bind(order.created_at)
        .bind(order.updated_at)
        .execute(&app_state.db)
        .await;

    match result {
        Ok(_) => {
            info!("Order saved to database: {}", order.id);
            
            // Process order based on type
            match order.order_type {
                OrderType::BridgeIn => {
                    // Add to matching engine for P2P matching
                    let mut engine = app_state.matching_engine.lock().await;
                    if let Err(e) = engine.add_order(order.clone()) {
                        error!("Failed to add order to matching engine: {}", e);
                    } else {
                        info!("Order added to matching engine: {}", order.id);
                    }
                }
                OrderType::Transfer | OrderType::BridgeOut => {
                    // Add directly to batch processor
                    let mut processor = app_state.batch_processor.lock().await;
                    
                    // Start batch if none exists
                    if processor.get_current_batch().is_none() {
                        if let Err(e) = processor.start_batch() {
                            error!("Failed to start batch: {}", e);
                        }
                    }
                    
                    if let Err(e) = processor.add_order_to_batch(order.clone()) {
                        error!("Failed to add order to batch: {}", e);
                    } else {
                        info!("Order added to batch: {}", order.id);
                    }
                }
            }
            
            let response = OrderResponse::from(&order);
            
            info!("Order created successfully: {}", order.id);
            Ok(Json(response))
        }
        Err(e) => {
            error!("Database error creating order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get order status for tracking (GET /orders/:id/status)
pub async fn get_order_status(
    Path(order_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<OrderStatusResponse>, StatusCode> {
    info!("Getting order status for: {}", order_id);

    let query = "SELECT * FROM orders WHERE id = $1";
    let row = sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching order status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match row {
        Some(row) => {
            let order = Order {
                id: row.try_get("id").unwrap_or_default(),
                order_type: OrderType::from(row.try_get::<i32, _>("order_type").unwrap_or(0)),
                status: OrderStatus::from(row.try_get::<i32, _>("status").unwrap_or(0)),
                from_address: row.try_get("from_address").ok(),
                to_address: row.try_get("to_address").ok(),
                token_id: row.try_get::<i32, _>("token_id").unwrap_or(1) as u32,
                amount: row.try_get("amount").unwrap_or_default(),
                bank_account: row.try_get("bank_account").ok(),
                bank_service: row.try_get("bank_service").ok(),
                banking_hash: row.try_get("banking_hash").ok(),
                filler_id: row.try_get("filler_id").ok(),
                locked_amount: row.try_get("locked_amount").ok(),
                batch_id: row.try_get::<Option<i32>, _>("batch_id").unwrap_or(None).map(|id| id as u32),
                created_at: row.try_get("created_at").unwrap_or_default(),
                updated_at: row.try_get("updated_at").unwrap_or_default(),
            };
            
            let status_response = OrderStatusResponse::from(order);
            Ok(Json(status_response))
        }
        None => {
            warn!("Order not found for status: {}", order_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Mark an order as paid (triggers Transfer order creation)
pub async fn mark_paid(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Marking order as paid: {}", order_id);
    
    // Get order from database
    let query = "SELECT * FROM orders WHERE id = ?";
    let order_row = sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await;

    match order_row {
        Ok(Some(row)) => {
            // Update order status to MarkPaid
            let update_query = "UPDATE orders SET status = ?, updated_at = ? WHERE id = ?";
            sqlx::query(update_query)
                .bind(OrderStatus::MarkPaid as i32)
                .bind(Utc::now())
                .bind(&order_id)
                .execute(&app_state.db)
                .await
                .map_err(|e| {
                    error!("Failed to update order status: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Create Transfer order (seller → filler)
            let transfer_order = Order {
                id: Uuid::new_v4().to_string(),
                order_type: OrderType::Transfer,
                status: OrderStatus::Pending,
                from_address: row.try_get("to_address").ok(),
                to_address: Some("filler_address".to_string()), // TODO: Get from matching
                token_id: row.try_get::<i32, _>("token_id").unwrap_or(1) as u32,
                amount: row.try_get("amount").unwrap_or_default(),
                bank_account: None,
                bank_service: None,
                banking_hash: None,
                filler_id: None,
                locked_amount: None,
                batch_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // Add Transfer order to batch
            let mut processor = app_state.batch_processor.lock().await;
            if processor.get_current_batch().is_none() {
                processor.start_batch().map_err(|e| {
                    error!("Failed to start batch: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            }
            
            processor.add_order_to_batch(transfer_order.clone()).map_err(|e| {
                error!("Failed to add transfer order to batch: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            info!("Order marked as paid and transfer order created: {}", order_id);
            Ok(Json(serde_json::json!({
                "status": "success",
                "order_id": order_id,
                "transfer_order_id": transfer_order.id,
                "message": "Order marked as paid, transfer order created"
            })))
        }
        Ok(None) => {
            warn!("Order not found: {}", order_id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Database error fetching order: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get orders list with optional filtering
pub async fn list_orders(
    State(app_state): State<AppState>,
    Query(params): Query<OrderQuery>,
) -> Result<Json<OrdersListResponse>, StatusCode> {
    info!("Listing orders with params: {:?}", params);
    
    let mut query = "SELECT id, order_type, status, amount, created_at FROM orders".to_string();
    let mut conditions = Vec::new();
    
    // Add status filter
    if let Some(status) = &params.status {
        match status.as_str() {
            "pending" => conditions.push("status = 0"),
            "locked" => conditions.push("status = 1"),
            "mark_paid" => conditions.push("status = 2"),
            "settled" => conditions.push("status = 3"),
            "failed" => conditions.push("status = 4"),
            _ => {}
        }
    }
    
    // Add order type filter
    if let Some(order_type) = &params.order_type {
        match order_type.as_str() {
            "bridge_in" => conditions.push("order_type = 0"),
            "bridge_out" => conditions.push("order_type = 1"),
            "transfer" => conditions.push("order_type = 2"),
            _ => {}
        }
    }
    
    if !conditions.is_empty() {
        query.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
    }
    
    query.push_str(" ORDER BY created_at DESC");
    
    if let Some(limit) = params.limit {
        query.push_str(&format!(" LIMIT {}", limit.min(100))); // Cap at 100
    }
    
    let rows = sqlx::query(&query)
        .fetch_all(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error listing orders: {}", e);
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
    
    info!("Found {} orders", total);
    Ok(Json(OrdersListResponse { orders, total }))
}

/// Get specific order by ID
pub async fn get_order(
    State(app_state): State<AppState>,
    Path(order_id): Path<String>,
) -> Result<Json<OrderResponse>, StatusCode> {
    info!("Getting order: {}", order_id);
    
    let query = "SELECT id, order_type, status, amount, created_at FROM orders WHERE id = ?";
    let row = sqlx::query(query)
        .bind(&order_id)
        .fetch_optional(&app_state.db)
        .await
        .map_err(|e| {
            error!("Database error fetching order: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match row {
        Some(row) => {
            let order = OrderResponse {
                id: row.try_get("id").unwrap_or_default(),
                order_type: OrderType::from(row.try_get::<i32, _>("order_type").unwrap_or(0)),
                status: OrderStatus::from(row.try_get::<i32, _>("status").unwrap_or(0)),
                amount: row.try_get("amount").unwrap_or_default(),
                bank_account: row.try_get("bank_account").ok(),
                bank_service: row.try_get("bank_service").ok(),
                filler_id: row.try_get("filler_id").ok(),
                locked_amount: row.try_get("locked_amount").ok(),
                created_at: row.try_get("created_at").unwrap_or_default(),
            };
            
            Ok(Json(order))
        }
        None => {
            warn!("Order not found: {}", order_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Trigger order matching manually
pub async fn match_orders(
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Triggering order matching");
    
    let mut engine = app_state.matching_engine.lock().await;
    let matches = engine.match_orders().map_err(|e| {
        error!("Failed to match orders: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let match_responses: Vec<MatchResponse> = matches.iter()
        .map(|m| MatchResponse {
            order_id: m.order_id.clone(),
            filler_id: m.filler_id.clone(),
            amount_usd: m.amount_usd,
            locked_until: m.locked_until.to_rfc3339(),
        })
        .collect();

    info!("Matched {} orders", matches.len());
    Ok(Json(serde_json::json!({
        "status": "success",
        "matches": match_responses,
        "count": matches.len()
    })))
}