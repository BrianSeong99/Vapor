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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Order, OrderType, OrderStatus};
    use chrono::Utc;

    fn create_test_order(id: &str, amount: u64) -> Order {
        Order {
            id: id.to_string(),
            order_type: OrderType::BridgeIn,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1, // USDC token ID
            amount: amount.to_string(),
            banking_hash: None,
            status: OrderStatus::Pending,
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_matching_engine_creation() {
        let engine = MatchingEngine::new();
        assert_eq!(engine.pending_orders.len(), 0);
        assert_eq!(engine.fillers.len(), 0);
        
        let stats = engine.get_stats();
        assert_eq!(stats.pending_orders, 0);
        assert_eq!(stats.active_fillers, 0);
        assert_eq!(stats.total_capacity, 0);
    }

    #[test]
    fn test_add_filler() {
        let mut engine = MatchingEngine::new();
        
        // Add a filler
        let result = engine.add_filler(
            "filler1".to_string(),
            "0x1111111111111111111111111111111111111111".to_string(),
            1000
        );
        assert!(result.is_ok());
        
        // Check filler was added
        assert_eq!(engine.fillers.len(), 1);
        let filler = engine.fillers.get("filler1").unwrap();
        assert_eq!(filler.id, "filler1");
        assert_eq!(filler.capacity_usd, 1000);
        assert!(filler.is_active);
        
        // Check stats
        let stats = engine.get_stats();
        assert_eq!(stats.active_fillers, 1);
        assert_eq!(stats.total_capacity, 1000);
    }

    #[test]
    fn test_remove_filler() {
        let mut engine = MatchingEngine::new();
        
        // Add and then remove a filler
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        assert_eq!(engine.fillers.len(), 1);
        
        let result = engine.remove_filler("filler1");
        assert!(result.is_ok());
        assert_eq!(engine.fillers.len(), 0);
        
        // Check stats
        let stats = engine.get_stats();
        assert_eq!(stats.active_fillers, 0);
        assert_eq!(stats.total_capacity, 0);
    }

    #[test]
    fn test_add_order_valid() {
        let mut engine = MatchingEngine::new();
        
        let order = create_test_order("order1", 100);
        let result = engine.add_order(order);
        assert!(result.is_ok());
        
        assert_eq!(engine.pending_orders.len(), 1);
        let stats = engine.get_stats();
        assert_eq!(stats.pending_orders, 1);
    }

    #[test]
    fn test_add_order_invalid_type() {
        let mut engine = MatchingEngine::new();
        
        let mut order = create_test_order("order1", 100);
        order.order_type = OrderType::BridgeOut; // Invalid type
        
        let result = engine.add_order(order);
        assert!(result.is_err());
        assert_eq!(engine.pending_orders.len(), 0);
    }

    #[test]
    fn test_simple_match() {
        let mut engine = MatchingEngine::new();
        
        // Add a filler with sufficient capacity
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        
        // Add an order
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        // Match orders
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 1);
        
        let match_result = &matches[0];
        assert_eq!(match_result.order_id, "order1");
        assert_eq!(match_result.filler_id, "filler1");
        assert_eq!(match_result.amount_usd, 100);
        
        // Check order was removed from queue
        assert_eq!(engine.pending_orders.len(), 0);
        
        // Check filler capacity was reduced
        let filler = engine.fillers.get("filler1").unwrap();
        assert_eq!(filler.capacity_usd, 900); // 1000 - 100
    }

    #[test]
    fn test_no_match_insufficient_capacity() {
        let mut engine = MatchingEngine::new();
        
        // Add a filler with insufficient capacity
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 50).unwrap();
        
        // Add an order that exceeds capacity
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        // Try to match - should not match
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 0);
        
        // Order should still be in queue
        assert_eq!(engine.pending_orders.len(), 1);
        
        // Filler capacity should be unchanged
        let filler = engine.fillers.get("filler1").unwrap();
        assert_eq!(filler.capacity_usd, 50);
    }

    #[test]
    fn test_no_match_no_fillers() {
        let mut engine = MatchingEngine::new();
        
        // Add an order but no fillers
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        // Try to match - should not match
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 0);
        
        // Order should still be in queue
        assert_eq!(engine.pending_orders.len(), 1);
    }

    #[test]
    fn test_fifo_order_processing() {
        let mut engine = MatchingEngine::new();
        
        // Add a filler with capacity for only one order
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 100).unwrap();
        
        // Add multiple orders
        let order1 = create_test_order("order1", 100);
        let order2 = create_test_order("order2", 100);
        engine.add_order(order1).unwrap();
        engine.add_order(order2).unwrap();
        
        // Match orders - only first order should match
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].order_id, "order1"); // First order matched
        
        // Second order should still be in queue
        assert_eq!(engine.pending_orders.len(), 1);
        assert_eq!(engine.pending_orders.front().unwrap().id, "order2");
    }

    #[test]
    fn test_multiple_fillers() {
        let mut engine = MatchingEngine::new();
        
        // Add multiple fillers
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 100).unwrap();
        engine.add_filler("filler2".to_string(), "0x2222".to_string(), 200).unwrap();
        
        // Add multiple orders
        let order1 = create_test_order("order1", 100);
        let order2 = create_test_order("order2", 150);
        engine.add_order(order1).unwrap();
        engine.add_order(order2).unwrap();
        
        // Match orders
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 2);
        
        // First order should match with first available filler
        assert_eq!(matches[0].order_id, "order1");
        assert_eq!(matches[0].filler_id, "filler1");
        
        // Second order should match with second filler (first has insufficient capacity)
        assert_eq!(matches[1].order_id, "order2");
        assert_eq!(matches[1].filler_id, "filler2");
        
        // Check capacities
        assert_eq!(engine.fillers.get("filler1").unwrap().capacity_usd, 0); // 100 - 100
        assert_eq!(engine.fillers.get("filler2").unwrap().capacity_usd, 50); // 200 - 150
    }

    #[test]
    fn test_inactive_filler() {
        let mut engine = MatchingEngine::new();
        
        // Add a filler and make it inactive
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        engine.fillers.get_mut("filler1").unwrap().is_active = false;
        
        // Add an order
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        // Try to match - should not match with inactive filler
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 0);
        
        // Order should still be in queue
        assert_eq!(engine.pending_orders.len(), 1);
        
        // Stats should reflect inactive filler
        let stats = engine.get_stats();
        assert_eq!(stats.active_fillers, 0);
        assert_eq!(stats.total_capacity, 0);
    }

    #[test]
    fn test_release_order() {
        let mut engine = MatchingEngine::new();
        
        // Add filler and order
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        // Match order
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(engine.fillers.get("filler1").unwrap().capacity_usd, 900);
        
        // Release the order
        let result = engine.release_order("order1", "filler1", 100);
        assert!(result.is_ok());
        
        // Check capacity was restored
        assert_eq!(engine.fillers.get("filler1").unwrap().capacity_usd, 1000);
    }

    #[test]
    fn test_release_order_unknown_filler() {
        let mut engine = MatchingEngine::new();
        
        // Try to release order for non-existent filler
        let result = engine.release_order("order1", "unknown_filler", 100);
        assert!(result.is_ok()); // Should not error, just do nothing
    }

    #[test]
    fn test_match_result_lock_time() {
        let mut engine = MatchingEngine::new();
        
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        let order = create_test_order("order1", 100);
        engine.add_order(order).unwrap();
        
        let before_match = Utc::now();
        let matches = engine.match_orders().unwrap();
        let after_match = Utc::now();
        
        assert_eq!(matches.len(), 1);
        let match_result = &matches[0];
        
        // Check that lock time is approximately 30 minutes from now
        let lock_duration = match_result.locked_until - before_match;
        assert!(lock_duration.num_minutes() >= 29);
        assert!(lock_duration.num_minutes() <= 31);
        
        // Lock time should be after match time
        assert!(match_result.locked_until > after_match);
    }

    #[test]
    fn test_large_capacity_operations() {
        let mut engine = MatchingEngine::new();
        
        // Test with large numbers
        engine.add_filler("whale_filler".to_string(), "0x1111".to_string(), 1_000_000).unwrap();
        
        let order = create_test_order("large_order", 500_000);
        engine.add_order(order).unwrap();
        
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].amount_usd, 500_000);
        
        // Check remaining capacity
        assert_eq!(engine.fillers.get("whale_filler").unwrap().capacity_usd, 500_000);
    }

    #[test]
    fn test_zero_amount_order() {
        let mut engine = MatchingEngine::new();
        
        engine.add_filler("filler1".to_string(), "0x1111".to_string(), 1000).unwrap();
        
        let order = create_test_order("zero_order", 0);
        engine.add_order(order).unwrap();
        
        let matches = engine.match_orders().unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].amount_usd, 0);
        
        // Filler capacity should be unchanged
        assert_eq!(engine.fillers.get("filler1").unwrap().capacity_usd, 1000);
    }

    #[test]
    fn test_concurrent_operations_simulation() {
        let mut engine = MatchingEngine::new();
        
        // Simulate multiple fillers joining
        for i in 1..=5 {
            engine.add_filler(
                format!("filler{}", i),
                format!("0x{:040}", i),
                i * 100
            ).unwrap();
        }
        
        // Simulate orders coming in
        for i in 1..=10 {
            let order = create_test_order(&format!("order{}", i), i * 50);
            engine.add_order(order).unwrap();
        }
        
        // Process all matches
        let matches = engine.match_orders().unwrap();
        
        // Should match several orders based on capacity
        assert!(matches.len() > 0);
        assert!(matches.len() <= 10);
        
        // Verify no double-matching
        let mut matched_orders = std::collections::HashSet::new();
        for match_result in &matches {
            assert!(matched_orders.insert(match_result.order_id.clone()));
        }
        
        // Check total stats make sense
        let stats = engine.get_stats();
        assert_eq!(stats.active_fillers, 5);
        assert_eq!(stats.pending_orders + matches.len(), 10);
    }
}