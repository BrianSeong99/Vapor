#!/bin/bash

# Start Anvil blockchain and deploy Vapor contracts only
# Does not start backend or frontend

set -e

echo "🔗 Starting Anvil Blockchain with Vapor Contracts..."

# Configuration
ANVIL_PORT=8545

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_step() { echo -e "${BLUE}🔄 $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_info() { echo -e "${YELLOW}ℹ️  $1${NC}"; }

# Cleanup existing Anvil processes
cleanup_anvil() {
    log_step "Cleaning up existing Anvil processes..."
    pkill -f "anvil" 2>/dev/null || true
    sleep 2
    log_success "Cleanup complete"
}

# Start Anvil blockchain
start_anvil() {
    log_step "Starting Anvil blockchain..."
    
    echo ""
    log_info "🔗 Anvil will run in foreground. Press Ctrl+C to stop."
    log_info "📄 Logs will be saved to: anvil.log"
    echo ""
    sleep 2
    
    # Start Anvil in foreground but also log to file
    anvil --port $ANVIL_PORT --gas-limit 30000000 --base-fee 0 2>&1 | tee anvil.log
}



# Deploy contracts and setup accounts (runs in background)
setup_blockchain() {
    sleep 3  # Give Anvil time to start
    
    log_step "Deploying Vapor contracts..."
    
    cd /Users/polygonbrian/Developer/Chainless/Vapor/contracts
    
    # Deploy MockUSDC
    log_info "Deploying MockUSDC..."
    USDC_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
        src/MockUSDC.sol:MockUSDC 2>&1)
    
    USDC_ADDRESS=$(echo "$USDC_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
    if [ -z "$USDC_ADDRESS" ]; then
        log_error "Failed to deploy MockUSDC"
        return 1
    fi
    
    # Deploy MockSP1Verifier
    log_info "Deploying MockSP1Verifier..."
    SP1_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
        script/Deploy.s.sol:MockSP1Verifier 2>&1)
    
    MOCK_SP1_ADDRESS=$(echo "$SP1_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
    if [ -z "$MOCK_SP1_ADDRESS" ]; then
        log_error "Failed to deploy MockSP1Verifier"
        return 1
    fi
    
    # Deploy ProofVerifier
    log_info "Deploying ProofVerifier..."
    PROOF_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
        src/ProofVerifier.sol:ProofVerifier \
        --constructor-args $MOCK_SP1_ADDRESS 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef false 2>&1)
    
    PROOF_VERIFIER_ADDRESS=$(echo "$PROOF_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
    if [ -z "$PROOF_VERIFIER_ADDRESS" ]; then
        log_error "Failed to deploy ProofVerifier"
        return 1
    fi
    
    # Deploy VaporBridge
    log_info "Deploying VaporBridge..."
    BRIDGE_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
        src/VaporBridge.sol:VaporBridge \
        --constructor-args $PROOF_VERIFIER_ADDRESS 2>&1)
    
    VAPOR_BRIDGE_ADDRESS=$(echo "$BRIDGE_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
    if [ -z "$VAPOR_BRIDGE_ADDRESS" ]; then
        log_error "Failed to deploy VaporBridge"
        return 1
    fi
    
    # Add USDC as supported token
    log_info "Configuring VaporBridge..."
    cast send $VAPOR_BRIDGE_ADDRESS "addSupportedToken(uint256,address)" 1 $USDC_ADDRESS \
        --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null
    
    log_success "All contracts deployed successfully!"
    log_info "Contract Addresses:"
    log_info "  MockUSDC: $USDC_ADDRESS"
    log_info "  MockSP1Verifier: $MOCK_SP1_ADDRESS"
    log_info "  ProofVerifier: $PROOF_VERIFIER_ADDRESS"
    log_info "  VaporBridge: $VAPOR_BRIDGE_ADDRESS"
    
    # Save addresses to file for easy reference
    cat > ../deployed_addresses.env << EOF
# Vapor Contract Addresses (Generated $(date))
USDC_ADDRESS=$USDC_ADDRESS
MOCK_SP1_ADDRESS=$MOCK_SP1_ADDRESS
PROOF_VERIFIER_ADDRESS=$PROOF_VERIFIER_ADDRESS
VAPOR_BRIDGE_ADDRESS=$VAPOR_BRIDGE_ADDRESS
EOF
    
    log_info "📄 Contract addresses saved to: deployed_addresses.env"
    
    # Setup test accounts
    log_step "Setting up test accounts with USDC..."
    
    TEST_ACCOUNTS=(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"  # Account 0
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"  # Account 1
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"  # Account 2
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906"  # Account 3
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"  # Account 4
    )
    
    for i in "${!TEST_ACCOUNTS[@]}"; do
        local address=${TEST_ACCOUNTS[$i]}
        local usdc_amount=$((1000 * 10**6)) # 1000 USDC
        local eth_amount="10000000000000000000" # 10 ETH in wei
        
        # Send ETH for gas fees
        cast send $address --value $eth_amount \
            --rpc-url http://localhost:$ANVIL_PORT \
            --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null
        
        # Mint USDC tokens
        cast send $USDC_ADDRESS "mint(address,uint256)" $address $usdc_amount \
            --rpc-url http://localhost:$ANVIL_PORT \
            --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null
        
        log_info "  Account $i ($address): 10 ETH + 1000 USDC"
    done
    
    log_success "Test accounts funded with USDC"
    log_success "🎉 Anvil Blockchain Ready!"
}

# Main execution
main() {
    echo "⚡ Anvil Blockchain Setup"
    echo "========================"
    
    cleanup_anvil
    
    # Start contract deployment in background
    setup_blockchain &
    
    # Start Anvil in foreground
    start_anvil
}

# Run main function
main "$@"
