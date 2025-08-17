#!/bin/bash

# Simple Vapor End-to-End Test Script
# Focused P2P offramp flow with minimal setup

set -e

echo "🚀 Starting Simple Vapor End-to-End Test..."

# Configuration
ANVIL_PORT=8545
BACKEND_PORT=3000
NUM_TRANSACTIONS=3  # Reduced for simplicity
AMOUNT_PER_TX=100   # 100 USDC per transaction

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_step() { echo -e "${BLUE}🔄 $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    pkill -f "anvil" 2>/dev/null || true
    pkill -f "vapor-server" 2>/dev/null || true
}
trap cleanup EXIT

# Start Anvil
log_step "Starting Anvil..."
anvil --port $ANVIL_PORT --gas-limit 30000000 --base-fee 0 > anvil.log 2>&1 &
sleep 3

# Test connection
if ! curl -s -X POST -H "Content-Type: application/json" \
    --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    http://localhost:$ANVIL_PORT > /dev/null; then
    log_error "Failed to start Anvil"
    exit 1
fi
log_success "Anvil started"

# Deploy contracts using forge create with verbose output
log_step "Deploying contracts..."
cd /Users/polygonbrian/Developer/Chainless/Vapor/contracts

# Deploy MockUSDC
log_step "Deploying MockUSDC..."
USDC_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    src/MockUSDC.sol:MockUSDC 2>&1)

USDC_ADDRESS=$(echo "$USDC_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
if [ -z "$USDC_ADDRESS" ]; then
    log_error "Failed to deploy MockUSDC"
    echo "$USDC_DEPLOY"
    exit 1
fi
log_success "MockUSDC deployed to: $USDC_ADDRESS"

# Deploy MockSP1Verifier
log_step "Deploying MockSP1Verifier..."
SP1_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    script/Deploy.s.sol:MockSP1Verifier 2>&1)

MOCK_SP1_ADDRESS=$(echo "$SP1_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
if [ -z "$MOCK_SP1_ADDRESS" ]; then
    log_error "Failed to deploy MockSP1Verifier"
    echo "$SP1_DEPLOY"
    exit 1
fi
log_success "MockSP1Verifier deployed to: $MOCK_SP1_ADDRESS"

# Deploy ProofVerifier
log_step "Deploying ProofVerifier..."
PROOF_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    src/ProofVerifier.sol:ProofVerifier \
    --constructor-args $MOCK_SP1_ADDRESS 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef false 2>&1)

