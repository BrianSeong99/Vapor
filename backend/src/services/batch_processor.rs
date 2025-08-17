use crate::models::{Order, AccountState};
use crate::merkle::MerkleTreeManager;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use chrono::{DateTime, Utc};

/// Batch processor for collecting orders and generating Merkle proofs
/// Handles the transition from one state to the next via batched operations
pub struct BatchProcessor {
    /// Merkle tree manager for state and order trees
    pub tree_manager: MerkleTreeManager,
    /// Current batch being processed
    pub current_batch: Option<Batch>,
    /// Next batch ID to assign
    pub next_batch_id: u32,
    /// Account states (address -> AccountState)
    pub accounts: HashMap<String, AccountState>,
}

/// A batch represents a collection of orders that transition the system state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    pub batch_id: u32,
    pub prev_batch_id: u32,
    pub prev_state_root: String,
    pub prev_orders_root: String,
    pub orders: Vec<Order>,
    pub new_state_root: String,
    pub new_orders_root: String,
    pub created_at: DateTime<Utc>,
    pub is_finalized: bool,
}

/// Result of batch processing
#[derive(Debug, Serialize)]
pub struct BatchResult {
    pub batch_id: u32,
    pub orders_count: usize,
    pub prev_state_root: String,
    pub new_state_root: String,
    pub prev_orders_root: String,
    pub new_orders_root: String,
    pub ready_for_proof: bool,
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            tree_manager: MerkleTreeManager::new(),
            current_batch: None,
            next_batch_id: 1,
            accounts: HashMap::new(),
        }
    }

    /// Start a new batch
    pub fn start_batch(&mut self) -> Result<u32> {
        if self.current_batch.is_some() {
            return Err(anyhow::anyhow!("Batch already in progress"));
        }

        let batch_id = self.next_batch_id;
        let prev_batch_id = batch_id - 1;
        
        // Get previous roots (empty for genesis batch)
        let prev_state_root = if batch_id == 1 {
            MerkleTreeManager::empty_state_root()
        } else {
            self.tree_manager.get_state_root()?
        };
        
        let prev_orders_root = if batch_id == 1 {
            MerkleTreeManager::empty_orders_root()
        } else {
            self.tree_manager.get_orders_root()?
        };

        let batch = Batch {
            batch_id,
            prev_batch_id,
            prev_state_root,
            prev_orders_root,
            orders: Vec::new(),
            new_state_root: String::new(), // Will be computed when finalized
            new_orders_root: String::new(), // Will be computed when finalized
            created_at: Utc::now(),
            is_finalized: false,
        };

        self.current_batch = Some(batch);
        self.next_batch_id += 1;

        info!("Started batch {}", batch_id);
        Ok(batch_id)
    }

    /// Add an order to the current batch
    pub fn add_order_to_batch(&mut self, order: Order) -> Result<()> {
        // Apply order to account states first
        self.apply_order_to_state(&order)?;
        
        // Then add to batch
        if let Some(batch) = self.current_batch.as_mut() {
            batch.orders.push(order.clone());
            info!("Added order {} to batch {}", order.id, batch.batch_id);
        } else {
            return Err(anyhow::anyhow!("No active batch"));
        }
        
        Ok(())
    }

    /// Finalize the current batch and compute new roots
    pub fn finalize_batch(&mut self) -> Result<BatchResult> {
        let mut batch = self.current_batch.take()
            .ok_or_else(|| anyhow::anyhow!("No active batch to finalize"))?;

        if batch.orders.is_empty() {
            warn!("Finalizing empty batch {}", batch.batch_id);
        }

        // Build new state tree from current accounts
        let accounts: Vec<AccountState> = self.accounts.values().cloned().collect();
        batch.new_state_root = self.tree_manager.build_state_tree(&accounts)?;

        // Build new orders tree
        batch.new_orders_root = self.tree_manager.build_orders_tree(&batch.orders, batch.batch_id)?;

        batch.is_finalized = true;

        let result = BatchResult {
            batch_id: batch.batch_id,
            orders_count: batch.orders.len(),
            prev_state_root: batch.prev_state_root.clone(),
            new_state_root: batch.new_state_root.clone(),
            prev_orders_root: batch.prev_orders_root.clone(),
            new_orders_root: batch.new_orders_root.clone(),
            ready_for_proof: true,
        };

        info!("Finalized batch {} with {} orders", batch.batch_id, batch.orders.len());
        info!("State root: {} -> {}", batch.prev_state_root, batch.new_state_root);
        info!("Orders root: {} -> {}", batch.prev_orders_root, batch.new_orders_root);

        // Store the finalized batch (could be saved to database here)
        // For now, we'll just log it
        
        Ok(result)
    }

    /// Apply an order's effects to account states
    fn apply_order_to_state(&mut self, order: &Order) -> Result<()> {
        use crate::models::OrderType;

        match order.order_type {
            OrderType::BridgeIn => {
                // Credit the account with deposited amount
                if let Some(to_addr) = &order.to_address {
                    self.credit_account(to_addr, order.token_id, &order.amount)?;
                    info!("BridgeIn: Credited {} {} to {}", order.amount, order.token_id, to_addr);
                }
            },
            
            OrderType::Transfer => {
                // Transfer from one account to another
                if let (Some(from_addr), Some(to_addr)) = (&order.from_address, &order.to_address) {
                    self.debit_account(from_addr, order.token_id, &order.amount)?;
                    self.credit_account(to_addr, order.token_id, &order.amount)?;
                    info!("Transfer: Moved {} {} from {} to {}", 
                        order.amount, order.token_id, from_addr, to_addr);
                }
            },
            
            OrderType::BridgeOut => {
                // Debit the account for withdrawal
                if let Some(from_addr) = &order.from_address {
                    self.debit_account(from_addr, order.token_id, &order.amount)?;
                    info!("BridgeOut: Debited {} {} from {}", order.amount, order.token_id, from_addr);
                }
            },
        }

        Ok(())
    }

    /// Credit an account with tokens
    fn credit_account(&mut self, address: &str, token_id: u32, amount: &str) -> Result<()> {
        let amount_value: u64 = amount.parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount: {}", amount))?;

        let account = self.accounts.entry(address.to_string())
            .or_insert_with(|| AccountState {
                address: address.to_string(),
                balances: Vec::new(),
                updated_at: Utc::now(),
            });

        // Find existing balance or create new one
        if let Some(balance) = account.balances.iter_mut().find(|b| b.token_id == token_id) {
            let current: u64 = balance.balance.parse().unwrap_or(0);
            balance.balance = (current + amount_value).to_string();
        } else {
            account.balances.push(crate::models::TokenBalance {
                token_id,
                balance: amount.to_string(),
            });
        }

        // Update timestamp
        account.updated_at = Utc::now();

        Ok(())
    }

    /// Debit an account
    fn debit_account(&mut self, address: &str, token_id: u32, amount: &str) -> Result<()> {
        let amount_value: u64 = amount.parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount: {}", amount))?;

        let account = self.accounts.get_mut(address)
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", address))?;

        // Find the balance
        let balance = account.balances.iter_mut()
            .find(|b| b.token_id == token_id)
            .ok_or_else(|| anyhow::anyhow!("Token balance not found: {} for {}", token_id, address))?;

        let current: u64 = balance.balance.parse().unwrap_or(0);
        if current < amount_value {
            return Err(anyhow::anyhow!("Insufficient balance: {} < {}", current, amount_value));
        }

        balance.balance = (current - amount_value).to_string();
        
        // Update timestamp
        account.updated_at = Utc::now();
        
        Ok(())
    }

    /// Get current batch info
    pub fn get_current_batch(&self) -> Option<&Batch> {
        self.current_batch.as_ref()
    }

    /// Get batch statistics
    pub fn get_stats(&self) -> BatchStats {
        BatchStats {
            next_batch_id: self.next_batch_id,
            current_batch_orders: self.current_batch.as_ref()
                .map(|b| b.orders.len())
                .unwrap_or(0),
            total_accounts: self.accounts.len(),
            has_active_batch: self.current_batch.is_some(),
        }
    }

    /// Initialize account (for testing/setup)
    pub fn init_account(&mut self, address: String, token_id: u32, initial_balance: String) -> Result<()> {
        let account = self.accounts.entry(address.clone())
            .or_insert_with(|| AccountState {
                address: address.clone(),
                balances: Vec::new(),
                updated_at: Utc::now(),
            });

        account.balances.push(crate::models::TokenBalance {
            token_id,
            balance: initial_balance.clone(),
        });

        info!("Initialized account {} with {} of token {}", address, initial_balance, token_id);
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct BatchStats {
    pub next_batch_id: u32,
    pub current_batch_orders: usize,
    pub total_accounts: usize,
    pub has_active_batch: bool,
}