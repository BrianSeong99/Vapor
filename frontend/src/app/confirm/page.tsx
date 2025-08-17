'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { usePrivy } from '@privy-io/react-auth';
import { usePyusd } from '../../hooks/usePyusd';

export default function ConfirmPage() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const { authenticated } = usePrivy();
  
  // Get form data from URL parameters
  const amount = searchParams.get('amount') || '0';
  const bankAccount = searchParams.get('bankAccount') || '';
  const service = searchParams.get('service') || '';
  
  const { 
    userAddress, 
    needsApproval, 
    approvePyusd, 
    depositPyusd,
    isApproveSuccess,
    isDepositSuccess,
    isApproveLoading,
    isDepositLoading,
    isApproveError,
    approveError,
    isDepositError,
    depositError,
    approveHash,
    depositHash
  } = usePyusd();

  const [showTransactionDetails, setShowTransactionDetails] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [step, setStep] = useState<'input' | 'approve' | 'deposit' | 'success'>('input');
  const [currentBankingHash, setCurrentBankingHash] = useState<`0x${string}` | null>(null);

  // Handle approval success - proceed to deposit
  useEffect(() => {
    if (isApproveSuccess && step === 'approve' && currentBankingHash) {
      console.log('Approval successful, proceeding to deposit');
      setStep('deposit');
      depositPyusd(amount, currentBankingHash);
    }
  }, [isApproveSuccess, step, currentBankingHash, amount, depositPyusd]);

  // Handle deposit success - show success and redirect
  useEffect(() => {
    if (isDepositSuccess && step === 'deposit') {
      console.log('Deposit successful');
      setStep('success');
      setTimeout(() => {
        router.push('/status');
      }, 2000);
    }
  }, [isDepositSuccess, step, router]);

  // Reset processing state when transactions complete or fail
  useEffect(() => {
    if (step === 'success') {
      setIsProcessing(false);
    }
  }, [step]);

  // Handle transaction errors
  useEffect(() => {
    if (isApproveError && step === 'approve') {
      console.error('Approval transaction failed:', approveError);
      alert(`Approval transaction failed: ${approveError?.message || 'Unknown error'}`);
      setStep('input');
      setIsProcessing(false);
      setCurrentBankingHash(null);
    }
  }, [isApproveError, approveError, step]);

  useEffect(() => {
    if (isDepositError && step === 'deposit') {
      console.error('Deposit transaction failed:', depositError);
      alert(`Deposit transaction failed: ${depositError?.message || 'Unknown error'}`);
      setStep('input');
      setIsProcessing(false);
      setCurrentBankingHash(null);
    }
  }, [isDepositError, depositError, step]);

  // Add timeout handling for stuck transactions
  useEffect(() => {
    if (!isProcessing) return;

    const timeout = setTimeout(() => {
      if (step === 'approve' && !isApproveSuccess && !isApproveLoading && !isApproveError) {
        console.log('Approval timeout - transaction may have failed');
        alert('Approval transaction timed out. Please try again.');
        setStep('input');
        setIsProcessing(false);
        setCurrentBankingHash(null);
      } else if (step === 'deposit' && !isDepositSuccess && !isDepositLoading && !isDepositError) {
        console.log('Deposit timeout - transaction may have failed');
        alert('Deposit transaction timed out. Please try again.');
        setStep('input');
        setIsProcessing(false);
        setCurrentBankingHash(null);
      }
    }, 60000); // 60 second timeout

    return () => clearTimeout(timeout);
  }, [isProcessing, step, isApproveSuccess, isApproveLoading, isApproveError, isDepositSuccess, isDepositLoading, isDepositError]);

  const handleConfirm = async () => {
    if (!authenticated || !userAddress) {
      alert('Please connect your wallet first');
      router.push('/');
      return;
    }

    // Create banking hash (simplified for demo)
    const bankingData = {
      amount: parseFloat(amount),
      bankAccount,
      service,
      userAddress,
      timestamp: Date.now()
    };
    
    const bankingHash = `0x${Array.from(new TextEncoder().encode(JSON.stringify(bankingData)))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('')
      .slice(0, 64)
      .padEnd(64, '0')}` as `0x${string}`;

    try {
      setIsProcessing(true);
      setCurrentBankingHash(bankingHash);
      
      // Check if approval is needed
      if (needsApproval(amount)) {
        console.log('Approval needed, starting approval process');
        setStep('approve');
        await approvePyusd(amount);
      } else {
        // No approval needed, go directly to deposit
        console.log('No approval needed, proceeding to deposit');
        setStep('deposit');
        await depositPyusd(amount, bankingHash);
      }

    } catch (error) {
      console.error('Transaction failed:', error);
      alert('Transaction failed. Please try again.');
      setStep('input');
      setIsProcessing(false);
      setCurrentBankingHash(null);
    }
  };

  const handleCancel = () => {
    router.push('/');
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col min-h-screen p-6 bg-white">
        {/* Header */}
        <div className="mb-8 mt-4">
          <h1 className="text-2xl font-bold text-gray-800 mb-1">
            Sending you: ${parseFloat(amount).toLocaleString()} USD
          </h1>
          <p className="text-gray-600 text-base">Confirm the details</p>
        </div>

        {/* Transaction Progress */}
        {isProcessing && (
          <div className="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <div className="flex items-center space-x-3">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
              <div>
                <p className="text-sm font-medium text-blue-800">
                  {step === 'approve' && 'Approving PYUSD spending...'}
                  {step === 'deposit' && 'Depositing PYUSD to Vapor Bridge...'}
                  {step === 'success' && 'Transaction completed successfully!'}
                </p>
                <p className="text-xs text-blue-600">
                  {step === 'approve' && !approveHash && 'Please confirm the approval transaction in your wallet'}
                  {step === 'approve' && approveHash && !isApproveSuccess && 'Waiting for approval confirmation...'}
                  {step === 'deposit' && !depositHash && 'Please confirm the deposit transaction in your wallet'}
                  {step === 'deposit' && depositHash && !isDepositSuccess && 'Waiting for deposit confirmation...'}
                  {step === 'success' && 'Redirecting to status page...'}
                </p>
                {(approveHash || depositHash) && (
                  <p className="text-xs text-gray-500 mt-1 font-mono">
                    {step === 'approve' && approveHash && `Tx: ${approveHash.slice(0, 10)}...`}
                    {step === 'deposit' && depositHash && `Tx: ${depositHash.slice(0, 10)}...`}
                  </p>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Transaction Details */}
        <div className="flex-1 space-y-6">
          {/* To */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">To</span>
            <span className="text-gray-900 font-mono">{bankAccount}</span>
          </div>

          {/* Service */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">Service</span>
            <span className="text-gray-900">{service}</span>
          </div>

          {/* Fee */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">Fee</span>
            <span className="text-gray-900">{(parseFloat(amount) * 0.005).toFixed(2)} PYUSD (0.5%)</span>
          </div>

          {/* Est Time */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">Est Time</span>
            <span className="text-gray-900">100 mins</span>
          </div>

          {/* Transaction Details Expandable */}
          <div className="mt-8">
            <button
              onClick={() => setShowTransactionDetails(!showTransactionDetails)}
              className="w-full p-4 bg-gray-100 rounded-lg flex justify-between items-center"
            >
              <span className="text-gray-700 font-medium">Transaction Details</span>
              <span className="text-gray-900 font-bold">
                {showTransactionDetails ? 'COLLAPSE' : 'EXPAND'}
              </span>
            </button>
            
            {showTransactionDetails && (
              <div className="mt-4 p-4 bg-gray-50 rounded-lg">
                <div className="font-mono text-sm text-gray-700 whitespace-pre-wrap">
                  {`{
  "bridgeTransaction": {
    "transactionHash": 
"0x7d3c9a93d95a3c3188182c0d3dc5f3
d95a3c3188182c0d3dc5f3d95a3c31881
8",
    "fromChain": {...`}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="pb-6 space-y-3">
          <button
            onClick={handleConfirm}
            disabled={isProcessing}
            className={`w-full py-4 font-semibold text-lg rounded-lg transition-colors ${
              isProcessing
                ? 'bg-gray-400 text-gray-600 cursor-not-allowed'
                : 'bg-[#8BC34A] hover:bg-[#689F38] text-white'
            }`}
          >
            {isProcessing ? (
              <div className="flex items-center justify-center space-x-2">
                <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white"></div>
                <span>
                  {step === 'approve' && 'APPROVING...'}
                  {step === 'deposit' && 'DEPOSITING...'}
                  {step === 'success' && 'SUCCESS!'}
                </span>
              </div>
            ) : (
              'CONFIRM'
            )}
          </button>
          
          <button
            onClick={handleCancel}
            disabled={isProcessing}
            className={`w-full py-4 font-semibold text-lg rounded-lg transition-colors ${
              isProcessing
                ? 'bg-gray-100 text-gray-400 cursor-not-allowed'
                : 'bg-gray-200 hover:bg-gray-300 text-gray-700'
            }`}
          >
            CANCEL
          </button>
        </div>
      </div>
    </div>
  );
}
