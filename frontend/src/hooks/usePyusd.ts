'use client';

import { useEffect, useState } from 'react';
import { usePrivy, useWallets } from '@privy-io/react-auth';
import { useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { parseUnits, formatUnits, Address } from 'viem';
import { ERC20_ABI, VAPOR_BRIDGE_ABI, getPyusdAddress, getVaporBridgeAddress } from '../lib/contracts';

export function usePyusd() {
  const { ready, authenticated, user } = usePrivy();
  const { wallets } = useWallets();
  const [chainId, setChainId] = useState<number>(11155111); // Default to Sepolia
  
  // Get the user's embedded wallet
  const embeddedWallet = wallets.find(wallet => wallet.walletClientType === 'privy');
  const userAddress = embeddedWallet?.address as Address | undefined;

  // Contract addresses
  const pyusdAddress = getPyusdAddress(chainId);
  const vaporBridgeAddress = getVaporBridgeAddress(chainId);

  // Read PYUSD balance
  const { data: balance, refetch: refetchBalance } = useReadContract({
    address: pyusdAddress,
    abi: ERC20_ABI,
    functionName: 'balanceOf',
    args: userAddress ? [userAddress] : undefined,
    query: {
      enabled: !!userAddress,
    },
  });

  // Read PYUSD allowance for VaporBridge
  const { data: allowance, refetch: refetchAllowance } = useReadContract({
    address: pyusdAddress,
    abi: ERC20_ABI,
    functionName: 'allowance',
    args: userAddress ? [userAddress, vaporBridgeAddress] : undefined,
    query: {
      enabled: !!userAddress,
    },
  });

  // Write contracts
  const { writeContract: writeApprove, data: approveHash } = useWriteContract();
  const { writeContract: writeDeposit, data: depositHash } = useWriteContract();

  // Wait for transactions
  const { isLoading: isApproveLoading, isSuccess: isApproveSuccess } = useWaitForTransactionReceipt({
    hash: approveHash,
  });

  const { isLoading: isDepositLoading, isSuccess: isDepositSuccess } = useWaitForTransactionReceipt({
    hash: depositHash,
  });

  // Helper functions
  const formatBalance = (balance: bigint | undefined) => {
    if (!balance) return '0';
    return formatUnits(balance, 6); // PYUSD has 6 decimals
  };

  const parseAmount = (amount: string) => {
    return parseUnits(amount, 6); // PYUSD has 6 decimals
  };

  const needsApproval = (amount: string) => {
    if (!allowance) return true;
    const amountBigInt = parseAmount(amount);
    return allowance < amountBigInt;
  };

  // Approve PYUSD spending
  const approvePyusd = async (amount: string) => {
    if (!userAddress) throw new Error('Wallet not connected');
    
    const amountBigInt = parseAmount(amount);
    
    writeApprove({
      address: pyusdAddress,
      abi: ERC20_ABI,
      functionName: 'approve',
      args: [vaporBridgeAddress, amountBigInt],
    });
  };

  // Deposit PYUSD to VaporBridge
  const depositPyusd = async (amount: string, orderHash: `0x${string}`) => {
    if (!userAddress) throw new Error('Wallet not connected');
    
    const amountBigInt = parseAmount(amount);
    
    writeDeposit({
      address: vaporBridgeAddress,
      abi: VAPOR_BRIDGE_ABI,
      functionName: 'deposit',
      args: [amountBigInt, orderHash],
    });
  };

  // Refresh data after successful transactions
  useEffect(() => {
    if (isApproveSuccess || isDepositSuccess) {
      refetchBalance();
      refetchAllowance();
    }
  }, [isApproveSuccess, isDepositSuccess, refetchBalance, refetchAllowance]);

  return {
    // State
    ready,
    authenticated,
    userAddress,
    chainId,
    
    // Balance data
    balance,
    formattedBalance: formatBalance(balance),
    allowance,
    
    // Transaction functions
    approvePyusd,
    depositPyusd,
    needsApproval,
    parseAmount,
    
    // Transaction states
    approveHash,
    depositHash,
    isApproveLoading,
    isApproveSuccess,
    isDepositLoading,
    isDepositSuccess,
    
    // Utility
    refetchBalance,
    refetchAllowance,
  };
}
