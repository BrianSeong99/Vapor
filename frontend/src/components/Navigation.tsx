'use client';

import { useRouter } from 'next/navigation';

export default function Navigation() {
  const router = useRouter();

  return (
    <div className="fixed top-4 right-4 z-50">
      <div className="bg-black bg-opacity-50 rounded-lg p-2 space-x-2 text-xs">
        <button 
          onClick={() => router.push('/')}
          className="px-2 py-1 bg-white rounded text-black hover:bg-gray-200"
        >
          Input
        </button>
        <button 
          onClick={() => router.push('/confirm')}
          className="px-2 py-1 bg-white rounded text-black hover:bg-gray-200"
        >
          Confirm
        </button>
        <button 
          onClick={() => router.push('/status')}
          className="px-2 py-1 bg-white rounded text-black hover:bg-gray-200"
        >
          Status
        </button>
        <button 
          onClick={() => router.push('/complete')}
          className="px-2 py-1 bg-white rounded text-black hover:bg-gray-200"
        >
          Complete
        </button>
      </div>
    </div>
  );
}
