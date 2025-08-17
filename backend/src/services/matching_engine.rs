use crate::models::{Order, OrderType};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use tracing::info;
use chrono::{DateTime, Utc};
use serde::Serialize;

/// Simple P2P Offramp Matching Engine
/// FIFO order matching with basic filler capacity management
pub struct MatchingEngine {
    /// FIFO queue of sell orders waiting for fillers
    pub pending_orders: VecDeque<Order>,
    /// Available fillers by ID
    pub fillers: HashMap<String, Filler>,
}

/// Simplified filler info
#[derive(Debug, Clone)]
pub struct Filler {
    pub id: String,
    pub address: String,
    pub capacity_usd: u64,      // How much USD they can provide
    pub is_active: bool,
}

/// Simple match result
#[derive(Debug, Clone, Serialize)]
pub struct MatchResult {
    pub order_id: String,
    pub filler_id: String,
    pub amount_usd: u64,
    pub locked_until: DateTime<Utc>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            pending_orders: VecDeque::new(),
            fillers: HashMap::new(),
        }
    }

    /// Add a filler to the system
    pub fn add_filler(&mut self, id: String, address: String, capacity_usd: u64) -> Result<()> {
        let filler = Filler {
            id: id.clone(),
            address,
            capacity_usd,
            is_active: true,
        };
        
        self.fillers.insert(id.clone(), filler);
        info!("Added filler {} with ${} capacity", id, capacity_usd);
        Ok(())
    }

    /// Remove a filler
    pub fn remove_filler(&mut self, filler_id: &str) -> Result<()> {
        self.fillers.remove(filler_id);
        info!("Removed filler {}", filler_id);
        Ok(())
    }

    /// Add a sell order to the queue
    pub fn add_order(&mut self, order: Order) -> Result<()> {
        if order.order_type != OrderType::BridgeIn {
            return Err(anyhow::anyhow!("Only BridgeIn orders supported"));
        }

        self.pending_orders.push_back(order.clone());
        info!("Added order {} for ${} to queue", order.id, order.amount);
        Ok(())
    }

    /// Match orders with fillers (FIFO)
    pub fn match_orders(&mut self) -> Result<Vec<MatchResult>> {
        let mut matches = Vec::new();

        // Process orders in FIFO order
        while let Some(order) = self.pending_orders.front() {
            let order_amount: u64 = order.amount.parse().unwrap_or(0);
            
            // Find any active filler with enough capacity
            let mut matched_filler = None;
            for filler in self.fillers.values_mut() {
                if filler.is_active && filler.capacity_usd >= order_amount {
                    matched_filler = Some(filler.id.clone());
                    filler.capacity_usd -= order_amount; // Reduce capacity
                    break;
                }
            }

            if let Some(filler_id) = matched_filler {
                let order = self.pending_orders.pop_front().unwrap();
                let lock_until = Utc::now() + chrono::Duration::minutes(30); // 30 min lock
                
                let match_result = MatchResult {
                    order_id: order.id.clone(),
                    filler_id: filler_id.clone(),
                    amount_usd: order_amount,
                    locked_until: lock_until,
                };

                info!("Matched order {} with filler {} for ${}", 
                    order.id, filler_id, order_amount);
                
                matches.push(match_result);
            } else {
                // No filler available, stop processing
                break;
            }
        }

        Ok(matches)
    }

    /// Get simple stats
    pub fn get_stats(&self) -> MatchingStats {
        MatchingStats {
            pending_orders: self.pending_orders.len(),
            active_fillers: self.fillers.values().filter(|f| f.is_active).count(),
            total_capacity: self.fillers.values()
                .filter(|f| f.is_active)
                .map(|f| f.capacity_usd)
                .sum(),
        }
    }

    /// Release a locked order back to queue (if payment fails)
    pub fn release_order(&mut self, order_id: &str, filler_id: &str, amount: u64) -> Result<()> {
        // Restore filler capacity
        if let Some(filler) = self.fillers.get_mut(filler_id) {
            filler.capacity_usd += amount;
            info!("Released order {} and restored ${} to filler {}", 
                order_id, amount, filler_id);
        }
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct MatchingStats {
    pub pending_orders: usize,
    pub active_fillers: usize,
    pub total_capacity: u64,
}