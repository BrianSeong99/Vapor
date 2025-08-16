// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ICashlinkBridge.sol";
import "./interfaces/IProofVerifier.sol";

// For production, we'd use actual USDC
// For MVP, we'll use a simple mock token
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

/**
 * @title CashlinkBridge
 * @dev Handles USDC claims using Merkle proofs from verified batches
 */
contract CashlinkBridge is ICashlinkBridge {
    // Order types for Merkle leaf validation
    uint8 constant ORDER_TYPE_BRIDGE_IN = 0;
    uint8 constant ORDER_TYPE_BRIDGE_OUT = 1;
    uint8 constant ORDER_TYPE_TRANSFER = 2;

    // State variables
    IProofVerifier public immutable proofVerifier;
    IERC20 public immutable usdcToken;
    
    // Mapping to track claimed orders
    mapping(uint256 => bool) public claimed;
    
    // For MVP: Owner can manage the contract
    address public owner;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }
    
    constructor(address _proofVerifier, address _usdcToken) {
        proofVerifier = IProofVerifier(_proofVerifier);
        usdcToken = IERC20(_usdcToken);
        owner = msg.sender;
    }
    
    /**
     * @dev Claim USDC using a Merkle proof for a BridgeOut order
     * @param batchId The batch ID containing the order
     * @param orderId The order ID to claim
     * @param to The recipient address
     * @param amount The amount to claim
     * @param merkleProof The Merkle proof for the order
     */
    function claim(
        uint256 batchId,
        uint256 orderId,
        address to,
        uint256 amount,
        bytes32[] calldata merkleProof
    ) external override {
        // Check if order has already been claimed
        if (claimed[orderId]) {
            revert OrderAlreadyClaimed();
        }
        
        // Get the batch data from the proof verifier
        IProofVerifier.Batch memory batch = proofVerifier.getBatch(batchId);
        
        // Check if batch exists (verified)
        if (batch.ordersRoot == bytes32(0)) {
            revert BatchNotVerified();
        }
        
        // Construct the order leaf for BridgeOut
        // Leaf format: keccak256(abi.encode(batchId, orderId, ORDER_TYPE_BRIDGE_OUT, to, to, tokenId, amount))
        // Note: Using tokenId = 1 for USDC, from/to both set to recipient for BridgeOut
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId,
            orderId,
            ORDER_TYPE_BRIDGE_OUT,
            to,      // from (not really used for BridgeOut)
            to,      // to
            uint256(1), // tokenId (1 = USDC)
            amount
        ));
        
        // Verify Merkle proof
        if (!_verifyMerkleProof(merkleProof, orderLeaf, batch.ordersRoot)) {
            revert InvalidMerkleProof();
        }
        
        // Check contract has sufficient balance
        if (usdcToken.balanceOf(address(this)) < amount) {
            revert InsufficientBalance();
        }
        
        // Mark order as claimed
        claimed[orderId] = true;
        
        // Transfer USDC to recipient
        require(usdcToken.transfer(to, amount), "USDC transfer failed");
        
        emit Claimed(batchId, orderId, to, amount);
    }
    
    /**
     * @dev Deposit USDC to trigger a BridgeIn order
     * @param amount The amount to deposit
     */
    function deposit(uint256 amount) external override {
        require(amount > 0, "Amount must be greater than 0");
        
        // Transfer USDC from user to this contract
        require(
            usdcToken.transferFrom(msg.sender, address(this), amount),
            "USDC transfer failed"
        );
        
        emit Deposited(msg.sender, amount);
    }
    
    /**
     * @dev Check if an order has been claimed
     * @param orderId The order ID
     * @return True if claimed, false otherwise
     */
    function isClaimed(uint256 orderId) external view override returns (bool) {
        return claimed[orderId];
    }
    
    /**
     * @dev Get the USDC token address
     * @return The USDC token address
     */
    function getUSDCToken() external view override returns (address) {
        return address(usdcToken);
    }
    
    /**
     * @dev Verify a Merkle proof
     * @param proof The Merkle proof
     * @param leaf The leaf to verify
     * @param root The Merkle root
     * @return True if proof is valid
     */
    function _verifyMerkleProof(
        bytes32[] memory proof,
        bytes32 leaf,
        bytes32 root
    ) internal pure returns (bool) {
        bytes32 computedHash = leaf;
        
        for (uint256 i = 0; i < proof.length; i++) {
            bytes32 proofElement = proof[i];
            
            if (computedHash <= proofElement) {
                // Hash(current computed hash + current element of the proof)
                computedHash = keccak256(abi.encodePacked(computedHash, proofElement));
            } else {
                // Hash(current element of the proof + current computed hash)
                computedHash = keccak256(abi.encodePacked(proofElement, computedHash));
            }
        }
        
        return computedHash == root;
    }
    
    /**
     * @dev Emergency function to withdraw USDC (MVP only)
     * @param to Recipient address
     * @param amount Amount to withdraw
     */
    function emergencyWithdraw(address to, uint256 amount) external onlyOwner {
        require(usdcToken.transfer(to, amount), "USDC transfer failed");
    }
    
    /**
     * @dev Get contract USDC balance
     * @return The USDC balance of this contract
     */
    function getBalance() external view returns (uint256) {
        return usdcToken.balanceOf(address(this));
    }
}
