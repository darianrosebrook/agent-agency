/**
 * Comprehensive E2E Test Runner
 *
 * @author @darianrosebrook
 * @description Orchestrates all E2E tests with comprehensive reporting and analysis
 */

import fs from "fs";
import path from "path";
import {
  E2ETestResult,
  E2ETestSuite,
  runComprehensiveE2ETests,
} from "./evaluation-runner";

export interface E2ETestReport {
  suite: E2ETestSuite;
  results: E2ETestResult[];
  summary: {
    totalTests: number;
    passedTests: number;
    failedTests: number;
    averageScore: number;
    totalDuration: number;
    successRate: number;
    performanceMetrics: {
      averageTestDuration: number;
      fastestTest: { name: string; duration: number };
      slowestTest: { name: string; duration: number };
    };
    agentMetrics: {
      totalInteractions: number;
      averageInteractionsPerTest: number;
      toolCallsByType: Record<string, number>;
      evaluationCount: number;
    };
    qualityMetrics: {
      criteriaEvaluated: number;
      averageCriteriaPerTest: number;
      topPerformingCriteria: Array<{ name: string; averageScore: number }>;
      challengingCriteria: Array<{ name: string; averageScore: number }>;
    };
  };
  recommendations: string[];
  timestamp: string;
  duration: number;
}

/**
 * Run all E2E tests with comprehensive reporting
 */
export async function runAllE2ETests(): Promise<E2ETestReport> {
  const startTime = Date.now();

  console.log("🚀 Starting Comprehensive E2E Test Suite");
  console.log("========================================");

  try {
    // Run the comprehensive test suite
    const { results, summary } = await runComprehensiveE2ETests();

    // Create detailed report
    const report = await generateDetailedReport(results, summary);

    const totalDuration = Date.now() - startTime;

    const finalReport: E2ETestReport = {
      suite: {
        name: "Agent Agency E2E Test Suite",
        description:
          "Comprehensive end-to-end testing of agent evaluation capabilities",
        scenarios: results.map((r) => r.scenario),
      },
      results,
      summary: {
        ...summary,
        successRate: (summary.passed / summary.total) * 100,
        performanceMetrics: report.performanceMetrics,
        agentMetrics: report.agentMetrics,
        qualityMetrics: report.qualityMetrics,
      },
      recommendations: report.recommendations,
      timestamp: new Date().toISOString(),
      duration: totalDuration,
    };

    // Display final results
    displayFinalResults(finalReport);

    // Save report to file
    await saveReport(finalReport);

    return finalReport;
  } catch (error) {
    console.error("❌ E2E Test Suite Failed:", error);
    throw error;
  }
}

/**
 * Generate detailed analysis from test results
 */
async function generateDetailedReport(
  results: E2ETestResult[],
  _basicSummary: any
): Promise<{
  performanceMetrics: any;
  agentMetrics: any;
  qualityMetrics: any;
  recommendations: string[];
}> {
  // Performance metrics
  const durations = results.map((r) => r.executionTime);
  const performanceMetrics = {
    averageTestDuration:
      durations.reduce((a, b) => a + b, 0) / durations.length,
    fastestTest: {
      name: results.reduce((prev, curr) =>
        prev.executionTime < curr.executionTime ? prev : curr
      ).scenario.name,
      duration: Math.min(...durations),
    },
    slowestTest: {
      name: results.reduce((prev, curr) =>
        prev.executionTime > curr.executionTime ? prev : curr
      ).scenario.name,
      duration: Math.max(...durations),
    },
  };

  // Agent interaction metrics
  const allInteractions = results.flatMap((r) => r.agentInteractions);
  const toolCalls = allInteractions.filter((i) => i.type === "tool_call");
  const evaluations = allInteractions.filter((i) => i.type === "evaluation");

  const toolCallsByType: Record<string, number> = {};
  toolCalls.forEach((call) => {
    const toolName = (call.details as any).tool || "unknown";
    toolCallsByType[toolName] = (toolCallsByType[toolName] || 0) + 1;
  });

  const agentMetrics = {
    totalInteractions: allInteractions.length,
    averageInteractionsPerTest: allInteractions.length / results.length,
    toolCallsByType,
    evaluationCount: evaluations.length,
  };

  // Quality metrics
  const allCriteria = results.flatMap((r) =>
    r.report.criteria.map((c) => ({
      name: c.criteria.name,
      score: c.result.score,
      passed: c.result.passed,
    }))
  );

  const criteriaByName: Record<string, number[]> = {};
  allCriteria.forEach((c) => {
    if (!criteriaByName[c.name]) {
      criteriaByName[c.name] = [];
    }
    criteriaByName[c.name].push(c.score);
  });

  const criteriaAverages = Object.entries(criteriaByName).map(
    ([name, scores]) => ({
      name,
      averageScore: scores.reduce((a, b) => a + b, 0) / scores.length,
    })
  );

  const qualityMetrics = {
    criteriaEvaluated: allCriteria.length,
    averageCriteriaPerTest: allCriteria.length / results.length,
    topPerformingCriteria: criteriaAverages
      .sort((a, b) => b.averageScore - a.averageScore)
      .slice(0, 3),
    challengingCriteria: criteriaAverages
      .sort((a, b) => a.averageScore - b.averageScore)
      .slice(0, 3),
  };

  // Generate recommendations
  const recommendations = generateRecommendations(results, {
    performanceMetrics,
    agentMetrics,
    qualityMetrics,
  });

  return {
    performanceMetrics,
    agentMetrics,
    qualityMetrics,
    recommendations,
  };
}

