'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import PaymentModal from '../../components/filler/PaymentModal';
import ProofUploadModal from '../../components/filler/ProofUploadModal';

interface Order {
  id: string;
  amount: string;
  currency: string;
  destination: string;
  service: string;
  fee: string;
  status: 'available' | 'locked' | 'payment_sent' | 'completed';
}

export default function FillerPage() {
  const router = useRouter();
  const [selectedOrder, setSelectedOrder] = useState<Order | null>(null);
  const [showPaymentModal, setShowPaymentModal] = useState(false);
  const [showProofModal, setShowProofModal] = useState(false);

  // Mock orders data
  const [orders, setOrders] = useState<Order[]>([
    {
      id: '1',
      amount: '$100K',
      currency: 'USD',
      destination: 'PayPal Hong Kong',
      service: 'PayPal Hong Kong',
      fee: '$500',
      status: 'available'
    },
    {
      id: '2', 
      amount: '$300K',
      currency: 'USD',
      destination: 'ACH Hong Kong',
      service: 'ACH Hong Kong',
      fee: '$2,000',
      status: 'available'
    },
    {
      id: '3',
      amount: '$60K',
      currency: 'USD', 
      destination: 'PayNow Singapore',
      service: 'PayNow Singapore',
      fee: '$100',
      status: 'available'
    },
    {
      id: '4',
      amount: '$600K',
      currency: 'HKD',
      destination: 'ACH Hong Kong',
      service: 'ACH Hong Kong',
      fee: '$3K',
      status: 'completed'
    }
  ]);

  const handleFillOrder = (order: Order) => {
    setSelectedOrder(order);
    
    if (order.status === 'available') {
      setShowPaymentModal(true);
    } else if (order.status === 'locked') {
      setShowPaymentModal(true);
    } else if (order.status === 'payment_sent') {
      setShowProofModal(true);
    }
  };

  const handleSendPayment = () => {
    if (selectedOrder) {
      // Update order status to locked (orange state)
      setOrders(prev => prev.map(order => 
        order.id === selectedOrder.id 
          ? { ...order, status: 'locked' }
          : order
      ));
      setShowPaymentModal(false);
      setSelectedOrder(null);
    }
  };

  const handlePaymentSent = () => {
    if (selectedOrder) {
      // Update order status to payment_sent
      setOrders(prev => prev.map(order => 
        order.id === selectedOrder.id 
          ? { ...order, status: 'payment_sent' }
          : order
      ));
      setShowPaymentModal(false);
      setShowProofModal(true);
    }
  };

  const handleUploadProof = () => {
    if (selectedOrder) {
      // Update order status to completed
      setOrders(prev => prev.map(order => 
        order.id === selectedOrder.id 
          ? { ...order, status: 'completed' }
          : order
      ));
      setShowProofModal(false);
      setSelectedOrder(null);
    }
  };

  const getOrderButtonText = (order: Order) => {
    switch (order.status) {
      case 'available':
        return `Fill for ${order.fee} USD`;
      case 'locked':
        return 'Pending to Fill';
      case 'payment_sent':
        return 'Upload Proof';
      case 'completed':
        return `Filled for ${order.fee} USD`;
      default:
        return 'Fill Order';
    }
  };

  const getOrderButtonColor = (order: Order) => {
    switch (order.status) {
      case 'available':
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
      case 'locked':
        return 'bg-orange-400 hover:bg-orange-500';
      case 'payment_sent':
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
      case 'completed':
        return 'bg-gray-400';
      default:
        return 'bg-[#4FC3F7] hover:bg-[#29B6F6]';
    }
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

        {/* Orders List */}
        <div className="flex-1 p-6 space-y-4">
          {orders.map((order) => (
            <div key={order.id} className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm">
              {/* Order Header */}
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center space-x-2">
                  <span className="text-lg font-semibold text-gray-800">
                    {order.amount} {order.currency}
                  </span>
                  <svg width="20" height="16" viewBox="0 0 20 16" fill="none">
                    <path d="M2 8h16m-8-6l6 6-6 6" stroke="#E91E63" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
                  </svg>
                  <span className="text-gray-600 font-medium">
                    {order.destination}
                  </span>
                </div>
              </div>

              {/* Fill Button */}
              <button
                onClick={() => handleFillOrder(order)}
                disabled={order.status === 'completed'}
                className={`w-full py-3 text-white font-semibold rounded-lg transition-colors ${getOrderButtonColor(order)} ${
                  order.status === 'completed' ? 'cursor-not-allowed' : 'cursor-pointer'
                }`}
              >
                {getOrderButtonText(order)}
              </button>
            </div>
          ))}
        </div>

        {/* Payment Modal */}
        <PaymentModal
          isOpen={showPaymentModal}
          onClose={() => setShowPaymentModal(false)}
          onSendPayment={handleSendPayment}
          onPaymentSent={handlePaymentSent}
          orderStatus={selectedOrder?.status || 'available'}
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
