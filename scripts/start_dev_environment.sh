#!/bin/bash

# Vapor Development Environment Setup
# Starts Anvil, deploys contracts, and runs backend persistently

set -e

echo "🚀 Starting Vapor Development Environment..."

# Configuration
ANVIL_PORT=8545
BACKEND_PORT=3000

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

# Cleanup existing processes
cleanup_existing() {
    log_step "Cleaning up existing processes..."
    pkill -f "anvil" 2>/dev/null || true
    pkill -f "vapor-server" 2>/dev/null || true
    sleep 2
    log_success "Cleanup complete"
}

# Start Anvil blockchain
start_anvil() {
    log_step "Starting Anvil blockchain..."
    
    anvil --port $ANVIL_PORT --gas-limit 30000000 --base-fee 0 > anvil.log 2>&1 &
    ANVIL_PID=$!
    
    # Wait for Anvil to start
    for i in {1..10}; do
        if curl -s -X POST -H "Content-Type: application/json" \
            --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            http://localhost:$ANVIL_PORT > /dev/null 2>&1; then
            log_success "Anvil started on port $ANVIL_PORT (PID: $ANVIL_PID)"
            echo "📄 Anvil logs: anvil.log"
            return 0
        fi
        sleep 1
    done
    
    log_error "Failed to start Anvil"
    exit 1
}

# Deploy contracts
deploy_contracts() {
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
        exit 1
    fi
    
    # Deploy MockSP1Verifier
    log_info "Deploying MockSP1Verifier..."
    SP1_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
        script/Deploy.s.sol:MockSP1Verifier 2>&1)
    
    MOCK_SP1_ADDRESS=$(echo "$SP1_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
    if [ -z "$MOCK_SP1_ADDRESS" ]; then
        log_error "Failed to deploy MockSP1Verifier"
        exit 1
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
        exit 1
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
        exit 1
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
    cat > deployed_addresses.env << EOF
# Vapor Contract Addresses (Generated $(date))
USDC_ADDRESS=$USDC_ADDRESS
MOCK_SP1_ADDRESS=$MOCK_SP1_ADDRESS
PROOF_VERIFIER_ADDRESS=$PROOF_VERIFIER_ADDRESS
VAPOR_BRIDGE_ADDRESS=$VAPOR_BRIDGE_ADDRESS
EOF
    
    log_info "📄 Contract addresses saved to: deployed_addresses.env"
    
    export USDC_ADDRESS MOCK_SP1_ADDRESS PROOF_VERIFIER_ADDRESS VAPOR_BRIDGE_ADDRESS
}

# Setup test accounts
setup_test_accounts() {
    log_step "Setting up test accounts with USDC..."
    
    # Mint USDC to test accounts
    TEST_ACCOUNTS=(
        "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"  # Account 0
        "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"  # Account 1
        "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"  # Account 2
        "0x90F79bf6EB2c4f870365E785982E1f101E93b906"  # Account 3
        "0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65"  # Account 4
    )
    
    for i in "${!TEST_ACCOUNTS[@]}"; do
        local address=${TEST_ACCOUNTS[$i]}
        local amount=$((1000 * 10**6)) # 1000 USDC
        
        cast send $USDC_ADDRESS "mint(address,uint256)" $address $amount \
            --rpc-url http://localhost:$ANVIL_PORT \
            --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null
        
        log_info "  Account $i ($address): 1000 USDC"
    done
    
    log_success "Test accounts funded with USDC"
}

# Start backend server
start_backend() {
    log_step "Starting Vapor backend server..."
    
    cd /Users/polygonbrian/Developer/Chainless/Vapor/backend
    
    # Create environment file
    cat > .env << EOF
# Vapor Backend Configuration (Generated $(date))
DATABASE_URL=sqlite:vapor_dev.db
CHAIN_RPC_URL=http://localhost:$ANVIL_PORT
VAPOR_BRIDGE_CONTRACT=$VAPOR_BRIDGE_ADDRESS
PROOF_VERIFIER_CONTRACT=$PROOF_VERIFIER_ADDRESS
USDC_CONTRACT=$USDC_ADDRESS
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
SERVER_PORT=$BACKEND_PORT
EOF
    
    # Remove old database and create new one
    rm -f vapor_dev.db
    touch vapor_dev.db
    chmod 666 vapor_dev.db
    
    # Start backend server
    cargo run --bin vapor-server > backend.log 2>&1 &
    BACKEND_PID=$!
    
    # Wait for backend to start
    for i in {1..30}; do
        if curl -s http://localhost:$BACKEND_PORT/health > /dev/null 2>&1; then
            log_success "Backend server started on port $BACKEND_PORT (PID: $BACKEND_PID)"
            log_info "📄 Backend logs: backend.log"
            return 0
        fi
        sleep 1
    done
    
    log_error "Backend server failed to start"
    exit 1
}

# Save process information
save_dev_info() {
    cat > dev_environment.info << EOF
# Vapor Development Environment
# Started: $(date)

## Process Information
Anvil PID: $ANVIL_PID
Backend PID: $BACKEND_PID

## Service URLs
Anvil RPC: http://localhost:$ANVIL_PORT
Backend API: http://localhost:$BACKEND_PORT
Frontend: http://localhost:8080 (run 'npm run dev' in frontend/)

## Contract Addresses
MockUSDC: $USDC_ADDRESS
MockSP1Verifier: $MOCK_SP1_ADDRESS  
ProofVerifier: $PROOF_VERIFIER_ADDRESS
VaporBridge: $VAPOR_BRIDGE_ADDRESS

## Log Files
Anvil: anvil.log
Backend: backend.log

## Useful Commands
# Check Anvil status
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://localhost:$ANVIL_PORT

# Check Backend health
curl http://localhost:$BACKEND_PORT/health

# Stop environment
kill $ANVIL_PID $BACKEND_PID

## Test Accounts (All have 1000 USDC)
Account 0: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
Account 1: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
Account 2: 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC
Account 3: 0x90F79bf6EB2c4f870365E785982E1f101E93b906
Account 4: 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65

Private Keys:
Account 0: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
Account 1: 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
Account 2: 0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
Account 3: 0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
Account 4: 0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
EOF

    log_info "📄 Environment info saved to: dev_environment.info"
}

# Main execution
main() {
    echo "🎯 Vapor Development Environment Setup"
    echo "======================================"
    
    cleanup_existing
    start_anvil
    deploy_contracts
    setup_test_accounts
    start_backend
    save_dev_info
    
    echo ""
    log_success "🎉 Vapor Development Environment Ready!"
    echo ""
    log_info "Services Running:"
    log_info "  🔗 Anvil Blockchain: http://localhost:$ANVIL_PORT"
    log_info "  🖥️  Backend API: http://localhost:$BACKEND_PORT"
    log_info "  📱 Frontend: http://localhost:8080 (start with 'npm run dev')"
    echo ""
    log_info "📋 Quick Test Commands:"
    log_info "  curl http://localhost:$BACKEND_PORT/health"
    log_info "  curl http://localhost:$BACKEND_PORT/api/v1/fillers/discovery"
    echo ""
    log_info "📄 Files Created:"
    log_info "  • deployed_addresses.env - Contract addresses"
    log_info "  • dev_environment.info - Complete environment info"
    log_info "  • anvil.log - Blockchain logs"
    log_info "  • backend.log - Backend server logs"
    echo ""
    log_info "🔄 To stop environment: kill $ANVIL_PID $BACKEND_PID"
    echo ""
    log_success "Environment is ready for development! 🚀"
}

# Run main function
main "$@"
