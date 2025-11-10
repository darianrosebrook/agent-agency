/**
 * Testing API Client
 *
 * Provides functions for running integrated tests and fetching test scenarios.
 *
 * @author @darianrosebrook
 */

import { apiGet, apiPost } from "../utils/api";

/**
 * Test scenario information
 */
export interface TestScenario {
  id: string;
  name: string;
  file_type: string;
  description: string;
}

/**
 * Test execution result
 */
export interface TestResult {
  scenario_id?: string;
  status: "completed" | "failed" | "running";
  exit_code?: number | null;
  stdout: string;
  stderr: string;
  report: string;
  timestamp: string;
}

/**
 * Test execution request
 */
export interface RunTestRequest {
  scenario_id?: string;
}

const API_BASE = "/api/proxy/api/v1";

/**
 * List all available test scenarios
 */
export async function listTestScenarios(): Promise<TestScenario[]> {
  const response = await apiGet<{ scenarios: TestScenario[] }>(
    `${API_BASE}/testing/scenarios`
  );
  return response.scenarios;
}

/**
 * Run a specific integrated test scenario
 */
export async function runIntegratedTest(
  scenarioId: string
): Promise<TestResult> {
  return apiPost<TestResult>(`${API_BASE}/testing/integrated-test`, {
    scenario_id: scenarioId,
  });
}

/**
 * Run all integrated tests
 */
export async function runAllIntegratedTests(): Promise<TestResult> {
  return apiPost<TestResult>(`${API_BASE}/testing/integrated-test/all`, {});
}
