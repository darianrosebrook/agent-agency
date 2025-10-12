/**
 * @fileoverview Consistency Validator (ARBITER-007)
 *
 * Checks internal logical consistency and detects contradictions
 * within content to identify logical errors and inconsistencies.
 *
 * @author @darianrosebrook
 */

import {
  VerificationMethodResult,
  VerificationRequest,
  VerificationType,
  VerificationVerdict,
} from "../../types/verification";

/**
 * Configuration for consistency validation
 */
export interface ConsistencyConfig {
  logicEngine: "default" | "strict";
  strictMode: boolean;
  maxStatements?: number;
}

/**
 * Parsed statement from content
 */
interface Statement {
  text: string;
  type: "claim" | "fact" | "opinion" | "temporal";
  subject?: string;
  predicate?: string;
  object?: string;
  temporal?: {
    type: "date" | "sequence" | "duration";
    value: string;
    year?: number;
  };
  negated: boolean;
}

/**
 * Detected contradiction
 */
interface Contradiction {
  statement1: Statement;
  statement2: Statement;
  type: "direct" | "temporal" | "logical";
  severity: "high" | "medium" | "low";
  explanation: string;
}

/**
 * Consistency Validator
 *
 * Validates internal logical consistency and detects contradictions
 * within content.
 */
export class ConsistencyValidator {
  private config: ConsistencyConfig;

  constructor(config: Partial<ConsistencyConfig> = {}) {
    this.config = {
      logicEngine: config.logicEngine ?? "default",
      strictMode: config.strictMode ?? false,
      maxStatements: config.maxStatements ?? 50,
    };
  }

  /**
   * Verify content consistency
   */
  async verify(
    request: VerificationRequest
  ): Promise<VerificationMethodResult> {
    const startTime = Date.now();

    try {
      // Parse content into statements
      const statements = this.parseStatements(request.content);

      if (statements.length === 0) {
        return {
          method: VerificationType.CONSISTENCY_CHECK,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No verifiable statements found in content"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
        };
      }

      // Detect contradictions
      const contradictions = this.detectContradictions(statements);

      // Check temporal consistency
      const temporalIssues = this.checkTemporalConsistency(statements);

      // Assess overall consistency
      const assessment = this.assessConsistency(
        statements,
        contradictions,
        temporalIssues
      );

      return {
        method: VerificationType.CONSISTENCY_CHECK,
        verdict: assessment.verdict,
        confidence: assessment.confidence,
        reasoning: assessment.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: statements.length,
      };
    } catch (error) {
      return {
        method: VerificationType.CONSISTENCY_CHECK,
        verdict: VerificationVerdict.ERROR,
        confidence: 0,
        reasoning: [
          `Consistency check failed: ${
            error instanceof Error ? error.message : String(error)
          }`,
        ],
        processingTimeMs: Date.now() - startTime,
        evidenceCount: 0,
      };
    }
  }

  /**
   * Parse content into individual statements
   */
  private parseStatements(content: string): Statement[] {
    const statements: Statement[] = [];

    // Split into sentences
    const sentences = content
      .split(/[.!?]+/)
      .map((s) => s.trim())
      .filter((s) => s.length > 5);

    for (const sentence of sentences.slice(0, this.config.maxStatements)) {
      const statement = this.parseStatement(sentence);
      if (statement) {
        statements.push(statement);
      }
    }

    return statements;
  }

  /**
   * Parse a single statement
   */
  private parseStatement(text: string): Statement | null {
    const negated = /\b(not|no|never|neither|nor)\b/i.test(text);

    // Check for temporal indicators
    const temporalMatch = text.match(
      /\b(in|on|during|since|until|before|after)\s+(\d{4}|\w+\s+\d{1,2},\s+\d{4})/i
    );

    const temporal = temporalMatch
      ? {
          type: "date" as const,
          value: temporalMatch[2],
          year: this.extractYear(temporalMatch[2]),
        }
      : undefined;

    // Determine statement type
    let type: Statement["type"] = "claim";
    if (temporal) {
      type = "temporal";
    } else if (
      /\b(is|are|was|were|has|have|had)\b/i.test(text) &&
      /\d+/.test(text)
    ) {
      type = "fact";
    } else if (/\b(think|believe|feel|opinion|seems|appears)\b/i.test(text)) {
      type = "opinion";
    }

    // Simple subject-predicate-object extraction
    const spo = this.extractSPO(text);

    return {
      text,
      type,
      subject: spo.subject,
      predicate: spo.predicate,
      object: spo.object,
      temporal,
      negated,
    };
  }

