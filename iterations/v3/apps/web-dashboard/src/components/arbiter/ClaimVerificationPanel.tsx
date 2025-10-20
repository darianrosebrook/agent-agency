import React, { useState, useEffect } from "react";
import { useClaimVerificationData } from "@/hooks/useArbiter";
import { Card, CardHeader, CardTitle, CardContent } from "../shared/Card";
import { Badge } from "../shared/Badge";
import { Button } from "../shared/Button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../shared/Tabs";
import { Progress } from "../shared/Progress";
import { Alert, AlertDescription } from "../shared/Alert";
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  Search,
  FileText,
  Shield,
  TrendingUp,
  Eye,
  EyeOff,
} from "lucide-react";

interface ClaimVerificationData {
  taskId: string;
  totalClaims: number;
  verifiedClaims: number;
  unverifiedClaims: number;
  averageConfidence: number;
  claimsByCategory: Record<string, number>;
  topVerifications: ClaimVerification[];
  failedVerifications: ClaimVerification[];
  evidenceQuality: EvidenceQualityMetrics;
}

interface ClaimVerification {
  claimId: string;
  claimText: string;
  subject: string;
  predicate: string;
  object?: string;
  verificationStatus: "Verified" | "Unverified" | "PartiallyVerified";
  confidence: number;
  evidenceCount: number;
  verificationMethod: string;
  timestamp: string;
  category: string;
}

interface EvidenceQualityMetrics {
  averageRelevance: number;
  averageConfidence: number;
  sourcesUsed: string[];
  freshEvidencePercentage: number;
  crossReferencedPercentage: number;
}

interface ClaimVerificationPanelProps {
  taskId: string;
  onClaimDetails?: (claimId: string) => void;
  onEvidenceReview?: (claimId: string) => void;
}

