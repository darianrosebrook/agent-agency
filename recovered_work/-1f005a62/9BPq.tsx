import React from "react";
import { useArbiterVerdict } from "@/hooks/useArbiter";
import {
  Clock,
  CheckCircle,
  XCircle,
  AlertTriangle,
  Shield,
  FileText,
  Users,
  Scale,
} from "lucide-react";

interface ArbiterVerdict {
  taskId: string;
  workingSpecId: string;
  status:
    | "Approved"
    | "Rejected"
    | "WaiverRequired"
    | "NeedsClarification"
    | "Error";
  confidence: number;
  evidenceManifest: EvidenceManifest;
  waiverRequired: boolean;
  waiverReason?: string;
  debateRounds: number;
  provenanceId: string;
  timestamp: string;
  violations: string[];
  recommendedActions: string[];
}

interface EvidenceManifest {
  claims: AtomicClaim[];
  verificationResults: VerificationResult[];
  factualAccuracyScore: number;
  cawsComplianceScore: number;
}

interface AtomicClaim {
  id: string;
  claimText: string;
  subject: string;
  predicate: string;
  object?: string;
  confidence: number;
  verificationRequirements: VerificationRequirement[];
}

interface VerificationResult {
  claimId: string;
  verificationStatus: "Verified" | "Unverified";
  confidence: number;
  evidence: Evidence[];
  timestamp: string;
}

interface Evidence {
  source: string;
  content: string;
  relevance: number;
  confidence: number;
  timestamp: string;
}

interface VerificationRequirement {
  method: string;
  evidenceType: string;
  minimumConfidence: number;
  requiredSources: string[];
}

interface ArbiterVerdictPanelProps {
  taskId: string;
  onWaiverRequest?: (reason: string) => void;
  onAppealVerdict?: () => void;
}

const getStatusColor = (status: string) => {
  switch (status) {
    case "Approved":
      return "bg-green-100 text-green-800 border-green-200";
    case "Rejected":
      return "bg-red-100 text-red-800 border-red-200";
    case "WaiverRequired":
      return "bg-yellow-100 text-yellow-800 border-yellow-200";
    case "NeedsClarification":
      return "bg-blue-100 text-blue-800 border-blue-200";
    case "Error":
      return "bg-gray-100 text-gray-800 border-gray-200";
    default:
      return "bg-gray-100 text-gray-800 border-gray-200";
  }
};

const getStatusIcon = (status: string) => {
  switch (status) {
    case "Approved":
      return <CheckCircle className="w-4 h-4" />;
    case "Rejected":
      return <XCircle className="w-4 h-4" />;
    case "WaiverRequired":
      return <AlertTriangle className="w-4 h-4" />;
    case "NeedsClarification":
      return <Clock className="w-4 h-4" />;
    case "Error":
      return <XCircle className="w-4 h-4" />;
    default:
      return <Clock className="w-4 h-4" />;
  }
};

