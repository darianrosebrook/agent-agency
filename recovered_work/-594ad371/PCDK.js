#!/usr/bin/env node

/**
 * @fileoverview CAWS Modules Index
 * Central export point for all CAWS dashboard modules
 * @author @darianrosebrook
 */

// Coverage Analysis
const { getRealCoverage } = require('./coverage-analysis');

// Mutation Analysis
const { getRealMutationScore } = require('./mutation-analysis');

// Test Analysis
const {
  parseTestResults,
  parseJUnitXML,
  parseCargoTestOutput,
  analyzeTestExecutionHistory
} = require('./test-analysis');

// Compliance Checking
const {
  checkContractCompliance,
  checkAccessibilityCompliance,
  checkPerformanceCompliance
} = require('./compliance-checker');

// Data Generation
const {
  generateRealProvenanceData,
  simulateTestHistoryFromGit,
  countRustFiles,
  getCurrentCommitHash,
  getCurrentBranch
} = require('./data-generator');

module.exports = {
  // Coverage
  getRealCoverage,

  // Mutation
  getRealMutationScore,

  // Test Analysis
  parseTestResults,
  parseJUnitXML,
  parseCargoTestOutput,
  analyzeTestExecutionHistory,

  // Compliance
  checkContractCompliance,
  checkAccessibilityCompliance,
  checkPerformanceCompliance,

  // Data Generation
  generateRealProvenanceData,
  simulateTestHistoryFromGit,
  countRustFiles,
  getCurrentCommitHash,
  getCurrentBranch
};
