/**
 * Iterative Guidance Types
 *
 * Type definitions for the IterativeGuidance system that provides
 * intelligent progress tracking and developer guidance.
 *
 * @author @darianrosebrook
 */

import type {
  BudgetStatistics,
  BudgetUsage,
} from "../../monitoring/types/budget-monitor-types.js";
import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * Progress status for acceptance criteria
 */
export type AcceptanceStatus =
  | "not_started"
  | "in_progress"
  | "completed"
  | "blocked"
  | "cancelled";

/**
 * Confidence level for estimates and analysis
 */
export type ConfidenceLevel = "low" | "medium" | "high";

/**
 * Priority level for next steps
 */
export type StepPriority = "critical" | "high" | "medium" | "low";

/**
 * Complexity level for work estimates
 */
export type ComplexityLevel = "simple" | "moderate" | "complex" | "expert";

/**
 * Acceptance criterion progress tracking
 */
export interface AcceptanceProgress {
  /** Unique identifier (e.g., "A1", "A2") */
  id: string;

  /** Current status */
  status: AcceptanceStatus;

  /** Progress percentage (0-100) */
  progressPercent: number;

  /** Given condition */
  given: string;

  /** When trigger condition */
  when: string;

  /** Then expected outcome */
  then: string;

  /** Evidence of progress (file paths, test names, etc.) */
  evidence: string[];

  /** Blocking issues */
  blockers: string[];

  /** Estimated hours remaining */
  estimatedHoursRemaining: number;

  /** Confidence in progress assessment */
  confidence: ConfidenceLevel;

  /** Last updated timestamp */
  lastUpdated: string;
}

/**
 * Gap analysis result
 */
export interface GapAnalysis {
  /** Category of gap */
  category:
    | "implementation"
    | "testing"
    | "documentation"
    | "integration"
    | "validation";

  /** Specific gap description */
  description: string;

  /** Impact severity */
  severity: "low" | "medium" | "high" | "critical";

  /** Affected acceptance criteria */
  affectedCriteria: string[];

  /** Estimated effort to fix */
  estimatedEffort: {
    hours: number;
    complexity: ComplexityLevel;
  };

  /** Suggested remediation steps */
  remediationSteps: string[];

  /** Priority for addressing */
  priority: StepPriority;
}

/**
 * Actionable next step
 */
export interface NextStep {
  /** Step identifier */
  id: string;

  /** Step title */
  title: string;

  /** Detailed description */
  description: string;

  /** Step priority */
  priority: StepPriority;

  /** Category of work */
  category:
    | "implementation"
    | "testing"
    | "refactoring"
    | "documentation"
    | "integration";

  /** Estimated effort */
  estimatedEffort: {
    hours: number;
    complexity: ComplexityLevel;
    confidence: ConfidenceLevel;
  };

  /** Prerequisites for this step */
  prerequisites: string[];

  /** Files that need to be created/modified */
  affectedFiles: string[];

  /** Expected outcomes */
  expectedOutcomes: string[];

  /** Risk level */
  risk: "low" | "medium" | "high";

  /** Dependencies on other steps */
  dependencies: string[];

  /** Whether this step can be done in parallel */
  parallelizable: boolean;
}

/**
 * Work estimate summary
 */
export interface WorkEstimate {
  /** Total estimated hours remaining */
  totalHours: number;

  /** Breakdown by category */
  hoursByCategory: {
    implementation: number;
    testing: number;
    refactoring: number;
    documentation: number;
    integration: number;
  };

  /** Breakdown by priority */
  hoursByPriority: {
    critical: number;
    high: number;
    medium: number;
    low: number;
  };

  /** Confidence intervals */
  confidenceIntervals: {
    pessimistic: number; // +50% hours
    optimistic: number; // -30% hours
    mostLikely: number; // baseline
  };

  /** Parallelization factor (how much can be done in parallel) */
  parallelizationFactor: number;

  /** Estimated completion date ranges */
  completionEstimates: {
    earliest: string; // ISO date string
    mostLikely: string; // ISO date string
    latest: string; // ISO date string
  };

  /** Risk factors affecting estimate */
  riskFactors: {
    technicalDebt: number; // 0-1
    teamExperience: number; // 0-1
    requirementClarity: number; // 0-1
    externalDependencies: number; // 0-1
  };
}

/**
 * Overall progress summary
 */
export interface ProgressSummary {
  /** Overall completion percentage (0-100) */
  overallProgress: number;

  /** Acceptance criteria progress */
  acceptanceCriteria: AcceptanceProgress[];

  /** Identified gaps */
  gaps: GapAnalysis[];

  /** Recommended next steps (prioritized) */
  nextSteps: NextStep[];

  /** Work estimate summary */
  workEstimate: WorkEstimate;

  /** Critical blockers */
  criticalBlockers: string[];

