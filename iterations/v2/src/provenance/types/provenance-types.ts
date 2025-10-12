/**
 * Provenance Tracking Types
 *
 * Type definitions for the ProvenanceTracker system that tracks AI attribution,
 * commit provenance, and integrates with CAWS provenance tracking.
 *
 * @author @darianrosebrook
 */

import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * AI tool attribution types
 */
export type AIToolType =
  | "cursor-composer"
  | "cursor-tab-completion"
  | "cursor-chat"
  | "github-copilot"
  | "github-copilot-chat"
  | "claude"
  | "gpt-4"
  | "gpt-3.5"
  | "gemini"
  | "other";

/**
 * Provenance entry types
 */
export type ProvenanceEntryType =
  | "commit"
  | "ai_assistance"
  | "human_review"
  | "validation"
  | "budget_check"
  | "quality_gate";

/**
 * Confidence level for provenance attribution
 */
export type AttributionConfidence = "low" | "medium" | "high" | "certain";

/**
 * AI attribution record
 */
export interface AIAttribution {
  /** Unique attribution ID */
  id: string;

  /** AI tool used */
  toolType: AIToolType;

  /** Tool version or model (if known) */
  toolVersion?: string;

  /** Confidence in this attribution */
  confidence: AttributionConfidence;

  /** Timestamp of attribution */
  timestamp: string;

  /** Code regions attributed to AI */
  codeRegions?: Array<{
    file: string;
    startLine: number;
    endLine: number;
    content?: string;
  }>;

  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Provenance chain entry
 */
export interface ProvenanceEntry {
  /** Unique entry ID */
  id: string;

  /** Entry type */
  type: ProvenanceEntryType;

  /** Associated working spec ID */
  specId: string;

  /** Git commit hash (if applicable) */
  commitHash?: string;

  /** Timestamp of entry */
  timestamp: string;

  /** Actor (human or AI tool) */
  actor: {
    type: "human" | "ai";
    identifier: string; // email, tool name, etc.
    name?: string;
  };

  /** Action performed */
  action: {
    type: string; // "generated", "reviewed", "modified", "validated", etc.
    description: string;
    details?: Record<string, unknown>;
  };

  /** Files affected by this entry */
  affectedFiles?: Array<{
    path: string;
    changeType: "added" | "modified" | "deleted" | "renamed";
    linesChanged?: number;
  }>;

  /** Quality metrics at time of entry */
  qualityMetrics?: {
    testCoverage?: number;
    lintErrors?: number;
    typeErrors?: number;
    budgetUsage?: {
      files: number;
      loc: number;
    };
  };

  /** AI attributions for this entry */
  aiAttributions?: AIAttribution[];

  /** Parent entries in the provenance chain */
  parentEntries?: string[];

  /** Child entries in the provenance chain */
  childEntries?: string[];

  /** Verification status */
  verificationStatus?: {
    verified: boolean;
    verifiedBy?: string;
    verifiedAt?: string;
    verificationMethod?: string;
  };

  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Provenance chain summary
 */
export interface ProvenanceChain {
  /** Chain ID (usually spec ID) */
  id: string;

  /** Working spec */
  spec: WorkingSpec;

  /** All entries in the chain */
  entries: ProvenanceEntry[];

  /** Chain statistics */
  statistics: {
    totalEntries: number;
    aiAssistedEntries: number;
    humanEntries: number;
    aiToolsUsed: AIToolType[];
    timeSpan: {
      start: string;
      end: string;
      durationMs: number;
    };
    qualityTrends: {
      testCoverage: Array<{ timestamp: string; value: number }>;
      lintErrors: Array<{ timestamp: string; value: number }>;
      budgetUsage: Array<{ timestamp: string; files: number; loc: number }>;
    };
  };

  /** Chain integrity verification */
  integrity: {
    verified: boolean;
    lastVerified: string;
    hash: string; // Chain hash for integrity checking
  };
}

/**
 * AI attribution statistics
 */
export interface AIAttributionStats {
  /** Total attributions */
  total: number;

  /** Attributions by tool type */
  byToolType: Record<AIToolType, number>;

  /** Attributions by confidence level */
  byConfidence: Record<AttributionConfidence, number>;

