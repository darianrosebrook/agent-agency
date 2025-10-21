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
    time?: number;
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
  private healthMetrics: {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    responseTimes: number[];
    lastHealthCheck: Date;
    consecutiveFailures: number;
    lastResponseTime: number;
    errorRate: number;
  } = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    responseTimes: [],
    lastHealthCheck: new Date(),
    consecutiveFailures: 0,
    lastResponseTime: 0,
    errorRate: 0,
  };

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
      // Handle empty content
      if (!request.content || request.content.trim().length === 0) {
        return {
          method: VerificationType.CONSISTENCY_CHECK,
          verdict: VerificationVerdict.UNVERIFIED,
          confidence: 0,
          reasoning: ["Empty content cannot be verified for consistency"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
          metadata: {
            contradictions: [],
            circularReasoning: false,
            temporalIssues: [],
          },
        };
      }

      // Parse content into statements
      const statements = this.parseStatements(request.content);
      console.log(
        "DEBUG: Parsed statements:",
        JSON.stringify(statements, null, 2)
      );

      if (statements.length === 0) {
        return {
          method: VerificationType.CONSISTENCY_CHECK,
          verdict: VerificationVerdict.INSUFFICIENT_DATA,
          confidence: 0,
          reasoning: ["No verifiable statements found in content"],
          processingTimeMs: Date.now() - startTime,
          evidenceCount: 0,
          metadata: {
            contradictions: [],
            circularReasoning: false,
            temporalIssues: [],
          },
        };
      }

      // Detect contradictions
      const contradictions = this.detectContradictions(statements);
      console.log(
        "DEBUG: Contradictions:",
        JSON.stringify(contradictions, null, 2)
      );

      // Debug: Check if we have temporal statements
      const temporalStatements = statements.filter((s) => s.temporal);
      console.log("DEBUG: Temporal statements:", temporalStatements.length);

      // Check temporal consistency
      const temporalIssues = this.checkTemporalConsistency(statements);

      // Check for circular reasoning
      const circularReasoning = this.detectCircularReasoning(statements);

      // Check for numerical contradictions
      const numericalContradictions =
        this.detectNumericalContradictions(statements);

      // Combine all contradictions
      const allContradictions = [...contradictions, ...numericalContradictions];

      // Assess overall consistency
      const assessment = this.assessConsistency(
        statements,
        allContradictions,
        temporalIssues,
        circularReasoning
      );

      return {
        method: VerificationType.CONSISTENCY_CHECK,
        verdict: assessment.verdict,
        confidence: assessment.confidence,
        reasoning: assessment.reasoning,
        processingTimeMs: Date.now() - startTime,
        evidenceCount: statements.length,
        metadata: {
          contradictions: allContradictions.map((c) => ({
            type: c.type,
            severity: c.severity,
            explanation: c.explanation,
          })),
          circularReasoning,
          temporalIssues,
        },
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
        metadata: {
          contradictions: [],
          circularReasoning: false,
          temporalIssues: [],
        },
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
    const temporalMatch =
      text.match(
        /\b(in|on|during|since|until|before|after)\s+(\d{4}|\w+\s+\d{1,2},\s+\d{4})/i
      ) ||
      text.match(/\b(\d{4})\b/) || // Also match standalone years
      text.match(/\b(\d{1,2})\s*(AM|PM|am|pm)\b/i); // Also match time patterns like "2 PM"

    console.log(
      "DEBUG: Temporal parsing for text:",
      text,
      "match:",
      temporalMatch
    );

    const temporal = temporalMatch
      ? {
          type: "date" as const,
          value: temporalMatch[0], // Use the full match (e.g., "2 PM", "2010", etc.)
          year: this.extractYear(temporalMatch[0]),
          time: this.extractTime(temporalMatch[0]), // Extract time for time patterns
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
   * Extract time from temporal value (for time patterns like "2 PM")
   */
  private extractTime(value: string): number | undefined {
    const timeMatch = value.match(/(\d{1,2})\s*(AM|PM|am|pm)/i);
    if (timeMatch) {
      let hour = parseInt(timeMatch[1]);
      const period = timeMatch[2].toUpperCase();

      if (period === "PM" && hour !== 12) {
        hour += 12;
      } else if (period === "AM" && hour === 12) {
        hour = 0;
      }

      return hour;
    }
    return undefined;
  }

  /**
   * Detect contradictions between statements
   */
  private detectContradictions(statements: Statement[]): Contradiction[] {
    const contradictions: Contradiction[] = [];
    console.log(
      "DEBUG: detectContradictions called with",
      statements.length,
      "statements"
    );

    // Compare each pair of statements
    for (let i = 0; i < statements.length; i++) {
      for (let j = i + 1; j < statements.length; j++) {
        const s1 = statements[i];
        const s2 = statements[j];

        console.log("DEBUG: Comparing statements:", {
          s1: { text: s1.text, hasTemporal: !!s1.temporal },
          s2: { text: s2.text, hasTemporal: !!s2.temporal },
        });

        // Check for direct contradictions
        const directContradiction = this.checkDirectContradiction(s1, s2);
        if (directContradiction) {
          contradictions.push(directContradiction);
        }

        // Check for temporal contradictions
        // Check if either statement has temporal info OR contains temporal keywords
        const s1HasTemporal =
          s1.temporal ||
          /\b(scheduled|planned|set|concluded|ended|finished|started|began|meeting|event)\b/i.test(
            s1.text
          );
        const s2HasTemporal =
          s2.temporal ||
          /\b(scheduled|planned|set|concluded|ended|finished|started|began|meeting|event)\b/i.test(
            s2.text
          );

        if (s1HasTemporal || s2HasTemporal) {
          console.log(
            "DEBUG: At least one statement has temporal info, checking temporal contradiction"
          );
          const temporalContradiction = this.checkTemporalContradiction(s1, s2);
          if (temporalContradiction) {
            contradictions.push(temporalContradiction);
          }
        } else {
          console.log(
            "DEBUG: Neither statement has temporal info, skipping temporal check"
          );
        }
      }
    }

    console.log(
      "DEBUG: detectContradictions found",
      contradictions.length,
      "contradictions"
    );
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
    console.log("DEBUG: checkTemporalContradiction called with:", {
      s1: { text: s1.text, temporal: s1.temporal },
      s2: { text: s2.text, temporal: s2.temporal },
    });

    // Check if we have temporal information in either statement
    const s1HasTemporal = s1.temporal && (s1.temporal.year || s1.temporal.time);
    const s2HasTemporal = s2.temporal && (s2.temporal.year || s2.temporal.time);

    if (!s1HasTemporal && !s2HasTemporal) {
      console.log("DEBUG: Neither statement has temporal info, returning null");
      return null;
    }

    // Check for time contradictions within a single statement (e.g., "scheduled for 2 PM but concluded at 1 PM")
    const allTimes1 = this.extractAllTimes(s1.text);
    const allTimes2 = this.extractAllTimes(s2.text);

    console.log("DEBUG: All times for s1:", allTimes1, "s2:", allTimes2);

    // Check both statements for time contradictions
    if (allTimes1.length >= 2) {
      const hasScheduledKeywords = /\b(scheduled|planned|set)\b/i.test(s1.text);
      const hasConcludedKeywords = /\b(concluded|ended|finished)\b/i.test(
        s1.text
      );

      console.log("DEBUG: Keywords check s1:", {
        hasScheduledKeywords,
        hasConcludedKeywords,
      });

      if (hasScheduledKeywords && hasConcludedKeywords) {
        // Find scheduled and concluded times in the same statement
        const scheduledTime = this.findTimeNearKeyword(
          s1.text,
          /scheduled|planned|set/i,
          allTimes1
        );
        const concludedTime = this.findTimeNearKeyword(
          s1.text,
          /concluded|ended|finished/i,
          allTimes1
        );

        console.log("DEBUG: Times found s1:", { scheduledTime, concludedTime });

        if (scheduledTime && concludedTime && concludedTime < scheduledTime) {
          console.log("DEBUG: Found time contradiction in s1!");
          return {
            statement1: s1,
            statement2: s2,
            type: "temporal",
            severity: "high",
            explanation: `Temporal contradiction: event concluded at ${concludedTime}:00 before it was scheduled for ${scheduledTime}:00`,
          };
        }
      }
    }

    if (allTimes2.length >= 2) {
      const hasScheduledKeywords = /\b(scheduled|planned|set)\b/i.test(s2.text);
      const hasConcludedKeywords = /\b(concluded|ended|finished)\b/i.test(
        s2.text
      );

      console.log("DEBUG: Keywords check s2:", {
        hasScheduledKeywords,
        hasConcludedKeywords,
      });

      if (hasScheduledKeywords && hasConcludedKeywords) {
        // Find scheduled and concluded times in the same statement
        const scheduledTime = this.findTimeNearKeyword(
          s2.text,
          /scheduled|planned|set/i,
          allTimes2
        );
        const concludedTime = this.findTimeNearKeyword(
          s2.text,
          /concluded|ended|finished/i,
          allTimes2
        );

        console.log("DEBUG: Times found s2:", { scheduledTime, concludedTime });

        if (scheduledTime && concludedTime && concludedTime < scheduledTime) {
          console.log("DEBUG: Found time contradiction in s2!");
          return {
            statement1: s1,
            statement2: s2,
            type: "temporal",
            severity: "high",
            explanation: `Temporal contradiction: event concluded at ${concludedTime}:00 before it was scheduled for ${scheduledTime}:00`,
          };
        }
      }
    }

    // Check for impossible temporal ordering (year-based)
    if (s1.temporal?.year && s2.temporal?.year) {
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

      // Check for company founding vs starting contradictions
      const hasFoundedKeywords =
        /\b(founded|established|created|incorporated)\b/i.test(
          s1.text + s2.text
        );
      const hasStartedKeywords = /\b(started|began|launched|initiated)\b/i.test(
        s1.text + s2.text
      );

      if (hasFoundedKeywords && hasStartedKeywords) {
        const foundedYear = /founded|established|created|incorporated/i.test(
          s1.text
        )
          ? s1.temporal.year
          : s2.temporal.year;
        const startedYear = /started|began|launched|initiated/i.test(s1.text)
          ? s1.temporal.year
          : s2.temporal.year;

        // Can't start a company before it was founded
        if (startedYear < foundedYear) {
          return {
            statement1: s1,
            statement2: s2,
            type: "temporal",
            severity: "high",
            explanation: `Temporal contradiction: company started (${startedYear}) before it was founded (${foundedYear})`,
          };
        }
      }

      // Check for general temporal contradictions (same entity, different years)
      // This handles cases like "The company was founded in 2010. The CEO started the company in 2015."
      if (s1.temporal && s2.temporal && s1.temporal.year && s2.temporal.year) {
        // Look for same entity references
        const hasCompanyKeywords =
          /\b(company|corporation|business|organization)\b/i.test(
            s1.text + s2.text
          );
        const hasFoundedKeywords2 =
          /\b(founded|established|created|incorporated)\b/i.test(
            s1.text + s2.text
          );
        const hasStartedKeywords2 =
          /\b(started|began|launched|initiated)\b/i.test(s1.text + s2.text);

        console.log("DEBUG: Temporal check:", {
          hasCompanyKeywords,
          hasFoundedKeywords2,
          hasStartedKeywords2,
          s1Text: s1.text,
          s2Text: s2.text,
          s1Year: s1.temporal.year,
          s2Year: s2.temporal.year,
        });

        if (hasCompanyKeywords && hasFoundedKeywords2 && hasStartedKeywords2) {
          const foundedYear = /founded|established|created|incorporated/i.test(
            s1.text
          )
            ? s1.temporal.year
            : s2.temporal.year;
          const startedYear = /started|began|launched|initiated/i.test(s1.text)
            ? s1.temporal.year
            : s2.temporal.year;

          console.log("DEBUG: Years:", { foundedYear, startedYear });

          // Can't start a company before it was founded
          if (startedYear < foundedYear) {
            console.log("DEBUG: Found temporal contradiction!");
            return {
              statement1: s1,
              statement2: s2,
              type: "temporal",
              severity: "high",
              explanation: `Temporal contradiction: company started (${startedYear}) before it was founded (${foundedYear})`,
            };
          }
        }
      }
    } // Close the year-based check

    // Check for meeting time contradictions
    const hasMeetingKeywords = /\b(meeting|event|conference)\b/i.test(
      s1.text + s2.text
    );
    const hasEndedKeywords = /\b(ended|concluded|finished)\b/i.test(
      s1.text + s2.text
    );
    const hasStartedKeywords2 = /\b(started|began|opened)\b/i.test(
      s1.text + s2.text
    );

    if (hasMeetingKeywords && hasEndedKeywords && hasStartedKeywords2) {
      const endedTime = this.extractTimeFromText(
        s1.text + s2.text,
        /ended|concluded|finished/i
      );
      const startedTime = this.extractTimeFromText(
        s1.text + s2.text,
        /started|began|opened/i
      );

      if (endedTime && startedTime && endedTime < startedTime) {
        return {
          statement1: s1,
          statement2: s2,
          type: "temporal",
          severity: "high",
          explanation: `Temporal contradiction: meeting ended (${endedTime}) before it started (${startedTime})`,
        };
      }
    }

    // Check for time contradictions using temporal.time field
    if (s1.temporal && s2.temporal && s1.temporal.time && s2.temporal.time) {
      const hasScheduledKeywords = /\b(scheduled|planned|set)\b/i.test(
        s1.text + s2.text
      );
      const hasConcludedKeywords = /\b(concluded|ended|finished)\b/i.test(
        s1.text + s2.text
      );

      if (hasScheduledKeywords && hasConcludedKeywords) {
        // Find which statement has scheduled time and which has concluded time
        const scheduledTime = /scheduled|planned|set/i.test(s1.text)
          ? s1.temporal.time
          : s2.temporal.time;
        const concludedTime = /concluded|ended|finished/i.test(s1.text)
          ? s1.temporal.time
          : s2.temporal.time;

        // Can't conclude before being scheduled
        if (concludedTime < scheduledTime) {
          return {
            statement1: s1,
            statement2: s2,
            type: "temporal",
            severity: "high",
            explanation: `Temporal contradiction: event concluded at ${concludedTime}:00 before it was scheduled for ${scheduledTime}:00`,
          };
        }
      }
    }

    return null;
  }

  /**
   * TODO: Implement comprehensive temporal information extraction
   * - Use NLP libraries for advanced date/time parsing (chrono, dateutil)
   * - Support multiple time formats and cultural variations
   * - Implement relative time parsing (yesterday, last week, in 2 hours)
   * - Add timezone detection and normalization
   * - Support temporal range and duration extraction
   * - Implement temporal consistency validation across documents
   * - Add temporal reasoning and conflict detection
   * - Support temporal metadata extraction and indexing
   */
  private extractTimeFromText(
    text: string,
    keywordPattern: RegExp
  ): number | null {
    const timeMatch = text.match(/(\d{1,2})\s*(AM|PM|am|pm)/i);
    if (timeMatch) {
      let hour = parseInt(timeMatch[1]);
      const period = timeMatch[2].toUpperCase();

      if (period === "PM" && hour !== 12) {
        hour += 12;
      } else if (period === "AM" && hour === 12) {
        hour = 0;
      }

      return hour;
    }
    return null;
  }

  /**
   * Extract all time references from text
   */
  private extractAllTimes(
    text: string
  ): Array<{ time: number; index: number; text: string }> {
    const times: Array<{ time: number; index: number; text: string }> = [];
    const timeRegex = /\b(\d{1,2})\s*(AM|PM|am|pm)\b/gi;
    let match;

    // Reset regex lastIndex to ensure we start from the beginning
    timeRegex.lastIndex = 0;

    while ((match = timeRegex.exec(text)) !== null) {
      let hour = parseInt(match[1]);
      const period = match[2].toUpperCase();

      if (period === "PM" && hour !== 12) {
        hour += 12;
      } else if (period === "AM" && hour === 12) {
        hour = 0;
      }

      times.push({
        time: hour,
        index: match.index,
        text: match[0],
      });
    }

    return times;
  }

  /**
   * Find the time closest to a keyword
   */
  private findTimeNearKeyword(
    text: string,
    keywordPattern: RegExp,
    allTimes: Array<{ time: number; index: number; text: string }>
  ): number | null {
    const keywordMatch = text.match(keywordPattern);
    if (!keywordMatch || allTimes.length === 0) {
      return null;
    }

    const keywordIndex = keywordMatch.index!;
    console.log(
      "DEBUG: findTimeNearKeyword - keyword:",
      keywordMatch[0],
      "at index:",
      keywordIndex,
      "allTimes:",
      allTimes
    );

    // Find the time that comes after the keyword (within a reasonable distance)
    const timesAfterKeyword = allTimes.filter(
      (time) => time.index > keywordIndex
    );
    console.log("DEBUG: Times after keyword:", timesAfterKeyword);

    if (timesAfterKeyword.length > 0) {
      // Find the closest time after the keyword
      let closestTime = timesAfterKeyword[0];
      let minDistance = timesAfterKeyword[0].index - keywordIndex;

      for (const time of timesAfterKeyword) {
        const distance = time.index - keywordIndex;
        console.log(
          "DEBUG: Time",
          time.text,
          "at index",
          time.index,
          "distance from keyword:",
          distance
        );
        if (distance < minDistance) {
          minDistance = distance;
          closestTime = time;
        }
      }

      console.log(
        "DEBUG: Closest time after keyword:",
        closestTime.text,
        "at",
        closestTime.time
      );
      return closestTime.time;
    }

    // Fallback: find the closest time overall
    let closestTime = allTimes[0];
    let minDistance = Math.abs(allTimes[0].index - keywordIndex);

    for (const time of allTimes) {
      const distance = Math.abs(time.index - keywordIndex);
      console.log(
        "DEBUG: Time",
        time.text,
        "at index",
        time.index,
        "distance from keyword:",
        distance
      );
      if (distance < minDistance) {
        minDistance = distance;
        closestTime = time;
      }
    }

    console.log(
      "DEBUG: Closest time to keyword (fallback):",
      closestTime.text,
      "at",
      closestTime.time
    );
    return closestTime.time;
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
   * Detect circular reasoning patterns
   */
  private detectCircularReasoning(statements: Statement[]): boolean {
    if (statements.length < 2) {
      return false;
    }

    // Check for A is true because B is true, B is true because A is true
    for (let i = 0; i < statements.length; i++) {
      for (let j = i + 1; j < statements.length; j++) {
        const s1 = statements[i];
        const s2 = statements[j];

        // Look for circular dependency patterns
        if (this.hasCircularDependency(s1.text, s2.text)) {
          return true;
        }
      }
    }

    return false;
  }

  /**
   * Check if two statements have circular dependency
   */
  private hasCircularDependency(text1: string, text2: string): boolean {
    const lower1 = text1.toLowerCase();
    const lower2 = text2.toLowerCase();

    // Pattern: "A is true because B is true" and "B is true because A is true"
    const becausePattern =
      /\b(\w+)\s+is\s+true\s+because\s+(\w+)\s+is\s+true\b/i;
    const match1 = lower1.match(becausePattern);
    const match2 = lower2.match(becausePattern);

    if (match1 && match2) {
      const [, a1, b1] = match1;
      const [, a2, b2] = match2;

      // Check if A1 == B2 and B1 == A2 (circular)
      if (a1 === b2 && b1 === a2) {
        return true;
      }
    }

    // Pattern: "A because B" and "B because A"
    const simpleBecausePattern = /\b(\w+)\s+because\s+(\w+)\b/i;
    const simpleMatch1 = lower1.match(simpleBecausePattern);
    const simpleMatch2 = lower2.match(simpleBecausePattern);

    if (simpleMatch1 && simpleMatch2) {
      const [, a1, b1] = simpleMatch1;
      const [, a2, b2] = simpleMatch2;

      // Check if A1 == B2 and B1 == A2 (circular)
      if (a1 === b2 && b1 === a2) {
        return true;
      }
    }

    return false;
  }

  /**
   * Detect numerical contradictions
   */
  private detectNumericalContradictions(
    statements: Statement[]
  ): Contradiction[] {
    const contradictions: Contradiction[] = [];

    for (let i = 0; i < statements.length; i++) {
      for (let j = i + 1; j < statements.length; j++) {
        const s1 = statements[i];
        const s2 = statements[j];

        const numericalContradiction = this.checkNumericalContradiction(s1, s2);
        if (numericalContradiction) {
          contradictions.push(numericalContradiction);
        }
      }
    }

    return contradictions;
  }

  /**
   * Check for numerical contradictions
   */
  private checkNumericalContradiction(
    s1: Statement,
    s2: Statement
  ): Contradiction | null {
    // Extract numbers from statements
    const numbers1 = this.extractNumbers(s1.text);
    const numbers2 = this.extractNumbers(s2.text);

    if (numbers1.length === 0 || numbers2.length === 0) {
      return null;
    }

    // Check for total vs parts contradiction
    // Pattern: "The total is X. A is Y, B is Z." where Y + Z > X
    const totalPattern =
      /\b(total|sum|combined|together)\s+(is|equals?)\s+(\d+)\b/i;
    const partsPattern = /\b(\w+)\s+(is|equals?)\s+(\d+)\b/g;

    const totalMatch1 = s1.text.match(totalPattern);
    const totalMatch2 = s2.text.match(totalPattern);

    if (totalMatch1 || totalMatch2) {
      const totalStatement = totalMatch1 ? s1 : s2;
      const partsStatement = totalMatch1 ? s2 : s1;

      const totalValue = parseInt(
        totalMatch1 ? totalMatch1[3] : totalMatch2![3]
      );
      const partsMatches = Array.from(
        partsStatement.text.matchAll(partsPattern)
      );

      if (partsMatches.length >= 2) {
        const partsSum = partsMatches.reduce(
          (sum, match) => sum + parseInt(match[3]),
          0
        );

        if (partsSum > totalValue) {
          return {
            statement1: totalStatement,
            statement2: partsStatement,
            type: "logical",
            severity: "high",
            explanation: `Numerical contradiction: parts sum to ${partsSum} but total is ${totalValue}`,
          };
        }
      }
    }

    return null;
  }

  /**
   * Extract numbers from text
   */
  private extractNumbers(text: string): number[] {
    const numberMatches = text.match(/\b\d+\b/g);
    return numberMatches ? numberMatches.map((n) => parseInt(n)) : [];
  }

  /**
   * Assess overall consistency
   */
  private assessConsistency(
    statements: Statement[],
    contradictions: Contradiction[],
    temporalIssues: string[],
    circularReasoning: boolean
  ): {
    verdict: VerificationVerdict;
    confidence: number;
    reasoning: string[];
  } {
    const reasoning: string[] = [];

    reasoning.push(`Analyzed ${statements.length} statements`);

    // Check for circular reasoning first (highest priority)
    if (circularReasoning) {
      reasoning.push("Circular reasoning detected");
      return {
        verdict: VerificationVerdict.VERIFIED_FALSE,
        confidence: 0.9,
        reasoning,
      };
    }

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
          verdict: VerificationVerdict.VERIFIED_FALSE,
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

  /**
   * Get method health status
   */
  getHealth(): { available: boolean; responseTime: number; errorRate: number } {
    // Update error rate based on recent metrics
    this.updateErrorRate();

    // Check availability based on consecutive failures and recent activity
    const now = new Date();
    const timeSinceLastCheck =
      now.getTime() - this.healthMetrics.lastHealthCheck.getTime();
    const available: boolean =
      this.healthMetrics.consecutiveFailures < 3 && timeSinceLastCheck < 300000; // 5 minutes

    // Calculate average response time
    const avgResponseTime =
      this.healthMetrics.responseTimes.length > 0
        ? this.healthMetrics.responseTimes.reduce(
            (sum, time) => sum + time,
            0
          ) / this.healthMetrics.responseTimes.length
        : this.healthMetrics.lastResponseTime || 0;

    return {
      available,
      responseTime: Math.round(avgResponseTime),
      errorRate: Math.round(this.healthMetrics.errorRate * 100) / 100,
    };
  }

  /**
   * Record a successful verification request
   */
  private recordSuccess(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.successfulRequests++;
    this.healthMetrics.consecutiveFailures = 0;
    this.healthMetrics.lastResponseTime = responseTime;
    this.healthMetrics.responseTimes.push(responseTime);

    // Keep only last 100 response times for rolling average
    if (this.healthMetrics.responseTimes.length > 100) {
      this.healthMetrics.responseTimes.shift();
    }

    this.healthMetrics.lastHealthCheck = new Date();
  }

  /**
   * Record a failed verification request
   */
  private recordFailure(responseTime: number): void {
    this.healthMetrics.totalRequests++;
    this.healthMetrics.failedRequests++;
    this.healthMetrics.consecutiveFailures++;
    this.healthMetrics.lastResponseTime = responseTime;
    this.healthMetrics.responseTimes.push(responseTime);

    // Keep only last 100 response times for rolling average
    if (this.healthMetrics.responseTimes.length > 100) {
      this.healthMetrics.responseTimes.shift();
    }

    this.healthMetrics.lastHealthCheck = new Date();
  }

  /**
   * Update error rate based on recent metrics
   */
  private updateErrorRate(): void {
    if (this.healthMetrics.totalRequests > 0) {
      this.healthMetrics.errorRate =
        this.healthMetrics.failedRequests / this.healthMetrics.totalRequests;
    } else {
      this.healthMetrics.errorRate = 0;
    }
  }
}
