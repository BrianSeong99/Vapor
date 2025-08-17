'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';

type StepStatus = 'completed' | 'active' | 'pending';

interface Step {
  id: number;
  title: string;
  subtitle: string;
  duration: string;
  details: string[];
  status: StepStatus;
}

export default function StatusPage() {
  const router = useRouter();
  const [currentStep, setCurrentStep] = useState(1);
  const [isComplete, setIsComplete] = useState(false);

  const steps: Step[] = [
    {
      id: 1,
      title: 'Privately Listing',
      subtitle: '~5 mins',
      duration: '5 mins',
      details: [
        'Storing your transaction',
        'Privately submitting your job'
      ],
      status: currentStep > 1 ? 'completed' : currentStep === 1 ? 'active' : 'pending'
    },
    {
      id: 2,
      title: 'Finding Fillers',
      subtitle: '~30 mins',
      duration: '30 mins',
      details: [
        'Matching your order for the best price',
        'Verifying fillers have the money'
      ],
      status: currentStep > 2 ? 'completed' : currentStep === 2 ? 'active' : 'pending'
    },
    {
      id: 3,
      title: 'Sending your USD',
      subtitle: '~60 mins',
      duration: '60 mins',
      details: [
        'Sending money to PayPal Hong Kong',
        'Confirming Receipt'
      ],
      status: currentStep > 3 ? 'completed' : currentStep === 3 ? 'active' : 'pending'
    },
    {
      id: 4,
      title: 'View Receipt',
      subtitle: '',
      duration: '',
      details: [],
      status: currentStep > 4 ? 'completed' : currentStep === 4 ? 'active' : 'pending'
    }
  ];

  // Simulate step progression
  useEffect(() => {
    const intervals: NodeJS.Timeout[] = [];

    // Step 1 -> 2 after 3 seconds
    intervals.push(setTimeout(() => setCurrentStep(2), 3000));
    
    // Step 2 -> 3 after 8 seconds
    intervals.push(setTimeout(() => setCurrentStep(3), 8000));
    
    // Step 3 -> 4 after 12 seconds
    intervals.push(setTimeout(() => {
      setCurrentStep(4);
      setIsComplete(true);
    }, 12000));

    return () => intervals.forEach(clearInterval);
  }, []);

  const handleReceived = () => {
    router.push('/complete');
  };

  const getStepIcon = (step: Step) => {
    if (step.status === 'completed') {
      return (
        <div className="w-6 h-6 bg-[#8BC34A] rounded-full flex items-center justify-center">
          <svg width="12" height="9" viewBox="0 0 12 9" fill="none">
            <path d="M1 4.5L4.5 8L11 1.5" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
          </svg>
        </div>
      );
    } else if (step.status === 'active') {
      return (
        <div className="w-6 h-6 bg-[#8BC34A] rounded-full flex items-center justify-center">
          <div className="w-2 h-2 bg-white rounded-full"></div>
        </div>
      );
    } else {
      return <div className="w-6 h-6 bg-gray-300 rounded-full"></div>;
    }
  };

  const getConnectorColor = (index: number) => {
    return currentStep > index + 1 ? '#8BC34A' : '#D1D5DB';
  };

  return (
    <div className="mobile-container">
      <div className="flex flex-col min-h-screen p-6 bg-white">
        {/* Header */}
        <div className="mb-8 mt-4">
          <h1 className="text-2xl font-bold text-gray-800 mb-1">
            Off Ramp {isComplete ? 'Complete' : 'Starting'}
          </h1>
          <p className="text-gray-600 text-base">Bringing it all together</p>
        </div>

        {/* Progress Steps */}
        <div className="flex-1">
          <div className="space-y-0">
            {steps.map((step, index) => (
              <div key={step.id} className="relative">
                {/* Step Content */}
                <div className="flex items-start space-x-4 pb-6">
                  {/* Icon */}
                  <div className="flex-shrink-0 mt-1">
                    {getStepIcon(step)}
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center space-x-2">
                      <h3 className={`font-semibold ${
                        step.status === 'completed' 
                          ? 'text-gray-400 line-through' 
                          : step.status === 'active'
                          ? 'text-gray-900'
                          : 'text-gray-400'
                      }`}>
                        {step.id}. {step.title}
                      </h3>
                      {step.subtitle && (
                        <span className={`text-sm ${
                          step.status === 'completed' 
                            ? 'text-gray-400 line-through' 
                            : step.status === 'active'
                            ? 'text-gray-600'
                            : 'text-gray-400'
                        }`}>
                          {step.subtitle}
                        </span>
                      )}
                    </div>

                    {/* Step Details */}
                    {step.details.length > 0 && (
                      <ul className={`mt-2 space-y-1 text-sm ${
                        step.status === 'completed' 
                          ? 'text-gray-400' 
                          : step.status === 'active'
                          ? 'text-gray-600'
                          : 'text-gray-400'
                      }`}>
                        {step.details.map((detail, detailIndex) => (
                          <li key={detailIndex} className="flex items-center space-x-2">
                            <span>•</span>
                            <span className={step.status === 'completed' ? 'line-through' : ''}>
                              {detail}
                            </span>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>
                </div>

                {/* Connector Line */}
                {index < steps.length - 1 && (
                  <div 
                    className="absolute left-3 top-8 w-0.5 h-6 -translate-x-0.5"
                    style={{ backgroundColor: getConnectorColor(index) }}
                  ></div>
                )}
              </div>
            ))}
          </div>
        </div>

        {/* Action Button */}
        <div className="pb-6">
          {isComplete ? (
            <button
              onClick={handleReceived}
              className="w-full py-4 bg-[#8BC34A] hover:bg-[#689F38] text-white font-semibold text-lg rounded-lg transition-colors"
            >
              CONFIRM RECEIPT
            </button>
          ) : (
            <button
              disabled
              className="w-full py-4 bg-gray-200 text-gray-500 font-semibold text-lg rounded-lg cursor-not-allowed"
            >
              I RECEIVED MY USD
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