export function ClaimVerificationPanel({
  taskId,
  onClaimDetails,
  onEvidenceReview,
}: ClaimVerificationPanelProps) {
  const { data, loading, error } = useClaimVerificationData(taskId);
  const [expandedClaims, setExpandedClaims] = useState<Set<string>>(new Set());

  const toggleClaimExpansion = (claimId: string) => {
    const newExpanded = new Set(expandedClaims);
    if (newExpanded.has(claimId)) {
      newExpanded.delete(claimId);
    } else {
      newExpanded.add(claimId);
    }
    setExpandedClaims(newExpanded);
  };

  if (loading) {
    return (
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center justify-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            <span className="ml-2 text-gray-600">
              Loading claim verification data...
            </span>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (error || !data) {
    return (
      <Alert>
        <AlertTriangle className="h-4 w-4" />
        <AlertDescription>
          {error || "No claim verification data available for this task."}
        </AlertDescription>
      </Alert>
    );
  }

  const verificationRate =
    data.totalClaims > 0 ? (data.verifiedClaims / data.totalClaims) * 100 : 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="w-5 h-5" />
          Claim Verification Analysis
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Overview Metrics */}
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
            <div className="text-2xl font-bold text-red-600">
              {data.unverifiedClaims}
            </div>
            <div className="text-sm text-gray-600">Unverified</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-purple-600">
              {verificationRate.toFixed(0)}%
            </div>
            <div className="text-sm text-gray-600">Verification Rate</div>
          </div>
        </div>

        {/* Average Confidence */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Average Confidence</span>
            <span className="text-sm text-gray-600">
              {(data.averageConfidence * 100).toFixed(1)}%
            </span>
          </div>
          <Progress value={data.averageConfidence * 100} className="h-2" />
        </div>

        {/* Evidence Quality Metrics */}
        <div className="bg-gray-50 rounded-lg p-4">
          <h4 className="font-medium mb-3 flex items-center gap-2">
            <FileText className="w-4 h-4" />
            Evidence Quality
          </h4>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <span className="text-sm text-gray-600">Relevance</span>
              <Progress
                value={data.evidenceQuality.averageRelevance * 100}
                className="h-2"
              />
              <span className="text-xs text-gray-500">
                {(data.evidenceQuality.averageRelevance * 100).toFixed(0)}%
              </span>
            </div>
            <div className="space-y-2">
              <span className="text-sm text-gray-600">Freshness</span>
              <Progress
                value={data.evidenceQuality.freshEvidencePercentage}
                className="h-2"
              />
              <span className="text-xs text-gray-500">
                {data.evidenceQuality.freshEvidencePercentage.toFixed(0)}%
              </span>
            </div>
          </div>
          <div className="mt-3">
            <span className="text-sm text-gray-600">Sources Used:</span>
            <div className="flex flex-wrap gap-1 mt-1">
              {data.evidenceQuality.sourcesUsed.map((source, index) => (
                <Badge key={index} variant="outline" className="text-xs">
                  {source}
                </Badge>
              ))}
            </div>
          </div>
        </div>

        {/* Claims by Category */}
        {Object.keys(data.claimsByCategory).length > 0 && (
          <div className="space-y-2">
            <h4 className="font-medium">Claims by Category</h4>
            <div className="flex flex-wrap gap-2">
              {Object.entries(data.claimsByCategory).map(
                ([category, count]) => (
                  <Badge key={category} variant="outline" className="text-xs">
                    {category}: {count}
                  </Badge>
                )
              )}
            </div>
          </div>
        )}

        {/* Detailed Claim Lists */}
        <Tabs defaultValue="verified" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="verified">
              Verified Claims ({data.topVerifications.length})
            </TabsTrigger>
            <TabsTrigger value="failed">
              Failed Verifications ({data.failedVerifications.length})
            </TabsTrigger>
          </TabsList>

          <TabsContent value="verified" className="space-y-4">
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {data.topVerifications.map((claim) => (
                <div
                  key={claim.claimId}
                  className="border rounded-lg p-3 bg-green-50 border-green-200"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <CheckCircle className="w-4 h-4 text-green-600" />
                        <span className="font-medium text-sm text-green-800">
                          Verified
                        </span>
                        <Badge className="text-xs bg-green-100 text-green-800">
                          {(claim.confidence * 100).toFixed(0)}%
                        </Badge>
                      </div>
                      <p className="text-sm text-gray-700">{claim.claimText}</p>
                    </div>
                    <div className="flex gap-1 ml-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => toggleClaimExpansion(claim.claimId)}
                      >
                        {expandedClaims.has(claim.claimId) ? (
                          <EyeOff className="w-3 h-3" />
                        ) : (
                          <Eye className="w-3 h-3" />
                        )}
                      </Button>
                    </div>
                  </div>

                  {expandedClaims.has(claim.claimId) && (
                    <div className="mt-3 space-y-2 text-xs text-gray-600">
                      <div>
                        <strong>Subject:</strong> {claim.subject}
                      </div>
                      <div>
                        <strong>Predicate:</strong> {claim.predicate}
                      </div>
                      {claim.object && (
                        <div>
                          <strong>Object:</strong> {claim.object}
                        </div>
                      )}
                      <div>
                        <strong>Method:</strong> {claim.verificationMethod}
                      </div>
                      <div>
                        <strong>Evidence:</strong> {claim.evidenceCount} sources
                      </div>
                      <div>
                        <strong>Category:</strong> {claim.category}
                      </div>
                    </div>
                  )}

                  <div className="flex gap-2 mt-3">
                    {onClaimDetails && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onClaimDetails(claim.claimId)}
                      >
                        Details
                      </Button>
                    )}
                    {onEvidenceReview && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onEvidenceReview(claim.claimId)}
                      >
                        Evidence
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </TabsContent>

          <TabsContent value="failed" className="space-y-4">
            <div className="space-y-2 max-h-96 overflow-y-auto">
              {data.failedVerifications.map((claim) => (
                <div
                  key={claim.claimId}
                  className="border rounded-lg p-3 bg-red-50 border-red-200"
                >
                  <div className="flex items-start justify-between mb-2">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <XCircle className="w-4 h-4 text-red-600" />
                        <span className="font-medium text-sm text-red-800">
                          {claim.verificationStatus}
                        </span>
                        <Badge className="text-xs bg-red-100 text-red-800">
                          {(claim.confidence * 100).toFixed(0)}%
                        </Badge>
                      </div>
                      <p className="text-sm text-gray-700">{claim.claimText}</p>
                    </div>
                    <div className="flex gap-1 ml-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => toggleClaimExpansion(claim.claimId)}
                      >
                        {expandedClaims.has(claim.claimId) ? (
                          <EyeOff className="w-3 h-3" />
                        ) : (
                          <Eye className="w-3 h-3" />
                        )}
                      </Button>
                    </div>
                  </div>

                  {expandedClaims.has(claim.claimId) && (
                    <div className="mt-3 space-y-2 text-xs text-gray-600">
                      <div>
                        <strong>Subject:</strong> {claim.subject}
                      </div>
                      <div>
                        <strong>Predicate:</strong> {claim.predicate}
                      </div>
                      {claim.object && (
                        <div>
                          <strong>Object:</strong> {claim.object}
                        </div>
                      )}
                      <div>
                        <strong>Method:</strong> {claim.verificationMethod}
                      </div>
                      <div>
                        <strong>Evidence:</strong> {claim.evidenceCount} sources
                      </div>
                      <div>
                        <strong>Category:</strong> {claim.category}
                      </div>
                    </div>
                  )}

                  <div className="flex gap-2 mt-3">
                    {onClaimDetails && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onClaimDetails(claim.claimId)}
                      >
                        Details
                      </Button>
                    )}
                    {onEvidenceReview && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onEvidenceReview(claim.claimId)}
                      >
                        Evidence
                      </Button>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </TabsContent>
        </Tabs>

        {/* Quality Insights */}
        <div className="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <h4 className="font-medium text-blue-900 mb-2 flex items-center gap-2">
            <TrendingUp className="w-4 h-4" />
            Verification Insights
          </h4>
          <ul className="text-sm text-blue-800 space-y-1">
            <li>
              • {data.evidenceQuality.crossReferencedPercentage.toFixed(0)}% of
              claims had cross-referenced evidence
            </li>
            <li>
              • Average evidence relevance:{" "}
              {(data.evidenceQuality.averageRelevance * 100).toFixed(0)}%
            </li>
            <li>
              • {data.evidenceQuality.sourcesUsed.length} different evidence
              sources used
            </li>
          </ul>
        </div>
      </CardContent>
    </Card>
  );
}
