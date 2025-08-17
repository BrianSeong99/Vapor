# Vapor Smart Contracts

Smart contracts for the Vapor P2P offramp system, enabling trustless multi-token claims through ZK proof verification with banking integration.

## Architecture

### Core Contracts

#### ProofVerifier.sol
- **Purpose**: Manages batch state and order roots with SP1 ZK proof verification
- **Key Functions**:
  - `submitProof()`: Verifies ZK proofs and updates batch roots
  - `getBatch()`: Returns batch data for Merkle proof verification
  - MVP/Production modes for flexible deployment

#### VaporBridge.sol
- **Purpose**: Handles multi-token deposits and claims using Merkle proofs with banking integration
- **Key Functions**:
  - `claim()`: Claims tokens using Merkle proof from verified batch (supports multiple tokens)
  - `deposit()`: Deposits tokens with banking hash to trigger BridgeIn orders
  - `addSupportedToken()`: Dynamically add ERC20 tokens via token ID mapping
  - `removeSupportedToken()`: Remove supported tokens
  - Merkle proof verification against verified batch roots

#### MockUSDC.sol
- **Purpose**: OpenZeppelin-based ERC20 token for testing and demo
- **Features**: Inherits from OpenZeppelin's ERC20 and Ownable contracts

## Quick Start

### Prerequisites
- Foundry installed
- OpenZeppelin contracts (auto-installed)
- SP1 contracts (auto-installed)
- Environment variables configured

### Setup

1. **Install dependencies**:
```bash
forge install
```

2. **Run tests**:
```bash
forge test
```

3. **Deploy locally**:
```bash
# Start local node
anvil

# Deploy contracts
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

## Deployment

### Local/Testing
```bash
# Uses mock contracts for SP1 and USDC
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast
```

### Sepolia Testnet
```bash
forge script script/Deploy.s.sol \
  --rpc-url $SEPOLIA_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify
```

### Mainnet
```bash
forge script script/Deploy.s.sol \
  --rpc-url $MAINNET_RPC_URL \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify
```

## Configuration

### Environment Variables
```bash
# Required
PRIVATE_KEY=your_private_key_here
RPC_URL=your_rpc_url

# Optional for verification
ETHERSCAN_API_KEY=your_api_key
```

### SP1 Integration

#### MVP Mode (Default)
- Deploys with `useActualSP1Verification = false`
- Accepts any non-empty proof for testing
- Perfect for hackathon demos

#### Production Mode
- Set `useActualSP1Verification = true` via `setUseActualSP1Verification()`
- Requires actual SP1 proofs with correct public inputs
- Integrates with official SP1 verifier contracts

## Testing

### Test Coverage
- **ProofVerifier**: 12 tests covering all functionality
- **VaporBridge**: 15 tests including multi-token and banking hash validation
- **Total**: 27 tests with 100% pass rate

### Run Tests
```bash
# All tests
forge test

# Specific contract
forge test --match-contract ProofVerifierTest
forge test --match-contract VaporBridgeTest

# With gas reporting
forge test --gas-report
```

## Contract Interactions

### Basic Flow

1. **Setup** (Owner):
```solidity
// Deploy contracts
ProofVerifier verifier = new ProofVerifier(sp1Verifier, programVKey, false);
VaporBridge bridge = new VaporBridge(address(verifier));

// Add supported tokens
bridge.addSupportedToken(1, usdcAddress); // Token ID 1 = USDC
bridge.addSupportedToken(2, usdtAddress); // Token ID 2 = USDT
```

2. **Deposit** (User):
```solidity
// Approve and deposit USDC with banking info
bytes32 bankingHash = keccak256(abi.encode("bank_account_123", "ref_456"));
usdc.approve(address(bridge), amount);
bridge.deposit(1, amount, bankingHash); // tokenId=1 for USDC
```

3. **Submit Batch** (Operator):
```solidity
// Submit ZK proof for batch
verifier.submitProof(batchId, prevBatchId, prevStateRoot, prevOrdersRoot, 
                    newStateRoot, newOrdersRoot, zkProof);
```

4. **Claim** (User):
```solidity
// Claim tokens with Merkle proof
bridge.claim(batchId, orderId, recipient, tokenId, amount, merkleProof);
```

## Multi-Token & Banking Integration

### Supported Token Management

#### Adding Tokens
```solidity
// Add USDC as token ID 1
bridge.addSupportedToken(1, usdcAddress);

// Add USDT as token ID 2  
bridge.addSupportedToken(2, usdtAddress);

// Check if token is supported
bool isSupported = bridge.isTokenSupported(1); // true
```

#### Token Operations
```solidity
// Get token address by ID
address tokenAddr = bridge.getSupportedToken(1);

// Get contract balance for specific token
uint256 balance = bridge.getTokenBalance(1);

// Remove token support (owner only)
bridge.removeSupportedToken(2);
```

### Banking Hash Integration

The banking hash links on-chain deposits with off-chain fiat payments:

```solidity
// Example banking hash construction
bytes32 bankingHash = keccak256(abi.encode(
    "bank_account_number",
    "routing_number", 
    "reference_id",
    "user_identifier",
    block.timestamp
));

// Deposit with banking information
bridge.deposit(tokenId, amount, bankingHash);
```

#### Use Cases
- **Payment Matching**: Backend matches deposits to bank transfers
- **Compliance**: Auditable trail linking crypto and fiat
- **Dispute Resolution**: Verifiable payment references
- **Multi-Bank Support**: Different hashes for different banks

## Security Considerations

### MVP Limitations
- ⚠️ Owner-only proof submission (not decentralized)
- ⚠️ MVP mode accepts any non-empty proof
- ⚠️ Emergency withdraw function for testing

### Production Ready
- ✅ SP1 ZK proof verification
- ✅ Multi-token support with dynamic token management
- ✅ Banking hash integration for fiat payment linking
- ✅ OpenZeppelin security standards
- ✅ Merkle proof validation for claims
- ✅ Double-claim prevention
- ✅ Batch sequence validation
- ✅ Comprehensive test coverage

## Gas Usage

Average gas costs:
- **Submit Proof**: ~90k gas
- **Claim**: ~150k gas (includes token validation)
- **Deposit**: ~48k gas (includes banking hash)
- **Add Token**: ~35k gas
- **Remove Token**: ~25k gas

## Deployed Addresses

Deployment addresses are saved to `deployments/{chainId}.json` after each deployment.

## Development

### Project Structure
```
contracts/
├── src/
│   ├── ProofVerifier.sol       # ZK proof verification
│   ├── VaporBridge.sol         # Multi-token claims with Merkle proofs
│   ├── MockUSDC.sol           # OpenZeppelin-based test token
│   └── interfaces/            # Contract interfaces
├── test/                      # Comprehensive tests (27 tests)
├── script/                    # Deployment scripts
├── lib/                       # Dependencies
│   ├── openzeppelin-contracts/ # Security-audited ERC20 contracts
│   └── sp1-contracts/         # Official SP1 ZK verification
└── deployments/              # Deployment artifacts
```

### Adding New Features

1. Update interfaces in `src/interfaces/`
2. Implement in main contracts
3. Add comprehensive tests
4. Update deployment script if needed

## License

MIT License - see LICENSE file for details.