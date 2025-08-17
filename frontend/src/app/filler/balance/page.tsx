'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';

interface WalletAddress {
  address: string;
  balance: string;
  percentage: number;
}

interface FillerBalance {
  filler_id: string;
  total_balance: string;
  available_balance: string;
  locked_balance: string;
  completed_jobs: number;
  wallets: WalletAddress[];
}

export default function FillerBalancePage() {
  const router = useRouter();
  const [fillerBalance, setFillerBalance] = useState<FillerBalance | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Mock filler ID - in a real app, this would come from authentication
  const fillerId = 'filler-123';

  // Fetch filler balance from backend
  const fetchFillerBalance = async () => {
    try {
      setLoading(true);
      const response = await fetch(`http://localhost:3000/api/v1/fillers/${fillerId}/balance`);
      if (!response.ok) {
        throw new Error(`Failed to fetch balance: ${response.statusText}`);
      }
      
      const data = await response.json();
      setFillerBalance(data);
      setError(null);
    } catch (err) {
      console.error('Error fetching filler balance:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch balance');
    } finally {
      setLoading(false);
    }
  };

  // Add wallet address
  const handleAddAddress = async () => {
    const newAddress = '0X' + Math.random().toString(36).substring(2, 15) + '...';
    
    try {
      const response = await fetch(`http://localhost:3000/api/v1/fillers/${fillerId}/wallets`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          wallet_address: newAddress,
          balance: '0'
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to add wallet: ${response.statusText}`);
      }

      // Refresh balance to get updated wallet list
      await fetchFillerBalance();
    } catch (err) {
      console.error('Error adding wallet:', err);
      alert('Failed to add wallet. Please try again.');
    }
  };

  const handleUpdatePercentage = (index: number, percentage: number) => {
    if (!fillerBalance) return;
    
    const updatedWallets = fillerBalance.wallets.map((wallet, i) => 
      i === index ? { ...wallet, percentage } : wallet
    );
    
    setFillerBalance(prev => prev ? { ...prev, wallets: updatedWallets } : null);
  };

  const handleClaimMoney = async () => {
    if (!fillerBalance) return;
    
    try {
      const claims = fillerBalance.wallets
        .filter(wallet => parseFloat(wallet.balance) > 0)
        .map(wallet => ({
          amount: wallet.balance,
          destination_address: wallet.address
        }));

      if (claims.length === 0) {
        alert('No balance available to claim');
        return;
      }

      const response = await fetch('http://localhost:3000/api/v1/fillers/claim', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          filler_id: fillerId,
          claims
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to claim tokens: ${response.statusText}`);
      }

      const result = await response.json();
      alert(`Claim successful! Transaction: ${result.transaction_hash}`);
      
      // Refresh balance
      await fetchFillerBalance();
    } catch (err) {
      console.error('Error claiming tokens:', err);
      alert('Failed to claim tokens. Please try again.');
    }
  };

  // Fetch balance on component mount
  useEffect(() => {
    fetchFillerBalance();
  }, []);

  // Format balance for display (convert from wei to readable format)
  const formatBalance = (balance: string) => {
    const num = parseFloat(balance);
    if (num >= 1e21) return (num / 1e21).toFixed(0); // Convert from wei (assuming 18 decimals)
    return (num / 1e6).toFixed(0); // Convert from 6 decimal places
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

        {/* Loading State */}
        {loading && (
          <div className="p-6">
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <span className="ml-3 text-gray-600">Loading balance...</span>
            </div>
          </div>
        )}

        {/* Error State */}
        {error && (
          <div className="p-6">
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <p className="text-red-800">{error}</p>
              <button 
                onClick={fetchFillerBalance}
                className="mt-2 text-red-600 hover:text-red-800 underline"
              >
                Retry
              </button>
            </div>
          </div>
        )}

        {/* Balance Info */}
        {!loading && !error && fillerBalance && (
          <div className="p-6 space-y-6">
            {/* Balance */}
            <div>
              <div className="flex justify-between items-center mb-2">
                <span className="text-gray-700 font-medium">Your Balance</span>
                <span className="text-2xl font-bold text-gray-900">
                  {formatBalance(fillerBalance.total_balance)} USDT
                </span>
              </div>
              <div className="flex justify-between items-center text-sm text-gray-600">
                <span>Available: {formatBalance(fillerBalance.available_balance)} USDT</span>
                <span>Locked: {formatBalance(fillerBalance.locked_balance)} USDT</span>
              </div>
            </div>

            {/* Jobs Filled */}
            <div>
              <div className="flex justify-between items-center">
                <span className="text-gray-700 font-medium">Jobs Filled</span>
                <span className="text-xl font-semibold text-gray-900">{fillerBalance.completed_jobs}</span>
              </div>
            </div>

            {/* Wallet Addresses */}
            <div>
              <div className="flex items-center mb-4">
                <span className="text-gray-700 font-medium">Wallet Addresses</span>
                <button className="ml-2 w-5 h-5 rounded-full bg-gray-200 flex items-center justify-center">
                  <span className="text-gray-500 text-xs font-bold">i</span>
                </button>
              </div>

              <div className="space-y-3">
                {fillerBalance.wallets.map((wallet, index) => (
                  <div key={index} className="flex items-center space-x-3">
                    {/* Address Display */}
                    <div className="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm font-mono bg-gray-50">
                      {wallet.address}
                    </div>
                    
                    {/* Balance Display */}
                    <div className="px-3 py-2 border border-gray-300 rounded-lg text-sm bg-gray-50 text-center min-w-[80px]">
                      {formatBalance(wallet.balance)} USDT
                    </div>
                    
                    {/* Percentage Input */}
                    <div className="relative">
                      <input
                        type="number"
                        value={wallet.percentage}
                        onChange={(e) => handleUpdatePercentage(index, parseFloat(e.target.value) || 0)}
                        className="w-16 px-2 py-2 border border-gray-300 rounded-lg text-sm text-center focus:border-[#4FC3F7] focus:outline-none"
                        min="0"
                        max="100"
                        step="0.1"
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
        )}

        {/* Claim Button */}
        {!loading && !error && fillerBalance && (
          <div className="p-6 mt-auto">
            <button
              onClick={handleClaimMoney}
              disabled={parseFloat(fillerBalance.available_balance) === 0}
              className={`w-full py-4 font-semibold text-lg rounded-lg transition-colors ${
                parseFloat(fillerBalance.available_balance) === 0
                  ? 'bg-gray-400 text-gray-600 cursor-not-allowed'
                  : 'bg-[#4FC3F7] hover:bg-[#29B6F6] text-white'
              }`}
            >
              Claim Money
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
