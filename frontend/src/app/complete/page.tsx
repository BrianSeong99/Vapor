'use client';

import { useRouter } from 'next/navigation';

export default function CompletePage() {
  const router = useRouter();

  const handleNewTransaction = () => {
    router.push('/');
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col items-center justify-center min-h-screen p-6 bg-white">
        {/* Success Circle */}
        <div className="relative mb-8">
          {/* Outer ring */}
          <div className="w-64 h-64 rounded-full border-4 border-[#8BC34A] flex items-center justify-center">
            {/* Inner circle */}
            <div className="w-48 h-48 rounded-full bg-[#8BC34A] flex items-center justify-center">
              {/* Thank you text */}
              <div className="text-center">
                <span className="text-white text-2xl font-semibold">Thank you</span>
                <span className="text-white text-2xl ml-2">🤙</span>
              </div>
            </div>
          </div>
          
          {/* Subtle inner ring */}
          <div className="absolute inset-8 rounded-full border-2 border-white opacity-30"></div>
        </div>

        {/* Optional: Add some celebration or completion message */}
        <div className="text-center mb-8">
          <p className="text-gray-600 text-lg">
            Your off-ramp transaction has been completed successfully!
          </p>
        </div>

        {/* Optional: New Transaction Button */}
        <button
          onClick={handleNewTransaction}
          className="px-8 py-3 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold rounded-lg transition-colors"
        >
          New Transaction
        </button>
      </div>
    </div>
  );
}
