'use client';

import { PrivyProvider } from '@privy-io/react-auth';
import { WagmiProvider } from '@privy-io/wagmi';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { http } from 'viem';
import { mainnet, sepolia } from 'viem/chains';
import { createConfig } from 'wagmi';

// Wagmi configuration
const wagmiConfig = createConfig({
  chains: [mainnet, sepolia],
  transports: {
    [mainnet.id]: http(),
    [sepolia.id]: http(),
  },
});

const queryClient = new QueryClient();

export default function Providers({ children }: { children: React.ReactNode }) {
  const privyAppId = process.env.NEXT_PUBLIC_PRIVY_APP_ID;
  
  // If no Privy App ID is configured, show setup instructions
  if (!privyAppId || privyAppId === 'clxxx-xxx-xxx') {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 p-6">
        <div className="max-w-md w-full bg-white rounded-lg shadow-md p-6">
          <h2 className="text-xl font-bold text-gray-800 mb-4">Privy Setup Required</h2>
          <div className="space-y-3 text-sm text-gray-600">
            <p>To use Vapor with Privy integration:</p>
            <ol className="list-decimal list-inside space-y-2">
              <li>Create a Privy app at <a href="https://console.privy.id/" className="text-blue-600 underline" target="_blank" rel="noopener noreferrer">console.privy.id</a></li>
              <li>Copy your App ID</li>
              <li>Create <code className="bg-gray-100 px-1 rounded">.env.local</code> file:</li>
            </ol>
            <pre className="bg-gray-100 p-3 rounded text-xs overflow-x-auto">
{`NEXT_PUBLIC_PRIVY_APP_ID=your-app-id-here`}
            </pre>
            <p className="text-xs text-gray-500">
              See <code>PRIVY_SETUP.md</code> for detailed instructions.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <PrivyProvider
      appId={privyAppId}
      config={{
        loginMethods: ['email', 'wallet', 'google'],
        appearance: {
          theme: 'light',
          accentColor: '#8BC34A',
        },
        embeddedWallets: {
          createOnLogin: 'users-without-wallets',
          requireUserPasswordOnCreate: false,
        },
        defaultChain: sepolia, // Use Sepolia for testing
      }}
    >
      <QueryClientProvider client={queryClient}>
        <WagmiProvider config={wagmiConfig}>
          {children}
        </WagmiProvider>
      </QueryClientProvider>
    </PrivyProvider>
  );
}
