# Vapor Backend

Rust backend service for the Vapor P2P offramp system, handling order management, batch processing, and blockchain integration.

## Architecture

### Module Structure

```
src/
├── main.rs                 # Server entry point
├── config.rs              # Configuration management
├── database.rs            # Database initialization and migrations
├── models.rs              # Data models and types
├── api/                   # REST API endpoints
│   ├── mod.rs
│   ├── health.rs          # Health check endpoint
│   ├── orders.rs          # Order management endpoints
│   ├── batch.rs           # Batch processing endpoints
│   └── proofs.rs          # Merkle proof endpoints
├── services/              # Business logic services
│   ├── mod.rs
│   ├── order_service.rs   # Order management service
│   ├── matching_engine.rs # Order matching logic
│   └── batch_processor.rs # Batch processing service
├── blockchain.rs          # Smart contract integration
└── merkle.rs             # Merkle tree operations
```

## Current Status

### ✅ **Completed (Phase 1)**
- [x] **Project structure** - Organized module hierarchy
- [x] **Dependencies** - All required Rust crates configured
- [x] **Configuration** - Environment-based config system
- [x] **Database setup** - SQLite with migrations
- [x] **Basic API** - REST endpoints structure
- [x] **Data models** - Order types, batch structure, account states

### 🔄 **In Progress (Phase 2)**
- [ ] **Database operations** - CRUD for orders and batches
- [ ] **API implementation** - Complete endpoint logic
- [ ] **Order matching** - Seller/filler matching engine

### 📋 **Planned (Phase 3+)**
- [ ] **Batch processing** - Merkle tree generation
- [ ] **Blockchain integration** - Smart contract interaction
- [ ] **SP1 integration** - ZK proof generation
- [ ] **Background workers** - Automated batch processing

## API Endpoints

### Health Check
```
GET /health
```

### Order Management
```
POST /api/v1/orders
POST /api/v1/orders/:order_id/mark-paid
```

### Batch Processing
```
POST /api/v1/batch/prove
```

### Proof Service
```
GET /api/v1/proofs/:batch_id/:order_id
```

## Data Models

### Order Types
- **BridgeIn** (0) - User deposits tokens
- **Transfer** (2) - Move tokens between accounts
- **BridgeOut** (1) - User withdraws tokens

### Order Status
- **Pending** (0) - Newly created
- **Locked** (1) - Assigned to filler
- **MarkPaid** (2) - Fiat payment confirmed
- **Completed** (3) - Batch processed
- **Failed** (4) - Processing failed

### Batch Status
- **Building** (0) - Collecting orders
- **Proving** (1) - Generating ZK proof
- **Submitting** (2) - Submitting to blockchain
- **Submitted** (3) - On-chain confirmed
- **Failed** (4) - Processing failed

## Configuration

Set environment variables:

```bash
# API Configuration
PORT=8080

# Database Configuration  
DATABASE_URL=sqlite:cashlink.db

# Blockchain Configuration
RPC_URL=http://localhost:8545
CONTRACT_ADDRESS=0x1234567890123456789012345678901234567890
PRIVATE_KEY=0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890

# Batch Processing
BATCH_INTERVAL_SECONDS=60
MAX_ORDERS_PER_BATCH=100

# Logging
RUST_LOG=info
```

## Running

### Development
```bash
# Install dependencies
cargo build

# Run server
cargo run --bin cashlink-server

# Check compilation
cargo check

# Run tests
cargo test
```

### Database
The server automatically creates and migrates the SQLite database on startup:
- `orders` - Order management
- `batches` - Batch tracking
- `account_states` - Account balances

## Implementation Phases

### Phase 1: Foundation ✅
Basic project structure, configuration, and API skeleton.

### Phase 2: Core Logic (Next)
- Database operations with SQLX
- Complete API endpoint implementations
- Order matching engine
- Banking hash integration

### Phase 3: Batch Processing
- Merkle tree generation with `rs_merkle`
- Batch building and validation
- Order aggregation and state transitions

### Phase 4: Blockchain Integration
- Smart contract interaction with `ethers`
- Event listening for deposits
- Proof submission to contracts

### Phase 5: SP1 Integration
- ZK proof generation
- Guest program integration
- Production-ready proving

### Phase 6: Production Features
- Background workers
- Monitoring and metrics
- Error handling and recovery
- Load balancing and scaling

## Dependencies

Key Rust crates used:
- **axum** - Web framework
- **sqlx** - Database operations
- **ethers** - Ethereum integration
- **rs_merkle** - Merkle tree operations
- **tokio** - Async runtime
- **serde** - Serialization
- **tracing** - Logging

## Next Steps

1. **Implement database operations** in `services/order_service.rs`
2. **Complete API endpoints** with actual database integration
3. **Build order matching engine** for seller/filler pairing
4. **Add banking hash validation** for fiat payment linking

The backend is structured for step-by-step development with clear separation of concerns and modular architecture.
