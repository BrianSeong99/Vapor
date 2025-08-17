use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

/// Generic Sparse Merkle Tree with dynamic sizing
/// Supports any data type that can be hashed and indexed by a key
pub struct SparseMerkleTree<T> {
    /// Tree depth (can be adjusted based on data size)
    pub depth: usize,
    /// Data indexed by key
    pub data: HashMap<String, T>,
    /// Cached intermediate nodes for efficiency (path -> hash)
    pub cached_nodes: HashMap<String, [u8; 32]>,
    /// Current root hash
    pub root: Option<[u8; 32]>,
    /// Zero hash for empty nodes at each level
    pub zero_hashes: Vec<[u8; 32]>,
    /// Minimum depth to prevent too shallow trees
    pub min_depth: usize,
    /// Maximum depth to prevent memory issues
    pub max_depth: usize,
}

/// Trait for data types that can be stored in sparse Merkle trees
pub trait SparseMerkleLeaf {
    /// Hash the leaf data to a 32-byte hash
    fn hash_leaf(&self, key: &str) -> Result<[u8; 32]>;
    
    /// Convert key to bit path for tree indexing
    fn key_to_path(&self, key: &str, depth: usize) -> String;
}

/// Merkle proof data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub key: String,
    pub leaf_hash: String,
    pub proof: Vec<String>, // Sibling hashes from leaf to root
    pub root: String,
}

/// Tree statistics for monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStats {
    pub depth: usize,
    pub item_count: usize,
    pub cache_size: usize,
    pub optimal_depth: usize,
    pub memory_usage: usize,
}

/// Batch proof generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProofResult {
    pub root: String,
    pub proofs: Vec<MerkleProof>,
    pub stats: TreeStats,
}

/// Generic Sparse Merkle Tree implementation
impl<T: SparseMerkleLeaf + Clone> SparseMerkleTree<T> {
    pub fn new(depth: usize) -> Self {
        Self::new_with_bounds(depth, 4, 32) // Default: min 4, max 32 levels
    }
    
    pub fn new_with_bounds(depth: usize, min_depth: usize, max_depth: usize) -> Self {
        let actual_depth = depth.max(min_depth).min(max_depth);
        let mut zero_hashes = Vec::with_capacity(actual_depth + 1);
        
        // Compute zero hashes for each level
        let mut current_zero = [0u8; 32];
        zero_hashes.push(current_zero);
        
        for _ in 0..actual_depth {
            let mut hasher = Keccak256::new();
            hasher.update(current_zero);
            hasher.update(current_zero);
            current_zero = hasher.finalize().into();
            zero_hashes.push(current_zero);
        }
        
        Self {
            depth: actual_depth,
            data: HashMap::new(),
            cached_nodes: HashMap::new(),
            root: None,
            zero_hashes,
            min_depth,
            max_depth,
        }
    }
    
    /// Create tree with optimal depth based on expected data size
    pub fn new_for_size(expected_items: usize) -> Self {
        let optimal_depth = if expected_items <= 1 {
            4 // Minimum depth
        } else {
            // Calculate depth needed: log2(expected_items) + 1 for safety margin
            ((expected_items as f64).log2().ceil() as usize + 1).max(4).min(32)
        };
        Self::new_with_bounds(optimal_depth, 4, 32)
    }
    
    /// Dynamically resize tree if needed
    pub fn resize_if_needed(&mut self, current_items: usize) -> Result<()> {
        let needed_depth = if current_items <= 1 {
            self.min_depth
        } else {
            ((current_items as f64).log2().ceil() as usize + 1).max(self.min_depth).min(self.max_depth)
        };
        
        if needed_depth != self.depth {
            self.resize(needed_depth)?;
        }
        Ok(())
    }
    
    /// Resize the tree to a new depth
    fn resize(&mut self, new_depth: usize) -> Result<()> {
        let bounded_depth = new_depth.max(self.min_depth).min(self.max_depth);
        
        if bounded_depth == self.depth {
            return Ok(());
        }
        
        // Rebuild zero hashes
        let mut zero_hashes = Vec::with_capacity(bounded_depth + 1);
        let mut current_zero = [0u8; 32];
        zero_hashes.push(current_zero);
        
        for _ in 0..bounded_depth {
            let mut hasher = Keccak256::new();
            hasher.update(current_zero);
            hasher.update(current_zero);
            current_zero = hasher.finalize().into();
            zero_hashes.push(current_zero);
        }
        
        self.depth = bounded_depth;
        self.zero_hashes = zero_hashes;
        self.cached_nodes.clear(); // Invalidate cache
        self.root = None;
        
        Ok(())
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
        self.cached_nodes.clear();
        self.root = None;
    }
    
    pub fn insert(&mut self, key: String, value: T) -> Result<()> {
        self.data.insert(key, value);
        self.invalidate_cache();
        Ok(())
    }
    
