#!/usr/bin/env node

/**
 * @fileoverview Workflow Base Module
 * Provides base workflow structure and configuration for CAWS CI/CD
 * @author @darianrosebrook
 */

/**
 * Create base workflow structure
 * @param {Object} options - Workflow options
 * @returns {Object} Base workflow object
 */
function createBaseWorkflow(options = {}) {
  const { name = 'CAWS Optimized CI/CD' } = options;

  return {
    name,
    on: {
      push: { branches: ['main', 'develop'] },
      pull_request: { branches: ['main', 'develop'] },
    },
    jobs: {},
  };
}

/**
 * Get workflow triggers based on options
 * @param {Object} options - Workflow options
 * @returns {Object} Trigger configuration
 */
function getWorkflowTriggers(options = {}) {
  const { branches = ['main', 'develop'] } = options;

  return {
    push: { branches },
    pull_request: { branches },
  };
}

/**
 * Create setup job configuration
 * @param {Object} options - Job options
 * @returns {Object} Setup job configuration
 */
function createSetupJob(options = {}) {
  const {
    runner = 'ubuntu-latest',
    nodeVersion = '18',
    enableRiskDetection = true,
  } = options;

  const job = {
    'runs-on': runner,
    outputs: {
      risk_tier: '${{ steps.detect.outputs.tier }}',
      changed_files: '${{ steps.detect.outputs.files }}',
      is_experimental: '${{ steps.detect.outputs.experimental }}',
    },
    steps: [
      {
        name: 'Checkout code',
        uses: 'actions/checkout@v4',
        with: { 'fetch-depth': 2 },
      },
    ],
  };

  if (enableRiskDetection) {
    job.steps.push({
      id: 'detect',
      name: 'Detect CAWS configuration',
      run: `
        # Detect risk tier from working spec
        if [ -f .caws/working-spec.yaml ]; then
          TIER=$(grep 'risk_tier:' .caws/working-spec.yaml | cut -d':' -f2 | tr -d ' ')
          echo "tier=$TIER" >> $GITHUB_OUTPUT

          # Check for experimental mode
          if grep -q 'experimental_mode:' .caws/working-spec.yaml; then
            echo "experimental=true" >> $GITHUB_OUTPUT
          else
            echo "experimental=false" >> $GITHUB_OUTPUT
          fi
        else
          echo "tier=2" >> $GITHUB_OUTPUT
          echo "experimental=false" >> $GITHUB_OUTPUT
        fi

        # Detect changed files
        if [ "$GITHUB_EVENT_NAME" = "pull_request" ]; then
          CHANGED_FILES=$(git diff --name-only HEAD~1)
        else
          CHANGED_FILES=$(git diff --name-only HEAD~1)
        fi
        echo "files=$CHANGED_FILES" >> $GITHUB_OUTPUT
      `,
    });
  }

  // Setup Node.js if needed
  if (nodeVersion) {
    job.steps.push({
      name: 'Setup Node.js',
      uses: 'actions/setup-node@v4',
      with: {
        'node-version': nodeVersion,
        cache: 'npm',
      },
    });
  }

  return job;
}

module.exports = {
  createBaseWorkflow,
  getWorkflowTriggers,
  createSetupJob,
};
