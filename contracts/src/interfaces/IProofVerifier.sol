// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title IProofVerifier
 * @dev Interface for the Cashlink Proof Verifier
 * Manages batch state and order roots with ZK proof verification
 */
interface IProofVerifier {
    // Events
    event ProofSubmitted(
        uint256 indexed batchId,
        bytes32 newStateRoot,
        bytes32 newOrdersRoot
    );

    // Errors
    error InvalidBatchId();
    error InvalidProof();
    error BatchAlreadyExists();

    /**
     * @dev Submit a ZK proof for a new batch
     * @param batchId The ID of the new batch
     * @param prevBatchId The ID of the previous batch
     * @param prevStateRoot The previous state root
     * @param prevOrdersRoot The previous orders root
     * @param newStateRoot The new state root after applying orders
     * @param newOrdersRoot The new orders root after applying orders
     * @param proof The ZK proof bytes
     */
    function submitProof(
        uint256 batchId,
        uint256 prevBatchId,
        bytes32 prevStateRoot,
        bytes32 prevOrdersRoot,
        bytes32 newStateRoot,
        bytes32 newOrdersRoot,
        bytes calldata proof
    ) external;

    /**
     * @dev Get the state root for a specific batch
     * @param batchId The batch ID
     * @return The state root
     */
    function getStateRoot(uint256 batchId) external view returns (bytes32);

    /**
     * @dev Get the orders root for a specific batch
     * @param batchId The batch ID
     * @return The orders root
     */
    function getOrdersRoot(uint256 batchId) external view returns (bytes32);

    /**
     * @dev Get the latest batch ID
     * @return The latest batch ID
     */
    function getLatestBatchId() external view returns (uint256);
}
