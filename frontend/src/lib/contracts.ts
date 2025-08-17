import { Address } from 'viem';

// PYUSD Contract Addresses
export const PYUSD_ADDRESSES = {
  // Ethereum Mainnet
  1: '0x6c3ea9036406852006290770BEdFcAbA0e23A0e8' as Address,
  // Sepolia Testnet (using MockPYUSD from our deployment - will be updated by deployment script)
  11155111: '0x9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0' as Address,
  // Local Anvil (using MockPYUSD from our deployment)
  31337: '0x5FC8d32690cc91D4c39d9d3abcBD16989F875707' as Address,
} as const;

// VaporBridge Contract Addresses (from our deployment)
export const VAPOR_BRIDGE_ADDRESSES = {
  // Local Anvil
  31337: '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9' as Address,
  // Sepolia Testnet
  11155111: '0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9' as Address,
} as const;

// ERC20 ABI (simplified for PYUSD operations)
export const ERC20_ABI = [
  {
    inputs: [{ name: 'account', type: 'address' }],
    name: 'balanceOf',
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [
      { name: 'spender', type: 'address' },
      { name: 'amount', type: 'uint256' },
    ],
    name: 'approve',
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [
      { name: 'spender', type: 'address' },
      { name: 'owner', type: 'address' },
    ],
    name: 'allowance',
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
    type: 'function',
  },
  {
    inputs: [],
    name: 'decimals',
    outputs: [{ name: '', type: 'uint8' }],
    stateMutability: 'view',
    type: 'function',
  },
] as const;

// VaporBridge ABI (simplified for deposit operation)
export const VAPOR_BRIDGE_ABI = [
  {
    inputs: [
      { name: 'tokenId', type: 'uint256' },
      { name: 'amount', type: 'uint256' },
      { name: 'bankingHash', type: 'bytes32' },
    ],
    name: 'deposit',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
  {
    inputs: [
      { name: 'tokenId', type: 'uint256' },
      { name: 'tokenAddress', type: 'address' },
    ],
    name: 'addSupportedToken',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function',
  },
] as const;

// Helper function to get contract addresses based on chain
export function getPyusdAddress(chainId: number): Address {
  return PYUSD_ADDRESSES[chainId as keyof typeof PYUSD_ADDRESSES] || PYUSD_ADDRESSES[11155111];
}

export function getVaporBridgeAddress(chainId: number): Address {
  return VAPOR_BRIDGE_ADDRESSES[chainId as keyof typeof VAPOR_BRIDGE_ADDRESSES] || VAPOR_BRIDGE_ADDRESSES[31337];
}
