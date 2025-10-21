#!/usr/bin/env node

/**
 * @fileoverview Quality Jobs Module
 * Provides lint and test job configurations for CAWS CI/CD workflows
 * @author @darianrosebrook
 */

/**
 * Create lint job configuration
 * @param {Object} options - Job options
 * @returns {Object} Lint job configuration
 */
function createLintJob(options = {}) {
  const {
    runner = "ubuntu-latest",
    enableTierConditionals = true,
    enableSelectiveLinting = true,
  } = options;

  const job = {
    "runs-on": runner,
    needs: "setup",
  };

  // Add conditional execution for tier-based optimization
  if (enableTierConditionals) {
    job.if = `needs.setup.outputs.risk_tier != '1' || needs.setup.outputs.is_experimental == 'true'`;
  }

  job.steps = [
    {
      name: "Checkout code",
      uses: "actions/checkout@v4",
    },
    {
      name: "Setup Node.js",
      uses: "actions/setup-node@v4",
      with: { "node-version": "18", cache: "npm" },
    },
    {
      name: "Install dependencies",
      run: "npm ci",
    },
    {
      name: "Run linting",
      run: enableSelectiveLinting
        ? `
          # Selective linting based on changed files
          if [ -n "\${{ needs.setup.outputs.changed_files }}" ]; then
            echo "Running selective linting..."
            # Run linting only on changed files
            npm run lint:changed
          else
            npm run lint
          fi
        `
        : "npm run lint",
    },
  ];

  return job;
}

/**
 * Create test job configuration
 * @param {Object} options - Job options
 * @returns {Object} Test job configuration
 */
function createTestJob(options = {}) {
  const {
    runner = "ubuntu-latest",
    language = "javascript",
    enableTierConditionals = true,
    enableSelectiveTests = true,
  } = options;

  const job = {
    "runs-on": runner,
    needs: "setup",
    services: {},
  };

  // Add conditional execution
  if (enableTierConditionals) {
    job.if = `needs.setup.outputs.risk_tier != '1' || needs.setup.outputs.is_experimental == 'true'`;
  }

  // Add database services if needed
  if (language === "javascript" || language === "typescript") {
    job.services = {
      postgres: {
        image: "postgres:15",
        env: {
          POSTGRES_PASSWORD: "postgres",
          POSTGRES_DB: "test_db",
        },
        options:
          "--health-cmd pg_isready --health-interval 10s --health-timeout 5s --health-retries 5",
      },
      redis: {
        image: "redis:7-alpine",
        options:
          '--health-cmd "redis-cli ping" --health-interval 10s --health-timeout 5s --health-retries 5',
      },
    };
  }

  job.steps = [
    {
      name: "Checkout code",
      uses: "actions/checkout@v4",
    },
    {
      name: "Setup Node.js",
      uses: "actions/setup-node@v4",
      with: { "node-version": "18", cache: "npm" },
    },
    {
      name: "Install dependencies",
      run: "npm ci",
    },
    {
      name: "Wait for services",
      run: "sleep 10",
    },
    {
      name: "Run tests",
      run: getTestCommand(language, options),
      env: {
        DATABASE_URL: "postgresql://postgres:postgres@localhost:5432/test_db",
        REDIS_URL: "redis://localhost:6379",
      },
    },
  ];

  return job;
}

/**
 * Get test command based on language and options
 * @param {string} language - Programming language
 * @param {Object} options - Test options
 * @returns {string} Test command
 */
function getTestCommand(language, options = {}) {
  const { tier = 2, enableSelectiveTests = true } = options;

  // Tier-based test selection
  if (enableSelectiveTests) {
    switch (tier) {
      case 1: // Critical tier - full test suite
        return "npm run test:full";
      case 2: // Standard tier - comprehensive but optimized
        return "npm run test:ci";
      case 3: // Low risk - minimal tests
        return "npm run test:smoke";
      default:
        return "npm test";
    }
  }

  return "npm test";
}

/**
 * Create security scan job configuration
 * @param {Object} options - Job options
 * @returns {Object} Security scan job configuration
 */
function createSecurityJob(options = {}) {
  const { runner = "ubuntu-latest", enableTierConditionals = true } = options;

  const job = {
    "runs-on": runner,
    needs: "setup",
  };

  // Security scans always run for Tier 1 and experimental
  if (enableTierConditionals) {
    job.if = `needs.setup.outputs.risk_tier == '1' || needs.setup.outputs.is_experimental == 'true'`;
  }

  job.steps = [
    {
      name: "Checkout code",
      uses: "actions/checkout@v4",
    },
    {
      name: "Run security scan",
      uses: "github/super-linter/slim@v5",
      env: {
        DEFAULT_BRANCH: "main",
        GITHUB_TOKEN: "${{ secrets.GITHUB_TOKEN }}",
        VALIDATE_ALL_CODEBASE: false,
        VALIDATE_JAVASCRIPT_ES: true,
        VALIDATE_TYPESCRIPT_ES: true,
        VALIDATE_RUST_CLIPPY: true,
      },
    },
  ];

  return job;
}

module.exports = {
  createLintJob,
  createTestJob,
  createSecurityJob,
  getTestCommand,
};