  /**
   * Extract subject-predicate-object from text
   */
  private extractSPO(text: string): {
    subject?: string;
    predicate?: string;
    object?: string;
  } {
    // Simplified SPO extraction
    const words = text.split(/\s+/);

    // Find verb position
    const verbIndex = words.findIndex((w) =>
      /\b(is|are|was|were|has|have|had|do|does|did)\b/i.test(w)
    );

    if (verbIndex === -1) {
      return {};
    }

    const subject = words.slice(0, verbIndex).join(" ");
    const predicate = words[verbIndex];
    const object = words.slice(verbIndex + 1).join(" ");

    return { subject, predicate, object };
  }

  /**
   * Extract year from temporal value
   */
  private extractYear(value: string): number | undefined {
    const yearMatch = value.match(/\d{4}/);
    return yearMatch ? parseInt(yearMatch[0], 10) : undefined;
  }

  /**
   * Detect contradictions between statements
   */
  private detectContradictions(statements: Statement[]): Contradiction[] {
    const contradictions: Contradiction[] = [];

    // Compare each pair of statements
    for (let i = 0; i < statements.length; i++) {
      for (let j = i + 1; j < statements.length; j++) {
        const s1 = statements[i];
        const s2 = statements[j];

        // Check for direct contradictions
        const directContradiction = this.checkDirectContradiction(s1, s2);
        if (directContradiction) {
          contradictions.push(directContradiction);
        }

        // Check for temporal contradictions
        if (s1.temporal && s2.temporal) {
          const temporalContradiction = this.checkTemporalContradiction(s1, s2);
          if (temporalContradiction) {
            contradictions.push(temporalContradiction);
          }
        }
      }
    }

    return contradictions;
  }

  /**
   * Check for direct logical contradictions
   */
  private checkDirectContradiction(
    s1: Statement,
    s2: Statement
  ): Contradiction | null {
    // Same subject, opposite negation
    if (
      s1.subject &&
      s2.subject &&
      s1.subject.toLowerCase() === s2.subject.toLowerCase() &&
      s1.negated !== s2.negated
    ) {
      // Check if predicates/objects are similar
      const predicateSimilar =
        s1.predicate &&
        s2.predicate &&
        s1.predicate.toLowerCase() === s2.predicate.toLowerCase();

      const objectSimilar =
        s1.object &&
        s2.object &&
        this.calculateSimilarity(s1.object, s2.object) > 0.7;

      if (predicateSimilar || objectSimilar) {
        return {
          statement1: s1,
          statement2: s2,
          type: "direct",
          severity: "high",
          explanation: `Direct contradiction: statements make opposite claims about "${s1.subject}"`,
        };
      }
    }

    return null;
  }

