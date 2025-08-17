'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import PaymentModal from '../../components/filler/PaymentModal';
import ProofUploadModal from '../../components/filler/ProofUploadModal';

interface Order {
  id: string;
  order_type: string;
  status: string;
  amount: string;
  bank_account?: string;
  bank_service?: string;
  filler_id?: string;
  locked_amount?: string;
  created_at: string;
}

export default function FillerPage() {
  const router = useRouter();
  const [selectedOrder, setSelectedOrder] = useState<Order | null>(null);
  const [showPaymentModal, setShowPaymentModal] = useState(false);
  const [showProofModal, setShowProofModal] = useState(false);
  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Mock filler ID - in a real app, this would come from authentication
  const fillerId = 'filler-123';

  // Fetch discovery orders from backend
  const fetchDiscoveryOrders = async () => {
    try {
      setLoading(true);
      const response = await fetch('http://localhost:3000/api/v1/fillers/discovery?limit=20');
      if (!response.ok) {
        throw new Error(`Failed to fetch orders: ${response.statusText}`);
      }
      
      const data = await response.json();
      setOrders(data.orders);
      setError(null);
    } catch (err) {
      console.error('Error fetching discovery orders:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch orders');
    } finally {
      setLoading(false);
    }
  };

  // Lock an order
  const lockOrder = async (orderId: string, amount: string) => {
    try {
      const response = await fetch(`http://localhost:3000/api/v1/fillers/orders/${orderId}/lock`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          filler_id: fillerId,
          amount: amount
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to lock order: ${response.statusText}`);
      }

      const updatedOrder = await response.json();
      
      // Update local state
      setOrders(prev => prev.map(order => 
        order.id === orderId ? updatedOrder : order
      ));
      
      return updatedOrder;
    } catch (err) {
      console.error('Error locking order:', err);
      throw err;
    }
  };

  // Submit payment proof
  const submitPaymentProof = async (orderId: string, bankingHash: string) => {
    try {
      const response = await fetch(`http://localhost:3000/api/v1/fillers/orders/${orderId}/payment-proof`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          banking_hash: bankingHash,
          filler_id: fillerId
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to submit payment proof: ${response.statusText}`);
      }

      const updatedOrder = await response.json();
      
      // Update local state
      setOrders(prev => prev.map(order => 
        order.id === orderId ? updatedOrder : order
      ));
      
      return updatedOrder;
    } catch (err) {
      console.error('Error submitting payment proof:', err);
      throw err;
    }
  };

  // Fetch orders on component mount and set up polling
  useEffect(() => {
    fetchDiscoveryOrders();
    
    // Poll for updates every 10 seconds
    const interval = setInterval(fetchDiscoveryOrders, 10000);
    
    return () => clearInterval(interval);
  }, []);

  const handleFillOrder = (order: Order) => {
    setSelectedOrder(order);
    
    if (order.status === 'Discovery') {
      setShowPaymentModal(true);
    } else if (order.status === 'Locked') {
      setShowPaymentModal(true);
    } else if (order.status === 'MarkPaid') {
      setShowProofModal(true);
    }
  };

  const handleSendPayment = async () => {
    if (selectedOrder) {
      try {
        await lockOrder(selectedOrder.id, selectedOrder.amount);
        setShowPaymentModal(false);
        setSelectedOrder(null);
      } catch (error) {
        alert('Failed to lock order. Please try again.');
      }
    }
  };

  const handlePaymentSent = () => {
    if (selectedOrder) {
      setShowPaymentModal(false);
      setShowProofModal(true);
    }
  };

  const handleUploadProof = async () => {
    if (selectedOrder) {
      try {
        // Generate a mock banking hash for the proof
        const bankingHash = `0x${Math.random().toString(36).substring(2, 15)}${Math.random().toString(36).substring(2, 15)}`.padEnd(66, '0');
        
        await submitPaymentProof(selectedOrder.id, bankingHash);
        setShowProofModal(false);
        setSelectedOrder(null);
      } catch (error) {
        alert('Failed to submit payment proof. Please try again.');
      }
    }
  };

  const calculateFee = (amount: string) => {
    const numAmount = parseFloat(amount);
    return (numAmount * 0.005).toFixed(2); // 0.5% fee
  };

  const getOrderButtonText = (order: Order) => {
    switch (order.status) {
      case 'Discovery':
        return `Fill for $${calculateFee(order.amount)} USD`;
      case 'Locked':
        return 'Pending Payment';
      case 'MarkPaid':
        return 'Upload Proof';
      case 'Settled':
        return `Filled for $${calculateFee(order.amount)} USD`;
      default:
        return 'Fill Order';
    }
  };

  const getOrderButtonColor = (order: Order) => {
    switch (order.status) {
      case 'Discovery':
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
      case 'Locked':
        return 'bg-orange-400 hover:bg-orange-500';
      case 'MarkPaid':
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
      case 'Settled':
        return 'bg-gray-400';
      default:
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
    }
  };

  const isOrderDisabled = (order: Order) => {
    return order.status === 'Settled';
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col min-h-screen bg-white">
        {/* Header */}
        <div className="flex justify-between items-center p-6 border-b border-gray-200">
          <h1 className="text-2xl font-bold text-gray-800">Vapor Filler Orders</h1>
          <button 
            onClick={() => router.push('/filler/balance')}
            className="w-8 h-8 rounded-full bg-gray-200 flex items-center justify-center"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 8a3 3 0 100-6 3 3 0 000 6zM8 10c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" fill="#666"/>
            </svg>
          </button>
        </div>

        {/* Loading State */}
        {loading && (
          <div className="flex-1 p-6">
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <span className="ml-3 text-gray-600">Loading orders...</span>
            </div>
          </div>
        )}

        {/* Error State */}
        {error && (
          <div className="flex-1 p-6">
            <div className="bg-red-50 border border-red-200 rounded-lg p-4">
              <p className="text-red-800">{error}</p>
              <button 
                onClick={fetchDiscoveryOrders}
                className="mt-2 text-red-600 hover:text-red-800 underline"
              >
                Retry
              </button>
            </div>
          </div>
        )}

        {/* Orders List */}
        {!loading && !error && (
          <div className="flex-1 p-6 space-y-4">
            {orders.length === 0 ? (
              <div className="text-center py-8">
                <p className="text-gray-600">No orders available for filling</p>
                <button 
                  onClick={fetchDiscoveryOrders}
                  className="mt-2 text-blue-600 hover:text-blue-800 underline"
                >
                  Refresh
                </button>
              </div>
            ) : (
              orders.map((order) => (
                <div key={order.id} className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
                  {/* Order Header */}
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center space-x-2">
                      <span className="text-lg font-semibold text-gray-800">
                        ${parseFloat(order.amount).toLocaleString()} PYUSD
                      </span>
                      <svg width="20" height="16" viewBox="0 0 20 16" fill="none">
                        <path d="M2 8h16m-8-6l6 6-6 6" stroke="#E91E63" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                      </svg>
                      <span className="text-gray-600 font-medium">
                        {order.bank_service || 'Bank Transfer'}
                      </span>
                    </div>
                  </div>

                  {/* Order Details */}
                  <div className="mb-3 text-sm text-gray-600">
                    <div className="flex justify-between">
                      <span>Status:</span>
                      <span className="font-medium">{order.status}</span>
                    </div>
                    {order.bank_account && (
                      <div className="flex justify-between">
                        <span>Account:</span>
                        <span className="font-mono">{order.bank_account}</span>
                      </div>
                    )}
                    {order.filler_id && (
                      <div className="flex justify-between">
                        <span>Locked by:</span>
                        <span className="font-mono text-xs">{order.filler_id}</span>
                      </div>
                    )}
                  </div>

                  {/* Fill Button */}
                  <button
                    onClick={() => handleFillOrder(order)}
                    disabled={isOrderDisabled(order)}
                    className={`w-full py-3 text-white font-semibold rounded-lg transition-colors ${getOrderButtonColor(order)} ${
                      isOrderDisabled(order) ? 'cursor-not-allowed' : 'cursor-pointer'
                    }`}
                  >
                    {getOrderButtonText(order)}
                  </button>
                </div>
              ))
            )}
          </div>
        )}

        {/* Payment Modal */}
        <PaymentModal
          isOpen={showPaymentModal}
          onClose={() => setShowPaymentModal(false)}
          onSendPayment={handleSendPayment}
          onPaymentSent={handlePaymentSent}
          orderStatus={(selectedOrder?.status as any) || 'Discovery'}
        />

        {/* Proof Upload Modal */}
        <ProofUploadModal
          isOpen={showProofModal}
          onClose={() => setShowProofModal(false)}
          onUpload={handleUploadProof}
        />
      </div>
    </div>
  );
}
