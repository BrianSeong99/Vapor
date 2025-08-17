#!/bin/bash

# Start just the Vapor backend server
# Assumes Anvil and contracts are already deployed

set -e

# Configuration
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

echo "🖥️  Starting Vapor Backend Server..."

# Navigate to backend directory
cd /Users/polygonbrian/Developer/Chainless/Vapor/backend

# Kill any existing backend
pkill -f "vapor-server" 2>/dev/null || true
sleep 2

# Create environment file with existing contract addresses
log_step "Configuring backend environment..."

# Check if contract addresses exist
if [ -f "../contracts/deployed_addresses.env" ]; then
    source ../contracts/deployed_addresses.env
    log_info "Using existing contract addresses"
elif [ -f "../deployed_addresses.env" ]; then
    source ../deployed_addresses.env
    log_info "Using existing contract addresses"
else
    log_error "Contract addresses not found. Please deploy contracts first."
    log_info "Run: ./scripts/start_dev_environment.sh"
    exit 1
fi

# Create .env file
cat > .env << EOF
# Vapor Backend Configuration (Generated $(date))
DATABASE_URL=sqlite:vapor_dev.db
CHAIN_RPC_URL=http://localhost:8545
VAPOR_BRIDGE_CONTRACT=$VAPOR_BRIDGE_ADDRESS
PROOF_VERIFIER_CONTRACT=$PROOF_VERIFIER_ADDRESS
USDC_CONTRACT=$USDC_ADDRESS
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
SERVER_PORT=$BACKEND_PORT
EOF

# Prepare database
log_step "Preparing database..."
rm -f vapor_dev.db
touch vapor_dev.db
chmod 666 vapor_dev.db

# Start backend server
log_step "Starting backend server..."
echo ""
log_info "🖥️  Backend will run in foreground. Press Ctrl+C to stop."
log_info "📄 Logs will be saved to: backend.log"
echo ""
sleep 2

# Start backend in foreground but also log to file
cargo run --bin vapor-server 2>&1 | tee backend.log
