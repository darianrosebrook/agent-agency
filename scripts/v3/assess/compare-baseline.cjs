#!/usr/bin/env node
/**
 * Baseline Comparison Module for V3 Readiness Assessment Framework
 * Compares current assessment against previous baseline and shows trends
 * @author: @darianrosebrook
 */

const fs = require('fs');
const path = require('path');

const OUTPUT_DIR = path.join(__dirname, '../../../artifacts');
const BASELINE_FILE = path.join(OUTPUT_DIR, 'baseline.json');

function loadCurrentAssessment() {
    // Find the most recent assessment file
    const files = fs.readdirSync(OUTPUT_DIR)
        .filter(f => f.startsWith('readiness-assessment-') && f.endsWith('.json'))
        .sort()
        .reverse();
    
    if (files.length === 0) {
        throw new Error('No assessment files found');
    }
    
    return JSON.parse(fs.readFileSync(path.join(OUTPUT_DIR, files[0]), 'utf8'));
}

function loadBaseline() {
    if (!fs.existsSync(BASELINE_FILE)) {
        return null;
    }
    
    return JSON.parse(fs.readFileSync(BASELINE_FILE, 'utf8'));
}

function calculateDelta(current, baseline, path) {
    const currentVal = path.split('.').reduce((obj, key) => obj?.[key], current);
    const baselineVal = path.split('.').reduce((obj, key) => obj?.[key], baseline);
    
    if (currentVal === undefined || baselineVal === undefined) {
        return null;
    }
    
    if (typeof currentVal === 'number' && typeof baselineVal === 'number') {
        return currentVal - baselineVal;
    }
    
    return null;
}

function generateComparisonReport(current, baseline) {
    const report = {
        timestamp: new Date().toISOString(),
        current_assessment: current.timestamp,
        baseline_assessment: baseline.timestamp,
        deltas: {},
        trends: {},
        improvements: [],
        regressions: []
    };
    
    // Compare readiness scores
    const scoreDelta = calculateDelta(current, baseline, 'readiness_score.score');
    const percentageDelta = calculateDelta(current, baseline, 'readiness_score.percentage');
    
    report.deltas.readiness_score = {
        current: current.readiness_score.score,
        baseline: baseline.readiness_score.score,
        delta: scoreDelta,
        percentage_delta: percentageDelta
    };
    
    // Compare test results
    const unitTestDelta = calculateDelta(current, baseline, 'summary.tests.unit.failed');
    const integrationTestDelta = calculateDelta(current, baseline, 'summary.tests.integration.failed');
    
    report.deltas.tests = {
        unit_failed: {
            current: current.summary.tests.unit.failed,
            baseline: baseline.summary.tests.unit.failed,
            delta: unitTestDelta
        },
        integration_failed: {
            current: current.summary.tests.integration.failed,
            baseline: baseline.summary.tests.integration.failed,
            delta: integrationTestDelta
        }
    };
    
    // Compare coverage
    const lineCoverageDelta = calculateDelta(current, baseline, 'summary.coverage.overall.line_coverage');
    const branchCoverageDelta = calculateDelta(current, baseline, 'summary.coverage.overall.branch_coverage');
    
    report.deltas.coverage = {
        line: {
            current: current.summary.coverage.overall.line_coverage,
            baseline: baseline.summary.coverage.overall.line_coverage,
            delta: lineCoverageDelta
        },
        branch: {
            current: current.summary.coverage.overall.branch_coverage,
            baseline: baseline.summary.coverage.overall.branch_coverage,
            delta: branchCoverageDelta
        }
    };
    
    // Compare TODOs
    const todoDelta = calculateDelta(current, baseline, 'summary.todos.total');
    const blockingTodoDelta = calculateDelta(current, baseline, 'summary.todos.blocking');
    
    report.deltas.todos = {
        total: {
            current: current.summary.todos.total,
            baseline: baseline.summary.todos.total,
            delta: todoDelta
        },
        blocking: {
            current: current.summary.todos.blocking,
            baseline: baseline.summary.todos.blocking,
            delta: blockingTodoDelta
        }
    };
    
    // Identify improvements and regressions
    if (scoreDelta > 0) {
        report.improvements.push(`Readiness score improved by ${scoreDelta} points (${percentageDelta > 0 ? '+' : ''}${percentageDelta.toFixed(1)}%)`);
    } else if (scoreDelta < 0) {
        report.regressions.push(`Readiness score decreased by ${Math.abs(scoreDelta)} points (${percentageDelta.toFixed(1)}%)`);
    }
    
    if (unitTestDelta < 0) {
        report.improvements.push(`Unit test failures reduced by ${Math.abs(unitTestDelta)}`);
    } else if (unitTestDelta > 0) {
        report.regressions.push(`Unit test failures increased by ${unitTestDelta}`);
    }
    
    if (lineCoverageDelta > 0) {
        report.improvements.push(`Line coverage improved by ${(lineCoverageDelta * 100).toFixed(2)}%`);
    } else if (lineCoverageDelta < 0) {
        report.regressions.push(`Line coverage decreased by ${(Math.abs(lineCoverageDelta) * 100).toFixed(2)}%`);
    }
    
    if (blockingTodoDelta < 0) {
        report.improvements.push(`Blocking TODOs reduced by ${Math.abs(blockingTodoDelta)}`);
    } else if (blockingTodoDelta > 0) {
        report.regressions.push(`Blocking TODOs increased by ${blockingTodoDelta}`);
    }
    
    // Generate trends
    report.trends = {
        score_trend: scoreDelta > 0 ? 'improving' : scoreDelta < 0 ? 'declining' : 'stable',
        test_trend: unitTestDelta < 0 ? 'improving' : unitTestDelta > 0 ? 'declining' : 'stable',
        coverage_trend: lineCoverageDelta > 0 ? 'improving' : lineCoverageDelta < 0 ? 'declining' : 'stable',
        todo_trend: blockingTodoDelta < 0 ? 'improving' : blockingTodoDelta > 0 ? 'declining' : 'stable'
    };
    
    return report;
}

