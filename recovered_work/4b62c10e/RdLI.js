#!/usr/bin/env node

/**
 * @fileoverview Build Jobs Module
 * Provides build and deployment job configurations for CAWS CI/CD workflows
 * @author @darianrosebrook
 */

/**
 * Create build job configuration
 * @param {Object} options - Job options
 * @returns {Object} Build job configuration
 */
function createBuildJob(options = {}) {
  const {
    runner = 'ubuntu-latest',
    language = 'javascript',
    enableTierConditionals = true,
  } = options;

  const job = {
    'runs-on': runner,
    needs: ['setup', 'lint'],
  };

  // Build runs for all tiers except when tier 3 and no critical changes
  if (enableTierConditionals) {
    job.if = `needs.setup.outputs.risk_tier != '3' || needs.setup.outputs.is_experimental == 'true'`;
  }

  job.steps = [
    {
      name: 'Checkout code',
      uses: 'actions/checkout@v4',
    },
    {
      name: 'Setup Node.js',
      uses: 'actions/setup-node@v4',
      with: { 'node-version': '18', cache: 'npm' },
    },
    {
      name: 'Install dependencies',
      run: 'npm ci',
    },
    {
      name: 'Build application',
      run: getBuildCommand(language, options),
    },
    {
      name: 'Upload build artifacts',
      uses: 'actions/upload-artifact@v4',
      with: {
        name: 'build-artifacts',
        path: getBuildArtifactsPath(language),
      },
    },
  ];

  return job;
}

/**
 * Create deployment job configuration
 * @param {Object} options - Job options
 * @returns {Object} Deployment job configuration
 */
function createDeployJob(options = {}) {
  const {
    runner = 'ubuntu-latest',
    environment = 'staging',
    enableTierConditionals = true,
  } = options;

  const job = {
    'runs-on': runner,
    needs: ['setup', 'lint', 'test', 'build'],
    environment: environment,
  };

  // Deployment restrictions - only for main branch and approved tiers
  job.if = `github.ref == 'refs/heads/main' && (needs.setup.outputs.risk_tier == '1' || needs.setup.outputs.is_experimental == 'true')`;

  job.steps = [
    {
      name: 'Checkout code',
      uses: 'actions/checkout@v4',
    },
    {
      name: 'Download build artifacts',
      uses: 'actions/download-artifact@v4',
      with: {
        name: 'build-artifacts',
        path: './dist',
      },
    },
    {
      name: 'Deploy to staging',
      run: `
        echo "Deploying to ${environment} environment..."
        # Deployment logic would go here
        # This could include Docker builds, Kubernetes deployments, etc.
        echo "Deployment completed successfully"
      `,
      env: {
        DEPLOY_ENV: environment,
        API_KEY: '${{ secrets.DEPLOY_API_KEY }}',
      },
    },
    {
      name: 'Run smoke tests',
      run: 'npm run test:smoke',
      env: {
        TEST_ENV: environment,
      },
    },
  ];

  return job;
}

/**
 * Create Docker build job configuration
 * @param {Object} options - Job options
 * @returns {Object} Docker build job configuration
 */
function createDockerJob(options = {}) {
  const { runner = 'ubuntu-latest', enableTierConditionals = true } = options;

  const job = {
    'runs-on': runner,
    needs: ['setup', 'lint'],
  };

  // Docker builds for higher tiers
  if (enableTierConditionals) {
    job.if = `needs.setup.outputs.risk_tier <= '2' || needs.setup.outputs.is_experimental == 'true'`;
  }

  job.steps = [
    {
      name: 'Checkout code',
      uses: 'actions/checkout@v4',
    },
    {
      name: 'Set up Docker Buildx',
      uses: 'docker/setup-buildx-action@v3',
    },
    {
      name: 'Log in to container registry',
      uses: 'docker/login-action@v3',
      with: {
        registry: '${{ secrets.CONTAINER_REGISTRY }}',
        username: '${{ secrets.CONTAINER_USERNAME }}',
        password: '${{ secrets.CONTAINER_PASSWORD }}',
      },
    },
    {
      name: 'Build and push Docker image',
      uses: 'docker/build-push-action@v5',
      with: {
        context: '.',
        push: true,
        tags: [
          '${{ secrets.CONTAINER_REGISTRY }}/app:${{ github.sha }}',
          '${{ secrets.CONTAINER_REGISTRY }}/app:latest',
        ].join('\n'),
        cache_from: 'type=gha',
        cache_to: 'type=gha,mode=max',
      },
    },
  ];

  return job;
}

/**
 * Get build command based on language
 * @param {string} language - Programming language
 * @param {Object} options - Build options
 * @returns {string} Build command
 */
function getBuildCommand(language, options = {}) {
  const { optimize = true } = options;

  switch (language) {
    case 'javascript':
    case 'typescript':
      return optimize ? 'npm run build:production' : 'npm run build';
    case 'rust':
      return optimize ? 'cargo build --release' : 'cargo build';
    case 'python':
      return optimize ? 'python -m build --wheel' : 'python setup.py build';
    default:
      return 'npm run build';
  }
}

/**
 * Get build artifacts path based on language
 * @param {string} language - Programming language
 * @returns {string} Artifacts path
 */
function getBuildArtifactsPath(language) {
  switch (language) {
    case 'javascript':
    case 'typescript':
      return 'dist/';
    case 'rust':
      return 'target/release/';
    case 'python':
      return 'dist/';
    default:
      return 'dist/';
  }
}

module.exports = {
  createBuildJob,
  createDeployJob,
  createDockerJob,
  getBuildCommand,
  getBuildArtifactsPath,
};
