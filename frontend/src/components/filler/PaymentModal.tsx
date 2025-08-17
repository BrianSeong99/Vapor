'use client';

interface PaymentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSendPayment: () => void;
  onPaymentSent: () => void;
  orderStatus: 'Discovery' | 'Locked' | 'MarkPaid' | 'Settled' | 'available' | 'locked' | 'payment_sent' | 'completed';
}

export default function PaymentModal({ 
  isOpen, 
  onClose, 
  onSendPayment, 
  onPaymentSent, 
  orderStatus 
}: PaymentModalProps) {
  if (!isOpen) return null;

  const isPaymentSentStage = orderStatus === 'locked' || orderStatus === 'Locked';

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-sm">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-800">
            Receiver Details:
          </h3>
          <button 
            onClick={onClose}
            className="w-6 h-6 rounded-full bg-gray-200 flex items-center justify-center"
          >
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M9 3L3 9M3 3l6 6" stroke="#666" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
          </button>
        </div>
        
        <div className="space-y-3 mb-6">
          <div className="flex justify-between">
            <span className="text-gray-600">To</span>
            <span className="text-gray-900 font-mono">841273-1283712</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Service</span>
            <span className="text-gray-900">Pay Pal Hong Kong</span>
          </div>
          <div className="flex justify-between">
            <span className="text-gray-600">Fee</span>
            <span className="text-gray-900">500 PYUSD (0.5%)</span>
          </div>
        </div>

        <button
          onClick={isPaymentSentStage ? onPaymentSent : onSendPayment}
          className="w-full py-3 bg-[#4FC3F7] hover:bg-[#29B6F6] text-white font-semibold rounded-lg transition-colors"
        >
          {isPaymentSentStage ? 'PAYMENT SENT' : 'SEND PAYMENT'}
        </button>
      </div>
    </div>
  );
}
