// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Test.sol";
import "../src/ProofVerifier.sol";
import "../src/interfaces/IProofVerifier.sol";

contract MockSP1Verifier {
    mapping(bytes32 => bool) public shouldFail;
    
    function verifyProof(
        bytes32 /* programVKey */,
        bytes calldata /* publicValues */,
        bytes calldata proof
    ) external view {
        bytes32 proofHash = keccak256(proof);
        if (shouldFail[proofHash]) {
            revert("Proof verification failed");
        }
        // Success - no revert
    }
    
    function setShouldFail(bytes calldata proof, bool _shouldFail) external {
        bytes32 proofHash = keccak256(proof);
        shouldFail[proofHash] = _shouldFail;
    }
}

contract ProofVerifierTest is Test {
    ProofVerifier public verifier;
    MockSP1Verifier public mockSP1Verifier;
    
    address public owner;
    address public user;
    
    bytes32 public constant PROGRAM_VKEY = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
    
    event ProofSubmitted(
        uint256 indexed batchId,
        bytes32 newStateRoot,
        bytes32 newOrdersRoot
    );
    
    function setUp() public {
        owner = address(this);
        user = address(0x1);
        
        // Deploy mock SP1 verifier
        mockSP1Verifier = new MockSP1Verifier();
        
        // Deploy ProofVerifier with MVP mode enabled (SP1 disabled for tests)
        verifier = new ProofVerifier(
            address(mockSP1Verifier),
            PROGRAM_VKEY,
            false // Start with MVP mode
        );
    }
    
    function testInitialState() public {
        assertEq(verifier.latestBatchId(), 0);
        assertEq(verifier.owner(), owner);
        assertEq(address(verifier.sp1Verifier()), address(mockSP1Verifier));
        assertEq(verifier.programVKey(), PROGRAM_VKEY);
        assertFalse(verifier.useActualSP1Verification());
        
        // Check genesis batch
        IProofVerifier.Batch memory genesisBatch = verifier.getBatch(0);
        assertEq(genesisBatch.stateRoot, bytes32(0));
        assertEq(genesisBatch.ordersRoot, bytes32(0));
    }
    
    function testSubmitProofMVPMode() public {
        bytes32 prevStateRoot = bytes32(0);
        bytes32 prevOrdersRoot = bytes32(0);
        bytes32 newStateRoot = keccak256("newState1");
        bytes32 newOrdersRoot = keccak256("newOrders1");
        bytes memory proof = abi.encode("mockProof1");
        
        vm.expectEmit(true, false, false, true);
        emit ProofSubmitted(1, newStateRoot, newOrdersRoot);
        
        verifier.submitProof(
            1, // batchId
            0, // prevBatchId
            prevStateRoot,
            prevOrdersRoot,
            newStateRoot,
            newOrdersRoot,
            proof
        );
        
        // Verify batch was stored
        assertEq(verifier.latestBatchId(), 1);
        IProofVerifier.Batch memory batch = verifier.getBatch(1);
        assertEq(batch.stateRoot, newStateRoot);
        assertEq(batch.ordersRoot, newOrdersRoot);
        
        // Verify view functions
        assertEq(verifier.getStateRoot(1), newStateRoot);
        assertEq(verifier.getOrdersRoot(1), newOrdersRoot);
        assertEq(verifier.getLatestBatchId(), 1);
    }
    
    function testSubmitProofSP1Mode() public {
        // Enable SP1 verification
        verifier.setUseActualSP1Verification(true);
        assertTrue(verifier.useActualSP1Verification());
        
        bytes32 newStateRoot = keccak256("newState1");
        bytes32 newOrdersRoot = keccak256("newOrders1");
        bytes memory proof = abi.encode("mockProofSP1");
        
        // Should succeed with valid proof
        verifier.submitProof(
            1, // batchId
            0, // prevBatchId
            bytes32(0), // prevStateRoot
            bytes32(0), // prevOrdersRoot
            newStateRoot,
            newOrdersRoot,
            proof
        );
        
        assertEq(verifier.latestBatchId(), 1);
    }
    
    function testSubmitProofSP1ModeFailure() public {
        verifier.setUseActualSP1Verification(true);
        
        bytes memory proof = abi.encode("invalidProof");
        // Set this proof to fail in mock verifier
        mockSP1Verifier.setShouldFail(proof, true);
        
        vm.expectRevert(IProofVerifier.InvalidProof.selector);
        verifier.submitProof(
            1,
            0,
            bytes32(0),
            bytes32(0),
            keccak256("newState"),
            keccak256("newOrders"),
            proof
        );
    }
    
    function testSubmitProofSequentialBatches() public {
        // Submit batch 1
        bytes32 state1 = keccak256("state1");
        bytes32 orders1 = keccak256("orders1");
        verifier.submitProof(1, 0, bytes32(0), bytes32(0), state1, orders1, "proof1");
        
        // Submit batch 2
        bytes32 state2 = keccak256("state2");
        bytes32 orders2 = keccak256("orders2");
        verifier.submitProof(2, 1, state1, orders1, state2, orders2, "proof2");
        
        assertEq(verifier.latestBatchId(), 2);
        assertEq(verifier.getStateRoot(2), state2);
        assertEq(verifier.getOrdersRoot(2), orders2);
    }
    
    function testSubmitProofInvalidBatchId() public {
        // Try to submit batch 2 when latest is 0
        vm.expectRevert(IProofVerifier.InvalidBatchId.selector);
        verifier.submitProof(
            2, // Should be 1
            0,
            bytes32(0),
            bytes32(0),
            keccak256("state"),
            keccak256("orders"),
            "proof"
        );
    }
    
    function testSubmitProofInvalidPreviousRoots() public {
        // Submit batch 1 first
        bytes32 state1 = keccak256("state1");
        bytes32 orders1 = keccak256("orders1");
        verifier.submitProof(1, 0, bytes32(0), bytes32(0), state1, orders1, "proof1");
        
        // Try to submit batch 2 with wrong previous roots
        vm.expectRevert(IProofVerifier.InvalidBatchId.selector);
        verifier.submitProof(
            2,
            1,
            keccak256("wrongState"), // Should be state1
            orders1,
            keccak256("state2"),
            keccak256("orders2"),
            "proof2"
        );
    }
    
    function testSubmitProofEmptyProofMVPMode() public {
        vm.expectRevert(IProofVerifier.InvalidProof.selector);
        verifier.submitProof(
            1,
            0,
            bytes32(0),
            bytes32(0),
            keccak256("state"),
            keccak256("orders"),
            "" // Empty proof
        );
    }
    
    function testOnlyOwnerModifier() public {
        vm.prank(user);
        vm.expectRevert("Not authorized");
        verifier.submitProof(
            1,
            0,
            bytes32(0),
            bytes32(0),
            keccak256("state"),
            keccak256("orders"),
            "proof"
        );
        
        vm.prank(user);
        vm.expectRevert("Not authorized");
        verifier.setUseActualSP1Verification(true);
    }
    
    function testGetBatchNonexistent() public {
        IProofVerifier.Batch memory batch = verifier.getBatch(999);
        assertEq(batch.stateRoot, bytes32(0));
        assertEq(batch.ordersRoot, bytes32(0));
    }
    
    function testToggleVerificationMode() public {
        // Start in MVP mode
        assertFalse(verifier.useActualSP1Verification());
        
        // Toggle to SP1 mode
        verifier.setUseActualSP1Verification(true);
        assertTrue(verifier.useActualSP1Verification());
        
        // Toggle back to MVP mode
        verifier.setUseActualSP1Verification(false);
        assertFalse(verifier.useActualSP1Verification());
    }
    
    function testFuzzSubmitProof(
        bytes32 newStateRoot,
        bytes32 newOrdersRoot,
        bytes calldata proof
    ) public {
        vm.assume(proof.length > 0);
        
        // Simple single batch submission
        verifier.submitProof(
            1,
            0,
            bytes32(0),
            bytes32(0),
            newStateRoot,
            newOrdersRoot,
            proof
        );
        
        assertEq(verifier.latestBatchId(), 1);
        assertEq(verifier.getStateRoot(1), newStateRoot);
        assertEq(verifier.getOrdersRoot(1), newOrdersRoot);
    }
}
