/**
 * @fileoverview Spec Resolution System
 * Resolves spec files with priority: feature-specific > working-spec.yaml
 * Enables multi-agent workflows where each agent works on their own spec
 * @author @darianrosebrook
 */

const fs = require('fs-extra');
const path = require('path');
const chalk = require('chalk');

/**
 * Spec resolution priority:
 * 1. .caws/specs/<spec-id>.yaml (feature-specific, multi-agent safe)
 * 2. .caws/working-spec.yaml (legacy, single-agent only)
 */
const SPECS_DIR = '.caws/specs';
const LEGACY_SPEC = '.caws/working-spec.yaml';
const SPECS_REGISTRY = '.caws/specs/registry.json';

/**
 * Resolve spec file path based on priority
 * @param {Object} options - Resolution options
 * @param {string} [options.specId] - Feature-specific spec ID (e.g., 'user-auth', 'FEAT-001')
 * @param {string} [options.specFile] - Explicit file path override
 * @param {boolean} [options.warnLegacy=true] - Warn when falling back to legacy spec
 * @returns {Promise<{path: string, type: 'feature' | 'legacy', spec: Object}>}
 */
async function resolveSpec(options = {}) {
  const { specId, specFile, warnLegacy = true } = options;

  // 1. Explicit file path takes highest priority
  if (specFile) {
    const explicitPath = path.isAbsolute(specFile) ? specFile : path.join(process.cwd(), specFile);

    if (await fs.pathExists(explicitPath)) {
      const yaml = require('js-yaml');
      const content = await fs.readFile(explicitPath, 'utf8');
      const spec = yaml.load(content);

      return {
        path: explicitPath,
        type: explicitPath.includes('/specs/') ? 'feature' : 'legacy',
        spec,
      };
    }

    throw new Error(`Spec file not found: ${explicitPath}`);
  }

  // 2. Feature-specific spec (preferred for multi-agent)
  if (specId) {
    const featurePath = path.join(process.cwd(), SPECS_DIR, `${specId}.yaml`);

    if (await fs.pathExists(featurePath)) {
      const yaml = require('js-yaml');
      const content = await fs.readFile(featurePath, 'utf8');
      const spec = yaml.load(content);

      console.log(chalk.green(`✅ Using feature-specific spec: ${specId}`));

      return {
        path: featurePath,
        type: 'feature',
        spec,
      };
    }

    throw new Error(
      `Feature spec '${specId}' not found. Create it with: caws specs create ${specId}`
    );
  }

  // 3. Auto-detect from registry or list specs
  const registry = await loadSpecsRegistry();
  const specIds = Object.keys(registry.specs ?? {});

  if (specIds.length === 1) {
    // Single spec - use it automatically
    const singleSpecId = specIds[0];
    const singleSpecPath = path.join(process.cwd(), SPECS_DIR, registry.specs[singleSpecId].path);

    if (await fs.pathExists(singleSpecPath)) {
      const yaml = require('js-yaml');
      const content = await fs.readFile(singleSpecPath, 'utf8');
      const spec = yaml.load(content);

      console.log(chalk.blue(`📋 Auto-detected single spec: ${singleSpecId}`));

      return {
        path: singleSpecPath,
        type: 'feature',
        spec,
      };
    }
  } else if (specIds.length > 1) {
    // Multiple specs - require explicit selection
    console.error(chalk.red('❌ Multiple specs detected. Please specify which one:'));
    console.log(chalk.yellow('\n   Available specs:'));
    specIds.forEach((id) => {
      console.log(chalk.yellow(`   - ${id}`));
    });
    console.log(chalk.blue('\n   Usage: caws <command> --spec-id <spec-id>'));
    console.log(chalk.gray(`   Example: caws validate --spec-id ${specIds[0]}\n`));

    throw new Error('Spec ID required when multiple specs exist');
  }

  // 4. Fall back to legacy working-spec.yaml (with warning)
  const legacyPath = path.join(process.cwd(), LEGACY_SPEC);

  if (await fs.pathExists(legacyPath)) {
    const yaml = require('js-yaml');
    const content = await fs.readFile(legacyPath, 'utf8');
    const spec = yaml.load(content);

    if (warnLegacy) {
      console.log(chalk.yellow('⚠️  Using legacy working-spec.yaml'));
      console.log(chalk.gray('   For multi-agent workflows, use feature-specific specs:'));
      console.log(chalk.blue('   caws specs create <feature-id>'));
      console.log('');
    }

    return {
      path: legacyPath,
      type: 'legacy',
      spec,
    };
  }

  // 5. No specs found
  throw new Error(
    'No CAWS spec found. Initialize with: caws init or create a feature spec: caws specs create <id>'
  );
}

