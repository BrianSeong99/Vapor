#!/bin/bash

# Start just the Vapor frontend server
# Runs Next.js development server in foreground

set -e

echo "📱 Starting Vapor Frontend Server..."

# Configuration
FRONTEND_PORT=8080

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

# Navigate to frontend directory
cd /Users/polygonbrian/Developer/Chainless/Vapor/frontend

# Kill any existing frontend processes
log_step "Cleaning up existing frontend processes..."
pkill -f "next dev" 2>/dev/null || true
sleep 2
log_success "Cleanup complete"

# Check if node_modules exists
log_step "Checking frontend dependencies..."
if [ ! -d "node_modules" ]; then
    log_info "Installing frontend dependencies..."
    npm install
    log_success "Dependencies installed"
else
    log_info "Dependencies already installed"
fi

# Start frontend server
log_step "Starting frontend development server..."
echo ""
log_info "📱 Frontend will run in foreground. Press Ctrl+C to stop."
log_info "🌐 Frontend URL: http://localhost:$FRONTEND_PORT"
log_info "📄 Logs will be displayed live below"
echo ""
log_info "🔗 Make sure backend is running on: http://localhost:3000"
log_info "🔗 Make sure Anvil is running on: http://localhost:8545"
echo ""
sleep 2

# Start Next.js in development mode (foreground) without Turbopack
npm run dev