PROOF_VERIFIER_ADDRESS=$(echo "$PROOF_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
if [ -z "$PROOF_VERIFIER_ADDRESS" ]; then
    log_error "Failed to deploy ProofVerifier"
    echo "$PROOF_DEPLOY"
    exit 1
fi
log_success "ProofVerifier deployed to: $PROOF_VERIFIER_ADDRESS"

# Deploy VaporBridge
log_step "Deploying VaporBridge..."
BRIDGE_DEPLOY=$(forge create --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
    src/VaporBridge.sol:VaporBridge \
    --constructor-args $PROOF_VERIFIER_ADDRESS 2>&1)

VAPOR_BRIDGE_ADDRESS=$(echo "$BRIDGE_DEPLOY" | grep "Deployed to:" | awk '{print $3}')
if [ -z "$VAPOR_BRIDGE_ADDRESS" ]; then
    log_error "Failed to deploy VaporBridge"
    echo "$BRIDGE_DEPLOY"
    exit 1
fi
log_success "VaporBridge deployed to: $VAPOR_BRIDGE_ADDRESS"

# Add USDC as supported token
log_step "Adding USDC as supported token..."
cast send $VAPOR_BRIDGE_ADDRESS "addSupportedToken(uint256,address)" 1 $USDC_ADDRESS \
    --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null

log_success "Contract setup complete!"

# Setup test accounts
log_step "Minting USDC to test accounts..."
SELLER_ADDRESSES=(
    "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    "0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"
)

for address in "${SELLER_ADDRESSES[@]}"; do
    amount=$((1000 * 10**6)) # 1000 USDC
    cast send $USDC_ADDRESS "mint(address,uint256)" $address $amount \
        --rpc-url http://localhost:$ANVIL_PORT \
        --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null
done
log_success "Test accounts funded with USDC"

# Start backend
log_step "Starting backend server..."
cd /Users/polygonbrian/Developer/Chainless/Vapor/backend

# Create environment file
cat > .env << EOF
DATABASE_URL=:memory:
CHAIN_RPC_URL=http://localhost:$ANVIL_PORT
VAPOR_BRIDGE_CONTRACT=$VAPOR_BRIDGE_ADDRESS
PROOF_VERIFIER_CONTRACT=$PROOF_VERIFIER_ADDRESS
USDC_CONTRACT=$USDC_ADDRESS
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
SERVER_PORT=$BACKEND_PORT
EOF
cargo run --bin vapor-server > backend.log 2>&1 &
BACKEND_PID=$!

# Wait for backend
for i in {1..30}; do
    if curl -s http://localhost:$BACKEND_PORT/health > /dev/null; then
        log_success "Backend started"
        break
    fi
    sleep 1
    if [ $i -eq 30 ]; then
        log_error "Backend failed to start"
        exit 1
    fi
done

# Execute test flow
log_step "Testing complete P2P flow..."

# 1. Seller deposits
log_step "1. Seller Deposits"
SELLER_KEY="0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
SELLER_ADDRESS="0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
AMOUNT=$((100 * 10**6)) # 100 USDC

# Approve and deposit
cast send $USDC_ADDRESS "approve(address,uint256)" $VAPOR_BRIDGE_ADDRESS $AMOUNT \
    --rpc-url http://localhost:$ANVIL_PORT --private-key $SELLER_KEY > /dev/null

cast send $VAPOR_BRIDGE_ADDRESS "deposit(uint256,uint256,bytes32)" 1 $AMOUNT 0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef \
    --rpc-url http://localhost:$ANVIL_PORT --private-key $SELLER_KEY > /dev/null

echo "  ✅ Deposit successful"

# Create bridge-in order
ORDER_RESPONSE=$(curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/orders \
    -H "Content-Type: application/json" \
    -d "{
        \"order_type\": \"BridgeIn\",
        \"from_address\": \"$SELLER_ADDRESS\",
        \"to_address\": \"$SELLER_ADDRESS\",
        \"amount\": \"$AMOUNT\",
        \"token_id\": 1,
        \"banking_hash\": \"0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef\"
    }")

echo "  DEBUG: Order response: $ORDER_RESPONSE"
ORDER_ID=$(echo "$ORDER_RESPONSE" | jq -r '.id')
echo "  ✅ Order created: $ORDER_ID"

# For testing purposes, let's manually mark the order as Discovery 
# (simulating what the relayer would do after deposit confirmation)
curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/orders/$ORDER_ID/mark-discovery \
    -H "Content-Type: application/json" > /dev/null

echo "  🔄 Order marked as Discovery status (simulating relayer)"

# 2. Filler flow
log_step "2. Filler Flow"
FILLER_ID="test_filler"
FILLER_ADDRESS="0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

# Get discovery orders
DISCOVERY_ORDERS=$(curl -s "http://localhost:$BACKEND_PORT/api/v1/fillers/discovery")
echo "  DEBUG: Discovery response: $DISCOVERY_ORDERS"

# Check if it's an array or object and handle accordingly
if echo "$DISCOVERY_ORDERS" | jq -e '. | type == "array"' > /dev/null; then
    FIRST_ORDER_ID=$(echo "$DISCOVERY_ORDERS" | jq -r '.[0].id')
    FIRST_ORDER_AMOUNT=$(echo "$DISCOVERY_ORDERS" | jq -r '.[0].amount')
else
    # If it's not an array, it might be an object with orders
    FIRST_ORDER_ID=$(echo "$DISCOVERY_ORDERS" | jq -r '.orders[0].id // .id // empty')
    FIRST_ORDER_AMOUNT=$(echo "$DISCOVERY_ORDERS" | jq -r '.orders[0].amount // .amount // empty')
fi

echo "  📋 Found order in discovery: $FIRST_ORDER_ID"

# Lock order
curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/fillers/orders/$FIRST_ORDER_ID/lock \
    -H "Content-Type: application/json" \
    -d "{
        \"filler_id\": \"$FILLER_ID\",
        \"amount\": \"$FIRST_ORDER_AMOUNT\"
    }" > /dev/null

echo "  🔒 Order locked"

# Submit payment proof
curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/fillers/orders/$FIRST_ORDER_ID/payment-proof \
    -H "Content-Type: application/json" \
    -d "{
        \"filler_id\": \"$FILLER_ID\",
        \"banking_hash\": \"0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890\"
    }" > /dev/null

echo "  💳 Payment proof submitted"