/**
 * Load specs registry
 * @returns {Promise<Object>} Registry data
 */
async function loadSpecsRegistry() {
  const registryPath = path.join(process.cwd(), SPECS_REGISTRY);

  if (!(await fs.pathExists(registryPath))) {
    return {
      version: '1.0.0',
      specs: {},
      lastUpdated: new Date().toISOString(),
    };
  }

  try {
    const registry = await fs.readJson(registryPath);
    return registry;
  } catch (error) {
    return {
      version: '1.0.0',
      specs: {},
      lastUpdated: new Date().toISOString(),
    };
  }
}

/**
 * List all available specs
 * @returns {Promise<Array<{id: string, path: string, type: string}>>}
 */
async function listAvailableSpecs() {
  const specs = [];

  // Check feature-specific specs
  const specsDir = path.join(process.cwd(), SPECS_DIR);
  if (await fs.pathExists(specsDir)) {
    const files = await fs.readdir(specsDir);
    const yamlFiles = files.filter((f) => f.endsWith('.yaml') || f.endsWith('.yml'));

    for (const file of yamlFiles) {
      if (file === 'registry.json') continue;

      const specPath = path.join(specsDir, file);
      try {
        const yaml = require('js-yaml');
        const content = await fs.readFile(specPath, 'utf8');
        const spec = yaml.load(content);

        specs.push({
          id: spec.id || path.basename(file, path.extname(file)),
          path: path.relative(process.cwd(), specPath),
          type: 'feature',
          title: spec.title || 'Untitled',
        });
      } catch (error) {
        // Skip invalid specs
      }
    }
  }

  // Check legacy working-spec.yaml
  const legacyPath = path.join(process.cwd(), LEGACY_SPEC);
  if (await fs.pathExists(legacyPath)) {
    try {
      const yaml = require('js-yaml');
      const content = await fs.readFile(legacyPath, 'utf8');
      const spec = yaml.load(content);

      specs.push({
        id: spec.id || 'working-spec',
        path: LEGACY_SPEC,
        type: 'legacy',
        title: spec.title || 'Legacy Working Spec',
      });
    } catch (error) {
      // Skip invalid spec
    }
  }

  return specs;
}

/**
 * Check if project is using multi-spec architecture
 * @returns {Promise<{isMultiSpec: boolean, specCount: number, needsMigration: boolean}>}
 */
async function checkMultiSpecStatus() {
  const registry = await loadSpecsRegistry();
  const hasFeatureSpecs = Object.keys(registry.specs ?? {}).length > 0;
  const legacyPath = path.join(process.cwd(), LEGACY_SPEC);
  const hasLegacySpec = await fs.pathExists(legacyPath);

  return {
    isMultiSpec: hasFeatureSpecs,
    specCount: Object.keys(registry.specs ?? {}).length,
    needsMigration: hasLegacySpec && !hasFeatureSpecs,
  };
}

/**
 * Suggest migration from legacy to multi-spec
 * @returns {Promise<void>}
 */
async function suggestMigration() {
  const status = await checkMultiSpecStatus();

  if (status.needsMigration) {
    console.log(chalk.yellow('\n⚠️  Migration Recommended: Single-Spec → Multi-Spec'));
    console.log(chalk.gray('   Your project uses the legacy working-spec.yaml'));
    console.log(chalk.gray('   For multi-agent workflows, migrate to feature-specific specs:\n'));
    console.log(chalk.blue('   1. caws specs create <feature-id>'));
    console.log(chalk.blue('   2. Copy relevant content from working-spec.yaml'));
    console.log(chalk.blue('   3. Update agents to use --spec-id <feature-id>'));
    console.log(chalk.gray('\n   See: docs/guides/multi-agent-migration.md\n'));
  }
}

module.exports = {
  resolveSpec,
  listAvailableSpecs,
  checkMultiSpecStatus,
  suggestMigration,
  loadSpecsRegistry,
  SPECS_DIR,
  LEGACY_SPEC,
  SPECS_REGISTRY,
};
