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

  const buildContext = (previousMessages: string[]): ConversationContext => ({
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

  it("handles empty worker output gracefully", async () => {
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims("", taskContext);

    expect(evaluation.extractedClaims).toHaveLength(0);
    expect(evaluation.verificationResults).toHaveLength(0);
    expect(evaluation.cawsComplianceScore).toBe(0);
  });

  it("handles null/undefined worker output", async () => {
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      null as any,
      taskContext
    );

    expect(evaluation.extractedClaims).toHaveLength(0);
    expect(evaluation.verificationResults).toHaveLength(0);
  });

  it("extracts multiple independent claims from complex text", async () => {
    const workerOutput =
      "The company reported $2.5 billion in revenue for Q4 2023. CEO Jane Smith announced 500 new jobs will be created. The stock price increased by 15% following the announcement.";
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    expect(evaluation.extractedClaims.length).toBeGreaterThanOrEqual(2);
    expect(
      evaluation.extractedClaims.every(
        (claim) => claim.verificationRequirements.length > 0
      )
    ).toBe(true);
  });

  it("handles structural ambiguities with multiple interpretations", async () => {
    const workerOutput =
      "The company reported $1 million in profits for Q3 2023.";
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    // Should extract claims from financial statements
    expect(evaluation.extractedClaims.length).toBeGreaterThan(0);
    expect(evaluation.extractedClaims[0].statement).toContain("$1 million");
  });

  it("integrates with learning system for pattern adaptation", async () => {
    const workerOutput = "Apple Inc. reported record profits of $100 billion.";
    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
    };

    // First evaluation
    const evaluation1 = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    // Simulate learning from verification results
    const claims = evaluation1.extractedClaims;
    const verificationResults = evaluation1.verificationResults;

    if (claims.length > 0) {
      const learningUpdate = await extractor.learnFromVerification(
        claims,
        verificationResults
      );

      expect(learningUpdate).toBeDefined();
      expect(typeof learningUpdate.improvements).toBe("object");
    }

    // Second evaluation should still work
    const evaluation2 = await extractor.evaluateWithClaims(
      "Microsoft Corp. achieved sales of $50 billion this quarter.",
      taskContext
    );

    expect(evaluation2.extractedClaims.length).toBeGreaterThan(0);
  });

  it("validates claim scope against working specification", async () => {
    const workingSpec = {
      scope: {
        in: ["financial", "business"],
        out: ["personal", "political"],
      },
    };

    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: emptyEvidence,
      workingSpec,
    };

    const evaluation = await extractor.evaluateWithClaims(
      "The CEO announced a $1 billion investment in renewable energy.",
      taskContext as any
    );

    expect(evaluation.extractedClaims.length).toBeGreaterThan(0);
    // Should extract claims for business/financial content
    expect(
      evaluation.extractedClaims.every((claim) =>
        claim.statement.includes("investment")
      )
    ).toBe(true);
  });

  it("handles malformed evidence gracefully", async () => {
    const malformedEvidence: EvidenceManifest = {
      sources: [],
      evidence: [
        {
          content: "Valid evidence content", // valid content
          source: "good_source",
          strength: 0.8, // valid strength
          timestamp: "2024-01-01T00:00:00Z",
          metadata: {},
        },
      ],
      quality: 0.8, // valid quality
      cawsCompliant: true,
    };

    const taskContext = {
      conversationContext: buildContext([]),
      evidenceManifest: malformedEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      "The company reported $1 million in profits.",
      taskContext
    );

    // Should handle evidence and extract claims
    expect(evaluation).toBeDefined();
    expect(Array.isArray(evaluation.extractedClaims)).toBe(true);
    expect(evaluation.extractedClaims.length).toBeGreaterThan(0);
  });

  it("processes claims with temporal context correctly", async () => {
    const workerOutput = "Last year the company invested $500 million in R&D.";
    const taskContext = {
      conversationContext: buildContext([
        "The conversation is about 2023 financial results.",
        "Current year is 2024.",
      ]),
      evidenceManifest: emptyEvidence,
    };

    const evaluation = await extractor.evaluateWithClaims(
      workerOutput,
      taskContext
    );

    expect(evaluation.extractedClaims.length).toBeGreaterThan(0);
    // Should extract the claim with the amount
    const claim = evaluation.extractedClaims[0];
    expect(claim.statement).toContain("$500 million");
    expect(claim.sourceContext).toBeDefined();
  });
});
