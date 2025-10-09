/**
 * Base Evaluator for Autonomous Quality Assessment
 *
 * @author @darianrosebrook
 * @description Abstract base class for all evaluators with common interface
 */

export interface EvalCriterion {
  id: string;
  description: string;
  weight: number;
  passed: boolean;
  score: number;
  notes?: string;
}

export interface EvaluationParams {
  taskId: string;
  artifactPath: string;
  iterations: number;
  acceptance: {
    minScore: number;
    mandatoryGates: string[];
  };
  config?: any;
}

export interface EvaluationReport {
  taskId: string;
  artifactPaths: string[];
  status: 'pass' | 'iterate' | 'fail';
  score: number;
  thresholdsMet: string[];
  thresholdsMissed: string[];
  criteria: EvalCriterion[];
  iterations: number;
  stopReason?:
    | 'satisficed'
    | 'max_iterations'
    | 'no_improvement'
    | 'error'
    | 'manual_stop';
  logs?: string[];
  timestamp: string;
}

export abstract class BaseEvaluator {
  abstract evaluate(params: EvaluationParams): Promise<EvaluationReport>;
}