  /** Most used AI tools */
  topTools: Array<{
    toolType: AIToolType;
    count: number;
    percentage: number;
  }>;

  /** Attribution trends over time */
  trends: {
    daily: Array<{ date: string; count: number }>;
    weekly: Array<{ week: string; count: number }>;
    monthly: Array<{ month: string; count: number }>;
  };

  /** Average confidence score */
  averageConfidence: number;

  /** Code coverage by AI attribution */
  codeCoverage: {
    attributedLines: number;
    totalLines: number;
    percentage: number;
  };
}

/**
 * Provenance tracker configuration
 */
export interface ProvenanceTrackerConfig {
  /** Project root directory */
  projectRoot: string;

  /** Working spec to track */
  spec: WorkingSpec;

  /** Enable AI attribution tracking */
  enableAIAttribution?: boolean;

  /** AI tools to monitor */
  monitoredAITools?: AIToolType[];

  /** Git repository path */
  gitRepoPath?: string;

  /** CAWS integration settings */
  cawsIntegration?: {
    enabled: boolean;
    provenancePath?: string;
    autoSync?: boolean;
  };

  /** Storage settings */
  storage?: {
    type: "file" | "database" | "memory";
    path?: string;
    retentionDays?: number;
  };

  /** Verification settings */
  verification?: {
    enabled: boolean;
    intervalMs?: number;
    onIntegrityFailure?: (chain: ProvenanceChain) => void | Promise<void>;
  };
}

/**
 * Git commit information
 */
export interface GitCommitInfo {
  /** Commit hash */
  hash: string;

  /** Short hash (first 7 characters) */
  shortHash: string;

  /** Author information */
  author: {
    name: string;
    email: string;
  };

  /** Commit message */
  message: string;

  /** Timestamp */
  timestamp: string;

  /** Parent commit hashes */
  parents: string[];

  /** Files changed */
  filesChanged: Array<{
    path: string;
    status: "added" | "modified" | "deleted" | "renamed";
    linesAdded?: number;
    linesDeleted?: number;
  }>;

  /** Associated spec ID (if found in commit message) */
  associatedSpecId?: string;
}

/**
 * CAWS provenance integration
 */
export interface CAWSProvenanceIntegration {
  /** Integration status */
  status: "connected" | "disconnected" | "error";

  /** Last sync timestamp */
  lastSync?: string;

  /** Sync statistics */
  syncStats?: {
    entriesSynced: number;
    entriesFailed: number;
    lastSyncDuration: number;
  };

  /** CAWS provenance data */
  cawsData?: {
    specId: string;
    provenanceEntries: any[];
    qualityMetrics: Record<string, any>;
    aiAttributions: any[];
  };

  /** Error information */
  error?: {
    message: string;
    timestamp: string;
    details?: any;
  };
}

/**
 * Provenance report
 */
export interface ProvenanceReport {
  /** Report ID */
  id: string;

  /** Report type */
  type: "summary" | "detailed" | "compliance" | "audit";

  /** Time period covered */
  period: {
    start: string;
    end: string;
  };

  /** Associated spec */
  spec: WorkingSpec;

  /** AI attribution statistics */
  aiStats: AIAttributionStats;

  /** Provenance chain */
  provenanceChain: ProvenanceChain;

  /** CAWS integration status */
  cawsIntegration: CAWSProvenanceIntegration;

  /** Quality metrics over time */
  qualityMetrics: {
    testCoverage: Array<{ timestamp: string; value: number }>;
    lintErrors: Array<{ timestamp: string; value: number }>;
    typeErrors: Array<{ timestamp: string; value: number }>;
    budgetUsage: Array<{ timestamp: string; files: number; loc: number }>;
  };

  /** Risk assessment */
  riskAssessment: {
    overallRisk: "low" | "medium" | "high" | "critical";
    riskFactors: string[];
    recommendations: string[];
  };

  /** Compliance status */
  compliance: {
    cawsCompliant: boolean;
    issues: string[];
    recommendations: string[];
  };

  /** Generated timestamp */
  generatedAt: string;

  /** Report hash for integrity */
  hash: string;
}

/**
 * Provenance tracker events
 */
export interface ProvenanceTrackerEvents {
  /** Emitted when a new provenance entry is added */
  "entry:added": (entry: ProvenanceEntry) => void;

