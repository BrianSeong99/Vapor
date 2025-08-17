'use client';

import { useRouter } from 'next/navigation';
import { usePrivy } from '@privy-io/react-auth';
import { usePyusd } from '../hooks/usePyusd';
import { useState } from 'react';
import { fundWalletDirect } from '../utils/fundWallet';

export default function Navigation() {
  const router = useRouter();
  const { authenticated, user, login, logout } = usePrivy();
  const { userAddress, formattedBalance, refetchBalance, chainId, targetChainId } = usePyusd();
  const [showWallet, setShowWallet] = useState(false);

  const handleFundWallet = async () => {
    if (!userAddress) return;
    
    try {
      const result = await fundWalletDirect(userAddress);
      alert(`Funding command generated! Check console for details.\n\nCommand: ${result.command}`);
      
      // Refresh balance after funding
      setTimeout(() => {
        refetchBalance();
      }, 2000);
    } catch (error) {
      console.error('Fund wallet error:', error);
      alert('Failed to generate funding command');
    }
  };

  return (
    <div className="fixed top-4 right-4 z-50">
      <div className="bg-black bg-opacity-50 rounded-lg p-2 text-xs">
        {/* Seller Flow */}
        <div className="mb-2">
          <div className="text-white text-xs mb-1 px-1">Seller:</div>
          <div className="space-x-1">
            <button 
              onClick={() => router.push('/')}
              className="px-2 py-1 bg-[#8BC34A] rounded text-white hover:bg-[#689F38] text-xs"
            >
              Input
            </button>
            <button 
              onClick={() => router.push('/confirm')}
              className="px-2 py-1 bg-[#8BC34A] rounded text-white hover:bg-[#689F38] text-xs"
            >
              Confirm
            </button>
            <button 
              onClick={() => router.push('/status')}
              className="px-2 py-1 bg-[#8BC34A] rounded text-white hover:bg-[#689F38] text-xs"
            >
              Status
            </button>
            <button 
              onClick={() => router.push('/complete')}
              className="px-2 py-1 bg-[#8BC34A] rounded text-white hover:bg-[#689F38] text-xs"
            >
              Complete
            </button>
          </div>
        </div>

        {/* Filler Flow */}
        <div className="mb-2">
          <div className="text-white text-xs mb-1 px-1">Filler:</div>
          <div className="space-x-1">
            <button 
              onClick={() => router.push('/filler')}
              className="px-2 py-1 bg-[#4FC3F7] rounded text-white hover:bg-[#29B6F6] text-xs"
            >
              Orders
            </button>
            <button 
              onClick={() => router.push('/filler/balance')}
              className="px-2 py-1 bg-[#4FC3F7] rounded text-white hover:bg-[#29B6F6] text-xs"
            >
              Balance
            </button>
          </div>
        </div>

        {/* Privy Wallet Section */}
        <div>
          <div className="text-white text-xs mb-1 px-1">Wallet:</div>
          <div className="space-y-1">
            {!authenticated ? (
              <button 
                onClick={login}
                className="w-full px-2 py-1 bg-[#8BC34A] rounded text-white hover:bg-[#689F38] text-xs"
              >
                Connect Wallet
              </button>
            ) : (
              <>
                <button 
                  onClick={() => setShowWallet(!showWallet)}
                  className="w-full px-2 py-1 bg-purple-600 rounded text-white hover:bg-purple-700 text-xs"
                >
                  {showWallet ? 'Hide Wallet' : 'Show Wallet'}
                </button>
                
                {showWallet && (
                  <div className="bg-black bg-opacity-70 rounded p-2 space-y-1">
                    <div className="text-white text-xs">
                      <div className="text-gray-300">Network:</div>
                      <div className={`text-xs font-bold ${chainId === targetChainId ? 'text-green-400' : 'text-red-400'}`}>
                        Chain {chainId} {chainId === targetChainId ? '✅' : '❌'}
                      </div>
                      {chainId !== targetChainId && (
                        <div className="text-xs text-yellow-400">
                          Need Chain {targetChainId} (Anvil)
                        </div>
                      )}
                    </div>
                    
                    <div className="text-white text-xs">
                      <div className="text-gray-300">Address:</div>
                      <div className="font-mono text-xs break-all">
                        {userAddress ? `${userAddress.slice(0, 8)}...${userAddress.slice(-6)}` : 'Loading...'}
                      </div>
                      {userAddress && (
                        <div className="text-xs text-gray-400 mt-1">
                          Full: {userAddress}
                        </div>
                      )}
                    </div>
                    
                    <div className="text-white text-xs">
                      <div className="text-gray-300">PYUSD Balance:</div>
                      <div className="font-bold text-green-400">
                        {formattedBalance || '0'} PYUSD
                      </div>
                      <div className="flex space-x-1 mt-1">
                        <button 
                          onClick={() => refetchBalance()}
                          className="flex-1 px-2 py-1 bg-blue-600 rounded text-white hover:bg-blue-700 text-xs"
                        >
                          Refresh
                        </button>
                        {formattedBalance === '0' && userAddress && (
                          <button 
                            onClick={handleFundWallet}
                            className="flex-1 px-2 py-1 bg-yellow-600 rounded text-white hover:bg-yellow-700 text-xs"
                          >
                            Fund
                          </button>
                        )}
                      </div>
                    </div>
                    
                    {user?.email && (
                      <div className="text-white text-xs">
                        <div className="text-gray-300">Email:</div>
                        <div className="text-xs break-all">
                          {user.email.address}
                        </div>
                      </div>
                    )}
                    
                    <button 
                      onClick={logout}
                      className="w-full px-2 py-1 bg-red-600 rounded text-white hover:bg-red-700 text-xs mt-1"
                    >
                      Disconnect
                    </button>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
