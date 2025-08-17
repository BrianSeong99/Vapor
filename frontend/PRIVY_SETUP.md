# Privy Integration Setup

## 🔧 Setup Instructions

### 1. Create Privy App
1. Go to [Privy Console](https://console.privy.id/)
2. Create a new app
3. Copy your App ID

### 2. Environment Configuration
Create a `.env.local` file in the frontend directory:

```env
# Privy Configuration
NEXT_PUBLIC_PRIVY_APP_ID=your-privy-app-id-here

# Backend API Configuration  
NEXT_PUBLIC_BACKEND_URL=http://localhost:3000

# Contract Addresses (from deployment)
NEXT_PUBLIC_VAPOR_BRIDGE_ADDRESS=0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
NEXT_PUBLIC_PYUSD_ADDRESS=0x5FbDB2315678afecb367f032d93F642f64180aa3
```

### 3. Privy App Configuration
In your Privy console, configure:

- **Login Methods**: Email, Google, Wallet
- **Chains**: Ethereum Mainnet, Sepolia Testnet
- **Embedded Wallets**: Enable with auto-creation
- **Theme**: Light mode with accent color `#8BC34A`

## 🎯 Features Implemented

### ✅ Wallet Integration
- **Embedded Wallets**: Seamless wallet creation via Privy
- **Multi-Login**: Email, Google, or existing wallet connection
- **Balance Display**: Real-time PYUSD balance checking
- **Chain Support**: Mainnet and Sepolia testnet

### ✅ PYUSD Operations
- **Balance Checking**: Automatic PYUSD balance queries
- **Approval Flow**: Smart approval only when needed
- **Deposit Function**: Direct deposit to VaporBridge contract
- **Transaction Tracking**: Real-time transaction status

### ✅ User Experience
- **Smooth Onboarding**: No MetaMask required
- **Progress Indicators**: Clear transaction progress
- **Error Handling**: User-friendly error messages
- **Balance Validation**: Prevents overdraft attempts

## 🔄 Transaction Flow

1. **Connect Wallet**: User logs in via Privy (email/social/wallet)
2. **Check Balance**: Automatic PYUSD balance display
3. **Input Details**: Amount, bank account, payment service
4. **Approve PYUSD**: If needed, approve spending to VaporBridge
5. **Deposit**: Deposit PYUSD to VaporBridge contract
6. **Confirmation**: Success feedback and redirect

## 🧪 Testing

### With Testnet:
1. Use Sepolia testnet PYUSD (MockUSDC from our contracts)
2. Fund test wallet with test tokens
3. Test full deposit flow

### With Local Development:
1. Start Anvil: `./scripts/start_anvil_only.sh`
2. Start Backend: `./scripts/start_backend_only.sh`  
3. Start Frontend: `./scripts/start_frontend_only.sh`
4. Use local MockUSDC contract for testing

## 🏆 Hackathon Prize Alignment

### Privy Prizes:
- **Best Consumer App**: Seamless embedded wallet UX
- **Best Financial App**: PYUSD off-ramp functionality

### PYUSD Prizes:
- **Grand Prize**: Transformative PYUSD off-ramp use case
- **Most Innovative Payment**: Private P2P fiat conversion
- **Best Consumer Experience**: Smooth PYUSD → fiat UX
