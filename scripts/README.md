# Vapor Development Scripts

This directory contains scripts to help you manage the Vapor development environment.

## 🚀 Available Scripts

### 1. **Full Environment Setup**
```bash
./scripts/start_dev_environment.sh
```
**What it does:**
- ✅ Starts Anvil blockchain (port 8545)
- ✅ Deploys all Vapor contracts
- ✅ Funds test accounts with USDC
- ✅ Starts backend server (port 3000)
- ⏸️ Frontend ready to start (port 8080)

**Use when:** You want to start everything from scratch

---

### 2. **Anvil Blockchain Only** 🔗
```bash
./scripts/start_anvil_only.sh
```
**What it does:**
- ✅ Starts Anvil blockchain (port 8545) **in foreground**
- ✅ Deploys all Vapor contracts (in background)
- ✅ Funds test accounts with USDC
- 📄 Creates `deployed_addresses.env`

**Use when:** You only need the blockchain and contracts  
**Note:** Runs in foreground - press `Ctrl+C` to stop

---

### 3. **Backend Server Only** 🖥️
```bash
./scripts/start_backend_only.sh
```
**What it does:**
- ✅ Starts backend server (port 3000) **in foreground**
- ✅ Uses existing contract addresses
- ✅ Creates database and environment

**Use when:** Anvil is running, you just need the backend  
**Note:** Runs in foreground - press `Ctrl+C` to stop

---

### 4. **Frontend Only** 📱
```bash
./scripts/start_frontend_only.sh
```
**What it does:**
- ✅ Starts Next.js dev server (port 8080) **in foreground**
- ✅ Installs dependencies if needed
- ✅ Shows live development logs

**Use when:** Backend and Anvil are running, you just need the frontend  
**Note:** Runs in foreground - press `Ctrl+C` to stop

---

## 📋 Port Configuration

| Service | Port | URL |
|---------|------|-----|
| **Anvil Blockchain** | 8545 | `http://localhost:8545` |
| **Backend API** | 3000 | `http://localhost:3000` |
| **Frontend** | 8080 | `http://localhost:8080` |

## 🧪 Quick Tests

### Test Anvil
```bash
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  http://localhost:8545
```

### Test Backend
```bash
curl http://localhost:3000/health
curl http://localhost:3000/api/v1/fillers/discovery
```

### Start Frontend
```bash
./scripts/start_frontend_only.sh
# Opens on http://localhost:8080
```

## 🔄 Common Workflows

### **Full Restart**
```bash
# Kill everything
pkill -f anvil
pkill -f vapor-server
pkill -f "next dev"

# Start everything
./scripts/start_dev_environment.sh
# In another terminal:
./scripts/start_frontend_only.sh
```

### **Just Restart Blockchain**
```bash
pkill -f anvil
./scripts/start_anvil_only.sh
```

### **Just Restart Backend**
```bash
pkill -f vapor-server
./scripts/start_backend_only.sh
```

### **Just Restart Frontend**
```bash
pkill -f "next dev"
./scripts/start_frontend_only.sh
```

## 📄 Generated Files

Each script creates helpful files:

- **`deployed_addresses.env`** - Contract addresses
- **`anvil.log`** - Blockchain logs  
- **`backend.log`** - Backend server logs
- **`dev_environment.info`** - Complete environment info
- **`anvil_environment.info`** - Anvil-specific info

## 💰 Test Accounts

All scripts fund these accounts with 1000 USDC each:

| Account | Address | Private Key |
|---------|---------|-------------|
| **Account 0** | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` |
| **Account 1** | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` |
| **Account 2** | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | `0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a` |
| **Account 3** | `0x90F79bf6EB2c4f870365E785982E1f101E93b906` | `0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6` |
| **Account 4** | `0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65` | `0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a` |

## 🎯 Tips

- **Check what's running:** `ps aux | grep -E "(anvil|vapor|next)"`
- **View logs:** `tail -f anvil.log` or `tail -f backend/backend.log`
- **Kill specific service:** `pkill -f anvil` or `pkill -f vapor-server`
- **Contract addresses:** `cat deployed_addresses.env`