    /// Batch insert multiple items efficiently
    pub fn insert_batch(&mut self, items: Vec<(String, T)>) -> Result<()> {
        // Resize tree if needed for the new total size
        let new_total = self.data.len() + items.len();
        self.resize_if_needed(new_total)?;
        
        // Insert all items
        for (key, value) in items {
            self.data.insert(key, value);
        }
        
        self.invalidate_cache();
        Ok(())
    }
    
    /// Build tree from scratch with known items (most efficient)
    pub fn build_from_items(items: Vec<(String, T)>) -> Result<Self> {
        let mut tree = Self::new_for_size(items.len());
        
        for (key, value) in items {
            tree.data.insert(key, value);
        }
        
        Ok(tree)
    }
    
    /// Smart cache invalidation - only clear affected paths
    fn invalidate_cache(&mut self) {
        // For now, clear all cache. Could be optimized to only clear affected paths
        self.cached_nodes.clear();
        self.root = None;
    }
    
    /// Get statistics about the tree
    pub fn get_stats(&self) -> TreeStats {
        TreeStats {
            depth: self.depth,
            item_count: self.data.len(),
            cache_size: self.cached_nodes.len(),
            optimal_depth: if self.data.len() <= 1 { 
                4 
            } else { 
                ((self.data.len() as f64).log2().ceil() as usize + 1).max(4).min(32)
            },
            memory_usage: self.estimate_memory_usage(),
        }
    }
    
    fn estimate_memory_usage(&self) -> usize {
        let data_size = self.data.len() * (32 + 64); // rough estimate
        let cache_size = self.cached_nodes.len() * 32;
        let zero_hashes_size = self.zero_hashes.len() * 32;
        data_size + cache_size + zero_hashes_size
    }
    
    /// Compute root of sparse Merkle tree
    pub fn compute_root(&mut self) -> Result<[u8; 32]> {
        if let Some(root) = self.root {
            return Ok(root);
        }
        
        let root = self.compute_node_hash("".to_string(), 0)?;
        self.root = Some(root);
        Ok(root)
    }
    
    /// Generate Merkle proof for a given key
    pub fn generate_proof(&mut self, key: &str) -> Result<MerkleProof> {
        let root = self.compute_root()?;
        
        // Get the path for this key
        let sample_data = self.data.values().next()
            .ok_or_else(|| anyhow::anyhow!("No data in tree"))?;
        let path = sample_data.key_to_path(key, self.depth);
        
        let mut proof_hashes = Vec::new();
        
        // Collect sibling hashes from leaf to root
        for level in (0..self.depth).rev() {
            let current_path = &path[..level];
            let sibling_path = if path.chars().nth(level).unwrap() == '0' {
                format!("{}1", current_path)
            } else {
                format!("{}0", current_path)
            };
            
            let sibling_hash = self.compute_node_hash(sibling_path, level + 1)?;
            proof_hashes.push(hex::encode(sibling_hash));
        }
        
        // Get leaf hash
        let leaf_hash = if let Some(data) = self.data.get(key) {
            data.hash_leaf(key)?
        } else {
            self.zero_hashes[0]
        };
        
        Ok(MerkleProof {
            key: key.to_string(),
            leaf_hash: hex::encode(leaf_hash),
            proof: proof_hashes,
            root: hex::encode(root),
        })
    }
    
    /// Recursively compute node hash for sparse tree
    fn compute_node_hash(&mut self, path: String, level: usize) -> Result<[u8; 32]> {
        if let Some(cached) = self.cached_nodes.get(&path) {
            return Ok(*cached);
        }
        
        if level == self.depth {
            // Leaf level - hash data if it exists
            let hash = if let Some(data) = self.find_data_at_path(&path) {
                data.hash_leaf(&path)?
            } else {
                self.zero_hashes[0] // Empty leaf
            };
            
            self.cached_nodes.insert(path, hash);
            return Ok(hash);
        }
        
        // Internal node - compute left and right children
        let left_path = format!("{}0", path);
        let right_path = format!("{}1", path);
        
        let left_hash = self.compute_node_hash(left_path, level + 1)?;
        let right_hash = self.compute_node_hash(right_path, level + 1)?;
        
        // Hash left and right children
        let mut hasher = Keccak256::new();
        hasher.update(left_hash);
        hasher.update(right_hash);
        let hash = hasher.finalize().into();
        
        self.cached_nodes.insert(path, hash);
        Ok(hash)
    }
    
    /// Find data that matches the given bit path
    fn find_data_at_path(&self, path: &str) -> Option<&T> {
        for (key, data) in &self.data {
            if data.key_to_path(key, self.depth) == path {
                return Some(data);
            }
        }
        None
    }
    
