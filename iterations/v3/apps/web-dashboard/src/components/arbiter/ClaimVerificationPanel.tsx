import React from "react";
import { useClaimVerificationData } from "@/hooks/useArbiter";
import { Shield } from "lucide-react";

interface ClaimVerificationPanelProps {
  taskId: string;
  onClaimDetails?: (claimId: string) => void;
  onEvidenceReview?: (claimId: string) => void;
}

export function ClaimVerificationPanel({
  taskId,
  onClaimDetails: _onClaimDetails,
  onEvidenceReview: _onEvidenceReview,
}: ClaimVerificationPanelProps) {
  const { data, loading, error } = useClaimVerificationData(taskId);

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <span className="ml-2 text-gray-600">
            Loading claim verification data...
          </span>
        </div>
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center">
          <div className="h-4 w-4 text-red-600" />
          <p className="ml-2 text-red-800">
            {error || "No claim verification data available for this task."}
          </p>
        </div>
      </div>
    );
  }

  const verificationRate =
    data.totalClaims > 0 ? (data.verifiedClaims / data.totalClaims) * 100 : 0;

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="px-6 py-4 border-b border-gray-200">
        <h3 className="text-lg font-medium flex items-center gap-2">
          <Shield className="w-5 h-5" />
          Claim Verification
        </h3>
      </div>
      <div className="px-6 py-4 space-y-6">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div className="text-center">
            <div className="text-2xl font-bold text-blue-600">
              {data.totalClaims}
            </div>
            <div className="text-sm text-gray-600">Total Claims</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600">
              {data.verifiedClaims}
            </div>
            <div className="text-sm text-gray-600">Verified</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-yellow-600">
              {data.disputedClaims}
            </div>
            <div className="text-sm text-gray-600">Disputed</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600">
              {verificationRate.toFixed(0)}%
            </div>
            <div className="text-sm text-gray-600">Verification Rate</div>
          </div>
        </div>
      </div>
    </div>
  );
}
