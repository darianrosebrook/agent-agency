# CAWS Workflow Modules

This directory contains the refactored CAWS CI/CD workflow generation components, split into focused, maintainable modules.

## Architecture

The original monolithic `generateOptimizedWorkflow` function (311 lines) has been refactored into specialized modules:

- **`ci-optimizer.js`** (294 lines) - Main CLI interface and workflow orchestration
- **`workflow-modules/`** - Specialized workflow generation modules

## Modules

### `workflow-base.js`
- **Purpose**: Base workflow structure and setup job configuration
- **Functions**:
  - `createBaseWorkflow()` - Create basic workflow structure with triggers
  - `getWorkflowTriggers()` - Configure push/PR triggers
  - `createSetupJob()` - Setup job with CAWS detection and environment setup

### `quality-jobs.js`
- **Purpose**: Quality assurance job configurations (lint, test, security)
- **Functions**:
  - `createLintJob()` - ESLint/Prettier job with selective linting
  - `createTestJob()` - Test job with database services and tier-based execution
  - `createSecurityJob()` - Security scanning and vulnerability checks
  - `getTestCommand()` - Dynamic test command selection based on risk tier

### `build-jobs.js`
- **Purpose**: Build and deployment job configurations
- **Functions**:
  - `createBuildJob()` - Application build job with artifact upload
  - `createDeployJob()` - Deployment job with environment-specific configuration
  - `createDockerJob()` - Docker build and push job with registry integration
  - `getBuildCommand()` - Language-specific build command selection

### `index.js`
- **Purpose**: Central export point for all workflow modules
- **Usage**: `const { createSetupJob, createLintJob } = require('./workflow-modules')`

## Optimization Strategies

The modules implement several CI/CD optimization strategies:

### Tier-Based Conditional Execution
```javascript
// High-risk changes (Tier 1) run all checks
// Standard changes (Tier 2) skip expensive security scans
// Low-risk changes (Tier 3) run minimal validation
job.if = `needs.setup.outputs.risk_tier != '3'`;
```

### Selective Testing
```javascript
// Run different test suites based on risk assessment
// Tier 1: Full test suite
// Tier 2: CI-optimized tests
// Tier 3: Smoke tests only
const testCommand = getTestCommand(language, { tier, enableSelectiveTests });
```

### Change-Based Execution
```javascript
// Only run linting on changed files
if [ -n "${{ needs.setup.outputs.changed_files }}" ]; then
  npm run lint:changed
else
  npm run lint
fi
```

## Usage

```javascript
const {
  createBaseWorkflow,
  createSetupJob,
  createLintJob,
  createTestJob,
  createBuildJob
} = require('./workflow-modules');

// Generate optimized workflow
const workflow = createBaseWorkflow({ name: 'My CI/CD' });
workflow.jobs.setup = createSetupJob({ nodeVersion: '18' });
workflow.jobs.lint = createLintJob({ enableTierConditionals: true });
workflow.jobs.test = createTestJob({ language: 'javascript', tier: 2 });
workflow.jobs.build = createBuildJob({ language: 'javascript' });

// Convert to YAML
const yaml = require('js-yaml');
console.log(yaml.dump(workflow));
```

## File Size Reduction

- **Before**: `generateOptimizedWorkflow()` - 311 lines (monolithic function)
- **After**:
  - `ci-optimizer.js` - 294 lines (orchestration + CLI)
  - Total modules - 603 lines (distributed functionality)

This represents a **significant improvement** in maintainability while preserving all functionality.

## Benefits of Refactoring

1. **Modularity**: Each aspect of workflow generation is isolated
2. **Testability**: Individual modules can be unit tested separately
3. **Reusability**: Job configurations can be mixed and matched
4. **Maintainability**: Changes to specific job types are localized
5. **Extensibility**: New job types can be added without affecting existing code

## CLI Interface

The refactored `ci-optimizer.js` maintains the same CLI interface:

```bash
# Generate workflow for JavaScript project
node ci-optimizer.js generate --language javascript --tier 2

# Analyze repository and suggest optimizations
node ci-optimizer.js analyze

# Generate workflow with custom options
node ci-optimizer.js generate --language rust --tier 1 --enable-docker
```

## Integration with CAWS

The workflow modules integrate with CAWS risk assessment:

- **Risk Tier Detection**: Automatic tier detection from `.caws/working-spec.yaml`
- **Conditional Execution**: Jobs skip or run based on risk assessment
- **Selective Optimization**: Different optimization levels per risk tier
- **Change Detection**: Only run necessary checks based on what changed

This ensures CI/CD pipelines are both fast and comprehensive, running appropriate checks for each change's risk level.
