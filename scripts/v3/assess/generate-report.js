#!/usr/bin/env node
/**
 * Report Generator for V3 Readiness Assessment Framework
 * Generates human-readable and machine-readable reports
 * @author: @darianrosebrook
 */

const fs = require('fs');
const path = require('path');

const OUTPUT_DIR = path.join(__dirname, '../../../artifacts');
const TIMESTAMP = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5) + 'Z';

// Load assessment results
function loadResults() {
    const testResults = JSON.parse(fs.readFileSync(path.join(OUTPUT_DIR, 'test-results.json'), 'utf8'));
    const coverageResults = JSON.parse(fs.readFileSync(path.join(OUTPUT_DIR, 'coverage-results.json'), 'utf8'));
    const todoResults = JSON.parse(fs.readFileSync(path.join(OUTPUT_DIR, 'todo-results.json'), 'utf8'));
    const dashboardResults = JSON.parse(fs.readFileSync(path.join(OUTPUT_DIR, 'dashboard-readiness.json'), 'utf8'));
    
    return {
        tests: testResults,
        coverage: coverageResults,
        todos: todoResults,
        dashboard: dashboardResults,
        timestamp: TIMESTAMP
    };
}

// Calculate overall readiness score
function calculateReadinessScore(results) {
    let score = 0;
    let maxScore = 0;
    
    // Test status (30%)
    maxScore += 30;
    const unitTests = results.tests.unit_tests;
    const integrationTests = results.tests.integration_tests;
    const totalTests = unitTests.total + integrationTests.total;
    const passedTests = unitTests.passed + integrationTests.passed;
    
    if (totalTests > 0) {
        const testPassRate = passedTests / totalTests;
        score += testPassRate * 30;
    }
    
    // Coverage (25%)
    maxScore += 25;
    const lineCoverage = results.coverage.overall.line_coverage;
    const branchCoverage = results.coverage.overall.branch_coverage;
    const avgCoverage = (lineCoverage + branchCoverage) / 2;
    score += avgCoverage * 25;
    
    // TODO status (20%)
    maxScore += 20;
    const totalTodos = results.todos.summary.total_todos;
    const blockingTodos = results.todos.summary.blocking_todos;
    
    // Lower score for more blocking TODOs
    if (totalTodos === 0) {
        score += 20; // Perfect if no TODOs
    } else {
        const blockingRatio = blockingTodos / totalTodos;
        score += (1 - blockingRatio) * 20;
    }
    
    // Dashboard readiness (25%)
    maxScore += 25;
    if (results.dashboard.overall_readiness) {
        score += 25;
    } else {
        // Partial credit based on components
        let dashboardScore = 0;
        if (results.dashboard.build_status.compiles) dashboardScore += 10;
        if (results.dashboard.api_connectivity.server_running) dashboardScore += 5;
        if (results.dashboard.schema_alignment.aligned) dashboardScore += 5;
        if (results.dashboard.missing_apis.length === 0) dashboardScore += 5;
        score += dashboardScore;
    }
    
    return {
        score: Math.round(score),
        maxScore,
        percentage: Math.round((score / maxScore) * 100)
    };
}

