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