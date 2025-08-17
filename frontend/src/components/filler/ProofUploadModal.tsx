'use client';

interface ProofUploadModalProps {
  isOpen: boolean;
  onClose: () => void;
  onUpload: () => void;
}

export default function ProofUploadModal({ 
  isOpen, 
  onClose, 
  onUpload 
}: ProofUploadModalProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
      <div className="bg-white rounded-lg p-6 w-full max-w-sm">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-lg font-semibold text-gray-800">
            Upload Confirmation
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
        
        {/* Drag & Drop Area */}
        <div className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center mb-6 hover:border-[#4FC3F7] transition-colors cursor-pointer">
          <div className="text-gray-500 mb-2">
            <svg className="mx-auto w-8 h-8 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
          </div>
          <p className="text-gray-600 font-medium">DRAG & DROP</p>
          <p className="text-gray-400 text-sm mt-1">or click to browse</p>
        </div>

        <button
          onClick={onUpload}
          className="w-full py-3 bg-[#4FC3F7] hover:bg-[#29B6F6] text-white font-semibold rounded-lg transition-colors"
        >
          UPLOAD
        </button>
      </div>
    </div>
  );
}