  /** Recent achievements */
  recentAchievements: string[];

  /** Risk assessment */
  riskAssessment: {
    overallRisk: "low" | "medium" | "high" | "critical";
    riskFactors: string[];
    mitigationStrategies: string[];
  };

  /** Confidence in analysis */
  analysisConfidence: ConfidenceLevel;

  /** Generated timestamp */
  generatedAt: string;
}

/**
 * Guidance system configuration
 */
export interface GuidanceConfig {
  /** Working spec to analyze */
  spec: WorkingSpec;

  /** Current budget usage */
  budgetUsage?: BudgetUsage;

  /** Budget statistics */
  budgetStats?: BudgetStatistics;

  /** Project root directory */
  projectRoot: string;

  /** Files currently in the codebase */
  existingFiles?: string[];

  /** Test files and their coverage */
  testFiles?: string[];

  /** Recent git commits or changes */
  recentChanges?: Array<{
    file: string;
    type: "added" | "modified" | "deleted";
    timestamp: string;
  }>;

  /** AI assistance attribution */
  aiAttribution?: {
    totalCommits: number;
    aiAssistedCommits: number;
    aiToolsUsed: string[];
  };
}

/**
 * Guidance analysis result
 */
export interface GuidanceAnalysis {
  /** Success flag */
  success: boolean;

  /** Progress summary */
  summary?: ProgressSummary;

  /** Error details if failed */
  error?: {
    message: string;
    details?: any;
  };

  /** Analysis metadata */
  metadata: {
    duration: number;
    confidence: ConfidenceLevel;
    analyzedAt: string;
  };
}

/**
 * Step-by-step guidance for implementation
 */
export interface StepGuidance {
  /** Current step number */
  currentStep: number;

  /** Total number of steps */
  totalSteps: number;

  /** Current step details */
  step: NextStep;

  /** Progress within current step (0-100) */
  stepProgress: number;

  /** Time spent on current step */
  timeSpent: number;

  /** Estimated time remaining for current step */
  estimatedTimeRemaining: number;

  /** Tips for current step */
  tips: string[];

  /** Common pitfalls to avoid */
  pitfalls: string[];

  /** Quality checks for completion */
  qualityChecks: string[];

  /** Next steps preview */
  nextStepsPreview: NextStep[];
}

/**
 * Guidance system events
 */
export interface GuidanceEvents {
  /** Emitted when progress analysis is complete */
  "progress:analyzed": (summary: ProgressSummary) => void;

  /** Emitted when next steps are generated */
  "steps:generated": (steps: NextStep[]) => void;

  /** Emitted when work estimate is updated */
  "estimate:updated": (estimate: WorkEstimate) => void;

  /** Emitted when gaps are identified */
  "gaps:identified": (gaps: GapAnalysis[]) => void;

  /** Emitted when analysis starts */
  "analysis:start": () => void;

  /** Emitted when analysis completes */
  "analysis:complete": (result: GuidanceAnalysis) => void;

  /** Emitted on analysis error */
  "analysis:error": (error: Error) => void;
}

/**
 * Guidance system capabilities
 */
export interface GuidanceCapabilities {
  /** Can analyze acceptance criteria progress */
  analyzeAcceptanceCriteria: boolean;

  /** Can identify implementation gaps */
  identifyGaps: boolean;

  /** Can generate actionable next steps */
  generateNextSteps: boolean;

  /** Can estimate work effort */
  estimateWork: boolean;

  /** Can provide step-by-step guidance */
  provideStepGuidance: boolean;

  /** Can assess project risks */
  assessRisks: boolean;

  /** Can integrate with external systems */
  integrateExternal: boolean;
}

/**
 * Guidance context for analysis
 */
export interface GuidanceContext {
  /** Development phase */
  phase:
    | "planning"
    | "implementation"
    | "testing"
    | "refinement"
    | "completion";

  /** Team composition */
  teamSize: number;

  /** Experience level */
  experienceLevel: "junior" | "mid" | "senior" | "expert";

  /** Time pressure */
  timePressure: "low" | "medium" | "high" | "critical";

  /** Quality requirements */
  qualityRequirements: "basic" | "standard" | "high" | "enterprise";

  /** Technology familiarity */
  technologyFamiliarity: "new" | "familiar" | "expert";
}

/**
 * Guidance recommendations
 */
export interface GuidanceRecommendation {
  /** Recommendation type */
  type: "approach" | "tool" | "pattern" | "practice" | "resource";

  /** Recommendation title */
  title: string;

  /** Detailed explanation */
  explanation: string;

  /** Expected benefit */
  benefit: string;

  /** Implementation effort */
  effort: "low" | "medium" | "high";

  /** Urgency */
  urgency: "low" | "medium" | "high";

  /** Supporting evidence */
  evidence: string[];

  /** Prerequisites */
  prerequisites: string[];
}
