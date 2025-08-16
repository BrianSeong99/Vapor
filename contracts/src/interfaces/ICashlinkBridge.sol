// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

/**
 * @title ICashlinkBridge
 * @dev Interface for the Cashlink Bridge
 * Handles USDC claims using Merkle proofs from verified batches
 */
interface ICashlinkBridge {
    // Events
    event Claimed(
        uint256 indexed batchId,
        uint256 indexed orderId,
        address indexed to,
        uint256 amount
    );

    event Deposited(
        address indexed from,
        uint256 amount
    );

    // Errors
    error InvalidMerkleProof();
    error OrderAlreadyClaimed();
    error BatchNotVerified();
    error InsufficientBalance();

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
    ) external;

    /**
     * @dev Deposit USDC to trigger a BridgeIn order
     * @param amount The amount to deposit
     */
    function deposit(uint256 amount) external;

    /**
     * @dev Check if an order has been claimed
     * @param orderId The order ID
     * @return True if claimed, false otherwise
     */
    function isClaimed(uint256 orderId) external view returns (bool);

    /**
     * @dev Get the USDC token address
     * @return The USDC token address
     */
    function getUSDCToken() external view returns (address);
}
