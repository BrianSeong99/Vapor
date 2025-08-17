#!/bin/bash
set -e

echo "🚀 Setting up Vapor MVP..."

# 1. Copy environment config
echo "📝 Setting up environment..."
cd backend
cp .env.example .env
echo "✅ Environment config created"

# 2. Deploy contracts
echo "🔗 Deploying contracts..."
cd ../contracts
if ! command -v forge &> /dev/null; then
    echo "❌ Foundry not installed. Please install: curl -L https://foundry.paradigm.xyz | bash"
    exit 1
fi

# Start anvil in background if not running
if ! pgrep -f anvil > /dev/null; then
    echo "🔧 Starting local blockchain..."
    anvil --host 0.0.0.0 &
    ANVIL_PID=$!
    sleep 2
    echo "✅ Anvil started (PID: $ANVIL_PID)"
fi

# Deploy contracts
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# 3. Update backend config with deployed address  
echo "📋 Extracting contract addresses..."
# Note: Would need to parse deployment output and update .env

echo "🎉 MVP setup complete!"
echo ""
echo "Next steps:"
echo "1. Update CONTRACT_ADDRESS in backend/.env with deployed address"
echo "2. cd backend && cargo run"
echo "3. Test with: curl http://localhost:8080/health"
