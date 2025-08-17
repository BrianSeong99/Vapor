'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

interface WalletAddress {
  address: string;
  percentage: number;
}

export default function FillerBalancePage() {
  const router = useRouter();
  const [walletAddresses, setWalletAddresses] = useState<WalletAddress[]>([
    { address: '0X8aj81j2gasjd81as...', percentage: 32 },
    { address: '0X8aj81j2gasjd81as...', percentage: 68 }
  ]);

  const balance = '150,000';
  const jobsFilled = 2;

  const handleAddAddress = () => {
    setWalletAddresses(prev => [
      ...prev,
      { address: '0X' + Math.random().toString(36).substring(2, 15) + '...', percentage: 0 }
    ]);
  };

  const handleUpdatePercentage = (index: number, percentage: number) => {
    setWalletAddresses(prev => prev.map((wallet, i) => 
      i === index ? { ...wallet, percentage } : wallet
    ));
  };

  const handleClaimMoney = () => {
    // TODO: Implement claim functionality
    alert('Claim functionality will be implemented with backend integration');
  };

  const handleBack = () => {
    router.push('/filler');
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col min-h-screen bg-white">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200">
          <h1 className="text-2xl font-bold text-gray-800">Vapor Jobs</h1>
          <button 
            onClick={handleBack}
            className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M10 12l-4-4 4-4" stroke="#666" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        </div>

        {/* Balance Info */}
        <div className="p-6 space-y-6">
          {/* Balance */}
          <div>
            <div className="flex justify-between items-center mb-2">
              <span className="text-gray-700 font-medium">Your Balance</span>
              <span className="text-2xl font-bold text-gray-900">{balance} USDT</span>
            </div>
          </div>

          {/* Jobs Filled */}
          <div>
            <div className="flex justify-between items-center">
              <span className="text-gray-700 font-medium">Jobs Filled</span>
              <span className="text-xl font-semibold text-gray-900">{jobsFilled}</span>
            </div>
          </div>

          {/* Wallet Addresses */}
          <div>
            <div className="flex items-center mb-4">
              <span className="text-gray-700 font-medium">Wallet Address</span>
              <button className="ml-2 w-5 h-5 rounded-full bg-gray-200 flex items-center justify-center">
                <span className="text-gray-500 text-xs font-bold">i</span>
              </button>
            </div>

            <div className="space-y-3">
              {walletAddresses.map((wallet, index) => (
                <div key={index} className="flex items-center space-x-3">
                  {/* Address Input */}
                  <input
                    type="text"
                    value={wallet.address}
                    onChange={(e) => {
                      const newAddresses = [...walletAddresses];
                      newAddresses[index].address = e.target.value;
                      setWalletAddresses(newAddresses);
                    }}
                    className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono focus:border-[#4FC3F7] focus:outline-none"
                    placeholder="Wallet address"
                  />
                  
                  {/* Percentage Input */}
                  <div className="relative">
                    <input
                      type="number"
                      value={wallet.percentage}
                      onChange={(e) => handleUpdatePercentage(index, parseInt(e.target.value) || 0)}
                      className="w-16 px-2 py-2 border border-gray-300 rounded-lg text-sm text-center focus:border-[#4FC3F7] focus:outline-none"
                      min="0"
                      max="100"
                    />
                    <span className="absolute right-1 top-1/2 transform -translate-y-1/2 text-xs text-gray-500">%</span>
                  </div>
                </div>
              ))}

              {/* Add Address Button */}
              <button
                onClick={handleAddAddress}
                className="w-full py-2 text-[#4FC3F7] font-medium text-sm border border-[#4FC3F7] rounded-lg hover:bg-[#4FC3F7] hover:text-white transition-colors"
              >
                + Add Address
              </button>
            </div>
          </div>
        </div>

        {/* Claim Button */}
        <div className="p-6 mt-auto">
          <button
            onClick={handleClaimMoney}
            className="w-full py-4 bg-[#4FC3F7] hover:bg-[#29B6F6] text-white font-semibold text-lg rounded-lg transition-colors"
          >
            Claim Money
          </button>
        </div>
      </div>
    </div>
  );
}
