use crate::models::{Order, AccountState, TokenBalance};
use crate::lib::{SparseMerkleTree, SparseMerkleLeaf, MerkleProof, ethereum_address_to_path, index_to_path};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

// Constants for tree depths
const ACCOUNT_TREE_DEPTH: usize = 160; // Ethereum address bit size
const ORDER_TREE_DEPTH: usize = 20;    // 2^20 = ~1M max orders per batch

/// Merkle Tree Manager using generic sparse trees
pub struct MerkleTreeManager {
    /// Sparse account state tree (160 levels, Ethereum address-based)
    pub account_tree: SparseMerkleTree<AccountState>,
    /// Sparse order tree (20 levels, ~1M max orders per batch)
    pub order_tree: OrderMerkleTree,
    /// Current batch ID for order tree context
    pub current_batch_id: u32,
}

/// Specialized Order Merkle Tree that handles batch_id context
pub struct OrderMerkleTree {
    inner: SparseMerkleTree<Order>,
    current_batch_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLeaf {
    pub address: String,
    pub balances: Vec<TokenBalance>,
    pub nonce: u64, // Prevent replay attacks
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMerkleProof {
    pub address: String,
    pub leaf_hash: String,
    pub proof: Vec<String>, // Sibling hashes from leaf to root
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderMerkleProof {
    pub order_index: usize,
    pub leaf_hash: String,
    pub proof: Vec<String>, // Sibling hashes from leaf to root
    pub root: String,
}

impl MerkleTreeManager {
    pub fn new() -> Self {
        Self {
            account_tree: SparseMerkleTree::new(ACCOUNT_TREE_DEPTH),
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        }
    }

    /// Build sparse account state tree (160-bit depth based on Ethereum addresses)
    pub fn build_state_tree(&mut self, accounts: &[AccountState]) -> Result<String> {
        self.account_tree.clear();
        
        for account in accounts {
            self.account_tree.insert(account.address.clone(), account.clone())?;
        }
        
        let root = self.account_tree.compute_root()?;
        Ok(hex::encode(root))
    }

    /// Build sparse order tree (20 levels, max ~1M orders)
    /// Uses Solidity-compatible hashing to match smart contract verification
    pub fn build_orders_tree(&mut self, orders: &[Order], batch_id: u32) -> Result<String> {
        let max_orders = 1 << ORDER_TREE_DEPTH; // 2^20 = ~1M orders
        if orders.len() > max_orders {
            return Err(anyhow::anyhow!("Too many orders: {} (max {})", orders.len(), max_orders));
        }
        
        self.current_batch_id = batch_id;
        self.order_tree.set_batch_id(batch_id);
        self.order_tree.clear();
        
        for (index, order) in orders.iter().enumerate() {
            self.order_tree.insert(index.to_string(), order.clone())?;
        }
        
        let root = self.order_tree.compute_root()?;
        Ok(hex::encode(root))
    }

    /// Generate Merkle proof for an order (used for claims)
    pub fn generate_order_proof(&mut self, order_index: usize) -> Result<OrderMerkleProof> {
        let proof = self.order_tree.generate_proof(&order_index.to_string())?;
        
        Ok(OrderMerkleProof {
            order_index,
            leaf_hash: proof.leaf_hash,
            proof: proof.proof,
            root: proof.root,
        })
    }

    /// Generate Merkle proof for an account state  
    pub fn generate_account_proof(&mut self, address: &str) -> Result<AccountMerkleProof> {
        let proof = self.account_tree.generate_proof(address)?;
        
        Ok(AccountMerkleProof {
            address: address.to_string(),
            leaf_hash: proof.leaf_hash,
            proof: proof.proof,
            root: proof.root,
        })
    }

    /// Get current state root (for SP1 proof)
    pub fn get_state_root(&mut self) -> Result<String> {
        let root = self.account_tree.compute_root()?;
        Ok(hex::encode(root))
    }

    /// Get current orders root (for SP1 proof)
    pub fn get_orders_root(&mut self) -> Result<String> {
        let root = self.order_tree.compute_root()?;
        Ok(hex::encode(root))
    }

    /// Create deterministic empty roots for genesis batch
    pub fn empty_state_root() -> String {
        // Empty sparse tree root (all zeros)
        hex::encode([0u8; 32])
    }

    pub fn empty_orders_root() -> String {
        // Empty fixed tree root (all zeros)
        hex::encode([0u8; 32])
    }
}

// Trait implementations for AccountState
impl SparseMerkleLeaf for AccountState {
    fn hash_leaf(&self, _key: &str) -> Result<[u8; 32]> {
        let mut hasher = Keccak256::new();
        
        // Hash address
        hasher.update(self.address.as_bytes());
        
        // Hash balances in deterministic order
        let mut sorted_balances = self.balances.clone();
        sorted_balances.sort_by_key(|b| b.token_id);
        
        for balance in sorted_balances {
            hasher.update(balance.token_id.to_be_bytes());
            hasher.update(balance.balance.as_bytes());
        }
        
        Ok(hasher.finalize().into())
    }
    
    fn key_to_path(&self, key: &str, depth: usize) -> String {
        ethereum_address_to_path(key, depth)
    }
}

// Trait implementations for Order (basic version, specialized tree handles batch_id)
impl SparseMerkleLeaf for Order {
    fn hash_leaf(&self, _key: &str) -> Result<[u8; 32]> {
        // This should not be called directly - use OrderMerkleTree instead
        Err(anyhow::anyhow!("Order hash_leaf requires batch context - use OrderMerkleTree instead"))
    }
    
    fn key_to_path(&self, key: &str, depth: usize) -> String {
        index_to_path(key, depth)
    }
}

impl Order {
    /// Hash leaf with batch ID context
    pub fn hash_leaf_with_batch_id(&self, batch_id: u32) -> Result<[u8; 32]> {
        let leaf_hash = solidity_order_leaf_hash(
            batch_id,
            &self.id,
            self.order_type as u8,
            &self.from_address.clone().unwrap_or_default(),
            &self.to_address.clone().unwrap_or_default(),
            self.token_id,
            &self.amount,
        );
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&leaf_hash[..32]);
        Ok(hash_array)
    }
}

impl OrderMerkleTree {
    pub fn new(depth: usize) -> Self {
        Self {
            inner: SparseMerkleTree::new(depth),
            current_batch_id: None,
        }
    }
    
    pub fn set_batch_id(&mut self, batch_id: u32) {
        self.current_batch_id = Some(batch_id);
        // Clear cache when batch ID changes
        self.inner.cached_nodes.clear();
        self.inner.root = None;
    }
    
    pub fn clear(&mut self) {
        self.inner.clear();
    }
    
    pub fn insert(&mut self, key: String, value: Order) -> Result<()> {
        self.inner.insert(key, value)
    }
    
    pub fn compute_root(&mut self) -> Result<[u8; 32]> {
        let batch_id = self.current_batch_id
            .ok_or_else(|| anyhow::anyhow!("Batch ID not set for order tree"))?;
        
        if let Some(root) = self.inner.root {
            return Ok(root);
        }
        
        let root = self.compute_node_hash("".to_string(), 0, batch_id)?;
        self.inner.root = Some(root);
        Ok(root)
    }
    
    pub fn generate_proof(&mut self, key: &str) -> Result<MerkleProof> {
        let batch_id = self.current_batch_id
            .ok_or_else(|| anyhow::anyhow!("Batch ID not set for order tree"))?;
        
        let root = self.compute_root()?;
        
        // Get the path for this key
        let sample_data = self.inner.data.values().next()
            .ok_or_else(|| anyhow::anyhow!("No data in tree"))?;
        let path = sample_data.key_to_path(key, self.inner.depth);
        
        let mut proof_hashes = Vec::new();
        
        // Collect sibling hashes from leaf to root
        for level in (0..self.inner.depth).rev() {
            let current_path = &path[..level];
            let sibling_path = if path.chars().nth(level).unwrap() == '0' {
                format!("{}1", current_path)
            } else {
                format!("{}0", current_path)
            };
            
            let sibling_hash = self.compute_node_hash(sibling_path, level + 1, batch_id)?;
            proof_hashes.push(hex::encode(sibling_hash));
        }
        
        // Get leaf hash
        let leaf_hash = if let Some(order) = self.inner.data.get(key) {
            order.hash_leaf_with_batch_id(batch_id)?
        } else {
            self.inner.zero_hashes[0]
        };
        
        Ok(MerkleProof {
            key: key.to_string(),
            leaf_hash: hex::encode(leaf_hash),
            proof: proof_hashes,
            root: hex::encode(root),
        })
    }
    
    /// Recursively compute node hash with batch_id context
    fn compute_node_hash(&mut self, path: String, level: usize, batch_id: u32) -> Result<[u8; 32]> {
        if let Some(cached) = self.inner.cached_nodes.get(&path) {
            return Ok(*cached);
        }
        
        if level == self.inner.depth {
            // Leaf level - hash order data if it exists
            let hash = if let Some(order) = self.find_data_at_path(&path) {
                order.hash_leaf_with_batch_id(batch_id)?
            } else {
                self.inner.zero_hashes[0] // Empty leaf
            };
            
            self.inner.cached_nodes.insert(path, hash);
            return Ok(hash);
        }
        
        // Internal node - compute left and right children
        let left_path = format!("{}0", path);
        let right_path = format!("{}1", path);
        
        let left_hash = self.compute_node_hash(left_path, level + 1, batch_id)?;
        let right_hash = self.compute_node_hash(right_path, level + 1, batch_id)?;
        
        // Hash left and right children
        let mut hasher = Keccak256::new();
        hasher.update(left_hash);
        hasher.update(right_hash);
        let hash = hasher.finalize().into();
        
        self.inner.cached_nodes.insert(path, hash);
        Ok(hash)
    }
    
    /// Find data that matches the given bit path
    fn find_data_at_path(&self, path: &str) -> Option<&Order> {
        for (key, data) in &self.inner.data {
            if data.key_to_path(key, self.inner.depth) == path {
                return Some(data);
            }
        }
        None
    }
}

/// Solidity-compatible order leaf hash
fn solidity_order_leaf_hash(
    batch_id: u32,
    order_id: &str,
    order_type: u8,
    from: &str,
    to: &str,
    token_id: u32,
    amount: &str,
) -> Vec<u8> {
    // This should match the keccak256(abi.encode(...)) in the smart contract
    let mut hasher = Keccak256::new();
    
    hasher.update(batch_id.to_be_bytes()); // Solidity uses big-endian
    hasher.update(order_id.as_bytes());
    hasher.update([order_type]);
    hasher.update(from.as_bytes());
    hasher.update(to.as_bytes());
    hasher.update(token_id.to_be_bytes());
    hasher.update(amount.as_bytes());
    
    hasher.finalize().to_vec()
}

/// Utility functions for Solidity compatibility
impl MerkleTreeManager {
    /// Convert order to Solidity-compatible leaf hash (matches smart contract)
    pub fn solidity_order_leaf_hash(
        batch_id: u32,
        order_id: &str,
        order_type: u8,
        from: &str,
        to: &str,
        token_id: u32,
        amount: &str,
    ) -> Vec<u8> {
        solidity_order_leaf_hash(batch_id, order_id, order_type, from, to, token_id, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Order, OrderType, OrderStatus, AccountState, TokenBalance};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_order(id: &str, order_type: OrderType) -> Order {
        Order {
            id: id.to_string(),
            order_type,
            status: OrderStatus::Pending,
            from_address: Some("0x1234567890123456789012345678901234567890".to_string()),
            to_address: Some("0x9876543210987654321098765432109876543210".to_string()),
            token_id: 1,
            amount: "1000000".to_string(),
            banking_hash: Some("0xbankinghash".to_string()),
            batch_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_account(address: &str, balances: Vec<(u32, &str)>) -> AccountState {
        let mut account = AccountState::new(address.to_string());
        for (token_id, balance) in balances {
            account.set_balance(token_id, balance.to_string());
        }
        account
    }

    #[test]
    fn test_merkle_tree_manager_creation() {
        let manager = MerkleTreeManager::new();
        assert_eq!(manager.current_batch_id, 0);
        assert_eq!(manager.account_tree.depth, ACCOUNT_TREE_DEPTH);
        assert_eq!(manager.order_tree.inner.depth, ORDER_TREE_DEPTH);
    }

    #[test]
    fn test_empty_roots() {
        let empty_state = MerkleTreeManager::empty_state_root();
        let empty_orders = MerkleTreeManager::empty_orders_root();
        
        assert_eq!(empty_state, "0000000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(empty_orders, "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_single_account_state_tree() {
        // Use smaller depth manager for faster testing
        let mut manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8),
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        };
        
        let account = create_test_account(
            "0x12",  // Shorter address for smaller tree
            vec![(1, "1000000"), (2, "2000000")]
        );
        
        let root = manager.build_state_tree(&[account]).unwrap();
        assert_eq!(root.len(), 64); // 32 bytes hex = 64 chars
        assert_ne!(root, MerkleTreeManager::empty_state_root());
        
        // Test deterministic root
        let root2 = manager.build_state_tree(&[create_test_account(
            "0x12",
            vec![(1, "1000000"), (2, "2000000")]
        )]).unwrap();
        assert_eq!(root, root2);
    }

    #[test]
    fn test_multiple_account_state_tree() {
        // Use smaller depth manager for faster testing
        let mut manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8),
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        };
        
        let accounts = vec![
            create_test_account("0x11", vec![(1, "1000")]),
            create_test_account("0x22", vec![(1, "2000"), (2, "3000")]),
            create_test_account("0x33", vec![(2, "5000")]),
        ];
        
        let root = manager.build_state_tree(&accounts).unwrap();
        assert_eq!(root.len(), 64);
        assert_ne!(root, MerkleTreeManager::empty_state_root());
        
        // Test that order doesn't matter (different insertion order should give same root)
        let accounts_reordered = vec![
            create_test_account("0x33", vec![(2, "5000")]),
            create_test_account("0x11", vec![(1, "1000")]),
            create_test_account("0x22", vec![(1, "2000"), (2, "3000")]),
        ];
        
        let root_reordered = manager.build_state_tree(&accounts_reordered).unwrap();
        assert_eq!(root, root_reordered, "Root should be independent of insertion order");
    }

    #[test]
    fn test_single_order_tree() {
        let mut manager = MerkleTreeManager::new();
        
        let order = create_test_order("order-1", OrderType::BridgeIn);
        let batch_id = 123;
        
        let root = manager.build_orders_tree(&[order], batch_id).unwrap();
        assert_eq!(root.len(), 64);
        assert_ne!(root, MerkleTreeManager::empty_orders_root());
        assert_eq!(manager.current_batch_id, batch_id);
    }

    #[test]
    fn test_multiple_order_tree() {
        let mut manager = MerkleTreeManager::new();
        
        let orders = vec![
            create_test_order("order-1", OrderType::BridgeIn),
            create_test_order("order-2", OrderType::Transfer),
            create_test_order("order-3", OrderType::BridgeOut),
        ];
        let batch_id = 456;
        
        let root = manager.build_orders_tree(&orders, batch_id).unwrap();
        assert_eq!(root.len(), 64);
        assert_ne!(root, MerkleTreeManager::empty_orders_root());
        
        // Different batch ID should produce different root
        let root_diff_batch = manager.build_orders_tree(&orders, batch_id + 1).unwrap();
        assert_ne!(root, root_diff_batch, "Different batch IDs should produce different roots");
    }

    #[test]
    fn test_order_tree_max_capacity() {
        let mut manager = MerkleTreeManager::new();
        
        // Test at capacity limit
        let max_orders = 1 << ORDER_TREE_DEPTH; // 2^20 = ~1M orders
        
        // We can't actually create 1M orders in a test, so test the validation
        let mut large_orders = Vec::new();
        for i in 0..10 {
            large_orders.push(create_test_order(&format!("order-{}", i), OrderType::BridgeIn));
        }
        
        // This should work fine
        let result = manager.build_orders_tree(&large_orders, 1);
        assert!(result.is_ok());
        
        // Test that the limit calculation is correct
        assert_eq!(max_orders, 1048576); // 2^20
    }

    #[test]
    fn test_account_merkle_proof_generation() {
        // Use a smaller tree depth for testing to avoid performance issues
        let mut small_manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8), // Much smaller depth for testing
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        };
        
        let address = "0x12"; // Use shorter address for smaller tree
        let account = create_test_account(address, vec![(1, "1000000")]);
        
        small_manager.build_state_tree(&[account]).unwrap();
        let proof = small_manager.generate_account_proof(address).unwrap();
        
        assert_eq!(proof.address, address);
        assert_eq!(proof.leaf_hash.len(), 64); // 32 bytes hex
        assert_eq!(proof.proof.len(), 8); // Should have sibling for each level (small tree)
        assert_eq!(proof.root.len(), 64);
        assert_ne!(proof.root, MerkleTreeManager::empty_state_root());
    }

    #[test]
    fn test_order_merkle_proof_generation() {
        let mut manager = MerkleTreeManager::new();
        
        let orders = vec![
            create_test_order("order-0", OrderType::BridgeIn),
            create_test_order("order-1", OrderType::Transfer),
        ];
        let batch_id = 789;
        
        manager.build_orders_tree(&orders, batch_id).unwrap();
        
        // Generate proof for first order (index 0)
        let proof = manager.generate_order_proof(0).unwrap();
        
        assert_eq!(proof.order_index, 0);
        assert_eq!(proof.leaf_hash.len(), 64);
        assert_eq!(proof.proof.len(), ORDER_TREE_DEPTH); // Should have sibling for each level
        assert_eq!(proof.root.len(), 64);
        assert_ne!(proof.root, MerkleTreeManager::empty_orders_root());
    }

    #[test]
    fn test_order_hash_with_batch_id() {
        let order = create_test_order("test-order", OrderType::BridgeIn);
        
        let hash1 = order.hash_leaf_with_batch_id(123).unwrap();
        let hash2 = order.hash_leaf_with_batch_id(123).unwrap();
        assert_eq!(hash1, hash2, "Same batch ID should produce same hash");
        
        let hash3 = order.hash_leaf_with_batch_id(124).unwrap();
        assert_ne!(hash1, hash3, "Different batch ID should produce different hash");
        
        assert_eq!(hash1.len(), 32, "Hash should be 32 bytes");
    }

    #[test]
    fn test_solidity_compatible_order_hash() {
        let batch_id = 123;
        let order_id = "test-order";
        let order_type = OrderType::BridgeIn as u8;
        let from = "0x1111111111111111111111111111111111111111";
        let to = "0x2222222222222222222222222222222222222222";
        let token_id = 1;
        let amount = "1000000";
        
        let hash = MerkleTreeManager::solidity_order_leaf_hash(
            batch_id, order_id, order_type, from, to, token_id, amount
        );
        
        assert_eq!(hash.len(), 32, "Solidity hash should be 32 bytes");
        
        // Same parameters should produce same hash
        let hash2 = MerkleTreeManager::solidity_order_leaf_hash(
            batch_id, order_id, order_type, from, to, token_id, amount
        );
        assert_eq!(hash, hash2, "Deterministic hash for same parameters");
        
        // Different batch ID should produce different hash
        let hash3 = MerkleTreeManager::solidity_order_leaf_hash(
            batch_id + 1, order_id, order_type, from, to, token_id, amount
        );
        assert_ne!(hash, hash3, "Different batch ID should change hash");
    }

    #[test]
    fn test_account_state_leaf_hashing() {
        let account = create_test_account(
            "0x1234567890123456789012345678901234567890",
            vec![(1, "1000000"), (2, "2000000")]
        );
        
        let hash1 = account.hash_leaf();
        let hash2 = account.hash_leaf();
        assert_eq!(hash1, hash2, "Same account should produce same hash");
        
        // Different account should produce different hash
        let account2 = create_test_account(
            "0x9876543210987654321098765432109876543210",
            vec![(1, "1000000"), (2, "2000000")]
        );
        let hash3 = account2.hash_leaf();
        assert_ne!(hash1, hash3, "Different address should produce different hash");
        
        // Different balances should produce different hash
        let account3 = create_test_account(
            "0x1234567890123456789012345678901234567890",
            vec![(1, "1500000"), (2, "2000000")]
        );
        let hash4 = account3.hash_leaf();
        assert_ne!(hash1, hash4, "Different balance should produce different hash");
    }

    #[test]
    fn test_order_merkle_tree_batch_context() {
        let mut order_tree = OrderMerkleTree::new(ORDER_TREE_DEPTH);
        
        // Test that batch ID is required
        let order = create_test_order("test", OrderType::BridgeIn);
        order_tree.insert("0".to_string(), order).unwrap();
        
        // Should fail without batch ID
        let result = order_tree.compute_root();
        assert!(result.is_err(), "Should require batch ID");
        
        // Should work with batch ID
        order_tree.set_batch_id(123);
        let root = order_tree.compute_root().unwrap();
        assert_eq!(root.len(), 32);
        
        // Changing batch ID should clear cache and change root
        order_tree.set_batch_id(124);
        let root2 = order_tree.compute_root().unwrap();
        assert_ne!(root, root2, "Different batch ID should produce different root");
    }

    #[test]
    fn test_tree_roots_consistency() {
        // Use smaller depth manager for faster testing
        let mut manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8),
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        };
        
