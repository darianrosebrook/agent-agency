/**
 * DSPy Integration Module
 *
 * Exports TypeScript interface to Python-based DSPy service.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


export { DSPyClient } from "./DSPyClient.js";
export type {
  DSPyClientConfig,
  HealthResponse,
  JudgeEvaluationRequest,
  JudgeEvaluationResponse,
  RubricOptimizationRequest,
  RubricOptimizationResponse,
  SignatureOptimizationRequest,
  SignatureOptimizationResponse,
} from "./DSPyClient.js";
