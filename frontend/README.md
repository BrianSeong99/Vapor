# Vapor Frontend

A mobile-first Next.js application for the Vapor private, permissionless off-ramp platform.

## Overview

This frontend implements the seller flow for the Vapor off-ramp service, allowing users to:
1. **Input Transaction Details** - Amount, bank account, and service selection
2. **Confirm Transaction** - Review details before blockchain signing
3. **Track Status** - Real-time progress through the off-ramp process
4. **Complete Transaction** - Confirmation and receipt

## Features

### 🎨 Design System
- **Mobile-First**: Optimized for mobile browsers (max-width: 400px)
- **Vapor Green Theme**: `#8BC34A` primary color with hover states
- **Clean UI**: Minimalist design matching the provided mockups
- **Responsive**: Works on all mobile screen sizes

### 📱 User Flow (Seller)
1. **`/` (Input Page)**: 
   - Amount input with PYUSD denomination
   - Bank account field
   - Service selection dropdown (PayPal Hong Kong, etc.)
   - Connect Wallet button (Privy integration ready)

2. **`/confirm` (Confirmation Page)**:
   - Transaction summary with fee calculation
   - Expandable transaction details
   - Confirm/Cancel actions

3. **`/status` (Status Tracking)**:
   - Multi-step progress indicator
   - Real-time status updates
   - Steps: Private Listing → Finding Fillers → Sending USD → View Receipt

4. **`/complete` (Thank You Page)**:
   - Success celebration with circular design
   - Transaction completion confirmation

### 🛠 Technical Stack
- **Framework**: Next.js 15.4.6 with App Router
- **Styling**: Tailwind CSS v4
- **TypeScript**: Full type safety
- **Fonts**: Geist Sans & Geist Mono
- **State Management**: React useState hooks
- **Navigation**: Next.js router with programmatic navigation

## Getting Started

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start
```

Open [http://localhost:3000](http://localhost:3000) to view the application.

## Development Features

### 🧭 Navigation Helper
A floating navigation component (top-right) allows quick testing between pages:
- Input → Confirm → Status → Complete

### 🎯 Page Structure
```
src/app/
├── page.tsx           # Input form (seller entry point)
├── confirm/page.tsx   # Transaction confirmation
├── status/page.tsx    # Progress tracking
├── complete/page.tsx  # Success page
├── layout.tsx         # Root layout with navigation
└── globals.css        # Vapor theme & mobile styles
```

### 🎨 Styling
- CSS Custom Properties for Vapor green theme
- Mobile-first responsive design
- Tailwind CSS for utility-first styling
- Focus states and hover effects

## Future Integrations

### 🔐 Wallet Integration (Ready for Privy)
The connect wallet functionality is prepared for Privy integration:
```typescript
const handleConnectWallet = () => {
  // TODO: Integrate with Privy wallet
  setIsConnected(true);
};
```

### 🔌 API Integration (Ready for Backend)
Status tracking is prepared for real-time backend updates:
```typescript
// TODO: Replace with real API calls
useEffect(() => {
  // Simulate API polling for status updates
}, []);
```

### 📊 Real-time Updates
The status page includes automatic progression simulation and is ready for WebSocket or polling integration.

## Design Compliance

✅ **Pixel-perfect implementation** of provided mockups  
✅ **Vapor branding** throughout (green theme, typography)  
✅ **Mobile-optimized** layout and interactions  
✅ **Smooth transitions** between states  
✅ **Accessible** form inputs and buttons  

## Browser Support

- **Mobile Safari** (iOS 14+)
- **Chrome Mobile** (Android 8+)
- **Desktop browsers** (for development/testing)

---

Built with ❤️ for the Vapor ecosystem
