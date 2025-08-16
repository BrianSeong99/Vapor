// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "./interfaces/ICashlinkBridge.sol";
import "./interfaces/IProofVerifier.sol";

// Standard ERC20 interface
interface IERC20 {
    function transfer(address to, uint256 amount) external returns (bool);
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
    function balanceOf(address account) external view returns (uint256);
}

// Extended ERC20 interface with symbol
interface IERC20Extended {
    function symbol() external view returns (string memory);
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
    
    // Mapping to track claimed orders / nullifier
    mapping(uint256 => bool) public claimed;
    
    // Mapping from token ID to ERC20 token address
    mapping(uint256 => address) public supportedTokens;
    
    // For MVP: Owner can manage the contract
    address public owner;
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not authorized");
        _;
    }
    
    constructor(address _proofVerifier) {
        proofVerifier = IProofVerifier(_proofVerifier);
        owner = msg.sender;
    }
    
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
    ) external override {
        // Check if order has already been claimed
        if (claimed[orderId]) {
            revert OrderAlreadyClaimed();
        }
        
        // Get the batch data from the proof verifier
        IProofVerifier.Batch memory batch = proofVerifier.getBatch(batchId);
        
        // Check if token is supported
        address tokenAddress = supportedTokens[tokenId];
        if (tokenAddress == address(0)) {
            revert TokenNotSupported();
        }
        
        // Check if batch exists (verified)
        if (batch.ordersRoot == bytes32(0)) {
            revert BatchNotVerified();
        }
        
        // Construct the order leaf for BridgeOut
        // Leaf format: keccak256(abi.encode(batchId, orderId, ORDER_TYPE_BRIDGE_OUT, to, to, tokenId, amount))
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId,
            orderId,
            ORDER_TYPE_BRIDGE_OUT,
            to,      // from (not really used for BridgeOut)
            to,      // to
            tokenId, // tokenId
            amount
        ));
        
        // Verify Merkle proof
        if (!_verifyMerkleProof(merkleProof, orderLeaf, batch.ordersRoot)) {
            revert InvalidMerkleProof();
        }
        
        // Get token contract
        IERC20 token = IERC20(tokenAddress);
        
        // Check contract has sufficient balance
        if (token.balanceOf(address(this)) < amount) {
            revert InsufficientBalance();
        }
        
        // Mark order as claimed
        claimed[orderId] = true;
        
        // Transfer tokens to recipient
        require(token.transfer(to, amount), "Token transfer failed");
        
        emit Claimed(batchId, orderId, to, tokenId, amount);
    }
    
    /**
     * @dev Deposit tokens to trigger a BridgeIn order
     * @param tokenId The token ID to deposit
     * @param amount The amount to deposit
     */
    function deposit(uint256 tokenId, uint256 amount) external override {
        require(amount > 0, "Amount must be greater than 0");
        
        // Check if token is supported
        address tokenAddress = supportedTokens[tokenId];
        if (tokenAddress == address(0)) {
            revert TokenNotSupported();
        }
        
        // Get token contract
        IERC20 token = IERC20(tokenAddress);
        
        // Transfer tokens from user to this contract
        require(
            token.transferFrom(msg.sender, address(this), amount),
            "Token transfer failed"
        );
        
        emit Deposited(msg.sender, tokenId, amount);
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
     * @dev Add a supported ERC20 token
     * @param tokenId The token ID to assign
     * @param tokenAddress The ERC20 token address
     */
    function addSupportedToken(uint256 tokenId, address tokenAddress) external override onlyOwner {
        if (tokenAddress == address(0)) {
            revert InvalidTokenAddress();
        }
        
        if (supportedTokens[tokenId] != address(0)) {
            revert TokenAlreadyExists();
        }
        
        supportedTokens[tokenId] = tokenAddress;
        
        // Get token symbol for event (with fallback)
        string memory symbol = "UNKNOWN";
        try IERC20Extended(tokenAddress).symbol() returns (string memory _symbol) {
            symbol = _symbol;
        } catch {}
        
        emit TokenAdded(tokenId, tokenAddress, symbol);
    }

    /**
     * @dev Remove a supported ERC20 token
     * @param tokenId The token ID to remove
     */
    function removeSupportedToken(uint256 tokenId) external override onlyOwner {
        address tokenAddress = supportedTokens[tokenId];
        if (tokenAddress == address(0)) {
            revert TokenNotSupported();
        }
        
        delete supportedTokens[tokenId];
        
        emit TokenRemoved(tokenId, tokenAddress);
    }

    /**
     * @dev Get the token address for a given token ID
     * @param tokenId The token ID
     * @return The token address
     */
    function getSupportedToken(uint256 tokenId) external view override returns (address) {
        return supportedTokens[tokenId];
    }

    /**
     * @dev Check if a token ID is supported
     * @param tokenId The token ID
     * @return True if supported, false otherwise
     */
    function isTokenSupported(uint256 tokenId) external view override returns (bool) {
        return supportedTokens[tokenId] != address(0);
    }

    /**
     * @dev Get the contract balance for a specific token
     * @param tokenId The token ID
     * @return The token balance of this contract
     */
    function getTokenBalance(uint256 tokenId) external view override returns (uint256) {
        address tokenAddress = supportedTokens[tokenId];
        if (tokenAddress == address(0)) {
            return 0;
        }
        
        return IERC20(tokenAddress).balanceOf(address(this));
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
     * @dev Emergency function to withdraw tokens (MVP only)
     * @param tokenId The token ID to withdraw
     * @param to Recipient address
     * @param amount Amount to withdraw
     */
    function emergencyWithdraw(uint256 tokenId, address to, uint256 amount) external onlyOwner {
        address tokenAddress = supportedTokens[tokenId];
        if (tokenAddress == address(0)) {
            revert TokenNotSupported();
        }
        
        IERC20 token = IERC20(tokenAddress);
        require(token.transfer(to, amount), "Token transfer failed");
    }
}
