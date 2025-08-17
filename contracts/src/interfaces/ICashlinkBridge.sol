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
        uint256 tokenId,
        uint256 amount
    );

    event BatchClaimed(
        uint256 indexed batchId,
        address indexed filler,
        uint256 totalClaims,
        uint256 totalAmount
    );

    event Deposited(
        address indexed from,
        uint256 tokenId,
        uint256 amount,
        bytes32 indexed bankingHash
    );

    event TokenAdded(
        uint256 indexed tokenId,
        address indexed tokenAddress,
        string symbol
    );

    event TokenRemoved(
        uint256 indexed tokenId,
        address indexed tokenAddress
    );

    // Batch claim data structures
    struct ClaimData {
        uint256 orderId;
        address destinationAddress; // Where tokens should be sent
        uint256 tokenId;
        uint256 amount;
        bytes32[] merkleProof;
    }

    // Errors
    error InvalidMerkleProof();
    error OrderAlreadyClaimed();
    error BatchNotVerified();
    error InsufficientBalance();
    error TokenNotSupported();
    error TokenAlreadyExists();
    error InvalidTokenAddress();
    error EmptyClaimsArray();
    error MismatchedArrayLengths();

    /**
     * @dev Claim tokens using a Merkle proof for a BridgeOut order
     * @param batchId The batch ID containing the order
     * @param orderId The order ID to claim
     * @param to The recipient address
     * @param tokenId The token ID to claim
     * @param amount The amount to claim
     * @param merkleProof The Merkle proof for the order
     */
    function claim(
        uint256 batchId,
        uint256 orderId,
        address to,
        uint256 tokenId,
        uint256 amount,
        bytes32[] calldata merkleProof
    ) external;

    /**
     * @dev Batch claim tokens for multiple wallets using Merkle proofs
     * @param batchId The batch ID containing the orders
     * @param claims Array of claim data for each wallet
     */
    function batchClaim(
        uint256 batchId,
        ClaimData[] calldata claims
    ) external;

    /**
     * @dev Deposit tokens to trigger a BridgeIn order
     * @param tokenId The token ID to deposit
     * @param amount The amount to deposit
     * @param bankingHash Hash of the banking transfer information (account details, reference, etc.)
     */
    function deposit(uint256 tokenId, uint256 amount, bytes32 bankingHash) external;

    /**
     * @dev Check if an order has been claimed
     * @param orderId The order ID
     * @return True if claimed, false otherwise
     */
    function isClaimed(uint256 orderId) external view returns (bool);

    /**
     * @dev Add a supported ERC20 token
     * @param tokenId The token ID to assign
     * @param tokenAddress The ERC20 token address
     */
    function addSupportedToken(uint256 tokenId, address tokenAddress) external;

    /**
     * @dev Remove a supported ERC20 token
     * @param tokenId The token ID to remove
     */
    function removeSupportedToken(uint256 tokenId) external;

    /**
     * @dev Get the token address for a given token ID
     * @param tokenId The token ID
     * @return The token address
     */
    function getSupportedToken(uint256 tokenId) external view returns (address);

    /**
     * @dev Check if a token ID is supported
     * @param tokenId The token ID
     * @return True if supported, false otherwise
     */
    function isTokenSupported(uint256 tokenId) external view returns (bool);

    /**
     * @dev Get the contract balance for a specific token
     * @param tokenId The token ID
     * @return The token balance of this contract
     */
    function getTokenBalance(uint256 tokenId) external view returns (uint256);
}
