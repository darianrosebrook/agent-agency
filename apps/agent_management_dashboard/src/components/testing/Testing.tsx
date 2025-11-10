"use client";

/**
 * Testing Component
 *
 * Provides UI for running integrated playground + quality evaluation tests.
 * Displays test scenarios, allows running individual or all tests, and shows results.
 *
 * @author @darianrosebrook
 */

import { useState, useEffect } from "react";
import styles from "./Testing.module.scss";
import {
  listTestScenarios,
  runIntegratedTest,
  runAllIntegratedTests,
  type TestScenario,
  type TestResult,
} from "../../lib/api/testing";
import { ErrorDisplay } from "../ErrorDisplay";

export function Testing() {
  const [scenarios, setScenarios] = useState<TestScenario[]>([]);
  const [selectedScenario, setSelectedScenario] = useState<string | null>(null);
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isLoadingScenarios, setIsLoadingScenarios] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    async function fetchScenarios() {
      setIsLoadingScenarios(true);
      setError(null);
      try {
        const data = await listTestScenarios();
        setScenarios(data);
      } catch (err) {
        console.error("Failed to load test scenarios:", err);
        // Provide more detailed error message
        const errorMessage = err instanceof Error ? err.message : String(err);

        // Check if it's a 404 (endpoint not found - testing feature not enabled)
        if (
          errorMessage.includes("404") ||
          errorMessage.includes("Not Found")
        ) {
          setError(
            new Error(
              "Testing endpoints are not available. The API server needs to be started with the 'testing' feature: cargo run --bin api-server --features testing"
            )
          );
        } else {
          setError(
            err instanceof Error
              ? err
              : new Error(`Failed to load test scenarios: ${errorMessage}`)
          );
        }
      } finally {
        setIsLoadingScenarios(false);
      }
    }

    fetchScenarios();
  }, []);

  async function handleRunTest(scenarioId: string) {
    setIsLoading(true);
    setError(null);
    setTestResult(null);
    setSelectedScenario(scenarioId);

    try {
      const result = await runIntegratedTest(scenarioId);
      setTestResult(result);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to run test"));
    } finally {
      setIsLoading(false);
    }
  }

  async function handleRunAllTests() {
    setIsLoading(true);
    setError(null);
    setTestResult(null);
    setSelectedScenario(null);

    try {
      const result = await runAllIntegratedTests();
      setTestResult(result);
    } catch (err) {
      setError(err instanceof Error ? err : new Error("Failed to run tests"));
    } finally {
      setIsLoading(false);
    }
  }

  if (isLoadingScenarios) {
    return (
      <div className={styles.testingContainer}>
        <div className={styles.header}>
          <h1 className={styles.title}>Testing</h1>
          <p className={styles.description}>Loading test scenarios...</p>
        </div>
        <div className={styles.loadingState}>
          <div className={styles.spinner}></div>
          <p>Loading scenarios...</p>
        </div>
      </div>
    );
  }

  if (error && !testResult) {
    return (
      <div className={styles.testingContainer}>
        <div className={styles.header}>
          <h1 className={styles.title}>Testing</h1>
        </div>
        <ErrorDisplay
          error={error}
          onRetry={async () => {
            setIsLoadingScenarios(true);
            setError(null);
            try {
              const data = await listTestScenarios();
              setScenarios(data);
            } catch (err) {
              setError(
                err instanceof Error
                  ? err
                  : new Error("Failed to load test scenarios")
              );
            } finally {
              setIsLoadingScenarios(false);
            }
          }}
        />
      </div>
    );
  }

  return (
    <div className={styles.testingContainer}>
      <div className={styles.header}>
        <h1 className={styles.title}>Integrated Testing</h1>
        <p className={styles.description}>
          Run playground and quality evaluation tests to assess agent
          performance. Tests take 3-6 minutes to complete.
        </p>
      </div>

      <div className={styles.content}>
        <div className={styles.scenariosSection}>
          <h2 className={styles.sectionTitle}>Test Scenarios</h2>
          <div className={styles.scenariosGrid}>
            {scenarios.map((scenario) => (
              <div
                key={scenario.id}
                className={`${styles.scenarioCard} ${
                  selectedScenario === scenario.id ? styles.selected : ""
                }`}
              >
                <div className={styles.scenarioHeader}>
                  <h3 className={styles.scenarioName}>{scenario.name}</h3>
                  <span className={styles.scenarioType}>
                    {scenario.file_type}
                  </span>
                </div>
                <p className={styles.scenarioDescription}>
                  {scenario.description}
                </p>
                <button
                  className={styles.runButton}
                  onClick={() => handleRunTest(scenario.id)}
                  disabled={isLoading}
                >
                  {isLoading && selectedScenario === scenario.id
                    ? "Running..."
                    : "Run Test"}
                </button>
              </div>
            ))}
          </div>

          <div className={styles.actionsSection}>
            <button
              className={styles.runAllButton}
              onClick={handleRunAllTests}
              disabled={isLoading}
            >
              {isLoading && !selectedScenario
                ? "Running All Tests..."
                : "Run All Tests"}
            </button>
          </div>
        </div>

        {testResult !== null || isLoading ? (
          <div className={styles.resultsSection}>
            <h2 className={styles.sectionTitle}>Test Results</h2>
            {isLoading ? (
              <div className={styles.loadingState}>
                <div className={styles.spinner}></div>
                <p>Running test...</p>
                <p className={styles.loadingNote}>
                  This may take 3-6 minutes. Please wait...
                </p>
              </div>
            ) : testResult ? (
              <div className={styles.resultCard}>
                <div className={styles.resultHeader}>
                  <span
                    className={`${styles.statusBadge} ${
                      testResult.status === "completed"
                        ? styles.statusSuccess
                        : testResult.status === "failed"
                        ? styles.statusError
                        : styles.statusRunning
                    }`}
                  >
                    {testResult.status}
                  </span>
                  {testResult.scenario_id ? (
                    <span className={styles.scenarioId}>
                      {testResult.scenario_id}
                    </span>
                  ) : null}
                  {testResult.timestamp && (
                    <span className={styles.timestamp}>
                      {new Date(testResult.timestamp).toLocaleString()}
                    </span>
                  )}
                </div>

                {testResult.exit_code !== null &&
                  testResult.exit_code !== undefined && (
                    <div className={styles.resultMeta}>
                      <span>Exit Code: {testResult.exit_code}</span>
                    </div>
                  )}

                {testResult.stdout && (
                  <div className={styles.outputSection}>
                    <h3 className={styles.outputTitle}>Output</h3>
                    <pre className={styles.outputContent}>
                      {testResult.stdout}
                    </pre>
                  </div>
                )}

                {testResult.stderr && (
                  <div className={styles.outputSection}>
                    <h3 className={styles.outputTitle}>Errors</h3>
                    <pre
                      className={`${styles.outputContent} ${styles.errorOutput}`}
                    >
                      {testResult.stderr}
                    </pre>
                  </div>
                )}

                {testResult.report && (
                  <div className={styles.reportSection}>
                    <h3 className={styles.outputTitle}>Test Report</h3>
                    <div
                      className={styles.reportContent}
                      dangerouslySetInnerHTML={{
                        __html: testResult.report.replace(/\n/g, "<br />"),
                      }}
                    />
                  </div>
                )}
              </div>
            ) : null}
          </div>
        ) : null}
      </div>
    </div>
  );
}
