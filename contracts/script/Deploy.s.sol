// SPDX-License-Identifier: MIT
pragma solidity ^0.8.19;

import "forge-std/Script.sol";
import "../src/ProofVerifier.sol";
import "../src/VaporBridge.sol";
import "../src/MockUSDC.sol";

contract DeployScript is Script {
    // SP1 Verifier addresses for different networks
    // For production, use actual deployed SP1 verifiers
    // For MVP/testing, can use mock or placeholder
    address constant SP1_VERIFIER_MAINNET = 0x3B6041173B80E77f038f3F2C0f9744f04837185e;
    address constant SP1_VERIFIER_SEPOLIA = 0x3B6041173B80E77f038f3F2C0f9744f04837185e;
    
    // Program verification key (would be generated from SP1 program)
    bytes32 constant PROGRAM_VKEY = 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef;
    
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        console.log("Deploying contracts with the account:", deployer);
        console.log("Account balance:", deployer.balance);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy contracts based on network
        (address sp1Verifier, address usdcToken) = _getNetworkAddresses();
        
        // Deploy ProofVerifier
        // Start with MVP mode (SP1 disabled) for easy testing
        ProofVerifier proofVerifier = new ProofVerifier(
            sp1Verifier,
            PROGRAM_VKEY,
            false // useActualSP1Verification
        );
        
        // Deploy VaporBridge
        VaporBridge bridge = new VaporBridge(
            address(proofVerifier)
        );
        
        // Add USDC as token ID 1
        bridge.addSupportedToken(1, usdcToken);
        
        vm.stopBroadcast();
        
        // Log deployed addresses
        console.log("=== Deployment Complete ===");
        console.log("ProofVerifier deployed to:", address(proofVerifier));
        console.log("VaporBridge deployed to:", address(bridge));
        console.log("USDC Token address:", usdcToken);
        console.log("SP1 Verifier address:", sp1Verifier);
        console.log("==============================");
        
        // Save deployment info
        _saveDeploymentInfo(address(proofVerifier), address(bridge), usdcToken, sp1Verifier);
    }
    
    function _getNetworkAddresses() internal returns (address sp1Verifier, address usdcToken) {
        uint256 chainId = block.chainid;
        
        if (chainId == 1) {
            // Mainnet
            sp1Verifier = SP1_VERIFIER_MAINNET;
            usdcToken = 0xA0b86A33e6441C41c7C9c6C22E23Ed0b5c50d111; // Actual USDC on mainnet
        } else if (chainId == 11155111) {
            // Sepolia
            sp1Verifier = SP1_VERIFIER_SEPOLIA;
            usdcToken = 0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238; // USDC on Sepolia
        } else {
            // Local/test network - deploy mock contracts
            console.log("Deploying mock contracts for local/test network");
            
            // Deploy mock SP1 verifier
            MockSP1Verifier mockSP1 = new MockSP1Verifier();
            sp1Verifier = address(mockSP1);
            
            // Deploy mock USDC
            MockUSDC mockUSDC = new MockUSDC();
            usdcToken = address(mockUSDC);
            
            console.log("Mock SP1 Verifier deployed to:", sp1Verifier);
            console.log("Mock USDC deployed to:", usdcToken);
        }
    }
    
    function _saveDeploymentInfo(
        address proofVerifier,
        address bridge,
        address usdc,
        address sp1Verifier
    ) internal {
        string memory json = "deployment";
        vm.serializeAddress(json, "proofVerifier", proofVerifier);
        vm.serializeAddress(json, "bridge", bridge);
        vm.serializeAddress(json, "usdc", usdc);
        vm.serializeAddress(json, "sp1Verifier", sp1Verifier);
        vm.serializeUint(json, "chainId", block.chainid);
        vm.serializeUint(json, "blockNumber", block.number);
        
        string memory finalJson = vm.serializeUint(json, "timestamp", block.timestamp);
        
        string memory fileName = string.concat("deployments/", vm.toString(block.chainid), ".json");
        vm.writeJson(finalJson, fileName);
        
        console.log("Deployment info saved to:", fileName);
    }
}

// Mock SP1 Verifier for local testing
contract MockSP1Verifier {
    function verifyProof(
        bytes32 /* programVKey */,
        bytes calldata /* publicValues */,
        bytes calldata /* proof */
    ) external pure {
        // Always succeeds for testing
    }
}