// Generate JSON report
function generateJSONReport(results) {
    const readinessScore = calculateReadinessScore(results);
    
    const report = {
        timestamp: results.timestamp,
        readiness_score: readinessScore,
        summary: {
            tests: {
                unit: {
                    total: results.tests.unit_tests.total,
                    passed: results.tests.unit_tests.passed,
                    failed: results.tests.unit_tests.failed,
                    ignored: results.tests.unit_tests.ignored
                },
                integration: {
                    total: results.tests.integration_tests.total,
                    passed: results.tests.integration_tests.passed,
                    failed: results.tests.integration_tests.failed,
                    ignored: results.tests.integration_tests.ignored
                },
                mutation: results.tests.mutation_tests
            },
            coverage: {
                overall: results.coverage.overall,
                below_threshold_count: (results.coverage.below_threshold || []).length,
                high_value_areas_count: (results.coverage.high_value_areas || []).length
            },
            todos: {
                total: results.todos.summary.total_todos,
                blocking: results.todos.summary.blocking_todos,
                in_critical_paths: results.todos.blocking_in_critical_paths.length
            },
            dashboard: {
                ready: results.dashboard.overall_readiness,
                build_status: results.dashboard.build_status.compiles,
                api_connectivity: results.dashboard.api_connectivity.server_running,
                schema_aligned: results.dashboard.schema_alignment.aligned
            }
        },
        details: {
            tests: results.tests,
            coverage: results.coverage,
            todos: results.todos,
            dashboard: results.dashboard
        },
        recommendations: generateRecommendations(results, readinessScore)
    };
    
    return report;
}

// Generate recommendations
function generateRecommendations(results, readinessScore) {
    const recommendations = [];
    
    // Test recommendations
    if (results.tests.unit_tests.failed > 0) {
        recommendations.push({
            priority: 'high',
            category: 'tests',
            issue: `${results.tests.unit_tests.failed} unit tests failing`,
            action: 'Fix failing unit tests before proceeding',
            impact: 'Blocks development and deployment'
        });
    }
    
    if (results.tests.integration_tests.failed > 0) {
        recommendations.push({
            priority: 'high',
            category: 'tests',
            issue: `${results.tests.integration_tests.failed} integration tests failing`,
            action: 'Fix failing integration tests',
            impact: 'May indicate broken component integration'
        });
    }
    
    // Coverage recommendations
    const belowThreshold = results.coverage.below_threshold || [];
    if (belowThreshold.length > 0) {
        recommendations.push({
            priority: 'medium',
            category: 'coverage',
            issue: `${belowThreshold.length} crates below coverage threshold`,
            action: `Focus on: ${belowThreshold.slice(0, 5).join(', ')}`,
            impact: 'Insufficient test coverage may hide bugs'
        });
    }
    
    // TODO recommendations
    const blockingTodos = results.todos.summary.blocking_todos;
    if (blockingTodos > 0) {
        recommendations.push({
            priority: 'high',
            category: 'todos',
            issue: `${blockingTodos} blocking TODOs found`,
            action: 'Address blocking TODOs in critical paths',
            impact: 'May block training, conversion, or inference workflows'
        });
    }
    
    const criticalPathBlockers = results.todos.blocking_in_critical_paths.length;
    if (criticalPathBlockers > 0) {
        recommendations.push({
            priority: 'critical',
            category: 'todos',
            issue: `${criticalPathBlockers} blocking TODOs in critical paths`,
            action: 'Immediately address TODOs in training/conversion/inference paths',
            impact: 'Blocks core functionality'
        });
    }
    
    // Dashboard recommendations
    if (!results.dashboard.overall_readiness) {
        if (!results.dashboard.build_status.compiles) {
            recommendations.push({
                priority: 'high',
                category: 'dashboard',
                issue: 'Dashboard has TypeScript compilation errors',
                action: 'Fix TypeScript errors in dashboard',
                impact: 'Dashboard cannot be built or deployed'
            });
        }
        
        if (results.dashboard.missing_apis.length > 0) {
            recommendations.push({
                priority: 'medium',
                category: 'dashboard',
                issue: `${results.dashboard.missing_apis.length} API implementations missing`,
                action: 'Implement missing API client functions',
                impact: 'Dashboard features may not work'
            });
        }
    }
    
    return recommendations.sort((a, b) => {
        const priorityOrder = { critical: 0, high: 1, medium: 2, low: 3 };
        return priorityOrder[a.priority] - priorityOrder[b.priority];
    });
}