function generateMarkdownComparison(report) {
    let md = `# Baseline Comparison Report\n\n`;
    md += `**Generated:** ${new Date(report.timestamp).toLocaleString()}\n\n`;
    md += `**Current Assessment:** ${new Date(report.current_assessment).toLocaleString()}\n`;
    md += `**Baseline Assessment:** ${new Date(report.baseline_assessment).toLocaleString()}\n\n`;
    md += `---\n\n`;
    
    // Readiness Score Comparison
    md += `## Readiness Score Comparison\n\n`;
    const scoreDelta = report.deltas.readiness_score;
    md += `- **Current:** ${scoreDelta.current}/${scoreDelta.baseline.maxScore} (${scoreDelta.current / scoreDelta.baseline.maxScore * 100}%)\n`;
    md += `- **Baseline:** ${scoreDelta.baseline}/${scoreDelta.baseline.maxScore} (${scoreDelta.baseline / scoreDelta.baseline.maxScore * 100}%)\n`;
    md += `- **Change:** ${scoreDelta.delta > 0 ? '+' : ''}${scoreDelta.delta} points (${scoreDelta.percentage_delta > 0 ? '+' : ''}${scoreDelta.percentage_delta.toFixed(1)}%)\n\n`;
    
    // Improvements
    if (report.improvements.length > 0) {
        md += `## Improvements\n\n`;
        report.improvements.forEach(improvement => {
            md += `- ✅ ${improvement}\n`;
        });
        md += `\n`;
    }
    
    // Regressions
    if (report.regressions.length > 0) {
        md += `## Regressions\n\n`;
        report.regressions.forEach(regression => {
            md += `- ❌ ${regression}\n`;
        });
        md += `\n`;
    }
    
    // Trends
    md += `## Trends\n\n`;
    md += `- **Score:** ${report.trends.score_trend}\n`;
    md += `- **Tests:** ${report.trends.test_trend}\n`;
    md += `- **Coverage:** ${report.trends.coverage_trend}\n`;
    md += `- **TODOs:** ${report.trends.todo_trend}\n\n`;
    
    return md;
}

// Main execution
try {
    console.log('[compare-baseline] Loading current assessment...');
    const current = loadCurrentAssessment();
    
    console.log('[compare-baseline] Loading baseline...');
    const baseline = loadBaseline();
    
    if (!baseline) {
        console.log('[compare-baseline] No baseline found. Current assessment will be used as baseline.');
        process.exit(0);
    }
    
    console.log('[compare-baseline] Generating comparison...');
    const comparison = generateComparisonReport(current, baseline);
    
    const jsonPath = path.join(OUTPUT_DIR, `baseline-comparison-${Date.now()}.json`);
    fs.writeFileSync(jsonPath, JSON.stringify(comparison, null, 2));
    console.log(`[compare-baseline] Comparison JSON saved to: ${jsonPath}`);
    
    const mdReport = generateMarkdownComparison(comparison);
    const mdPath = path.join(OUTPUT_DIR, `baseline-comparison-${Date.now()}.md`);
    fs.writeFileSync(mdPath, mdReport);
    console.log(`[compare-baseline] Comparison Markdown saved to: ${mdPath}`);
    
    // Print summary to console
    console.log('\n=== Baseline Comparison Summary ===');
    console.log(`Readiness Score: ${comparison.deltas.readiness_score.current} (${comparison.deltas.readiness_score.delta > 0 ? '+' : ''}${comparison.deltas.readiness_score.delta})`);
    console.log(`Improvements: ${comparison.improvements.length}`);
    console.log(`Regressions: ${comparison.regressions.length}`);
    
    process.exit(0);
} catch (error) {
    console.error('[compare-baseline] Error:', error.message);
    process.exit(1);
}

