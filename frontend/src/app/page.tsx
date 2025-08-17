'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { usePrivy } from '@privy-io/react-auth';
import { usePyusd } from '../hooks/usePyusd';

export default function SellerInputPage() {
  const router = useRouter();
  const { login, logout, authenticated, user } = usePrivy();
  const { 
    userAddress, 
    formattedBalance, 
    needsApproval, 
    approvePyusd, 
    depositPyusd,
    isApproveLoading,
    isApproveSuccess,
    isDepositLoading,
    isDepositSuccess
  } = usePyusd();

  const [amount, setAmount] = useState('1000');
  const [bankAccount, setBankAccount] = useState('841273-1283712');
  const [service, setService] = useState('PayPal Hong Kong');
  const [isProcessing, setIsProcessing] = useState(false);
  const [step, setStep] = useState<'input' | 'approve' | 'deposit' | 'success'>('input');

  const handleConnectWallet = () => {
    login();
  };

  const handleSubmit = async () => {
    if (!authenticated || !userAddress) {
      handleConnectWallet();
      return;
    }

    // Validate amount
    const numAmount = parseFloat(amount.replace(/,/g, ''));
    const availableBalance = parseFloat(formattedBalance);
    
    if (numAmount > availableBalance) {
      alert(`Insufficient balance. You have ${formattedBalance} PYUSD available.`);
      return;
    }

    // Create order hash (simplified for demo)
    const orderData = {
      amount: numAmount,
      bankAccount,
      service,
      userAddress,
      timestamp: Date.now()
    };
    
    const orderHash = `0x${Array.from(new TextEncoder().encode(JSON.stringify(orderData)))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('')
      .slice(0, 64)
      .padEnd(64, '0')}` as `0x${string}`;

    try {
      setIsProcessing(true);
      
      // Step 1: Check if approval is needed
      if (needsApproval(amount.replace(/,/g, ''))) {
        setStep('approve');
        await approvePyusd(amount.replace(/,/g, ''));
        
        // Wait for approval to complete
        while (!isApproveSuccess) {
          await new Promise(resolve => setTimeout(resolve, 1000));
        }
      }

      // Step 2: Deposit PYUSD
      setStep('deposit');
      await depositPyusd(amount.replace(/,/g, ''), orderHash);
      
      // Wait for deposit to complete
      while (!isDepositSuccess) {
        await new Promise(resolve => setTimeout(resolve, 1000));
      }

      setStep('success');
      
      // Redirect after success
      setTimeout(() => {
        router.push('/confirm');
      }, 2000);

    } catch (error) {
      console.error('Transaction failed:', error);
      alert('Transaction failed. Please try again.');
      setStep('input');
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col min-h-screen p-6 bg-white">
        {/* Header */}
        <div className="mb-8 mt-4">
          <h1 className="text-3xl font-bold text-gray-800 mb-2">Vapor</h1>
          <p className="text-gray-600 text-lg">Private, Permissionless OffRamp</p>
          
          {/* Wallet Status */}
          {authenticated && userAddress && (
            <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium text-green-800">Wallet Connected</p>
                  <p className="text-xs text-green-600 font-mono">
                    {userAddress.slice(0, 6)}...{userAddress.slice(-4)}
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-sm font-medium text-green-800">PYUSD Balance</p>
                  <p className="text-sm font-bold text-green-900">{formattedBalance}</p>
                </div>
              </div>
            </div>
          )}
          
          {/* Transaction Progress */}
          {isProcessing && (
            <div className="mt-4 p-4 bg-blue-50 border border-blue-200 rounded-lg">
              <div className="flex items-center space-x-3">
                <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
                <div>
                  <p className="text-sm font-medium text-blue-800">
                    {step === 'approve' && 'Approving PYUSD spending...'}
                    {step === 'deposit' && 'Depositing PYUSD to Vapor Bridge...'}
                    {step === 'success' && 'Transaction completed successfully!'}
                  </p>
                  <p className="text-xs text-blue-600">
                    {step === 'approve' && 'Please confirm the approval transaction in your wallet'}
                    {step === 'deposit' && 'Please confirm the deposit transaction in your wallet'}
                    {step === 'success' && 'Redirecting to confirmation page...'}
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Form */}
        <div className="flex-1 space-y-6">
          {/* Amount Input */}
          <div>
            <div className="flex items-center mb-3">
              <label className="text-gray-700 font-medium text-base">
                Amount to Withdraw (PYUSD)
              </label>
              <button className="ml-2 w-5 h-5 rounded-full bg-gray-200 flex items-center justify-center">
                <span className="text-gray-500 text-xs font-bold">i</span>
              </button>
            </div>
            <div className="relative">
              <span className="absolute left-4 top-1/2 transform -translate-y-1/2 text-xl font-medium text-gray-700">
                $
              </span>
              <input
                type="text"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                className="w-full pl-8 pr-4 py-4 text-xl font-medium border-2 border-gray-200 rounded-lg focus:border-[#8BC34A] focus:outline-none"
                placeholder="100,000"
              />
            </div>
          </div>

          {/* Bank Account */}
          <div>
            <div className="flex items-center mb-3">
              <label className="text-gray-700 font-medium text-base">
                Bank Account
              </label>
              <button className="ml-2 w-5 h-5 rounded-full bg-gray-200 flex items-center justify-center">
                <span className="text-gray-500 text-xs font-bold">i</span>
              </button>
            </div>
            <input
              type="text"
              value={bankAccount}
              onChange={(e) => setBankAccount(e.target.value)}
              className="w-full px-4 py-4 text-base font-medium border-2 border-gray-200 rounded-lg focus:border-[#8BC34A] focus:outline-none"
              placeholder="Enter bank account"
            />
          </div>

          {/* Service Selection */}
          <div>
            <div className="flex items-center mb-3">
              <label className="text-gray-700 font-medium text-base">
                Service
              </label>
              <button className="ml-2 w-5 h-5 rounded-full bg-gray-200 flex items-center justify-center">
                <span className="text-gray-500 text-xs font-bold">i</span>
              </button>
            </div>
            <div className="relative">
              <select
                value={service}
                onChange={(e) => setService(e.target.value)}
                className="w-full px-4 py-4 text-base font-medium border-2 border-gray-200 rounded-lg focus:border-[#8BC34A] focus:outline-none appearance-none bg-white"
              >
                <option value="PayPal Hong Kong">PayPal Hong Kong</option>
                <option value="PayPal Singapore">PayPal Singapore</option>
                <option value="PayPal United States">PayPal United States</option>
                <option value="Wise">Wise</option>
                <option value="Bank Transfer">Bank Transfer</option>
              </select>
              <div className="absolute right-4 top-1/2 transform -translate-y-1/2">
                <svg width="12" height="8" viewBox="0 0 12 8" fill="none">
                  <path d="M1 1.5L6 6.5L11 1.5" stroke="#8BC34A" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              </div>
            </div>
          </div>
        </div>

        {/* Connect Wallet Button */}
        <div className="pb-6">
          {!isConnected ? (
            <button
              onClick={handleConnectWallet}
              className="w-full py-4 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold text-lg rounded-lg transition-colors"
            >
              CONNECT WALLET
            </button>
          ) : (
            <button
              onClick={handleSubmit}
              className="w-full py-4 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold text-lg rounded-lg transition-colors"
            >
              CONTINUE
            </button>
          )}
        </div>
      </div>
    </div>
  );
}