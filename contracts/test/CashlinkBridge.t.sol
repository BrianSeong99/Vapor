// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/CashlinkBridge.sol";
import "../src/ProofVerifier.sol";
import "../src/MockUSDC.sol";
import "../src/interfaces/ICashlinkBridge.sol";
import "../src/interfaces/IProofVerifier.sol";

contract MockSP1Verifier {
    function verifyProof(
        bytes32 /* programVKey */,
        bytes calldata /* publicValues */,
        bytes calldata /* proof */
    ) external pure {
        // Always succeeds for testing
    }
}

contract CashlinkBridgeTest is Test {
    CashlinkBridge public bridge;
    ProofVerifier public verifier;
    MockUSDC public usdc;
    MockSP1Verifier public mockSP1Verifier;
    
    address public owner;
    address public user1;
    address public user2;
    
    bytes32 public constant PROGRAM_VKEY = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
    
    // Order types
    uint8 constant ORDER_TYPE_BRIDGE_OUT = 1;
    
    event Claimed(uint256 indexed batchId, uint256 indexed orderId, address indexed to, uint256 tokenId, uint256 amount);
    event Deposited(address indexed from, uint256 tokenId, uint256 amount, bytes32 indexed bankingHash);
    event TokenAdded(uint256 indexed tokenId, address indexed tokenAddress, string symbol);
    event TokenRemoved(uint256 indexed tokenId, address indexed tokenAddress);
    
    function setUp() public {
        owner = address(this);
        user1 = address(0x1);
        user2 = address(0x2);
        
        // Deploy contracts
        mockSP1Verifier = new MockSP1Verifier();
        verifier = new ProofVerifier(address(mockSP1Verifier), PROGRAM_VKEY, false);
        usdc = new MockUSDC();
        bridge = new CashlinkBridge(address(verifier));
        
        // Add USDC as supported token (token ID 1)
        bridge.addSupportedToken(1, address(usdc));
        
        // Setup initial balances
        usdc.mint(address(bridge), 1000000 * 10**6); // 1M USDC to bridge
        usdc.mint(user1, 10000 * 10**6); // 10k USDC to user1
        usdc.mint(user2, 10000 * 10**6); // 10k USDC to user2
    }
    
    function testInitialState() public {
        assertEq(address(bridge.proofVerifier()), address(verifier));
        assertEq(bridge.owner(), owner);
        assertEq(bridge.getSupportedToken(1), address(usdc));
        assertTrue(bridge.isTokenSupported(1));
        assertFalse(bridge.isTokenSupported(2));
        assertEq(bridge.getTokenBalance(1), 1000000 * 10**6);
    }
    
    function testDeposit() public {
        uint256 depositAmount = 1000 * 10**6; // 1000 USDC
        uint256 tokenId = 1; // USDC token ID
        bytes32 bankingHash = keccak256("bank_account_123_ref_456");
        
        vm.startPrank(user1);
        usdc.approve(address(bridge), depositAmount);
        
        vm.expectEmit(true, true, true, true);
        emit Deposited(user1, tokenId, depositAmount, bankingHash);
        
        bridge.deposit(tokenId, depositAmount, bankingHash);
        vm.stopPrank();
        
        // Check balances
        assertEq(usdc.balanceOf(user1), 9000 * 10**6); // 10k - 1k
        assertEq(bridge.getTokenBalance(tokenId), 1001000 * 10**6); // 1M + 1k
    }
    
    function testDepositZeroAmount() public {
        vm.prank(user1);
        vm.expectRevert("Amount must be greater than 0");
        bridge.deposit(1, 0, keccak256("test"));
    }
    
    function testDepositEmptyBankingHash() public {
        vm.prank(user1);
        vm.expectRevert("Banking hash cannot be empty");
        bridge.deposit(1, 1000 * 10**6, bytes32(0));
    }
    
    function testDepositInsufficientBalance() public {
        uint256 depositAmount = 20000 * 10**6; // More than user1 has
        
        vm.startPrank(user1);
        usdc.approve(address(bridge), depositAmount);
        // OpenZeppelin uses ERC20InsufficientBalance error
        vm.expectRevert();
        bridge.deposit(1, depositAmount, keccak256("test"));
        vm.stopPrank();
    }
    
    function testClaimSuccessful() public {
        // First, create a verified batch
        uint256 batchId = 1;
        uint256 orderId = 123;
        address recipient = user1;
        uint256 amount = 500 * 10**6; // 500 USDC
        
        uint256 tokenId = 1; // USDC token ID
        
        // Create a simple Merkle tree with one leaf
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId,
            orderId,
            ORDER_TYPE_BRIDGE_OUT,
            recipient,
            recipient,
            tokenId,
            amount
        ));
        
        bytes32 ordersRoot = orderLeaf; // Single leaf = root
        
        // Submit batch to verifier
        verifier.submitProof(
            batchId,
            0, // prevBatchId
            bytes32(0), // prevStateRoot
            bytes32(0), // prevOrdersRoot
            keccak256("newState"), // newStateRoot
            ordersRoot, // newOrdersRoot
            "mockProof"
        );
        
        // Create empty proof for single leaf
        bytes32[] memory proof = new bytes32[](0);
        
        uint256 bridgeBalanceBefore = bridge.getTokenBalance(tokenId);
        uint256 userBalanceBefore = usdc.balanceOf(recipient);
        
        vm.expectEmit(true, true, true, true);
        emit Claimed(batchId, orderId, recipient, tokenId, amount);
        
        // Claim the order
        bridge.claim(batchId, orderId, recipient, tokenId, amount, proof);
        
        // Check balances
        assertEq(bridge.getTokenBalance(tokenId), bridgeBalanceBefore - amount);
        assertEq(usdc.balanceOf(recipient), userBalanceBefore + amount);
        assertTrue(bridge.isClaimed(orderId));
    }
    
    function testClaimWithMerkleProof() public {
        // Create a Merkle tree with multiple leaves
        uint256 batchId = 1;
        uint256 orderId1 = 100;
        uint256 orderId2 = 101;
        address recipient1 = user1;
        address recipient2 = user2;
        uint256 amount1 = 300 * 10**6;
        uint256 amount2 = 400 * 10**6;
        
        // Create order leaves
        bytes32 leaf1 = keccak256(abi.encode(
            batchId, orderId1, ORDER_TYPE_BRIDGE_OUT, recipient1, recipient1, uint256(1), amount1
        ));
        bytes32 leaf2 = keccak256(abi.encode(
            batchId, orderId2, ORDER_TYPE_BRIDGE_OUT, recipient2, recipient2, uint256(1), amount2
        ));
        
        // Create Merkle tree: root = hash(leaf1, leaf2)
        bytes32 ordersRoot = keccak256(abi.encodePacked(leaf1, leaf2));
        
        // Submit batch
        verifier.submitProof(
            batchId, 0, bytes32(0), bytes32(0), keccak256("newState"), ordersRoot, "mockProof"
        );
        
        // Create proof for leaf1 (proof = [leaf2])
        bytes32[] memory proof = new bytes32[](1);
        proof[0] = leaf2;
        
        // Claim order 1 
        bridge.claim(batchId, orderId1, recipient1, 1, amount1, proof);
        
        assertTrue(bridge.isClaimed(orderId1));
        assertFalse(bridge.isClaimed(orderId2));
    }
    
    function testClaimAlreadyClaimed() public {
        // Setup and submit a valid batch
        uint256 batchId = 1;
        uint256 orderId = 123;
        address recipient = user1;
        uint256 amount = 500 * 10**6;
        
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId, orderId, ORDER_TYPE_BRIDGE_OUT, recipient, recipient, uint256(1), amount
        ));
        
        verifier.submitProof(
            batchId, 0, bytes32(0), bytes32(0), keccak256("newState"), orderLeaf, "mockProof"
        );
        
        bytes32[] memory proof = new bytes32[](0);
        
        // Claim once
        bridge.claim(batchId, orderId, recipient, 1, amount, proof);
        
        // Try to claim again
        vm.expectRevert(ICashlinkBridge.OrderAlreadyClaimed.selector);
        bridge.claim(batchId, orderId, recipient, 1, amount, proof);
    }
    
    function testClaimBatchNotVerified() public {
        uint256 batchId = 999; // Non-existent batch
        uint256 orderId = 123;
        address recipient = user1;
        uint256 amount = 500 * 10**6;
        bytes32[] memory proof = new bytes32[](0);
        
        vm.expectRevert(ICashlinkBridge.BatchNotVerified.selector);
        bridge.claim(batchId, orderId, recipient, 1, amount, proof);
    }
    
    function testClaimInvalidMerkleProof() public {
        // Setup batch
        uint256 batchId = 1;
        uint256 orderId = 123;
        address recipient = user1;
        uint256 amount = 500 * 10**6;
        
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId, orderId, ORDER_TYPE_BRIDGE_OUT, recipient, recipient, uint256(1), amount
        ));
        
        verifier.submitProof(
            batchId, 0, bytes32(0), bytes32(0), keccak256("newState"), orderLeaf, "mockProof"
        );
        
        // Provide wrong proof
        bytes32[] memory wrongProof = new bytes32[](1);
        wrongProof[0] = keccak256("wrongProof");
        
        vm.expectRevert(ICashlinkBridge.InvalidMerkleProof.selector);
        bridge.claim(batchId, orderId, recipient, 1, amount, wrongProof);
    }
    
    function testClaimInsufficientBalance() public {
        // Setup batch
        uint256 batchId = 1;
        uint256 orderId = 123;
        address recipient = user1;
        uint256 amount = 2000000 * 10**6; // More than bridge has
        
        bytes32 orderLeaf = keccak256(abi.encode(
            batchId, orderId, ORDER_TYPE_BRIDGE_OUT, recipient, recipient, uint256(1), amount
        ));
        
        verifier.submitProof(
            batchId, 0, bytes32(0), bytes32(0), keccak256("newState"), orderLeaf, "mockProof"
        );
        
        bytes32[] memory proof = new bytes32[](0);
        
        vm.expectRevert(ICashlinkBridge.InsufficientBalance.selector);
        bridge.claim(batchId, orderId, recipient, 1, amount, proof);
    }
    
    function testEmergencyWithdraw() public {
        uint256 withdrawAmount = 10000 * 10**6;
        uint256 tokenId = 1;
        uint256 balanceBefore = bridge.getTokenBalance(tokenId);
        uint256 userBalanceBefore = usdc.balanceOf(user1);
        
        bridge.emergencyWithdraw(tokenId, user1, withdrawAmount);
        
        assertEq(bridge.getTokenBalance(tokenId), balanceBefore - withdrawAmount);
        assertEq(usdc.balanceOf(user1), userBalanceBefore + withdrawAmount);
    }
    
    function testEmergencyWithdrawOnlyOwner() public {
        vm.prank(user1);
        vm.expectRevert("Not authorized");
        bridge.emergencyWithdraw(1, user1, 1000 * 10**6);
    }
    
    function testTokenManagement() public {
        // Test adding a new token (use actual MockUSDC as newToken)
        MockUSDC newToken = new MockUSDC();
        uint256 newTokenId = 2;
        
        vm.expectEmit(true, true, false, true);
        emit TokenAdded(newTokenId, address(newToken), "USDC");
        
        bridge.addSupportedToken(newTokenId, address(newToken));
        
        assertTrue(bridge.isTokenSupported(newTokenId));
        assertEq(bridge.getSupportedToken(newTokenId), address(newToken));
        
        // Test removing token
        vm.expectEmit(true, true, false, false);
        emit TokenRemoved(newTokenId, address(newToken));
        
        bridge.removeSupportedToken(newTokenId);
        
        assertFalse(bridge.isTokenSupported(newTokenId));
        assertEq(bridge.getSupportedToken(newTokenId), address(0));
    }
    
    function testIsClaimedInitiallyFalse() public {
        assertFalse(bridge.isClaimed(123));
        assertFalse(bridge.isClaimed(999));
    }
}