  /** Emitted when AI attribution is recorded */
  "attribution:recorded": (attribution: AIAttribution) => void;

  /** Emitted when provenance chain is updated */
  "chain:updated": (chain: ProvenanceChain) => void;

  /** Emitted when CAWS integration syncs */
  "caws:synced": (integration: CAWSProvenanceIntegration) => void;

  /** Emitted when integrity verification runs */
  "integrity:checked": (chain: ProvenanceChain, verified: boolean) => void;

  /** Emitted on tracker error */
  "tracker:error": (error: Error) => void;

  /** Emitted when cleanup operation completes */
  "tracker:cleanup": (result: {
    entriesRemoved: number;
    attributionsRemoved: number;
  }) => void;

  /** Emitted when report is generated */
  "report:generated": (report: ProvenanceReport) => void;
}

/**
 * Provenance tracker capabilities
 */
export interface ProvenanceTrackerCapabilities {
  /** Can track AI attribution */
  trackAIAttribution: boolean;

  /** Can track human contributions */
  trackHumanContributions: boolean;

  /** Can integrate with CAWS */
  integrateWithCAWS: boolean;

  /** Can verify provenance integrity */
  verifyIntegrity: boolean;

  /** Can generate compliance reports */
  generateReports: boolean;

  /** Can analyze contribution patterns */
  analyzePatterns: boolean;
}

/**
 * AI tool detection patterns
 */
export interface AIToolDetectionPatterns {
  /** File patterns to scan for AI attribution */
  filePatterns: string[];

  /** Content patterns indicating AI usage */
  contentPatterns: Array<{
    toolType: AIToolType;
    patterns: RegExp[];
    confidence: AttributionConfidence;
  }>;

  /** Git patterns for AI attribution */
  gitPatterns: Array<{
    toolType: AIToolType;
    commitMessagePatterns: RegExp[];
    authorPatterns: RegExp[];
    confidence: AttributionConfidence;
  }>;
}

/**
 * Provenance storage interface
 */
export interface ProvenanceStorage {
  /** Store a provenance entry */
  storeEntry(entry: ProvenanceEntry): Promise<void>;

  /** Retrieve a provenance entry by ID */
  getEntry(id: string): Promise<ProvenanceEntry | null>;

  /** Get all entries for a spec */
  getEntriesForSpec(specId: string): Promise<ProvenanceEntry[]>;

  /** Update an existing entry */
  updateEntry(id: string, updates: Partial<ProvenanceEntry>): Promise<void>;

  /** Delete an entry */
  deleteEntry(id: string): Promise<void>;

  /** Get provenance chain for a spec */
  getProvenanceChain(specId: string): Promise<ProvenanceChain | null>;

  /** Store provenance chain */
  storeProvenanceChain(chain: ProvenanceChain): Promise<void>;

  /** Store AI attribution */
  storeAttribution(attribution: AIAttribution): Promise<void>;

  /** Get attributions for a time period */
  getAttributions(
    startDate: string,
    endDate: string,
    toolType?: AIToolType
  ): Promise<AIAttribution[]>;

  /** Verify storage integrity */
  verifyIntegrity(): Promise<{ verified: boolean; issues?: string[] }>;

  /** Clean up old data based on retention policy */
  cleanup(
    retentionDays: number
  ): Promise<{ entriesRemoved: number; attributionsRemoved: number }>;
}

/**
 * Git integration interface
 */
export interface GitIntegration {
  /** Get recent commits */
  getRecentCommits(count?: number): Promise<GitCommitInfo[]>;

  /** Get commits for a specific spec */
  getCommitsForSpec(specId: string): Promise<GitCommitInfo[]>;

  /** Get commit details */
  getCommitDetails(hash: string): Promise<GitCommitInfo | null>;

  /** Associate commits with specs based on commit messages */
  associateCommitsWithSpecs(
    commits: GitCommitInfo[]
  ): Promise<Map<string, string>>;

  /** Check if git repository exists */
  isGitRepo(): Promise<boolean>;

  /** Get current branch */
  getCurrentBranch(): Promise<string>;

  /** Get repository status */
  getRepoStatus(): Promise<{
    branch: string;
    ahead: number;
    behind: number;
    modified: string[];
    staged: string[];
    untracked: string[];
  }>;
}