export function ArbiterVerdictPanel({
  taskId,
  onWaiverRequest,
  onAppealVerdict,
}: ArbiterVerdictPanelProps) {
  const { verdict, loading, error, requestWaiver, appealVerdict } =
    useArbiterVerdict(taskId);

  if (loading) {
    return (
      <div className="bg-white rounded-lg shadow p-6">
        <div className="flex items-center justify-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          <span className="ml-2 text-gray-600">
            Loading arbiter verdict...
          </span>
        </div>
      </div>
    );
  }

  if (error || !verdict) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center">
          <AlertTriangle className="h-4 w-4 text-red-600" />
          <p className="ml-2 text-red-800">
            {error || "No arbiter verdict available for this task."}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow">
      <div className="px-6 py-4 border-b border-gray-200">
        <h3 className="text-lg font-medium flex items-center gap-2">
          <Scale className="w-5 h-5" />
          Arbiter Verdict
          <span className={`ml-auto px-2 py-1 rounded-full text-xs font-medium ${getStatusColor(verdict.status)}`}>
            {getStatusIcon(verdict.status)}
            <span className="ml-1">{verdict.status}</span>
          </span>
        </h3>
      </div>
      <div className="px-6 py-4 space-y-6">
        {/* Verdict Summary */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Shield className="w-4 h-4 text-gray-500" />
              <span className="text-sm font-medium">Confidence</span>
            </div>
            <Progress value={verdict.confidence * 100} className="h-2" />
            <span className="text-xs text-gray-600">
              {(verdict.confidence * 100).toFixed(1)}%
            </span>
          </div>
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <Users className="w-4 h-4 text-gray-500" />
              <span className="text-sm font-medium">Debate Rounds</span>
            </div>
            <span className="text-lg font-bold">{verdict.debateRounds}</span>
          </div>
        </div>

        {/* Evidence Scores */}
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <span className="text-sm font-medium">Factual Accuracy</span>
            <Progress
              value={verdict.evidenceManifest.factualAccuracyScore * 100}
              className="h-2"
            />
            <span className="text-xs text-gray-600">
              {(verdict.evidenceManifest.factualAccuracyScore * 100).toFixed(1)}
              %
            </span>
          </div>
          <div className="space-y-2">
            <span className="text-sm font-medium">CAWS Compliance</span>
            <Progress
              value={verdict.evidenceManifest.cawsComplianceScore * 100}
              className="h-2"
            />
            <span className="text-xs text-gray-600">
              {(verdict.evidenceManifest.cawsComplianceScore * 100).toFixed(1)}%
            </span>
          </div>
        </div>

        {/* Detailed Violations */}
        <div className="space-y-4">
            {verdict.violations.length > 0 ? (
              <div className="space-y-2">
                {verdict.violations.map((violation, index) => (
                  <div key={index} className="bg-yellow-50 border border-yellow-200 rounded-lg p-3">
                    <div className="flex items-center">
                      <AlertTriangle className="h-4 w-4 text-yellow-600" />
                      <p className="ml-2 text-yellow-800">{violation}</p>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="text-center text-gray-500 py-4">
                No violations found
              </div>
            )}

            {verdict.recommendedActions.length > 0 && (
              <div className="space-y-2">
                <h4 className="font-medium">Recommended Actions</h4>
                <ul className="list-disc list-inside space-y-1">
                  {verdict.recommendedActions.map((action, index) => (
                    <li key={index} className="text-sm">
                      {action}
                    </li>
                  ))}
                </ul>
              </div>
            )}
        </div>
                <div key={claim.id} className="border rounded-lg p-3">
                  <div className="flex items-center justify-between mb-2">
                    <span className="font-medium text-sm">
                      {claim.claimText}
                    </span>
                    <Badge variant="outline" className="text-xs">
                      {(claim.confidence * 100).toFixed(0)}%
                    </Badge>
                  </div>
                  <div className="text-xs text-gray-600">
                    Subject: {claim.subject} | Predicate: {claim.predicate}
                    {claim.object && ` | Object: ${claim.object}`}
                  </div>
                </div>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="evidence" className="space-y-4">
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {verdict.evidenceManifest.verificationResults.map((result) => (
                <div key={result.claimId} className="border rounded-lg p-3">
                  <div className="flex items-center justify-between mb-2">
                    <Badge
                      className={
                        result.verificationStatus === "Verified"
                          ? "bg-green-100 text-green-800"
                          : "bg-red-100 text-red-800"
                      }
                    >
                      {result.verificationStatus}
                    </Badge>
                    <span className="text-xs text-gray-600">
                      {(result.confidence * 100).toFixed(0)}% confidence
                    </span>
                  </div>
                  <div className="text-xs text-gray-600">
                    {result.evidence.length} evidence items
                  </div>
                </div>
              ))}
            </div>
          </TabsContent>
        </Tabs>

        {/* Action Buttons */}
        <div className="flex gap-2">
          {verdict.waiverRequired && (
            <button
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              onClick={() => {
                requestWaiver(
                  verdict.waiverReason || "Budget violation requires waiver"
                );
                onWaiverRequest?.(
                  verdict.waiverReason || "Budget violation requires waiver"
                );
              }}
            >
              Request Waiver
            </button>
          )}
          {verdict.status === "Rejected" && (
            <button
              className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
              onClick={() => {
                appealVerdict("Appealing arbiter verdict for review");
                onAppealVerdict?.("Appealing arbiter verdict for review");
              }}
            >
              Appeal Verdict
            </button>
          )}
        </div>

        {/* Provenance */}
        <div className="text-xs text-gray-500 border-t pt-2">
          <div className="flex items-center gap-1">
            <FileText className="w-3 h-3" />
            <span>Provenance ID: {verdict.provenanceId}</span>
          </div>
          <div className="flex items-center gap-1 mt-1">
            <Clock className="w-3 h-3" />
            <span>
              Adjudicated: {new Date(verdict.timestamp).toLocaleString()}
            </span>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