// Generate Markdown report
function generateMarkdownReport(jsonReport) {
    const results = jsonReport.details;
    const score = jsonReport.readiness_score;
    
    let md = `# V3 Readiness Assessment Report\n\n`;
    md += `**Generated:** ${new Date(jsonReport.timestamp).toLocaleString()}\n\n`;
    md += `---\n\n`;
    
    // Executive Summary
    md += `## Executive Summary\n\n`;
    md += `**Overall Readiness Score: ${score.percentage}%** (${score.score}/${score.maxScore})\n\n`;
    
    const statusEmoji = score.percentage >= 80 ? '✅' : score.percentage >= 60 ? '⚠️' : '❌';
    md += `${statusEmoji} **Status:** `;
    if (score.percentage >= 80) {
        md += `Ready for production use\n\n`;
    } else if (score.percentage >= 60) {
        md += `Ready with caution - address recommendations\n\n`;
    } else {
        md += `Not ready - critical issues must be addressed\n\n`;
    }
    
    // Test Status
    md += `## Test Status\n\n`;
    md += `### Unit Tests\n`;
    md += `- **Total:** ${results.tests.unit_tests.total}\n`;
    md += `- **Passed:** ${results.tests.unit_tests.passed}\n`;
    md += `- **Failed:** ${results.tests.unit_tests.failed}\n`;
    md += `- **Ignored:** ${results.tests.unit_tests.ignored}\n\n`;
    
    if (results.tests.unit_tests.failures.length > 0) {
        md += `**Failing Tests:**\n`;
        results.tests.unit_tests.failures.slice(0, 10).forEach(test => {
            md += `- \`${test}\`\n`;
        });
        if (results.tests.unit_tests.failures.length > 10) {
            md += `- ... and ${results.tests.unit_tests.failures.length - 10} more\n`;
        }
        md += `\n`;
    }
    
    md += `### Integration Tests\n`;
    md += `- **Total:** ${results.tests.integration_tests.total}\n`;
    md += `- **Passed:** ${results.tests.integration_tests.passed}\n`;
    md += `- **Failed:** ${results.tests.integration_tests.failed}\n`;
    md += `- **Ignored:** ${results.tests.integration_tests.ignored}\n\n`;
    
    if (results.tests.mutation_tests.enabled) {
        md += `### Mutation Tests\n`;
        md += `- **Score:** ${(results.tests.mutation_tests.score * 100).toFixed(2)}%\n`;
        md += `- **Mutants Killed:** ${results.tests.mutation_tests.mutants_killed}\n`;
        md += `- **Mutants Survived:** ${results.tests.mutation_tests.mutants_survived}\n\n`;
    }
    
    // Coverage Analysis
    md += `## Coverage Analysis\n\n`;
    md += `### Overall Coverage\n`;
    md += `- **Line Coverage:** ${(results.coverage.overall.line_coverage * 100).toFixed(2)}% (threshold: ${(results.coverage.thresholds.line * 100).toFixed(0)}%)\n`;
    md += `- **Branch Coverage:** ${(results.coverage.overall.branch_coverage * 100).toFixed(2)}% (threshold: ${(results.coverage.thresholds.branch * 100).toFixed(0)}%)\n\n`;
    
    if (results.coverage.below_threshold && results.coverage.below_threshold.length > 0) {
        md += `### Crates Below Threshold\n\n`;
        results.coverage.below_threshold.forEach(crate => {
            const crateData = results.coverage.crates[crate];
            if (crateData) {
                md += `- **${crate}**: `;
                md += `line=${(crateData.line_coverage * 100).toFixed(2)}%, `;
                md += `branch=${(crateData.branch_coverage * 100).toFixed(2)}%\n`;
            }
        });
        md += `\n`;
    }
    
    if (results.coverage.high_value_areas && results.coverage.high_value_areas.length > 0) {
        md += `### High-Value Areas Needing Coverage\n\n`;
        results.coverage.high_value_areas.forEach(crate => {
            md += `- ${crate}\n`;
        });
        md += `\n`;
    }
    
    // TODO Analysis
    md += `## TODO Analysis\n\n`;
    md += `- **Total TODOs:** ${results.todos.summary.total_todos}\n`;
    md += `- **Blocking TODOs:** ${results.todos.summary.blocking_todos}\n`;
    md += `- **High Confidence:** ${results.todos.summary.high_confidence}\n`;
    md += `- **Medium Confidence:** ${results.todos.summary.medium_confidence}\n`;
    md += `- **Low Confidence:** ${results.todos.summary.low_confidence}\n\n`;
    
    if (results.todos.blocking_in_critical_paths && results.todos.blocking_in_critical_paths.length > 0) {
        md += `### Blocking TODOs in Critical Paths\n\n`;
        results.todos.blocking_in_critical_paths.slice(0, 10).forEach(todo => {
            md += `- \`${todo}\`\n`;
        });
        if (results.todos.blocking_in_critical_paths.length > 10) {
            md += `- ... and ${results.todos.blocking_in_critical_paths.length - 10} more\n`;
        }
        md += `\n`;
    }
    
    // Dashboard Readiness
    md += `## Dashboard Readiness\n\n`;
    md += `- **Overall Status:** ${results.dashboard.overall_readiness ? '✅ Ready' : '❌ Not Ready'}\n`;
    md += `- **Build Status:** ${results.dashboard.build_status.compiles ? '✅ Compiles' : '❌ Has Errors'}\n`;
    md += `- **TypeScript Errors:** ${results.dashboard.build_status.error_count}\n`;
    md += `- **API Server:** ${results.dashboard.api_connectivity.server_running ? '✅ Running' : '⚠️ Not Running'}\n`;
    md += `- **Schema Alignment:** ${results.dashboard.schema_alignment.aligned ? '✅ Aligned' : '⚠️ Issues Found'}\n`;
    md += `- **Missing APIs:** ${results.dashboard.missing_apis.length}\n\n`;
    
    // Recommendations
    md += `## Recommendations\n\n`;
    jsonReport.recommendations.forEach((rec, idx) => {
        const priorityEmoji = {
            critical: '🔴',
            high: '🟠',
            medium: '🟡',
            low: '🟢'
        };
        
        md += `### ${idx + 1}. ${priorityEmoji[rec.priority]} ${rec.issue}\n\n`;
        md += `**Category:** ${rec.category}\n\n`;
        md += `**Action:** ${rec.action}\n\n`;
        md += `**Impact:** ${rec.impact}\n\n`;
    });
    
    return md;
}

