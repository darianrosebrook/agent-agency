#!/usr/bin/env node

/**
 * @fileoverview CAWS Dashboard and Analytics Tool
 * Provides comprehensive visualization and analytics for CAWS trust metrics
 * @author @darianrosebrook
 */

// Import modular components
const { getRealCoverage } = require('./modules/coverage-analysis');
const { getRealMutationScore } = require('./modules/mutation-analysis');
const { parseTestResults, analyzeTestExecutionHistory } = require('./modules/test-analysis');
const { checkContractCompliance, checkAccessibilityCompliance, checkPerformanceCompliance } = require('./modules/compliance-checker');
const { generateRealProvenanceData, simulateTestHistoryFromGit, countRustFiles } = require('./modules/data-generator');

const fs = require("fs");
const path = require("path");

/**
 * Main CAWS Dashboard function
 * @param {Object} options - Dashboard options
 */
function runCAWSDashboard(options = {}) {
  console.log("🚀 CAWS Dashboard - Trust & Compliance Analytics\n");

  // Generate comprehensive provenance data
  const provenanceData = generateRealProvenanceData();

  // Display current metrics
  displayMetrics(provenanceData.results);

  // Analyze test history if available
  analyzeAndDisplayTestHistory();

  // Display compliance status
  displayComplianceStatus(provenanceData.results);

  // Generate final trust score
  const trustScore = calculateTrustScore(provenanceData.results);
  console.log(`🏆 Final Trust Score: ${(trustScore * 100).toFixed(1)}%\n`);

  if (options.saveReport) {
    saveReport(provenanceData);
  }
}

/**
 * Display current metrics
 * @param {Object} results - Analysis results
 */
function displayMetrics(results) {
  console.log("📊 Current Metrics:");
  console.log(`  • Coverage: ${(results.coverage_branch * 100).toFixed(1)}%`);
  console.log(`  • Mutation Score: ${(results.mutation_score * 100).toFixed(1)}%`);
  console.log(`  • Contract Compliance: ${results.contract_compliance ? '✅' : '❌'}`);
  console.log(`  • Accessibility: ${results.accessibility_compliance ? '✅' : '❌'}`);
  console.log(`  • Performance: ${results.performance_compliance ? '✅' : '❌'}`);
  console.log(`  • Security Scan: ${results.security_scan_passed ? '✅' : '❌'}`);
  console.log(`  • Dependency Audit: ${results.dependency_audit_passed ? '✅' : '❌'}\n`);
}

/**
 * Analyze and display test execution history
 */
function analyzeAndDisplayTestHistory() {
  console.log("🧪 Test Execution Analysis:");

  try {
    // Try to parse real test results
    const testResults = parseTestResults(path.join(process.cwd(), 'test-results'));
    if (testResults.length > 0) {
      const analysis = analyzeTestExecutionHistory(testResults);
      console.log(`  • Total Test Runs: ${analysis.total_runs}`);
      console.log(`  • Average Pass Rate: ${(analysis.average_pass_rate * 100).toFixed(1)}%`);
      console.log(`  • Recent Trends: ${analysis.failure_trends.length} data points\n`);
    } else {
      // Fall back to simulated data
      const simulatedHistory = simulateTestHistoryFromGit();
      const analysis = analyzeTestExecutionHistory(simulatedHistory);
      console.log(`  • Simulated Test Runs: ${analysis.total_runs}`);
      console.log(`  • Average Pass Rate: ${(analysis.average_pass_rate * 100).toFixed(1)}%`);
      console.log("  • Using simulated data (no real test results found)\n");
    }
  } catch (error) {
    console.log("  • Test analysis unavailable\n");
  }
}

/**
 * Display compliance status
 * @param {Object} results - Compliance results
 */
function displayComplianceStatus(results) {
  console.log("📋 Compliance Status:");
  console.log(`  • Contracts: ${results.contract_compliance ? '✅ Compliant' : '❌ Non-compliant'}`);
  console.log(`  • Accessibility: ${results.accessibility_compliance ? '✅ Compliant' : '❌ Non-compliant'}`);
  console.log(`  • Performance: ${results.performance_compliance ? '✅ Compliant' : '❌ Non-compliant'}`);
  console.log("");
}

/**
 * Calculate overall trust score
 * @param {Object} results - Analysis results
 * @returns {number} Trust score (0-1)
 */
function calculateTrustScore(results) {
  const weights = {
    coverage: 0.20,
    mutation: 0.15,
    contracts: 0.15,
    accessibility: 0.10,
    performance: 0.15,
    security: 0.15,
    dependencies: 0.10
  };

  let score = 0;
  score += results.coverage_branch * weights.coverage;
  score += results.mutation_score * weights.mutation;
  score += (results.contract_compliance ? 1 : 0) * weights.contracts;
  score += (results.accessibility_compliance ? 1 : 0) * weights.accessibility;
  score += (results.performance_compliance ? 1 : 0) * weights.performance;
  score += (results.security_scan_passed ? 1 : 0) * weights.security;
  score += (results.dependency_audit_passed ? 1 : 0) * weights.dependencies;

  return Math.min(score, 1.0);
}

/**
 * Save analysis report to file
 * @param {Object} provenanceData - Complete provenance data
 */
function saveReport(provenanceData) {
  const reportPath = path.join(process.cwd(), 'caws-report.json');
  try {
    fs.writeFileSync(reportPath, JSON.stringify(provenanceData, null, 2));
    console.log(`💾 Report saved to: ${reportPath}`);
  } catch (error) {
    console.error(`❌ Failed to save report: ${error.message}`);
  }
}

// CLI Interface
if (require.main === module) {
  const args = process.argv.slice(2);
  const options = {
    saveReport: args.includes('--save') || args.includes('-s')
  };

  runCAWSDashboard(options);
}

module.exports = {
  runCAWSDashboard,
  calculateTrustScore
};