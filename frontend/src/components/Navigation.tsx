'use client';

import { useRouter } from 'next/navigation';

export default function Navigation() {
  const router = useRouter();

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
        <div>
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
      </div>
    </div>
  );
}