/**
 * Generate actionable recommendations based on test results
 */
function generateRecommendations(
  results: E2ETestResult[],
  metrics: any
): string[] {
  const recommendations: string[] = [];

  // Performance recommendations
  if (metrics.performanceMetrics.averageTestDuration > 30000) {
    recommendations.push(
      "⚡ Consider optimizing agent response times - average test duration exceeds 30 seconds"
    );
  }

  // Agent interaction recommendations
  if (metrics.agentMetrics.averageInteractionsPerTest < 2) {
    recommendations.push(
      "🤖 Agent interactions are minimal - consider adding more tool calls or evaluation steps"
    );
  }

  // Quality recommendations
  const failedTests = results.filter((r) => !r.success);
  if (failedTests.length > 0) {
    recommendations.push(
      `❌ ${failedTests.length} tests failed - review agent outputs and evaluation criteria`
    );
  }

  const lowScoringCriteria = metrics.qualityMetrics.challengingCriteria.filter(
    (c) => c.averageScore < 0.7
  );
  if (lowScoringCriteria.length > 0) {
    recommendations.push(
      `📈 Improve criteria: ${lowScoringCriteria.map((c) => c.name).join(", ")}`
    );
  }

  // Success recommendations
  if (results.every((r) => r.success)) {
    recommendations.push(
      "🎉 All tests passed! Consider adding more challenging scenarios"
    );
  }

  if (metrics.summary.averageScore > 0.9) {
    recommendations.push(
      "🏆 Excellent performance! Agent evaluation is highly effective"
    );
  }

  return recommendations;
}

/**
 * Display final test results in a comprehensive format
 */
function displayFinalResults(report: E2ETestReport): void {
  console.log("\n🎊 COMPREHENSIVE E2E TEST RESULTS");
  console.log("==================================");

  console.log(`\n📊 Overall Summary:`);
  console.log(
    `   ✅ Passed: ${report.summary.passedTests}/${report.summary.totalTests}`
  );
  console.log(`   📈 Success Rate: ${report.summary.successRate.toFixed(1)}%`);
  console.log(
    `   🎯 Average Score: ${(report.summary.averageScore * 100).toFixed(1)}%`
  );
  console.log(`   ⏱️ Total Duration: ${report.summary.totalDuration}ms`);

  console.log(`\n⚡ Performance Metrics:`);
  console.log(
    `   📏 Average Test Duration: ${report.summary.performanceMetrics.averageTestDuration.toFixed(
      0
    )}ms`
  );
  console.log(
    `   🏃 Fastest: ${report.summary.performanceMetrics.fastestTest.name} (${report.summary.performanceMetrics.fastestTest.duration}ms)`
  );
  console.log(
    `   🐌 Slowest: ${report.summary.performanceMetrics.slowestTest.name} (${report.summary.performanceMetrics.slowestTest.duration}ms)`
  );

  console.log(`\n🤖 Agent Metrics:`);
  console.log(
    `   💬 Total Interactions: ${report.summary.agentMetrics.totalInteractions}`
  );
  console.log(
    `   📊 Average per Test: ${report.summary.agentMetrics.averageInteractionsPerTest.toFixed(
      1
    )}`
  );
  console.log(`   🔧 Tool Calls by Type:`);
  Object.entries(report.summary.agentMetrics.toolCallsByType).forEach(
    ([tool, count]) => {
      console.log(`      - ${tool}: ${count}`);
    }
  );

  console.log(`\n🎯 Quality Metrics:`);
  console.log(
    `   📋 Criteria Evaluated: ${report.summary.qualityMetrics.criteriaEvaluated}`
  );
  console.log(
    `   📊 Average per Test: ${report.summary.qualityMetrics.averageCriteriaPerTest.toFixed(
      1
    )}`
  );
  console.log(`   🏆 Top Performing Criteria:`);
  report.summary.qualityMetrics.topPerformingCriteria.forEach((c) => {
    console.log(`      - ${c.name}: ${(c.averageScore * 100).toFixed(1)}%`);
  });
  console.log(`   📉 Challenging Criteria:`);
  report.summary.qualityMetrics.challengingCriteria.forEach((c) => {
    console.log(`      - ${c.name}: ${(c.averageScore * 100).toFixed(1)}%`);
  });

  console.log(`\n💡 Recommendations:`);
  report.recommendations.forEach((rec) => {
    console.log(`   ${rec}`);
  });

  console.log(`\n📈 Individual Test Results:`);
  report.results.forEach((result) => {
    const status = result.success ? "✅" : "❌";
    console.log(
      `   ${status} ${result.scenario.name}: ${(
        result.report.overallScore * 100
      ).toFixed(1)}% (${result.executionTime}ms)`
    );
  });

  console.log(`\n🎉 Test Suite Completed in ${report.duration}ms`);
}

