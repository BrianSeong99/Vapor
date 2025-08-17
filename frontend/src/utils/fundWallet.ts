// Helper function to fund a Privy wallet with test PYUSD from Account 0

export async function fundPrivyWallet(walletAddress: string, amount: string = "1000") {
  try {
    const response = await fetch('/api/fund-wallet', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        walletAddress,
        amount,
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to fund wallet: ${response.statusText}`);
    }

    const result = await response.json();
    return result;
  } catch (error) {
    console.error('Error funding wallet:', error);
    throw error;
  }
}

// Direct blockchain call to fund wallet (for development)
export async function fundWalletDirect(walletAddress: string) {
  // This would typically be done through the backend
  // For now, we'll show instructions to the user
  const testAccountPrivateKey = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
  const mockUsdcAddress = "0x5FbDB2315678afecb367f032d93F642f64180aa3";
  
  console.log(`To fund wallet ${walletAddress}:`);
  console.log(`cast send ${mockUsdcAddress} "transfer(address,uint256)" ${walletAddress} 1000000000 --rpc-url http://localhost:8545 --private-key ${testAccountPrivateKey}`);
  
  return {
    message: "Check console for funding command",
    command: `cast send ${mockUsdcAddress} "transfer(address,uint256)" ${walletAddress} 1000000000 --rpc-url http://localhost:8545 --private-key ${testAccountPrivateKey}`
  };
}
