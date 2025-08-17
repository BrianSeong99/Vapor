use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Order {
    pub id: String,
    pub order_type: OrderType,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_id: u32,
    pub amount: String,
    pub banking_hash: Option<String>,
    pub status: OrderStatus,
    pub batch_id: Option<u32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum OrderType {
    BridgeIn = 0,
    BridgeOut = 1,
    Transfer = 2,
}

impl From<i32> for OrderType {
    fn from(value: i32) -> Self {
        match value {
            0 => OrderType::BridgeIn,
            1 => OrderType::BridgeOut,
            2 => OrderType::Transfer,
            _ => OrderType::BridgeIn, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum OrderStatus {
    Pending = 0,
    Locked = 1,
    MarkPaid = 2,
    Settled = 3,
    Failed = 4,
}

impl From<i32> for OrderStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => OrderStatus::Pending,
            1 => OrderStatus::Locked,
            2 => OrderStatus::MarkPaid,
            3 => OrderStatus::Settled,
            4 => OrderStatus::Failed,
            _ => OrderStatus::Pending, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub id: u32,
    pub prev_state_root: String,
    pub prev_orders_root: String,
    pub new_state_root: String,
    pub new_orders_root: String,
    pub proof_data: Option<String>,
    pub status: BatchStatus,
    pub created_at: DateTime<Utc>,
    pub submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(i32)]
pub enum BatchStatus {
    Building = 0,
    Proving = 1,
    Submitting = 2,
    Submitted = 3,
    Failed = 4,
}

impl From<i32> for BatchStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => BatchStatus::Building,
            1 => BatchStatus::Proving,
            2 => BatchStatus::Submitting,
            3 => BatchStatus::Submitted,
            4 => BatchStatus::Failed,
            _ => BatchStatus::Building, // Default fallback
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token_id: u32,
    pub balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    pub address: String,
    pub balances: Vec<TokenBalance>, // Array-based dictionary of token balances
    pub updated_at: DateTime<Utc>,
}

// API request/response types
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub order_type: OrderType,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
    pub token_id: u32,
    pub amount: String,
    pub banking_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub amount: String,
    pub created_at: DateTime<Utc>,
}

impl Order {
    pub fn new(req: CreateOrderRequest) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            order_type: req.order_type,
            from_address: req.from_address,
            to_address: req.to_address,
            token_id: req.token_id,
            amount: req.amount,
            banking_hash: req.banking_hash,
            status: OrderStatus::Pending,
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