    /// Generate proofs for multiple keys in batch (most efficient)
    pub fn generate_batch_proofs(&mut self, keys: &[String]) -> Result<BatchProofResult> {
        if keys.is_empty() {
            return Ok(BatchProofResult {
                root: "0x".to_string(),
                proofs: vec![],
                stats: self.get_stats(),
            });
        }
        
        // Compute root once for all proofs
        let root = self.compute_root()?;
        let root_hex = hex::encode(root);
        
        let mut proofs = Vec::with_capacity(keys.len());
        
        for key in keys {
            // Generate proof for this key
            let proof = self.generate_proof_internal(key, root)?;
            proofs.push(proof);
        }
        
        Ok(BatchProofResult {
            root: root_hex,
            proofs,
            stats: self.get_stats(),
        })
    }
    
    /// Internal proof generation that reuses computed root
    fn generate_proof_internal(&mut self, key: &str, root: [u8; 32]) -> Result<MerkleProof> {
        // Get the path for this key
        let sample_data = self.data.values().next()
            .ok_or_else(|| anyhow::anyhow!("No data in tree"))?;
        let path = sample_data.key_to_path(key, self.depth);
        
        let mut proof_hashes = Vec::new();
        
        // Collect sibling hashes from leaf to root
        for level in (0..self.depth).rev() {
            let bit = path.chars().nth(level).unwrap_or('0');
            let current_path = &path[0..level];
            
            let sibling_path = if bit == '0' {
                format!("{}1", current_path)
            } else {
                format!("{}0", current_path)
            };
            
            let sibling_hash = self.compute_node_hash(sibling_path, self.depth - level - 1)?;
            proof_hashes.push(hex::encode(sibling_hash));
        }
        
        // Get leaf hash
        let leaf_hash = if let Some(data) = self.data.get(key) {
            data.hash_leaf(key)?
        } else {
            self.zero_hashes[0]
        };
        
        Ok(MerkleProof {
            key: key.to_string(),
            leaf_hash: hex::encode(leaf_hash),
            proof: proof_hashes,
            root: hex::encode(root),
        })
    }
    
    /// Check if tree needs optimization
    pub fn needs_optimization(&self) -> bool {
        let stats = self.get_stats();
        // Tree needs optimization if actual depth is much larger than optimal
        stats.depth > stats.optimal_depth + 2 || stats.depth < stats.optimal_depth - 1
    }
    
    /// Optimize tree structure based on current data
    pub fn optimize(&mut self) -> Result<()> {
        let stats = self.get_stats();
        if stats.optimal_depth != self.depth {
            self.resize(stats.optimal_depth)?;
        }
        Ok(())
    }
}

/// Utility functions for path conversion
pub fn ethereum_address_to_path(address: &str, depth: usize) -> String {
    // Remove 0x prefix if present
    let clean_addr = if address.starts_with("0x") {
        &address[2..]
    } else {
        address
    };
    
    // Convert hex to binary string
    let mut bit_path = String::new();
    for hex_char in clean_addr.chars() {
        let digit = u8::from_str_radix(&hex_char.to_string(), 16).unwrap_or(0);
        bit_path.push_str(&format!("{:04b}", digit));
    }
    
    // Ensure exactly the required depth
    bit_path.truncate(depth);
    while bit_path.len() < depth {
        bit_path.push('0');
    }
    
    bit_path
}

pub fn index_to_path(index_str: &str, depth: usize) -> String {
    let index: usize = index_str.parse().unwrap_or(0);
    format!("{:0width$b}", index, width = depth)
}

/// Solidity-compatible hashing utilities
pub fn solidity_keccak256_hash(data: &[&[u8]]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    for chunk in data {
        hasher.update(chunk);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test data structure
    #[derive(Clone, Debug)]
    struct TestData {
        value: String,
    }

    impl SparseMerkleLeaf for TestData {
        fn hash_leaf(&self, _key: &str) -> Result<[u8; 32]> {
            let mut hasher = Keccak256::new();
            hasher.update(self.value.as_bytes());
            Ok(hasher.finalize().into())
        }

        fn key_to_path(&self, key: &str, depth: usize) -> String {
            index_to_path(key, depth)
        }
    }

    #[test]
    fn test_sparse_merkle_tree_basic() {
        let mut tree = SparseMerkleTree::new(3);
        
        // Insert test data
        tree.insert("0".to_string(), TestData { value: "test0".to_string() }).unwrap();
        tree.insert("1".to_string(), TestData { value: "test1".to_string() }).unwrap();
        
        // Compute root
        let root = tree.compute_root().unwrap();
        assert_eq!(root.len(), 32);
        
        // Generate proof
        let proof = tree.generate_proof("0").unwrap();
        assert_eq!(proof.proof.len(), 4); // Actual tree depth (min 4 due to optimization)
    }

    #[test]
    fn test_path_conversion() {
        let addr = "0x742d35Cc6634C0532925a3b8D5C0B5Cc0532C75e";
        let path = ethereum_address_to_path(addr, 160);
        assert_eq!(path.len(), 160);
        
        let index_path = index_to_path("5", 8);
        assert_eq!(index_path, "00000101");
    }
}
