/**
 * Evaluation Framework Types
 *
 * @author @darianrosebrook
 * @description Type definitions for the agent evaluation system
 */

export type EvalStatus = "pass" | "iterate" | "fail";

export interface EvalCriterion {
  id: string;
  description: string;
  weight: number; // 0..1, sum ≤ 1.0
  passed: boolean;
  score: number; // 0..1
  notes?: string;
}

export interface EvalReport {
  taskId: string;
  artifactPaths: string[];
  // aggregate
  status: EvalStatus; // pass | iterate | fail
  score: number; // 0..1 weighted
  thresholdsMet: string[]; // names of thresholds met
  thresholdsMissed: string[]; // names missed
  criteria: EvalCriterion[];
  // meta for satisficing & yield
  iterations: number; // current iteration count
  promptTokens?: number; // optional for yield
  completionTokens?: number;
  elapsedMs?: number;
  // stopping guidance
  stopReason?:
    | "satisficed"
    | "max-iterations"
    | "quality-ceiling"
    | "failed-gates"
    | "unknown";
  nextActions?: string[]; // agent-facing hints
  // raw logs
  logs?: string[];
  // provenance / reproducibility
  seed?: number;
  toolVersions?: Record<string, string>;
  timestamp: string; // ISO-8601
}