// Main execution
try {
    console.log('[generate-report] Loading assessment results...');
    const results = loadResults();
    
    console.log('[generate-report] Generating JSON report...');
    const jsonReport = generateJSONReport(results);
    const jsonPath = path.join(OUTPUT_DIR, `readiness-assessment-${TIMESTAMP}.json`);
    fs.writeFileSync(jsonPath, JSON.stringify(jsonReport, null, 2));
    console.log(`[generate-report] JSON report saved to: ${jsonPath}`);
    
    console.log('[generate-report] Generating Markdown report...');
    const mdReport = generateMarkdownReport(jsonReport);
    const mdPath = path.join(OUTPUT_DIR, `readiness-assessment-${TIMESTAMP}.md`);
    fs.writeFileSync(mdPath, mdReport);
    console.log(`[generate-report] Markdown report saved to: ${mdPath}`);
    
    // Save as latest baseline
    const baselinePath = path.join(OUTPUT_DIR, 'baseline.json');
    fs.writeFileSync(baselinePath, JSON.stringify(jsonReport, null, 2));
    console.log(`[generate-report] Baseline saved to: ${baselinePath}`);
    
    console.log('[generate-report] Report generation complete!');
    process.exit(0);
} catch (error) {
    console.error('[generate-report] Error generating report:', error);
    process.exit(1);
}

