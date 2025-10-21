import { useState, useEffect } from "react";
import { taskApiClient } from "@/lib/task-api";

export interface ArbiterVerdict {
  taskId: string;
  status: "approved" | "rejected" | "waiver_required";
  confidence: number;
  waiverRequired: boolean;
  waiverReason?: string;
  cawsCompliance: number;
  factualAccuracy: number;
  debateRounds: number;
  evidenceCount: number;
  timestamp: string;
  violations: string[];
  recommendedActions: string[];
  provenanceId: string;
}

export interface ClaimVerificationData {
  taskId: string;
  totalClaims: number;
  verifiedClaims: number;
  disputedClaims: number;
  factualAccuracyScore: number;
  cawsComplianceScore: number;
  verificationTimestamp: string;
  claims: Array<{
    id: string;
    text: string;
    verificationStatus: "verified" | "disputed" | "unknown";
    confidence: number;
    evidence: string[];
  }>;
}

export interface DebateData {
  taskId: string;
  rounds: Array<{
    round: number;
    judge: string;
    verdict: "approve" | "reject" | "waiver";
    confidence: number;
    reasoning: string;
    evidenceUsed: string[];
    timestamp: string;
  }>;
  finalVerdict: ArbiterVerdict;
}

export function useArbiterVerdict(taskId: string) {
  const [verdict, setVerdict] = useState<ArbiterVerdict | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchVerdict = async () => {
      try {
        const data = await taskApiClient.getArbiterVerdict(taskId);
        setVerdict(data);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to fetch verdict"
        );
      } finally {
        setLoading(false);
      }
    };

    fetchVerdict();
  }, [taskId]);

  const requestWaiver = async (reason: string) => {
    try {
      await taskApiClient.requestWaiver(taskId, reason);
      // Refresh verdict after waiver request
      const data = await taskApiClient.getArbiterVerdict(taskId);
      setVerdict(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to request waiver");
    }
  };

  const appealVerdict = async (reason: string) => {
    try {
      await taskApiClient.appealVerdict(taskId, reason);
      // Refresh verdict after appeal
      const data = await taskApiClient.getArbiterVerdict(taskId);
      setVerdict(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to appeal verdict");
    }
  };

  return { verdict, loading, error, requestWaiver, appealVerdict };
}

export function useClaimVerificationData(taskId: string) {
  const [data, setData] = useState<ClaimVerificationData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const verificationData = await taskApiClient.getClaimVerificationData(
          taskId
        );
        setData(verificationData);
      } catch (err) {
        setError(
          err instanceof Error
            ? err.message
            : "Failed to fetch claim verification data"
        );
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [taskId]);

  return { data, loading, error };
}

export function useDebateData(taskId: string) {
  const [data, setData] = useState<DebateData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const debateData = await taskApiClient.getDebateData(taskId);
        setData(debateData);
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to fetch debate data"
        );
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, [taskId]);

  return { data, loading, error };
}
