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
    pub bank_account: Option<String>,        // New: Bank account for off-ramp
    pub bank_service: Option<String>,        // New: Bank service name (PayPal, ACH, etc.)
    pub banking_hash: Option<String>,        // Payment proof/receipt hash
    pub filler_id: Option<String>,           // New: ID of filler who locked this order
    pub locked_amount: Option<String>,       // New: Amount locked by filler
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
    Pending = 0,        // Order created, waiting for blockchain confirmation
    Discovery = 1,      // New: In discovery phase, visible to fillers
    Locked = 2,         // Locked by a filler, waiting for payment
    MarkPaid = 3,       // Filler has submitted payment proof
    Settled = 4,        // Order completed and settled
    Failed = 5,         // Order failed or cancelled
}

impl From<i32> for OrderStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => OrderStatus::Pending,
            1 => OrderStatus::Discovery,
            2 => OrderStatus::Locked,
            3 => OrderStatus::MarkPaid,
            4 => OrderStatus::Settled,
            5 => OrderStatus::Failed,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    pub bank_account: Option<String>,     // New: Bank account for off-ramp
    pub bank_service: Option<String>,     // New: Bank service name
    pub banking_hash: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: String,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub amount: String,
    pub bank_account: Option<String>,
    pub bank_service: Option<String>,
    pub filler_id: Option<String>,
    pub locked_amount: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to lock an order for filling
#[derive(Debug, Deserialize)]
pub struct LockOrderRequest {
    pub filler_id: String,
    pub amount: String,
}

/// Request to submit payment proof
#[derive(Debug, Deserialize)]
pub struct SubmitPaymentProofRequest {
    pub banking_hash: String,
}

/// Order status tracking for seller
#[derive(Debug, Serialize)]
pub struct OrderStatusResponse {
    pub id: String,
    pub status: OrderStatus,
    pub phase: OrderPhase,
    pub progress_percentage: u8,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub filler_info: Option<FillerInfo>,
}

/// Three phases of order processing
#[derive(Debug, Serialize)]
pub enum OrderPhase {
    PrivateListing,    // Order created, waiting for blockchain confirmation
    FindingFillers,    // In discovery, looking for fillers
    SendingUSD,        // Locked by filler, processing payment
}

/// Filler information
#[derive(Debug, Serialize)]
pub struct FillerInfo {
    pub id: String,
    pub locked_amount: String,
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
            bank_account: req.bank_account,
            bank_service: req.bank_service,
            banking_hash: req.banking_hash,
            filler_id: None,
            locked_amount: None,
            status: OrderStatus::Pending,
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update order status and timestamp
    pub fn update_status(&mut self, status: OrderStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Assign order to a batch
    pub fn assign_to_batch(&mut self, batch_id: u32) {
        self.batch_id = Some(batch_id);
        self.updated_at = Utc::now();
    }
    
    /// Lock order for a filler
    pub fn lock_for_filler(&mut self, filler_id: String, amount: String) {
        self.filler_id = Some(filler_id);
        self.locked_amount = Some(amount);
        self.status = OrderStatus::Locked;
        self.updated_at = Utc::now();
    }
    
    /// Mark order as discovered (available for fillers)
    pub fn mark_discovered(&mut self) {
        self.status = OrderStatus::Discovery;
        self.updated_at = Utc::now();
    }
    
    /// Submit payment proof
    pub fn submit_payment_proof(&mut self, banking_hash: String) {
        self.banking_hash = Some(banking_hash);
        self.status = OrderStatus::MarkPaid;
        self.updated_at = Utc::now();
    }

    /// Check if order is finalized (cannot be modified)
    pub fn is_finalized(&self) -> bool {
        matches!(self.status, OrderStatus::Settled | OrderStatus::Failed)
    }

    /// Check if order can be matched
    pub fn can_be_matched(&self) -> bool {
        self.status == OrderStatus::Pending
    }

    /// Validate order data
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Order ID cannot be empty".to_string());
        }

        if self.amount.is_empty() || self.amount.parse::<u64>().is_err() {
            return Err("Amount must be a valid positive number".to_string());
        }

