#!/usr/bin/env node

/**
 * Dependency Gate Enforcement Script
 *
 * Enforces acyclic dependency rules for the contracts-first architecture.
 * Blocks commits that violate the dependency hierarchy with contracts at the top.
 *
 * @author @darianrosebrook
 */

import { execSync } from 'child_process';
import { readFileSync, existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Dependency rules - FORBID violations, ALLOW informational
const DEPENDENCY_RULES = `
// Core forbidden cycles (direct dependencies that create cycles)
FORBID: agent-research -> agent-data-processing
FORBID: data-infrastructure -> agent-orchestration
FORBID: agent-agency-contracts -> agent-*,system-*,apps-*,data-*

// Allow feature-gated optional dependencies (these don't create cycles when features disabled)
ALLOW: agent-orchestration -> agent-memory
ALLOW: agent-orchestration -> agent-research
ALLOW: agent-orchestration -> agent-constitutional-council
ALLOW: agent-orchestration -> system-federated-ml
ALLOW: * -> agent-agency-contracts
ALLOW: agent-data-processing -> data-infrastructure
`;

// Parse dependency rules
function parseRules(rulesText) {
    const rules = {
        forbid: [],
        allow: []
    };

    const lines = rulesText.trim().split('\n');
    for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) continue;

        if (trimmed.startsWith('FORBID:')) {
            const rule = trimmed.substring(7).trim();
            const [source, targets] = rule.split('->').map(s => s.trim());
            const targetList = targets.split(',').map(s => s.trim());
            rules.forbid.push({ source, targets: targetList });
        } else if (trimmed.startsWith('ALLOW:')) {
            const rule = trimmed.substring(6).trim();
            const [source, targets] = rule.split('->').map(s => s.trim());
            const targetList = targets.split(',').map(s => s.trim());
            rules.allow.push({ source, targets: targetList });
        }
    }

    return rules;
}

// Get workspace members from Cargo.toml
function getWorkspaceMembers() {
    try {
        const cargoToml = readFileSync('Cargo.toml', 'utf8');
        const lines = cargoToml.split('\n');
        const members = [];
        let inMembersSection = false;

        for (const line of lines) {
            const trimmed = line.trim();

            if (trimmed === 'members = [') {
                inMembersSection = true;
                continue;
            }

            if (inMembersSection) {
                if (trimmed === ']') {
                    break;
                }

                // Extract quoted strings like "agent-agency-contracts",
                const match = trimmed.match(/"([^"]+)"/);
                if (match) {
                    members.push(match[1]);
                }
            }
        }

        return members;
    } catch (error) {
        console.error('Error reading Cargo.toml:', error.message);
        return [];
    }
}

// Get dependencies for a crate using cargo metadata
function getDependencies(crateName) {
    try {
        const output = execSync(`cargo metadata --format-version 1 --no-deps`, {
            encoding: 'utf8',
            cwd: process.cwd()
        });
        const metadata = JSON.parse(output);

        const crate = metadata.packages.find(pkg => pkg.name === crateName);
        if (!crate) return [];

        return crate.dependencies.map(dep => dep.name);
    } catch (error) {
        // If cargo metadata fails, try a simpler approach with Cargo.toml parsing
        try {
            const cargoTomlPath = join(crateName, 'Cargo.toml');
            if (!existsSync(cargoTomlPath)) return [];

            const cargoToml = readFileSync(cargoTomlPath, 'utf8');
            const dependencies = [];

            // Extract dependencies from [dependencies] section
            const depSectionMatch = cargoToml.match(/\[dependencies\]([^[]*)/s);
            if (depSectionMatch) {
                const depLines = depSectionMatch[1].split('\n');
                for (const line of depLines) {
                    const trimmed = line.trim();
                    if (trimmed && !trimmed.startsWith('#') && trimmed.includes('=')) {
                        const depName = trimmed.split('=')[0].trim().replace(/"/g, '');
                        if (depName) dependencies.push(depName);
                    }
                }
            }

            return dependencies;
        } catch (tomlError) {
            console.warn(`Warning: Could not parse dependencies for ${crateName}:`, tomlError.message);
            return [];
        }
    }
}

// Check if a rule matches (supports wildcards)
function ruleMatches(ruleSource, ruleTargets, actualSource, actualTarget) {
    const sourceMatches = ruleSource === '*' || ruleSource === actualSource ||
                         (ruleSource.includes('*') && new RegExp(ruleSource.replace(/\*/g, '.*')).test(actualSource));

    const targetMatches = ruleTargets.some(target =>
        target === '*' || target === actualTarget ||
        (target.includes('*') && new RegExp(target.replace(/\*/g, '.*')).test(actualTarget))
    );

    return sourceMatches && targetMatches;
}

// Check rules for a specific dependency
function checkRule(dependency, rules, violations) {
    const [source, target] = dependency.split(' -> ').map(s => s.trim());

    // Check FORBID rules first
    for (const forbid of rules.forbid) {
        if (ruleMatches(forbid.source, forbid.targets, source, target)) {
            violations.forbid.push({
                rule: `FORBID: ${forbid.source} -> ${forbid.targets.join(',')}`,
                violation: `${source} -> ${target}`,
                type: 'forbid'
            });
            return; // Found a forbid violation, no need to check allows
        }
    }

    // Check ALLOW rules (informational only)
    for (const allow of rules.allow) {
        if (ruleMatches(allow.source, allow.targets, source, target)) {
            violations.allow.push({
                rule: `ALLOW: ${allow.source} -> ${allow.targets.join(',')}`,
                dependency: `${source} -> ${target}`,
                type: 'allow'
            });
        }
    }
}

// Main function
function main() {
    console.log('🔍 Checking dependency gates...');

    const rules = parseRules(DEPENDENCY_RULES);
    const workspaceMembers = getWorkspaceMembers();

    console.log(`📦 Found ${workspaceMembers.length} workspace members`);
    console.log(`📋 Loaded ${rules.forbid.length} forbid rules, ${rules.allow.length} allow rules`);

    const violations = {
        forbid: [],
        allow: []
    };

    // Check all workspace member dependencies
    for (const member of workspaceMembers) {
        const dependencies = getDependencies(member);

        for (const dep of dependencies) {
            // Only check dependencies that are also workspace members
            if (workspaceMembers.includes(dep)) {
                checkRule(`${member} -> ${dep}`, rules, violations);
            }
        }
    }

    // Report results
    if (violations.forbid.length > 0) {
        console.log('\n❌ FORBIDDEN DEPENDENCY VIOLATIONS:');
        for (const violation of violations.forbid) {
            console.log(`  🚫 ${violation.violation} (violates: ${violation.rule})`);
        }
        console.log(`\n💥 ${violations.forbid.length} forbidden dependency violation(s) found!`);
        console.log('These must be fixed before committing.');
        process.exit(1);
    }

    if (violations.allow.length > 0) {
        console.log('\nℹ️ ALLOWED DEPENDENCIES (informational):');
        for (const allow of violations.allow) {
            console.log(`  ✅ ${allow.dependency} (allowed by: ${allow.rule})`);
        }
    }

    console.log('\n🎉 All dependency gates passed! No forbidden violations found.');

    if (violations.allow.length > 0) {
        console.log(`Found ${violations.allow.length} allowed dependencies (contracts-first architecture respected).`);
    }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
    main();
}

export { parseRules, getWorkspaceMembers, getDependencies, checkRule, main };
