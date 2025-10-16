/**
 * @fileoverview High-level pipeline tests for the ClaimExtractor implementation.
 *
 * These tests cover the four-stage claim extraction pipeline:
 * 1. Contextual disambiguation with conversation-aware resolution
 * 2. Verifiable content qualification
 * 3. Atomic claim decomposition with verification requirements
 * 4. CAWS-compliant verification integration
 */

import { createClaimExtractor } from "../../../src/verification/ClaimExtractor";
import type {
  ConversationContext,
  EvidenceManifest,
} from "../../../src/verification/types";

describe("ClaimExtractor - end-to-end pipeline", () => {
  const extractor = createClaimExtractor();

  const buildContext = (
    previousMessages: string[]
  ): ConversationContext => ({
    conversationId: "conv-001",
    tenantId: "tenant-123",
    previousMessages,
    metadata: {},
  });

  const emptyEvidence: EvidenceManifest = {
    sources: [],
    evidence: [],
    quality: 0,
    cawsCompliant: false,
  };

  it("resolves referential ambiguities and emits atomic claims with verification metadata", async () => {
    const workerOutput =
      "He announced the infrastructure policy on January 5, 2024 and promised $5 million in funding.";
    const taskContext = {
      conversationContext: buildContext([
        "President John Smith addressed the nation about infrastructure investments.",
      ]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    expect(evaluation.extractedClaims.length).toBeGreaterThanOrEqual(2);
    expect(
      evaluation.extractedClaims.every((claim) =>
        /^John Smith/i.test(claim.statement)
      )
    ).toBe(true);
    expect(
      evaluation.extractedClaims.every(
        (claim) => claim.verificationRequirements.length > 0
      )
    ).toBe(true);
    expect(
      evaluation.extractedClaims.every((claim) =>
        claim.sourceContext.includes("John Smith")
      )
    ).toBe(true);
  });

  it("skips extraction when ambiguities cannot be resolved", async () => {
    const workerOutput =
      "He announced the infrastructure policy on January 5, 2024 and promised $5 million in funding.";
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    expect(evaluation.extractedClaims).toHaveLength(0);
    expect(evaluation.verificationResults).toHaveLength(0);
  });

  it("scores verification based on available evidence", async () => {
    const workerOutput =
      "He announced the infrastructure policy on January 5, 2024 and promised $5 million in funding.";
    const evidence: EvidenceManifest = {
      sources: [
        {
          name: "official_transcript",
          type: "official_government",
          reliability: 0.95,
          responseTime: 120,
        },
      ],
      evidence: [
        {
          content:
            "Official transcript: John Smith announced the infrastructure policy on January 5, 2024.",
          source: "official_transcript",
          strength: 0.9,
          timestamp: "2024-01-05T18:00:00Z",
          metadata: {},
        },
      ],
      quality: 0.9,
      cawsCompliant: true,
    };

    const taskContext = {
      conversationContext: buildContext([
        "President John Smith addressed the nation about infrastructure investments.",
      ]),
      evidenceManifest: evidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    expect(evaluation.extractedClaims.length).toBeGreaterThan(0);
    expect(evaluation.verificationResults.length).toBeGreaterThan(0);
    expect(
      evaluation.verificationResults.some(
        (result) => result.status === "VERIFIED"
      )
    ).toBe(true);
    expect(evaluation.cawsComplianceScore).toBeGreaterThan(0);
  });
});