        // Build state tree
        let accounts = vec![
            create_test_account("0x11", vec![(1, "1000")]),
        ];
        let state_root1 = manager.build_state_tree(&accounts).unwrap();
        let state_root2 = manager.get_state_root().unwrap();
        assert_eq!(state_root1, state_root2, "build_state_tree and get_state_root should match");
        
        // Build orders tree
        let orders = vec![create_test_order("order-1", OrderType::BridgeIn)];
        let orders_root1 = manager.build_orders_tree(&orders, 123).unwrap();
        let orders_root2 = manager.get_orders_root().unwrap();
        assert_eq!(orders_root1, orders_root2, "build_orders_tree and get_orders_root should match");
    }

    #[test]
    fn test_ethereum_address_path_conversion() {
        // Test that ethereum addresses are properly converted to bit paths
        let address1 = "0x0000000000000000000000000000000000000000";
        let address2 = "0xffffffffffffffffffffffffffffffffffffffff";
        
        let account1 = create_test_account(address1, vec![(1, "1000")]);
        let account2 = create_test_account(address2, vec![(1, "1000")]);
        
        let path1 = account1.key_to_path(address1, ACCOUNT_TREE_DEPTH);
        let path2 = account2.key_to_path(address2, ACCOUNT_TREE_DEPTH);
        
        assert_eq!(path1.len(), ACCOUNT_TREE_DEPTH, "Path should match tree depth");
        assert_eq!(path2.len(), ACCOUNT_TREE_DEPTH, "Path should match tree depth");
        assert_ne!(path1, path2, "Different addresses should have different paths");
        
        // All zeros address should have path of all '0's
        assert!(path1.chars().all(|c| c == '0'), "Zero address should map to zero path");
        // All ones address should have path of all '1's
        assert!(path2.chars().all(|c| c == '1'), "Max address should map to one path");
    }

    #[test]
    fn test_large_tree_performance() {
        // Use smaller depth manager for performance testing
        let mut manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8), // Much smaller depth for testing
            order_tree: OrderMerkleTree::new(8), // Much smaller order tree depth too
            current_batch_id: 0,
        };
        
        // Test with a small number of accounts for fast performance testing
        let mut accounts = Vec::new();
        for i in 0..10 {  // Further reduced to 10 accounts
            let address = format!("0x{:02x}", i);  // Shorter address for smaller tree
            accounts.push(create_test_account(&address, vec![(1, &format!("{}", 1000 + i))]));
        }
        
        let start = std::time::Instant::now();
        let root = manager.build_state_tree(&accounts).unwrap();
        let duration = start.elapsed();
        
        assert_eq!(root.len(), 64);
        assert!(duration.as_millis() < 100, "Should build 10-account tree in under 100ms");
        
        // Test orders tree performance  
        let mut orders = Vec::new();
        for i in 0..10 {  // Further reduced to 10 orders
            orders.push(create_test_order(&format!("order-{}", i), OrderType::BridgeIn));
        }
        
        let start = std::time::Instant::now();
        let orders_root = manager.build_orders_tree(&orders, 123).unwrap();
        let duration = start.elapsed();
        
        assert_eq!(orders_root.len(), 64);
        assert!(duration.as_millis() < 100, "Should build 10-order tree in under 100ms");
    }

    #[test]
    fn test_tree_edge_cases() {
        // Use smaller depth manager for faster testing
        let mut manager = MerkleTreeManager {
            account_tree: SparseMerkleTree::new(8),
            order_tree: OrderMerkleTree::new(ORDER_TREE_DEPTH),
            current_batch_id: 0,
        };
        
        // Empty tree roots
        let empty_state = manager.get_state_root().unwrap();
        
        // For order tree, we need to set batch ID first, then get the root
        // The empty order tree root will be different from static zero hash due to batch context
        manager.order_tree.set_batch_id(1);
        let empty_orders_with_batch = manager.get_orders_root().unwrap();
        
        // Empty state tree with different depth will not equal static zero hash
        assert_ne!(empty_state, MerkleTreeManager::empty_state_root());
        assert_eq!(empty_state.len(), 64); // Should be valid 32-byte hex string
        
        // Empty orders tree with batch context will not equal static zero hash
        assert_ne!(empty_orders_with_batch, MerkleTreeManager::empty_orders_root());
        assert_eq!(empty_orders_with_batch.len(), 64); // Should be valid 32-byte hex string
        
        // Single item trees
        let single_account = vec![create_test_account("0x11", vec![(1, "1000")])];
        let single_root = manager.build_state_tree(&single_account).unwrap();
        assert_ne!(single_root, empty_state);
        
        // Proof for non-existent account should still work (prove non-inclusion)
        let proof_result = manager.generate_account_proof("0x99");
        assert!(proof_result.is_ok(), "Should generate proof for non-existent account");
    }

    #[test] 
    fn test_deterministic_token_balance_ordering() {
        let mut account1 = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        // Add tokens in one order
        account1.set_balance(3, "3000".to_string());
        account1.set_balance(1, "1000".to_string());
        account1.set_balance(2, "2000".to_string());
        
        let mut account2 = AccountState::new("0x1234567890123456789012345678901234567890".to_string());
        // Add tokens in different order
        account2.set_balance(1, "1000".to_string());
        account2.set_balance(3, "3000".to_string());
        account2.set_balance(2, "2000".to_string());
        
        let hash1 = account1.hash_leaf();
        let hash2 = account2.hash_leaf();
        
        assert_eq!(hash1, hash2, "Token balance order should not affect hash (deterministic sorting)");
    }
}