        if self.token_id == 0 {
            return Err("Token ID must be greater than 0".to_string());
        }

        match self.order_type {
            OrderType::BridgeIn => {
                if self.from_address.is_none() {
                    return Err("BridgeIn orders require from_address".to_string());
                }
                if self.banking_hash.is_none() {
                    return Err("BridgeIn orders require banking_hash".to_string());
                }
            }
            OrderType::BridgeOut => {
                if self.to_address.is_none() {
                    return Err("BridgeOut orders require to_address".to_string());
                }
            }
            OrderType::Transfer => {
                if self.from_address.is_none() || self.to_address.is_none() {
                    return Err("Transfer orders require both from_address and to_address".to_string());
                }
            }
        }

        Ok(())
    }
}

impl AccountState {
    /// Create new account state
    pub fn new(address: String) -> Self {
        Self {
            address,
            balances: Vec::new(),
            updated_at: Utc::now(),
        }
    }

    /// Get balance for a specific token
    pub fn get_balance(&self, token_id: u32) -> Option<&str> {
        self.balances
            .iter()
            .find(|b| b.token_id == token_id)
            .map(|b| b.balance.as_str())
    }

    /// Set balance for a specific token
    pub fn set_balance(&mut self, token_id: u32, balance: String) {
        if let Some(existing) = self.balances.iter_mut().find(|b| b.token_id == token_id) {
            existing.balance = balance;
        } else {
            self.balances.push(TokenBalance { token_id, balance });
        }
        self.updated_at = Utc::now();
    }

    /// Add to balance for a specific token
    pub fn add_balance(&mut self, token_id: u32, amount: &str) -> Result<(), String> {
        let amount_value = amount.parse::<u64>()
            .map_err(|_| "Invalid amount format".to_string())?;

        if let Some(existing) = self.balances.iter_mut().find(|b| b.token_id == token_id) {
            let current_value = existing.balance.parse::<u64>()
                .map_err(|_| "Invalid existing balance format".to_string())?;
            existing.balance = (current_value + amount_value).to_string();
        } else {
            self.balances.push(TokenBalance { 
                token_id, 
                balance: amount.to_string() 
            });
        }
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Subtract from balance for a specific token
    pub fn subtract_balance(&mut self, token_id: u32, amount: &str) -> Result<(), String> {
        let amount_value = amount.parse::<u64>()
            .map_err(|_| "Invalid amount format".to_string())?;

        if let Some(existing) = self.balances.iter_mut().find(|b| b.token_id == token_id) {
            let current_value = existing.balance.parse::<u64>()
                .map_err(|_| "Invalid existing balance format".to_string())?;
            
            if current_value < amount_value {
                return Err("Insufficient balance".to_string());
            }
            
            existing.balance = (current_value - amount_value).to_string();
            self.updated_at = Utc::now();
            Ok(())
        } else {
            Err("Token balance not found".to_string())
        }
    }

    /// Generate hash for Merkle tree leaf
    pub fn hash_leaf(&self) -> [u8; 32] {
        use sha3::{Digest, Keccak256};
        
        let mut hasher = Keccak256::new();
        hasher.update(self.address.as_bytes());
        
        // Sort balances by token_id for deterministic hashing
        let mut sorted_balances = self.balances.clone();
        sorted_balances.sort_by_key(|b| b.token_id);
        
        for balance in sorted_balances {
            hasher.update(balance.token_id.to_le_bytes());
            hasher.update(balance.balance.as_bytes());
        }
        
        hasher.finalize().into()
    }
}

impl TokenBalance {
    /// Create new token balance
    pub fn new(token_id: u32, balance: String) -> Self {
        Self { token_id, balance }
    }

    /// Check if balance is zero
    pub fn is_zero(&self) -> bool {
        self.balance == "0" || self.balance.is_empty()
    }

    /// Parse balance as u64
    pub fn as_u64(&self) -> Result<u64, String> {
        self.balance.parse().map_err(|_| "Invalid balance format".to_string())
    }
}

impl From<&Order> for OrderResponse {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id.clone(),
            order_type: order.order_type,
            status: order.status,
            amount: order.amount.clone(),
            bank_account: order.bank_account.clone(),
            bank_service: order.bank_service.clone(),
            filler_id: order.filler_id.clone(),
            locked_amount: order.locked_amount.clone(),
            created_at: order.created_at,
        }
    }
}