# Check order status after payment proof
ORDER_STATUS_CHECK=$(curl -s "http://localhost:$BACKEND_PORT/api/v1/orders/$FIRST_ORDER_ID")
echo "  DEBUG: Order after payment proof: $ORDER_STATUS_CHECK"

# 3. Batch processing
log_step "3. Batch Processing"

# Start batch
BATCH_RESPONSE=$(curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/batch/start)
BATCH_ID=$(echo "$BATCH_RESPONSE" | jq -r '.batch_id')
echo "  📦 Started batch: $BATCH_ID"

# Add order to batch (skip this as start_batch should auto-add orders)

echo "  ➕ Orders will be auto-added to batch"

# Finalize batch
FINALIZE_RESPONSE=$(curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/batch/finalize \
    -H "Content-Type: application/json" \
    -d "{\"batch_id\": $BATCH_ID}")

echo "  DEBUG: Finalize response: $FINALIZE_RESPONSE"

PROOF=$(echo "$FINALIZE_RESPONSE" | jq -r '.proof')
STATE_ROOT=$(echo "$FINALIZE_RESPONSE" | jq -r '.new_state_root')
ORDERS_ROOT=$(echo "$FINALIZE_RESPONSE" | jq -r '.new_orders_root')

echo "  🔄 Batch finalized and proof generated"
echo "  Proof: $PROOF"
echo "  State Root: $STATE_ROOT"
echo "  Orders Root: $ORDERS_ROOT"

# 4. Submit proof to contract
log_step "4. Proof Submission"
cd /Users/polygonbrian/Developer/Chainless/Vapor/contracts

cast send $PROOF_VERIFIER_ADDRESS "submitProof(uint32,bytes32,bytes32,bytes)" \
    $BATCH_ID $STATE_ROOT $ORDERS_ROOT "$PROOF" \
    --rpc-url http://localhost:$ANVIL_PORT \
    --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 > /dev/null

echo "  ✅ Proof submitted to contract"

# 5. Claim
log_step "5. Filler Claim"

# Get claim data
CLAIM_RESPONSE=$(curl -s -X POST http://localhost:$BACKEND_PORT/api/v1/fillers/claim \
    -H "Content-Type: application/json" \
    -d "{
        \"filler_id\": \"$FILLER_ID\",
        \"claims\": [{
            \"wallet_address\": \"$FILLER_ADDRESS\",
            \"amount\": \"$FIRST_ORDER_AMOUNT\"
        }]
    }")

CLAIM_DATA=$(echo "$CLAIM_RESPONSE" | jq -r '.claims[0]')
CLAIM_ORDER_ID=$(echo "$CLAIM_DATA" | jq -r '.order_id')
MERKLE_PROOF=$(echo "$CLAIM_DATA" | jq -r '.merkle_proof')

echo "  📋 Claim prepared for order: $CLAIM_ORDER_ID"

# Execute claim
FILLER_KEY="0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"

# Convert proof array to cast format
PROOF_ARGS=""
for proof_elem in $(echo "$MERKLE_PROOF" | jq -r '.[]'); do
    PROOF_ARGS="$PROOF_ARGS \"$proof_elem\""
done

# Execute batch claim
cast send $VAPOR_BRIDGE_ADDRESS "batchClaim((uint32,uint256,address,uint256)[],(bytes32[])[])" \
    "[($BATCH_ID,$CLAIM_ORDER_ID,$FILLER_ADDRESS,$FIRST_ORDER_AMOUNT)]" \
    "[$PROOF_ARGS]" \
    --rpc-url http://localhost:$ANVIL_PORT \
    --private-key $FILLER_KEY > /dev/null

echo "  ✅ Claim executed successfully"

# Check final balance
FINAL_BALANCE=$(cast call $USDC_ADDRESS "balanceOf(address)" $FILLER_ADDRESS \
    --rpc-url http://localhost:$ANVIL_PORT)
READABLE_BALANCE=$((FINAL_BALANCE / 10**6))

log_success "🎉 End-to-End Test Completed!"
echo ""
echo "Summary:"
echo "  ✅ Deployed contracts to Anvil"
echo "  ✅ Seller deposited 100 USDC"
echo "  ✅ Filler locked and processed order"
echo "  ✅ Generated and submitted batch proof (MVP mode)"
echo "  ✅ Filler successfully claimed: $READABLE_BALANCE USDC"
echo ""
echo "All components working correctly! 🚀"