/**
 * Save comprehensive report to file
 */
async function saveReport(report: E2ETestReport): Promise<void> {
  const reportDir = path.join(__dirname, "artifacts");
  if (!fs.existsSync(reportDir)) {
    fs.mkdirSync(reportDir, { recursive: true });
  }

  const reportPath = path.join(
    reportDir,
    `e2e-report-${new Date().toISOString().split("T")[0]}.json`
  );
  const summaryPath = path.join(
    reportDir,
    `e2e-summary-${new Date().toISOString().split("T")[0]}.md`
  );

  // Save detailed JSON report
  fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.log(`📄 Detailed report saved to: ${reportPath}`);

  // Save markdown summary
  const markdownSummary = generateMarkdownSummary(report);
  fs.writeFileSync(summaryPath, markdownSummary);
  console.log(`📝 Summary report saved to: ${summaryPath}`);
}

/**
 * Generate markdown summary for the report
 */
function generateMarkdownSummary(report: E2ETestReport): string {
  let markdown = `# Agent Agency E2E Test Report

**Date:** ${new Date(report.timestamp).toLocaleDateString()}
**Duration:** ${report.duration}ms

## Executive Summary

- **Tests Passed:** ${report.summary.passedTests}/${
    report.summary.totalTests
  } (${report.summary.successRate.toFixed(1)}%)
- **Average Score:** ${(report.summary.averageScore * 100).toFixed(1)}%
- **Total Duration:** ${report.summary.totalDuration}ms

## Performance Metrics

- **Average Test Duration:** ${report.summary.performanceMetrics.averageTestDuration.toFixed(
    0
  )}ms
- **Fastest Test:** ${report.summary.performanceMetrics.fastestTest.name} (${
    report.summary.performanceMetrics.fastestTest.duration
  }ms)
- **Slowest Test:** ${report.summary.performanceMetrics.slowestTest.name} (${
    report.summary.performanceMetrics.slowestTest.duration
  }ms)

## Agent Metrics

- **Total Interactions:** ${report.summary.agentMetrics.totalInteractions}
- **Average per Test:** ${report.summary.agentMetrics.averageInteractionsPerTest.toFixed(
    1
  )}

### Tool Usage
`;

  Object.entries(report.summary.agentMetrics.toolCallsByType).forEach(
    ([tool, count]) => {
      markdown += `- **${tool}:** ${count} calls\n`;
    }
  );

  markdown += `
## Quality Metrics

- **Criteria Evaluated:** ${report.summary.qualityMetrics.criteriaEvaluated}
- **Average per Test:** ${report.summary.qualityMetrics.averageCriteriaPerTest.toFixed(
    1
  )}

### Top Performing Criteria
`;

  report.summary.qualityMetrics.topPerformingCriteria.forEach((c) => {
    markdown += `- **${c.name}:** ${(c.averageScore * 100).toFixed(1)}%\n`;
  });

  markdown += `
### Challenging Criteria
`;

  report.summary.qualityMetrics.challengingCriteria.forEach((c) => {
    markdown += `- **${c.name}:** ${(c.averageScore * 100).toFixed(1)}%\n`;
  });

  markdown += `
## Recommendations
`;

  report.recommendations.forEach((rec) => {
    markdown += `- ${rec}\n`;
  });

  markdown += `
## Individual Test Results
`;

  report.results.forEach((result) => {
    const status = result.success ? "✅" : "❌";
    markdown += `- ${status} **${result.scenario.name}**: ${(
      result.report.overallScore * 100
    ).toFixed(1)}% (${result.executionTime}ms)\n`;
  });

  markdown += `
## Raw Data

See the accompanying JSON file for complete test data, including:
- Detailed evaluation criteria results
- Agent interaction logs
- Performance timing data
- Error messages and stack traces

---
*Generated by Agent Agency E2E Test Runner*
`;

  return markdown;
}

/**
 * Run E2E tests and exit with appropriate code
 */
export async function runAndExit(): Promise<void> {
  try {
    const report = await runAllE2ETests();

    // Exit with success/failure code
    const exitCode = report.summary.failedTests === 0 ? 0 : 1;
    process.exit(exitCode);
  } catch (error) {
    console.error("💥 E2E Test Runner failed:", error);
    process.exit(1);
  }
}

// If run directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runAndExit();
}
