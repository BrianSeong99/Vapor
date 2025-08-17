use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use tracing::{info, warn};
use chrono::Utc;
use std::time::Duration;
use tokio::time::sleep;

use crate::models::Order;

/// Mock proof data structure for MVP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockProof {
    pub batch_id: u32,
    pub prev_state_root: String,
    pub prev_orders_root: String,
    pub new_state_root: String,
    pub new_orders_root: String,
    pub orders_count: usize,
    pub proof_data: Vec<u8>, // Mock proof bytes
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub verification_key: String,
}

/// Proof generation result
#[derive(Debug, Clone, Serialize)]
pub struct ProofGenerationResult {
    pub success: bool,
    pub proof: Option<MockProof>,
    pub error_message: Option<String>,
    pub generation_time_ms: u64,
}

/// Configuration for MVP prover
#[derive(Debug, Clone)]
pub struct MvpProverConfig {
    /// Simulated proof generation time (for realism)
    pub generation_delay_ms: u64,
    /// Whether to simulate occasional failures
    pub simulate_failures: bool,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f64,
}

impl Default for MvpProverConfig {
    fn default() -> Self {
        Self {
            generation_delay_ms: 2000, // 2 seconds simulated proof time
            simulate_failures: false,   // No failures for MVP
            failure_rate: 0.1,         // 10% failure rate if enabled
        }
    }
}

/// MVP Prover service that mocks SP1 proof generation
pub struct MvpProverService {
    config: MvpProverConfig,
}

impl MvpProverService {
    /// Create a new MVP prover service
    pub fn new(config: MvpProverConfig) -> Self {
        Self { config }
    }

    /// Generate a mock proof for a batch
    pub async fn generate_proof_for_batch(
        &self,
        batch_id: u32,
        prev_state_root: &str,
        prev_orders_root: &str,
        new_state_root: &str,
        new_orders_root: &str,
        orders: &[Order],
    ) -> Result<ProofGenerationResult> {
        let start_time = std::time::Instant::now();
        
        info!(
            "Starting mock proof generation for batch {} with {} orders",
            batch_id, orders.len()
        );

        // Simulate proof generation time
        if self.config.generation_delay_ms > 0 {
            sleep(Duration::from_millis(self.config.generation_delay_ms)).await;
        }

        // Simulate occasional failures if enabled
        if self.config.simulate_failures {
            if rand::random::<f64>() < self.config.failure_rate {
                warn!("Simulated proof generation failure for batch {}", batch_id);
                return Ok(ProofGenerationResult {
                    success: false,
                    proof: None,
                    error_message: Some("Simulated proof generation failure".to_string()),
                    generation_time_ms: start_time.elapsed().as_millis() as u64,
                });
            }
        }

        // Generate mock proof
        let proof = self.create_mock_proof(
            batch_id,
            prev_state_root,
            prev_orders_root,
            new_state_root,
            new_orders_root,
            orders,
        );

        let generation_time = start_time.elapsed().as_millis() as u64;

        info!(
            "Mock proof generated successfully for batch {} in {}ms",
            batch_id, generation_time
        );

        Ok(ProofGenerationResult {
            success: true,
            proof: Some(proof),
            error_message: None,
            generation_time_ms: generation_time,
        })
    }

    /// Create a mock proof with deterministic but realistic-looking data
    fn create_mock_proof(
        &self,
        batch_id: u32,
        prev_state_root: &str,
        prev_orders_root: &str,
        new_state_root: &str,
        new_orders_root: &str,
        orders: &[Order],
    ) -> MockProof {
        // Create a deterministic but realistic proof based on batch data
        let mut hasher = Keccak256::new();
        hasher.update(batch_id.to_le_bytes());
        hasher.update(prev_state_root.as_bytes());
        hasher.update(prev_orders_root.as_bytes());
        hasher.update(new_state_root.as_bytes());
        hasher.update(new_orders_root.as_bytes());
        
        // Include order data in proof
        for order in orders {
            hasher.update(order.id.as_bytes());
            hasher.update(&[order.order_type as u8]);
            hasher.update(order.amount.as_bytes());
        }

        let proof_hash = hasher.finalize();
        
        // Generate mock proof data (in real SP1, this would be the actual proof)
        let mut proof_data = Vec::with_capacity(1024); // Typical proof size
        proof_data.extend_from_slice(&proof_hash);
        
        // Pad to make it look more realistic
        while proof_data.len() < 1024 {
            proof_data.push((proof_data.len() % 256) as u8);
        }

        // Generate verification key (deterministic for this batch)
        let vk_hash = Keccak256::digest(format!("vk_{}", batch_id).as_bytes());
        let verification_key = format!("0x{}", hex::encode(&vk_hash[..16])); // First 16 bytes as hex

        MockProof {
            batch_id,
            prev_state_root: prev_state_root.to_string(),
            prev_orders_root: prev_orders_root.to_string(),
            new_state_root: new_state_root.to_string(),
            new_orders_root: new_orders_root.to_string(),
            orders_count: orders.len(),
            proof_data,
            generated_at: Utc::now(),
            verification_key,
        }
    }

