use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;

/// Generic Sparse Merkle Tree with fixed height
/// Supports any data type that can be hashed and indexed by a key
pub struct SparseMerkleTree<T> {
    /// Tree depth (fixed at construction)
    pub depth: usize,
    /// Data indexed by key
    pub data: HashMap<String, T>,
    /// Cached intermediate nodes for efficiency (path -> hash)
    pub cached_nodes: HashMap<String, [u8; 32]>,
    /// Current root hash
    pub root: Option<[u8; 32]>,
    /// Zero hash for empty nodes at each level
    pub zero_hashes: Vec<[u8; 32]>,
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

/// Generic Sparse Merkle Tree implementation
impl<T: SparseMerkleLeaf + Clone> SparseMerkleTree<T> {
    pub fn new(depth: usize) -> Self {
        let mut zero_hashes = Vec::with_capacity(depth + 1);
        
        // Compute zero hashes for each level
        let mut current_zero = [0u8; 32];
        zero_hashes.push(current_zero);
        
        for _ in 0..depth {
            let mut hasher = Keccak256::new();
            hasher.update(current_zero);
            hasher.update(current_zero);
            current_zero = hasher.finalize().into();
            zero_hashes.push(current_zero);
        }
        
        Self {
            depth,
            data: HashMap::new(),
            cached_nodes: HashMap::new(),
            root: None,
            zero_hashes,
        }
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
        self.cached_nodes.clear();
        self.root = None;
    }
    
    pub fn insert(&mut self, key: String, value: T) -> Result<()> {
        self.data.insert(key, value);
        self.cached_nodes.clear(); // Invalidate cache
        self.root = None;
        Ok(())
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
        assert_eq!(proof.proof.len(), 3); // Tree depth
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
