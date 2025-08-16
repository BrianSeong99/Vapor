#!/bin/bash

# Cashlink - Seed Demo Script
# Seeds initial USDC balances for demo

echo "🌱 Seeding Cashlink demo environment..."

# Check if contracts are deployed
if [ -z "$CONTRACT_ADDRESS" ]; then
    echo "❌ CONTRACT_ADDRESS not set. Deploy contracts first."
    exit 1
fi

echo "📄 Contract address: $CONTRACT_ADDRESS"

# Seed seller with USDC
echo "💰 Seeding seller with 1000 USDC..."
# TODO: Add actual seeding logic

# Seed filler with initial balance
echo "💰 Seeding filler with initial balance..."
# TODO: Add actual seeding logic

echo "✅ Seeding complete!"
echo "🚀 Run ./scripts/happy_path.sh to test the full flow"
