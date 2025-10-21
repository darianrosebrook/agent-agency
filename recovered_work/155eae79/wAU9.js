// Test the Performance component checking functionality
// This mimics the calculatePerformanceScore function from gate-checker.ts

function calculatePerformanceScore(perfResults, workingSpec) {
  try {
    // If no performance results available, return minimum score
    if (!perfResults) {
      console.log("No performance results found");
      return 0.0;
    }

    // Get performance budgets from working spec
    const budgets = workingSpec?.non_functional?.perf || {};
    const apiBudget = budgets.api_p95_ms;
    const lcpBudget = budgets.lcp_ms;

    // If no budgets defined, give neutral score
    if (!apiBudget && !lcpBudget) {
      console.log("No performance budgets defined");
      return 0.5;
    }

    let score = 1.0; // Start with full score
    let violations = 0;

    // Check API performance budget
    if (apiBudget && perfResults.api_p95_ms !== undefined) {
      const actualApiTime = perfResults.api_p95_ms;
      if (actualApiTime > apiBudget) {
        const overrun = (actualApiTime - apiBudget) / apiBudget;
        score -= Math.min(0.5, overrun * 0.3); // Penalty based on overrun percentage
        violations++;
        console.log(
          `API p95 budget violation: ${actualApiTime}ms > ${apiBudget}ms (${(
            overrun * 100
          ).toFixed(1)}% over)`
        );
      } else {
        console.log(`API p95 budget met: ${actualApiTime}ms ≤ ${apiBudget}ms`);
      }
    }

    // Check LCP performance budget
    if (lcpBudget && perfResults.lcp_ms !== undefined) {
      const actualLcpTime = perfResults.lcp_ms;
      if (actualLcpTime > lcpBudget) {
        const overrun = (actualLcpTime - lcpBudget) / lcpBudget;
        score -= Math.min(0.5, overrun * 0.3); // Penalty based on overrun percentage
        violations++;
        console.log(
          `LCP budget violation: ${actualLcpTime}ms > ${lcpBudget}ms (${(
            overrun * 100
          ).toFixed(1)}% over)`
        );
      } else {
        console.log(`LCP budget met: ${actualLcpTime}ms ≤ ${lcpBudget}ms`);
      }
    }

    // Check for performance regressions vs baseline (if available)
    if (perfResults.baseline_comparison) {
      const regression = perfResults.baseline_comparison;
      if (regression.api_p95_regression > 0.05) {
        // 5% regression threshold
        score -= 0.1;
        violations++;
        console.log(
          `API performance regression: ${(
            regression.api_p95_regression * 100
          ).toFixed(1)}% slower than baseline`
        );
      }
      if (regression.lcp_regression > 0.05) {
        // 5% regression threshold
        score -= 0.1;
        violations++;
        console.log(
          `LCP performance regression: ${(
            regression.lcp_regression * 100
          ).toFixed(1)}% slower than baseline`
        );
      }
    }

    // Additional metrics that could affect score
    if (perfResults.error_rate !== undefined && perfResults.error_rate > 0.01) {
      // >1% errors
      score -= 0.2;
      violations++;
      console.log(
        `High error rate: ${(perfResults.error_rate * 100).toFixed(2)}%`
      );
    }

    // Ensure score doesn't go below 0
    score = Math.max(0.0, score);

    // Log final performance score
    console.log(
      `Performance score: ${(score * 100).toFixed(
        1
      )}% (${violations} violations)`
    );
    if (apiBudget) {
      console.log(
        `API budget: ${apiBudget}ms (actual: ${
          perfResults.api_p95_ms || "N/A"
        }ms)`
      );
    }
    if (lcpBudget) {
      console.log(
        `LCP budget: ${lcpBudget}ms (actual: ${perfResults.lcp_ms || "N/A"}ms)`
      );
    }

    return score;
  } catch (error) {
    console.error("Failed to calculate performance score:", error.message);
    return 0.0;
  }
}

// Test cases
console.log("Testing Performance component checking...\n");

// Test 1: All budgets met
console.log("Test 1: All budgets met");
const mockPerfResults1 = {
  api_p95_ms: 200,
  lcp_ms: 1500,
};
const mockWorkingSpec1 = {
  non_functional: {
    perf: {
      api_p95_ms: 250,
      lcp_ms: 2000,
    },
  },
};
const score1 = calculatePerformanceScore(mockPerfResults1, mockWorkingSpec1);
console.log("Result:", score1);
console.log();

// Test 2: Budget violations and regression
console.log("Test 2: Budget violations and regression");
const mockPerfResults2 = {
  api_p95_ms: 350, // 40% over budget (350 > 250)
  lcp_ms: 1800, // Within budget
  error_rate: 0.015, // 1.5% error rate (slightly over 1%)
  baseline_comparison: {
    api_p95_regression: 0.08, // 8% slower than baseline
    lcp_regression: 0.02, // 2% slower (below threshold)
  },
};
const score2 = calculatePerformanceScore(mockPerfResults2, mockWorkingSpec1);
console.log("Result:", score2);
console.log();

// Test 3: No budgets defined
console.log("Test 3: No budgets defined");
const score3 = calculatePerformanceScore(mockPerfResults1, {});
console.log("Result:", score3);
console.log();

// Test 4: No performance results
console.log("Test 4: No performance results");
const score4 = calculatePerformanceScore(null, mockWorkingSpec1);
console.log("Result:", score4);
