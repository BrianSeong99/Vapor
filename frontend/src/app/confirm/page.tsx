'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

export default function ConfirmPage() {
  const router = useRouter();
  const [showTransactionDetails, setShowTransactionDetails] = useState(false);

  const handleConfirm = () => {
    // TODO: Integrate wallet signing
    router.push('/status');
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
            Sending you: $100K USD
          </h1>
          <p className="text-gray-600 text-base">Confirm the details</p>
        </div>

        {/* Transaction Details */}
        <div className="flex-1 space-y-6">
          {/* To */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">To</span>
            <span className="text-gray-900 font-mono">841273-1283712</span>
          </div>

          {/* Service */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">Service</span>
            <span className="text-gray-900">Pay Pal Hong Kong</span>
          </div>

          {/* Fee */}
          <div className="flex justify-between items-center py-3 border-b border-gray-100">
            <span className="text-gray-700 font-medium">Fee</span>
            <span className="text-gray-900">500 PYUSD (0.5%)</span>
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
            className="w-full py-4 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold text-lg rounded-lg transition-colors"
          >
            CONFIRM
          </button>
          
          <button
            onClick={handleCancel}
            className="w-full py-4 bg-gray-200 hover:bg-gray-300 text-gray-700 font-semibold text-lg rounded-lg transition-colors"
          >
            CANCEL
          </button>
        </div>
      </div>
    </div>
  );
}