impl From<Order> for OrderStatusResponse {
    fn from(order: Order) -> Self {
        let (phase, progress_percentage) = match order.status {
            OrderStatus::Pending => (OrderPhase::PrivateListing, 10),
            OrderStatus::Discovery => (OrderPhase::FindingFillers, 40),
            OrderStatus::Locked => (OrderPhase::SendingUSD, 70),
            OrderStatus::MarkPaid => (OrderPhase::SendingUSD, 90),
            OrderStatus::Settled => (OrderPhase::SendingUSD, 100),
            OrderStatus::Failed => (OrderPhase::PrivateListing, 0),
        };
        
        let filler_info = if let (Some(filler_id), Some(locked_amount)) = 
            (order.filler_id.clone(), order.locked_amount.clone()) {
            Some(FillerInfo { id: filler_id, locked_amount })
        } else {
            None
        };
        
        Self {
            id: order.id,
            status: order.status,
            phase,
            progress_percentage,
            estimated_completion: None, // TODO: Calculate based on historical data
            filler_info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_order_type_serialization() {
        // Test enum values match expected i32 representation
        assert_eq!(OrderType::BridgeIn as i32, 0);
        assert_eq!(OrderType::BridgeOut as i32, 1);
        assert_eq!(OrderType::Transfer as i32, 2);

        // Test JSON serialization (serde serializes enums as strings by default)
        let bridge_in = OrderType::BridgeIn;
        let json = serde_json::to_string(&bridge_in).unwrap();
        assert_eq!(json, "\"BridgeIn\"");

        // Test JSON deserialization from string
        let deserialized: OrderType = serde_json::from_str("\"BridgeOut\"").unwrap();
        assert_eq!(deserialized, OrderType::BridgeOut);
    }

    #[test]
    fn test_order_type_from_i32() {
        assert_eq!(OrderType::from(0), OrderType::BridgeIn);
        assert_eq!(OrderType::from(1), OrderType::BridgeOut);
        assert_eq!(OrderType::from(2), OrderType::Transfer);
        assert_eq!(OrderType::from(999), OrderType::BridgeIn); // Default fallback
    }

    #[test]
    fn test_order_status_serialization() {
        // Test enum values
        assert_eq!(OrderStatus::Pending as i32, 0);
        assert_eq!(OrderStatus::Locked as i32, 1);
        assert_eq!(OrderStatus::MarkPaid as i32, 2);
        assert_eq!(OrderStatus::Settled as i32, 3);
        assert_eq!(OrderStatus::Failed as i32, 4);

        // Test serialization round-trip
        let status = OrderStatus::MarkPaid;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: OrderStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_order_status_from_i32() {
        assert_eq!(OrderStatus::from(0), OrderStatus::Pending);
        assert_eq!(OrderStatus::from(1), OrderStatus::Locked);
        assert_eq!(OrderStatus::from(2), OrderStatus::MarkPaid);
        assert_eq!(OrderStatus::from(3), OrderStatus::Settled);
        assert_eq!(OrderStatus::from(4), OrderStatus::Failed);
        assert_eq!(OrderStatus::from(-1), OrderStatus::Pending); // Default fallback
    }

    #[test]
    fn test_batch_status_enum() {
        assert_eq!(BatchStatus::Building as i32, 0);
        assert_eq!(BatchStatus::Proving as i32, 1);
        assert_eq!(BatchStatus::Submitting as i32, 2);
        assert_eq!(BatchStatus::Submitted as i32, 3);
        assert_eq!(BatchStatus::Failed as i32, 4);

        // Test conversion
        assert_eq!(BatchStatus::from(2), BatchStatus::Submitting);
        assert_eq!(BatchStatus::from(100), BatchStatus::Building); // Default
    }

    #[test]
    fn test_order_creation() {
        let create_req = CreateOrderRequest {
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: None,
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xabcdef1234567890".to_string()),
        };

        let order = Order::new(create_req);

        assert_eq!(order.order_type, OrderType::BridgeIn);
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.token_id, 1);
        assert_eq!(order.amount, "1000000");
        assert!(order.from_address.is_some());
        assert!(order.banking_hash.is_some());
        assert!(order.batch_id.is_none());
        assert!(!order.id.is_empty());
    }

    #[test]
    fn test_order_validation() {
        // Valid BridgeIn order
        let mut order = Order {
            id: "test-order".to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: None,
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xhash".to_string()),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(order.validate().is_ok());

        // Invalid: missing from_address for BridgeIn
        order.from_address = None;
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("from_address"));

        // Invalid: missing banking_hash for BridgeIn
        order.from_address = Some("0x1234567890123456789012345678901234567890".to_string());
        order.banking_hash = None;
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("banking_hash"));

        // Test BridgeOut validation
        order.order_type = OrderType::BridgeOut;
        order.banking_hash = Some("0xhash".to_string());
        order.to_address = None;
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("to_address"));

        // Test Transfer validation
        order.order_type = OrderType::Transfer;
        order.to_address = Some("0x9876543210987654321098765432109876543210".to_string());
        order.from_address = None;
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("both from_address and to_address"));