    /// Validate a mock proof (for testing)
    pub fn validate_proof(&self, proof: &MockProof) -> bool {
        // Basic validation checks
        if proof.proof_data.is_empty() {
            return false;
        }

        if proof.orders_count == 0 && proof.batch_id > 0 {
            // Allow empty batches only for genesis
            return false;
        }

        // Check that roots are valid hex strings
        if !proof.prev_state_root.starts_with("0x") ||
           !proof.prev_orders_root.starts_with("0x") ||
           !proof.new_state_root.starts_with("0x") ||
           !proof.new_orders_root.starts_with("0x") {
            return false;
        }

        // All validations passed
        true
    }

    /// Get prover statistics
    pub fn get_stats(&self) -> ProverStats {
        ProverStats {
            is_mock: true,
            generation_delay_ms: self.config.generation_delay_ms,
            simulate_failures: self.config.simulate_failures,
            failure_rate: self.config.failure_rate,
            total_proofs_generated: 0, // TODO: Track this
            total_failures: 0,         // TODO: Track this
            average_generation_time_ms: self.config.generation_delay_ms,
        }
    }

    /// Update prover configuration
    pub fn update_config(&mut self, new_config: MvpProverConfig) {
        info!("Updating MVP prover configuration: {:?}", new_config);
        self.config = new_config;
    }
}

/// Statistics for the MVP prover service
#[derive(Debug, Serialize)]
pub struct ProverStats {
    pub is_mock: bool,
    pub generation_delay_ms: u64,
    pub simulate_failures: bool,
    pub failure_rate: f64,
    pub total_proofs_generated: u64,
    pub total_failures: u64,
    pub average_generation_time_ms: u64,
}

/// Helper function to convert MockProof to bytes for blockchain submission
impl MockProof {
    pub fn to_submission_bytes(&self) -> Vec<u8> {
        // In real SP1, this would be the actual proof bytes
        // For MVP, we return the mock proof data
        self.proof_data.clone()
    }

    pub fn to_hex_string(&self) -> String {
        format!("0x{}", hex::encode(&self.proof_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrderType, OrderStatus, Batch, BatchStatus};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_mock_proof_generation() {
        let config = MvpProverConfig {
            generation_delay_ms: 10, // Fast for testing
            simulate_failures: false,
            failure_rate: 0.0,
        };
        
        let prover = MvpProverService::new(config);
        
        let orders = vec![Order {
            id: Uuid::new_v4().to_string(),
            order_type: OrderType::BridgeIn,
            status: OrderStatus::Pending,
            from_address: Some("0x123".to_string()),
            to_address: Some("0x456".to_string()),
            token_id: 1,
            amount: "1000".to_string(),
            banking_hash: Some("hash123".to_string()),
            batch_id: Some(1),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];

        let result = prover.generate_proof_for_batch(
            1,
            "0x1111",
            "0x2222",
            "0x3333",
            "0x4444",
            &orders,
        ).await.unwrap();

        assert!(result.success);
        assert!(result.proof.is_some());
        
        let proof = result.proof.unwrap();
        assert_eq!(proof.batch_id, 1);
        assert_eq!(proof.orders_count, 1);
        assert!(prover.validate_proof(&proof));
    }

    #[tokio::test]
    async fn test_simulated_failure() {
        let config = MvpProverConfig {
            generation_delay_ms: 10,
            simulate_failures: true,
            failure_rate: 1.0, // Always fail
        };
        
        let prover = MvpProverService::new(config);

        let result = prover.generate_proof_for_batch(
            1,
            "0x1111",
            "0x2222", 
            "0x3333",
            "0x4444",
            &[],
        ).await.unwrap();

        assert!(!result.success);
        assert!(result.proof.is_none());
        assert!(result.error_message.is_some());
    }
}
