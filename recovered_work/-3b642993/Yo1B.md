# CAWS Dashboard Modules

This directory contains the refactored CAWS dashboard components, split into focused, maintainable modules.

## Architecture

The original monolithic `dashboard.js` (1523 lines) has been refactored into specialized modules:

- **`dashboard.js`** (155 lines) - Main CLI interface and orchestration
- **`modules/`** - Specialized functionality modules

## Modules

### `coverage-analysis.js`
- **Purpose**: Coverage data analysis and integration
- **Functions**:
  - `getRealCoverage()` - Parse coverage reports and return metrics

### `mutation-analysis.js`
- **Purpose**: Mutation testing data analysis
- **Functions**:
  - `getRealMutationScore()` - Parse mutation reports and calculate scores

### `test-analysis.js`
- **Purpose**: Test result parsing and analysis
- **Functions**:
  - `parseTestResults()` - Parse various test result formats
  - `parseJUnitXML()` - Parse JUnit XML test results
  - `parseCargoTestOutput()` - Parse Rust cargo test output
  - `analyzeTestExecutionHistory()` - Analyze test execution patterns

### `compliance-checker.js`
- **Purpose**: Compliance validation for various standards
- **Functions**:
  - `checkContractCompliance()` - API contract validation
  - `checkAccessibilityCompliance()` - Accessibility standards
  - `checkPerformanceCompliance()` - Performance benchmarks

### `data-generator.js`
- **Purpose**: Data generation and simulation utilities
- **Functions**:
  - `generateRealProvenanceData()` - Generate provenance data
  - `simulateTestHistoryFromGit()` - Simulate test history
  - `countRustFiles()` - Count Rust source files
  - `getCurrentCommitHash()` - Get git commit hash
  - `getCurrentBranch()` - Get current git branch

### `index.js`
- **Purpose**: Central export point for all modules
- **Usage**: `const caws = require('./modules')`

## Benefits of Refactoring

1. **Maintainability**: Each module has a single responsibility
2. **Testability**: Individual modules can be unit tested separately
3. **Reusability**: Functions can be imported and used independently
4. **Readability**: Smaller files are easier to understand and navigate
5. **Collaboration**: Multiple developers can work on different modules simultaneously

## Usage

```javascript
// Import specific modules
const { getRealCoverage } = require('./modules/coverage-analysis');
const { parseTestResults } = require('./modules/test-analysis');

// Or import everything
const caws = require('./modules');
const coverage = caws.getRealCoverage();
const testResults = caws.parseTestResults('./test-results');
```

## File Size Reduction

- **Before**: `dashboard.js` - 1523 lines (monolithic)
- **After**:
  - `dashboard.js` - 155 lines (orchestration only)
  - Total modules - 533 lines (distributed functionality)

This represents a **90% reduction** in the main file size while maintaining all functionality.