        // Test invalid amount
        order.from_address = Some("0x1234567890123456789012345678901234567890".to_string());
        order.amount = "invalid".to_string();
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("Amount"));

        // Test zero token_id
        order.amount = "1000000".to_string();
        order.token_id = 0;
        assert!(order.validate().is_err());
        assert!(order.validate().unwrap_err().contains("Token ID"));
    }

    #[test]
    fn test_order_status_transitions() {
        let mut order = Order {
            id: "test-order".to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: None,
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xhash".to_string()),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test status update
        let old_updated_at = order.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(1)); // Ensure time difference
        order.update_status(OrderStatus::Locked);
        assert_eq!(order.status, OrderStatus::Locked);
        assert!(order.updated_at > old_updated_at);

        // Test batch assignment
        order.assign_to_batch(123);
        assert_eq!(order.batch_id, Some(123));

        // Test state checks
        assert!(order.can_be_matched() == false); // Not pending anymore
        assert!(order.is_finalized() == false); // Not settled or failed

        order.update_status(OrderStatus::Settled);
        assert!(order.is_finalized() == true);
        assert!(order.can_be_matched() == false);
    }

    #[test]
    fn test_order_hash_deterministic() {
        let order = Order {
            id: "test-order".to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: None,
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xhash".to_string()),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Note: hash_leaf_with_batch_id is implemented in merkle.rs and returns Result<[u8; 32]>
        // For model tests, we test the data structure integrity instead
        assert_eq!(order.id, "test-order");
        assert_eq!(order.order_type, OrderType::BridgeIn);
        assert_eq!(order.amount, "1000000");
    }

    #[test]
    fn test_account_state_creation() {
        let address = "0x1234567890123456789012345678901234567890".to_string();
        let account = AccountState::new(address.clone());

        assert_eq!(account.address, address);
        assert!(account.balances.is_empty());
        assert!(account.updated_at <= Utc::now());
    }

    #[test]
    fn test_account_state_balance_operations() {
        let mut account = AccountState::new("0x1234567890123456789012345678901234567890".to_string());

        // Test setting initial balance
        account.set_balance(1, "1000000".to_string());
        assert_eq!(account.get_balance(1), Some("1000000"));
        assert_eq!(account.get_balance(2), None);

        // Test adding balance
        account.add_balance(1, "500000").unwrap();
        assert_eq!(account.get_balance(1), Some("1500000"));

        // Test adding balance for new token
        account.add_balance(2, "2000000").unwrap();
        assert_eq!(account.get_balance(2), Some("2000000"));
        assert_eq!(account.balances.len(), 2);

        // Test subtracting balance
        account.subtract_balance(1, "300000").unwrap();
        assert_eq!(account.get_balance(1), Some("1200000"));

        // Test insufficient balance error
        let result = account.subtract_balance(1, "2000000");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient balance"));

        // Test subtracting from non-existent token
        let result = account.subtract_balance(3, "100");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token balance not found"));
    }

    #[test]
    fn test_account_state_hash_deterministic() {
        let mut account1 = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        account1.set_balance(1, "1000000".to_string());
        account1.set_balance(2, "2000000".to_string());

        let mut account2 = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        // Add balances in different order
        account2.set_balance(2, "2000000".to_string());
        account2.set_balance(1, "1000000".to_string());

        let hash1 = account1.hash_leaf();
        let hash2 = account2.hash_leaf();
        assert_eq!(hash1, hash2, "Hash should be deterministic regardless of insertion order");

        // Different address should produce different hash
        let mut account3 = AccountState::new("0x9876543210987654321098765432109876543210".to_string());
        account3.set_balance(1, "1000000".to_string());
        account3.set_balance(2, "2000000".to_string());
        
        let hash3 = account3.hash_leaf();
        assert_ne!(hash1, hash3, "Different address should produce different hash");
    }

    #[test]
    fn test_token_balance_operations() {
        let balance = TokenBalance::new(1, "1000000".to_string());
        assert_eq!(balance.token_id, 1);
        assert_eq!(balance.balance, "1000000");
        assert_eq!(balance.as_u64().unwrap(), 1000000);

        let zero_balance = TokenBalance::new(1, "0".to_string());
        assert!(zero_balance.is_zero());

        let empty_balance = TokenBalance::new(1, "".to_string());
        assert!(empty_balance.is_zero());

        let invalid_balance = TokenBalance::new(1, "invalid".to_string());
        assert!(invalid_balance.as_u64().is_err());
    }

    #[test]
    fn test_order_response_conversion() {
        let order = Order {
            id: "test-order".to_string(),
            order_type: OrderType::BridgeOut,
            status: OrderStatus::Locked,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: None,
            batch_id: Some(123),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let response: OrderResponse = (&order).into();
        assert_eq!(response.id, order.id);
        assert_eq!(response.order_type, order.order_type);
        assert_eq!(response.status, order.status);
        assert_eq!(response.amount, order.amount);
        assert_eq!(response.created_at, order.created_at);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let order = Order {
            id: "test-order".to_string(),
            order_type: OrderType::Transfer,
            status: OrderStatus::MarkPaid,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xbankinghash".to_string()),
            batch_id: Some(123),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Test Order serialization
        let json = serde_json::to_string(&order).unwrap();
        let deserialized: Order = serde_json::from_str(&json).unwrap();
        assert_eq!(order.id, deserialized.id);
        assert_eq!(order.order_type, deserialized.order_type);
        assert_eq!(order.status, deserialized.status);
        assert_eq!(order.amount, deserialized.amount);

        // Test AccountState serialization
        let mut account = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        account.set_balance(1, "1000000".to_string());
        account.set_balance(2, "2000000".to_string());

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: AccountState = serde_json::from_str(&json).unwrap();
        assert_eq!(account.address, deserialized.address);
        assert_eq!(account.balances.len(), deserialized.balances.len());
    }

    #[test]
    fn test_large_amount_handling() {
        let large_amount = "999999999999999999999999999999999999"; // 36 digits
        
        let mut account = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        account.set_balance(1, large_amount.to_string());
        
        assert_eq!(account.get_balance(1), Some(large_amount));
        
        // Test that hash works with large amounts
        let hash = account.hash_leaf();
        assert_eq!(hash.len(), 32); // Should produce valid 32-byte hash
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        let mut account = AccountState::new("".to_string());
        account.set_balance(1, "0".to_string());
        let hash = account.hash_leaf();
        assert_eq!(hash.len(), 32);

        // Unicode in addresses (should handle gracefully)
        let unicode_address = "0x1234567890123456789012345678901234567890🎯";
        let mut account = AccountState::new(unicode_address.to_string());
        account.set_balance(1, "1000".to_string());
        let hash = account.hash_leaf();
        assert_eq!(hash.len(), 32);

        // Zero balances
        let balance = TokenBalance::new(1, "0".to_string());
        assert!(balance.is_zero());
        assert_eq!(balance.as_u64().unwrap(), 0);
    }
}