  /**
   * Check for temporal contradictions
   */
  private checkTemporalContradiction(
    s1: Statement,
    s2: Statement
  ): Contradiction | null {
    if (
      !s1.temporal ||
      !s2.temporal ||
      !s1.temporal.year ||
      !s2.temporal.year
    ) {
      return null;
    }

    // Check for impossible temporal ordering
    const yearDiff = Math.abs(s1.temporal.year - s2.temporal.year);

    // If statements refer to same subject but different years
    if (
      s1.subject &&
      s2.subject &&
      s1.subject.toLowerCase() === s2.subject.toLowerCase() &&
      yearDiff > 0
    ) {
      // Check for impossible sequences (e.g., died before born)
      const hasDeathKeywords = /\b(died|death|deceased|passed away)\b/i.test(
        s1.text + s2.text
      );
      const hasBirthKeywords = /\b(born|birth|founded|established)\b/i.test(
        s1.text + s2.text
      );

      if (hasDeathKeywords && hasBirthKeywords) {
        const deathYear = /died|death|deceased|passed away/i.test(s1.text)
          ? s1.temporal.year
          : s2.temporal.year;
        const birthYear = /born|birth|founded|established/i.test(s1.text)
          ? s1.temporal.year
          : s2.temporal.year;

        if (deathYear < birthYear) {
          return {
            statement1: s1,
            statement2: s2,
            type: "temporal",
            severity: "high",
            explanation: `Temporal contradiction: death year (${deathYear}) before birth year (${birthYear})`,
          };
        }
      }
    }

    return null;
  }

  /**
   * Check temporal consistency across all statements
   */
  private checkTemporalConsistency(statements: Statement[]): string[] {
    const issues: string[] = [];

    // Extract temporal statements
    const temporalStatements = statements.filter((s) => s.temporal);

    if (temporalStatements.length < 2) {
      return issues;
    }

    // Check for chronological ordering issues
    const years = temporalStatements
      .map((s) => s.temporal?.year)
      .filter((y): y is number => y !== undefined)
      .sort((a, b) => a - b);

    const yearRange = years[years.length - 1] - years[0];

    if (yearRange > 100) {
      issues.push(
        `Wide temporal range detected: ${yearRange} years (${years[0]}-${
          years[years.length - 1]
        })`
      );
    }

    return issues;
  }

  /**
   * Calculate similarity between two strings
   */
  private calculateSimilarity(str1: string, str2: string): number {
    const words1 = new Set(str1.toLowerCase().split(/\s+/));
    const words2 = new Set(str2.toLowerCase().split(/\s+/));

    const intersection = new Set([...words1].filter((w) => words2.has(w)));
    const union = new Set([...words1, ...words2]);

    return intersection.size / union.size;
  }

  /**
   * Assess overall consistency
   */
  private assessConsistency(
    statements: Statement[],
    contradictions: Contradiction[],
    temporalIssues: string[]
  ): {
    verdict: VerificationVerdict;
    confidence: number;
    reasoning: string[];
  } {
    const reasoning: string[] = [];

    reasoning.push(`Analyzed ${statements.length} statements`);

    // No contradictions found
    if (contradictions.length === 0 && temporalIssues.length === 0) {
      reasoning.push("No internal contradictions detected");
      reasoning.push("Temporal consistency maintained");

      return {
        verdict: VerificationVerdict.VERIFIED_TRUE,
        confidence: 0.8,
        reasoning,
      };
    }

    // Minor issues
    if (
      contradictions.length === 0 &&
      temporalIssues.length > 0 &&
      !this.config.strictMode
    ) {
      reasoning.push(`${temporalIssues.length} temporal note(s):`);
      reasoning.push(...temporalIssues);

      return {
        verdict: VerificationVerdict.PARTIALLY_TRUE,
        confidence: 0.6,
        reasoning,
      };
    }

    // Contradictions found
    if (contradictions.length > 0) {
      reasoning.push(`Found ${contradictions.length} contradiction(s):`);

      for (const contradiction of contradictions.slice(0, 3)) {
        reasoning.push(
          `${contradiction.severity.toUpperCase()}: ${
            contradiction.explanation
          }`
        );
      }

      const highSeverityCount = contradictions.filter(
        (c) => c.severity === "high"
      ).length;

      if (highSeverityCount > 0) {
        return {
          verdict: VerificationVerdict.CONTRADICTORY,
          confidence: 0.8,
          reasoning,
        };
      }

      return {
        verdict: VerificationVerdict.PARTIALLY_TRUE,
        confidence: 0.4,
        reasoning,
      };
    }

    return {
      verdict: VerificationVerdict.UNVERIFIED,
      confidence: 0.3,
      reasoning,
    };
  }
}
