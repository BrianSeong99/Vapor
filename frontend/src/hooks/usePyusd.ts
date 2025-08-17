'use client';

import { useEffect, useState } from 'react';
import { usePrivy, useWallets } from '@privy-io/react-auth';
import { useReadContract, useWriteContract, useWaitForTransactionReceipt, useChainId, useSwitchChain } from 'wagmi';
import { parseUnits, formatUnits, Address } from 'viem';
import { ERC20_ABI, VAPOR_BRIDGE_ABI, getPyusdAddress, getVaporBridgeAddress } from '../lib/contracts';

export function usePyusd() {
  const { ready, authenticated } = usePrivy();
  const { wallets } = useWallets();
  const currentChainId = useChainId();
  const { switchChain } = useSwitchChain();
  const targetChainId = 31337; // Local Anvil
  
  // Auto-switch to Anvil chain if not already connected
  useEffect(() => {
    if (authenticated && currentChainId !== targetChainId) {
      console.log(`Switching from chain ${currentChainId} to ${targetChainId}`);
      switchChain({ chainId: targetChainId });
    }
  }, [authenticated, currentChainId, targetChainId, switchChain]);
  
  // Get the user's embedded wallet (try multiple detection methods)
  const embeddedWallet = wallets.find(wallet => 
    wallet.walletClientType === 'privy' || 
    wallet.connectorType === 'embedded' ||
    wallet.connectorType === 'privy'
  ) || wallets[0]; // Fallback to first wallet
  
  const userAddress = embeddedWallet?.address as Address | undefined;

  // Debug logging
  console.log('usePyusd Debug:', {
    ready,
    authenticated,
    walletsCount: wallets.length,
    currentChainId,
    targetChainId,
    wallets: wallets.map(w => ({ 
      address: w.address, 
      clientType: w.walletClientType, 
      connectorType: w.connectorType 
    })),
    embeddedWallet: embeddedWallet ? 'found' : 'not found',
    userAddress,
    pyusdAddress: getPyusdAddress(targetChainId),
    vaporBridgeAddress: getVaporBridgeAddress(targetChainId)
  });

  // Contract addresses
  const pyusdAddress = getPyusdAddress(targetChainId);
  const vaporBridgeAddress = getVaporBridgeAddress(targetChainId);

  // Read PYUSD balance
  const { data: balance, refetch: refetchBalance, error: balanceError } = useReadContract({
    address: pyusdAddress,
    abi: ERC20_ABI,
    functionName: 'balanceOf',
    args: userAddress ? [userAddress] : undefined,
    chainId: targetChainId, // Force use of local Anvil chain
    query: {
      enabled: !!userAddress && currentChainId === targetChainId,
    },
  });

  // Debug balance query
  console.log('Balance Query Debug:', {
    balance,
    balanceError,
    queryEnabled: !!userAddress,
    contractAddress: pyusdAddress,
    userAddress
  });

  // Test direct RPC call for comparison
  useEffect(() => {
    if (userAddress) {
      const testDirectRPC = async () => {
        try {
          const response = await fetch('http://localhost:8545', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              jsonrpc: '2.0',
              method: 'eth_call',
              params: [{
                to: pyusdAddress,
                data: `0x70a08231000000000000000000000000${userAddress.slice(2)}`
              }, 'latest'],
              id: 1
            })
          });
          const result = await response.json();
          console.log('Direct RPC Balance Test:', {
            result: result.result,
            decimal: result.result ? parseInt(result.result, 16) : 0,
            pyusd: result.result ? parseInt(result.result, 16) / 1000000 : 0
          });
        } catch (error) {
          console.error('Direct RPC test failed:', error);
        }
      };
      testDirectRPC();
    }
  }, [userAddress, pyusdAddress]);

  // Read PYUSD allowance for VaporBridge
  const { data: allowance, refetch: refetchAllowance } = useReadContract({
    address: pyusdAddress,
    abi: ERC20_ABI,
    functionName: 'allowance',
    args: userAddress ? [userAddress, vaporBridgeAddress] : undefined,
    chainId: targetChainId, // Force use of local Anvil chain
    query: {
      enabled: !!userAddress && currentChainId === targetChainId,
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
    chainId: currentChainId,
    targetChainId,
    
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
