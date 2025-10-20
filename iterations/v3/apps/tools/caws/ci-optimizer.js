#!/usr/bin/env node

/**
 * @fileoverview CAWS CI/CD Pipeline Optimizer
 * Implements risk-driven and change-driven optimizations for faster feedback
 * @author @darianrosebrook
 */

const fs = require('fs');
const path = require('path');
const yaml = require('js-yaml');

// Import workflow modules
const {
  createBaseWorkflow,
  createSetupJob,
  createLintJob,
  createTestJob,
  createSecurityJob,
  createBuildJob,
  createDeployJob,
  createDockerJob,
} = require('./workflow-modules');

/**
 * CI optimization strategies
 */
const OPTIMIZATION_STRATEGIES = {
  TIER_BASED_CONDITIONAL: {
    name: 'Tier-based Conditional Runs',
    description: 'Skip expensive checks for low-risk changes',
    impact: 'high',
  },
  SELECTIVE_TESTING: {
    name: 'Selective Test Execution',
    description: 'Run only relevant tests based on changes',
    impact: 'medium',
  },
  TWO_PHASE_VALIDATION: {
    name: 'Two-Phase Validation',
    description: 'Fast feedback first, comprehensive checks second',
    impact: 'high',
  },
};

/**
 * Generate optimized GitHub Actions workflow
 * @param {Object} options - Workflow generation options
 * @returns {Object} Optimized workflow configuration
 */
function generateOptimizedWorkflow(options = {}) {
  const {
    language = 'javascript',
    tier = 2,
    enableTwoPhase = true,
    enableSelectiveTests = true,
    enableTierConditionals = true,
    enableDocker = true,
    enableDeploy = false,
  } = options;

  // Create base workflow structure
  const workflow = createBaseWorkflow({ name: 'CAWS Optimized CI/CD' });

  // Add setup job
  workflow.jobs.setup = createSetupJob({
    nodeVersion: language === 'javascript' ? '18' : null,
  });

  // Add quality assurance jobs
  if (tier <= 2 || options.enableLint) {
    workflow.jobs.lint = createLintJob({
      enableTierConditionals,
      enableSelectiveLinting: enableSelectiveTests,
    });
  }

  if (tier <= 2 || options.enableTests) {
    workflow.jobs.test = createTestJob({
      language,
      enableTierConditionals,
      enableSelectiveTests,
      tier,
    });
  }

  if (tier === 1 || options.enableSecurity) {
    workflow.jobs.security = createSecurityJob({
      enableTierConditionals,
    });
  }

  // Add build jobs
  if (tier <= 2 || options.enableBuild) {
    workflow.jobs.build = createBuildJob({
      language,
      enableTierConditionals,
    });
  }

  // Add Docker build if enabled
  if (enableDocker && (tier <= 2 || options.forceDocker)) {
    workflow.jobs.docker = createDockerJob({
      enableTierConditionals,
    });
  }

  // Add deployment job if enabled (only for main branch and high-risk changes)
  if (enableDeploy && tier === 1) {
    workflow.jobs.deploy = createDeployJob({
      environment: 'staging',
      enableTierConditionals,
    });
  }

  return workflow;
}

/**
 * Generate workflow YAML from configuration
 * @param {Object} workflow - Workflow configuration
 * @returns {string} YAML string
 */
function generateWorkflowYAML(workflow) {
  try {
    return yaml.dump(workflow, {
      indent: 2,
      lineWidth: -1,
      noRefs: true,
    });
  } catch (error) {
    console.error('Error generating workflow YAML:', error.message);
    throw error;
  }
}

/**
 * Save workflow to file
 * @param {Object} workflow - Workflow configuration
 * @param {string} outputPath - Output file path
 */
function saveWorkflow(workflow, outputPath = '.github/workflows/caws-optimized.yml') {
  const yamlContent = generateWorkflowYAML(workflow);

  // Ensure directory exists
  const dir = path.dirname(outputPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  fs.writeFileSync(outputPath, yamlContent, 'utf8');
  console.log(`✅ Generated optimized workflow: ${outputPath}`);
}

/**
 * Analyze repository and suggest optimizations
 * @param {Object} options - Analysis options
 * @returns {Object} Analysis results and recommendations
 */
function analyzeRepository(options = {}) {
  const results = {
    currentWorkflow: null,
    recommendations: [],
    optimizations: [],
  };

  // Check for existing workflow
  const workflowPath = '.github/workflows/caws-optimized.yml';
  if (fs.existsSync(workflowPath)) {
    try {
      results.currentWorkflow = yaml.load(fs.readFileSync(workflowPath, 'utf8'));
    } catch (error) {
      console.warn('Could not parse existing workflow:', error.message);
    }
  }

  // Analyze repository structure and suggest optimizations
  if (fs.existsSync('package.json')) {
    results.recommendations.push({
      type: 'language',
      language: 'javascript',
      confidence: 'high',
    });
  }

  if (fs.existsSync('Cargo.toml')) {
    results.recommendations.push({
      type: 'language',
      language: 'rust',
      confidence: 'high',
    });
  }

  // Check for CAWS configuration
  if (fs.existsSync('.caws/working-spec.yaml')) {
    results.optimizations.push(OPTIMIZATION_STRATEGIES.TIER_BASED_CONDITIONAL);
    results.optimizations.push(OPTIMIZATION_STRATEGIES.SELECTIVE_TESTING);
  }

  return results;
}

/**
 * CLI interface
 */
function main() {
  const args = process.argv.slice(2);
  const command = args[0] || 'generate';

  switch (command) {
    case 'generate':
      const options = parseCLIOptions(args.slice(1));
      const workflow = generateOptimizedWorkflow(options);

      if (options.output) {
        saveWorkflow(workflow, options.output);
      } else {
        console.log(generateWorkflowYAML(workflow));
      }
      break;

    case 'analyze':
      const analysis = analyzeRepository();
      console.log('📊 Repository Analysis:');
      console.log(JSON.stringify(analysis, null, 2));
      break;

    default:
      console.log('Usage: node ci-optimizer.js [generate|analyze] [options]');
      console.log('');
      console.log('Commands:');
      console.log('  generate  Generate optimized workflow (default)');
      console.log('  analyze   Analyze repository and suggest optimizations');
      console.log('');
      console.log('Options:');
      console.log('  --language <lang>    Programming language (javascript, rust, python)');
      console.log('  --tier <1-3>         Risk tier (1=critical, 2=standard, 3=low)');
      console.log('  --output <file>      Output file path');
      console.log('  --no-selective       Disable selective testing');
      console.log('  --no-tier-checks     Disable tier-based conditionals');
      process.exit(1);
  }
}

/**
 * Parse CLI options
 * @param {Array} args - Command line arguments
 * @returns {Object} Parsed options
 */
function parseCLIOptions(args) {
  const options = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    switch (arg) {
      case '--language':
        options.language = args[++i];
        break;
      case '--tier':
        options.tier = parseInt(args[++i]);
        break;
      case '--output':
        options.output = args[++i];
        break;
      case '--no-selective':
        options.enableSelectiveTests = false;
        break;
      case '--no-tier-checks':
        options.enableTierConditionals = false;
        break;
      case '--enable-docker':
        options.enableDocker = true;
        break;
      case '--enable-deploy':
        options.enableDeploy = true;
        break;
    }
  }

  return options;
}

// Export functions for testing
module.exports = {
  generateOptimizedWorkflow,
  generateWorkflowYAML,
  saveWorkflow,
  analyzeRepository,
  OPTIMIZATION_STRATEGIES,
};

// Run CLI if called directly
if (require.main === module) {
  main();
}