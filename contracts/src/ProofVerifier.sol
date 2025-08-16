// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/IProofVerifier.sol";
import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

/**
 * @title ProofVerifier
 * @dev Manages batch state and order roots with ZK proof verification
 */
contract ProofVerifier is IProofVerifier {
    // Public inputs structure for SP1 verification
    struct PublicInputs {
        uint256 batchId;
        bytes32 prevStateRoot;
        bytes32 prevOrdersRoot;
        bytes32 newStateRoot;
        bytes32 newOrdersRoot;
    }

    // Mapping from batch ID to batch data
    mapping(uint256 => Batch) public batches;
    
    // Latest batch ID
    uint256 public latestBatchId;
    
    // For MVP: Simple owner-based access control  
    address public owner;
    
    // SP1 verification components
    ISP1Verifier public immutable sp1Verifier;
    bytes32 public immutable programVKey;
    
    // For MVP: Enable/disable actual SP1 verification
    bool public useActualSP1Verification;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }
    
    constructor(address _sp1Verifier, bytes32 _programVKey, bool _useActualSP1Verification) {
        owner = msg.sender;
        sp1Verifier = ISP1Verifier(_sp1Verifier);
        programVKey = _programVKey;
        useActualSP1Verification = _useActualSP1Verification;
        
        // Initialize genesis batch (batch 0) with empty roots
        batches[0] = Batch({
            stateRoot: bytes32(0),
            ordersRoot: bytes32(0)
        });
        latestBatchId = 0;
    }
    
    /**
     * @dev Submit a ZK proof for a new batch
     * For MVP: Simplified validation without actual ZK verification
     */
    function submitProof(
        uint256 batchId,
        uint256 prevBatchId,
        bytes32 prevStateRoot,
        bytes32 prevOrdersRoot,
        bytes32 newStateRoot,
        bytes32 newOrdersRoot,
        bytes calldata proof
    ) external onlyOwner {
        // Validate batch ID sequence
        if (batchId != latestBatchId + 1) {
            revert InvalidBatchId();
        }
        
        // Validate previous batch exists and roots match
        Batch memory prevBatch = batches[prevBatchId];
        if (prevBatch.stateRoot != prevStateRoot || prevBatch.ordersRoot != prevOrdersRoot) {
            revert InvalidBatchId();
        }
        
        // Verify proof based on configuration
        if (useActualSP1Verification) {
            _verifyProofSP1(proof, batchId, prevStateRoot, prevOrdersRoot, newStateRoot, newOrdersRoot);
        } else {
            _verifyProofMVP(proof, batchId, prevStateRoot, prevOrdersRoot, newStateRoot, newOrdersRoot);
        }
        
        // Store the new batch
        batches[batchId] = Batch({
            stateRoot: newStateRoot,
            ordersRoot: newOrdersRoot
        });
        
        // Update latest batch ID
        latestBatchId = batchId;
        
        emit ProofSubmitted(batchId, newStateRoot, newOrdersRoot);
    }
    
    /**
     * @dev Get the state root for a specific batch
     */
    function getStateRoot(uint256 batchId) external view returns (bytes32) {
        return batches[batchId].stateRoot;
    }
    
    /**
     * @dev Get the orders root for a specific batch
     */
    function getOrdersRoot(uint256 batchId) external view returns (bytes32) {
        return batches[batchId].ordersRoot;
    }
    
    /**
     * @dev Get the latest batch ID
     */
    function getLatestBatchId() external view returns (uint256) {
        return latestBatchId;
    }
    
    /**
     * @dev Get the batch data for a specific batch ID
     */
    function getBatch(uint256 batchId) external view returns (Batch memory) {
        return batches[batchId];
    }
    
    /**
     * @dev MVP proof verification (simplified)
     * @notice In production, this would be replaced with actual SP1 verification
     */
    function _verifyProofMVP(
        bytes calldata proof,
        uint256 /* batchId */,
        bytes32 /* prevStateRoot */,
        bytes32 /* prevOrdersRoot */,
        bytes32 /* newStateRoot */,
        bytes32 /* newOrdersRoot */
    ) internal pure {
        // Basic validation - proof must not be empty
        if (proof.length == 0) {
            revert InvalidProof();
        }
        
        // MVP: Accept any non-empty proof
        // TODO: Replace with actual SP1 verification call
    }
    
    /**
     * @dev Production SP1 proof verification
     * @notice Uses the official SP1 verifier contract as per documentation
     */
    function _verifyProofSP1(
        bytes calldata proof,
        uint256 batchId,
        bytes32 prevStateRoot,
        bytes32 prevOrdersRoot,
        bytes32 newStateRoot,
        bytes32 newOrdersRoot
    ) internal view {
        // Encode public inputs for SP1 verification
        PublicInputs memory publicInputs = PublicInputs({
            batchId: batchId,
            prevStateRoot: prevStateRoot,
            prevOrdersRoot: prevOrdersRoot,
            newStateRoot: newStateRoot,
            newOrdersRoot: newOrdersRoot
        });
        
        bytes memory publicInputsBytes = abi.encode(publicInputs);
        
        // Verify SP1 proof using the official verifier
        try sp1Verifier.verifyProof(programVKey, publicInputsBytes, proof) {
            // Proof verified successfully
        } catch {
            revert InvalidProof();
        }
    }
    
    /**
     * @dev Toggle SP1 verification mode (MVP helper function)
     * @notice Only owner can change verification mode
     */
    function setUseActualSP1Verification(bool _useActualSP1Verification) external onlyOwner {
        useActualSP1Verification = _useActualSP1Verification;
    }
}
