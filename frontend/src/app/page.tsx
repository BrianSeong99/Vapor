'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { usePrivy } from '@privy-io/react-auth';
import { usePyusd } from '../hooks/usePyusd';

export default function SellerInputPage() {
  const router = useRouter();
  const { login, logout, authenticated } = usePrivy();
  const { 
    userAddress, 
    formattedBalance
  } = usePyusd();

  const [amount, setAmount] = useState('1000');
  const [bankAccount, setBankAccount] = useState('841273-1283712');
  const [service, setService] = useState('PayPal Hong Kong');

  const handleConnectWallet = () => {
    login();
  };

  const handleContinue = () => {
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

    // Navigate to review/confirm page with the form data
    const params = new URLSearchParams({
      amount: amount.replace(/,/g, ''),
      bankAccount,
      service,
    });
    
    router.push(`/confirm?${params.toString()}`);
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

        {/* Action Button */}
        <div className="pb-6">
          {!authenticated ? (
            <button
              onClick={handleConnectWallet}
              className="w-full py-4 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold text-lg rounded-lg transition-colors"
            >
              CONNECT WALLET
            </button>
          ) : (
            <div className="space-y-3">
              {authenticated && userAddress && (
                <div className="flex justify-between items-center text-sm text-gray-600">
                  <span>Available: {formattedBalance} PYUSD</span>
                  <button
                    onClick={logout}
                    className="text-red-600 hover:text-red-800 text-xs underline"
                  >
                    Disconnect
                  </button>
                </div>
              )}
              
              <button
                onClick={handleContinue}
                disabled={!authenticated || !userAddress}
                className={`w-full py-4 font-semibold text-lg rounded-lg transition-colors ${
                  !authenticated || !userAddress
                    ? 'bg-gray-400 text-gray-600 cursor-not-allowed'
                    : 'bg-[#8BC34A] hover:bg-[#689F38] text-white'
                }`}
              >
                CONTINUE
              </button>
              
              {authenticated && (
                <p className="text-xs text-gray-500 text-center">
                  Review your withdrawal details on the next page before confirming.
                </p>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}