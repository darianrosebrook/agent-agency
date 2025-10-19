#!/usr/bin/env node
/**
 * Build Performance Benchmarking Script
 *
 * Measures and analyzes build performance across Rust, Node.js, and Python
 * environments with caching effectiveness analysis.
 *
 * @author @darianrosebrook
 */

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

class BuildBenchmarker {
    constructor() {
        this.results = {
            timestamp: new Date().toISOString(),
            system: {
                platform: os.platform(),
                arch: os.arch(),
                cpus: os.cpus().length,
                memory: Math.round(os.totalmem() / 1024 / 1024 / 1024) + 'GB'
            },
            benchmarks: []
        };
        this.startTime = null;
        this.endTime = null;
    }

    log(message, level = 'info') {
        const timestamp = new Date().toISOString();
        const prefix = level === 'error' ? '❌' : level === 'warn' ? '⚠️' : level === 'success' ? '✅' : 'ℹ️';
        console.log(`${prefix} [${timestamp}] ${message}`);
    }

    async timeOperation(name, operation, options = {}) {
        const { cwd = process.cwd(), env = process.env, retries = 1 } = options;

        this.log(`Starting benchmark: ${name}`);

        for (let attempt = 1; attempt <= retries; attempt++) {
            try {
                const startTime = process.hrtime.bigint();

                if (typeof operation === 'function') {
                    await operation();
                } else if (typeof operation === 'string') {
                    execSync(operation, { cwd, env, stdio: 'inherit' });
                }

                const endTime = process.hrtime.bigint();
                const durationMs = Number(endTime - startTime) / 1_000_000;

                const result = {
                    name,
                    duration: durationMs,
                    attempt,
                    success: true,
                    timestamp: new Date().toISOString()
                };

                this.results.benchmarks.push(result);
                this.log(`✅ ${name} completed in ${durationMs.toFixed(2)}ms`, 'success');
                return result;

            } catch (error) {
                this.log(`❌ ${name} failed (attempt ${attempt}/${retries}): ${error.message}`, 'error');

                if (attempt === retries) {
                    const result = {
                        name,
                        duration: 0,
                        attempt,
                        success: false,
                        error: error.message,
                        timestamp: new Date().toISOString()
                    };
                    this.results.benchmarks.push(result);
                    return result;
                }

                // Wait before retry
                await new Promise(resolve => setTimeout(resolve, 1000));
            }
        }
    }

    async benchmarkRustBuild(clean = false) {
        const rustDir = path.join(__dirname, '..', 'iterations', 'v3');

        if (!fs.existsSync(path.join(rustDir, 'Cargo.toml'))) {
            this.log('Rust project not found, skipping Rust benchmarks', 'warn');
            return;
        }

        process.chdir(rustDir);

        // Clean build if requested
        if (clean) {
            await this.timeOperation('Rust Clean', 'make clean');
        }

        // Cold build (no cache)
        await this.timeOperation('Rust Cold Build', './scripts/build-wrapper.sh release --quiet');

        // Warm build (with cache)
        await this.timeOperation('Rust Warm Build', './scripts/build-wrapper.sh release --quiet');

        // Test build
        await this.timeOperation('Rust Test Build', './scripts/build-wrapper.sh test --quiet');
    }

    async benchmarkNodeBuild(clean = false) {
        const nodeDir = path.join(__dirname, '..', 'iterations', 'v2');

        if (!fs.existsSync(path.join(nodeDir, 'package.json'))) {
            this.log('Node.js project not found, skipping Node.js benchmarks', 'warn');
            return;
        }

        process.chdir(nodeDir);

        // Clean if requested
        if (clean) {
            await this.timeOperation('Node Clean', 'rm -rf dist node_modules/.cache');
        }

        // Install dependencies
        await this.timeOperation('Node Install', 'pnpm install --frozen-lockfile');

        // Cold build
        await this.timeOperation('Node Cold Build', 'turbo build');

        // Warm build
        await this.timeOperation('Node Warm Build', 'turbo build');

        // Test run
        await this.timeOperation('Node Test Run', 'turbo test --filter=...{test}');
    }

    async benchmarkPythonBuild(clean = false) {
        const pythonDir = path.join(__dirname, '..', 'iterations', 'v2', 'python-services', 'dspy-integration');

        if (!fs.existsSync(path.join(pythonDir, 'pyproject.toml'))) {
            this.log('Python project not found, skipping Python benchmarks', 'warn');
            return;
        }

        process.chdir(pythonDir);

        // Clean if requested
        if (clean) {
            await this.timeOperation('Python Clean', 'make clean');
        }

        // Install dependencies
        await this.timeOperation('Python Install', 'uv sync --dev');

        // Test run
        await this.timeOperation('Python Test Run', 'uv run pytest tests/ -x --tb=short');

        // Parallel test run
        await this.timeOperation('Python Parallel Test', 'uv run pytest tests/ -x --tb=short -n auto');
    }

    async benchmarkCacheEffectiveness() {
        this.log('Analyzing cache effectiveness...');

        // Get cache stats
        const cacheStats = {
            sccache: this.getSccacheStats(),
            turbo: this.getTurboStats(),
            uv: this.getUvStats()
        };

        this.results.cacheStats = cacheStats;

        // Log cache effectiveness
        if (cacheStats.sccache) {
            this.log(`sccache: ${cacheStats.sccache.hitRate || 'N/A'} hit rate`);
        }
        if (cacheStats.turbo) {
            this.log(`Turbo: ${cacheStats.turbo.hitRate || 'N/A'} hit rate`);
        }
    }

    getSccacheStats() {
        try {
            const output = execSync('sccache --show-stats', { encoding: 'utf8' });
            // Parse sccache output for hit rates
            return { raw: output };
        } catch (e) {
            return null;
        }
    }

    getTurboStats() {
        try {
            // Turbo doesn't have a direct stats command, check cache directory size
            const cacheDir = path.join(os.homedir(), '.cache', 'turbo');
            const stats = fs.statSync(cacheDir);
            return { cacheSize: stats.size };
        } catch (e) {
            return null;
        }
    }

    getUvStats() {
        try {
            const cacheDir = path.join(os.homedir(), '.cache', 'uv');
            const stats = fs.statSync(cacheDir);
            return { cacheSize: stats.size };
        } catch (e) {
            return null;
        }
    }

    generateReport() {
        const report = {
            ...this.results,
            summary: this.generateSummary(),
            recommendations: this.generateRecommendations()
        };

        // Write to file
        const reportPath = path.join(__dirname, '..', 'build-performance-report.json');
        fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

        this.log(`📊 Performance report saved to: ${reportPath}`, 'success');

        // Print summary
        console.log('\n📈 Build Performance Summary:');
        console.log('='.repeat(50));

        report.summary.forEach(item => {
            console.log(`${item.name}: ${item.avgDuration.toFixed(2)}ms (σ: ${item.stdDev.toFixed(2)}ms)`);
        });

        console.log('\n💡 Recommendations:');
        report.recommendations.forEach(rec => {
            console.log(`• ${rec}`);
        });

        return report;
    }

    generateSummary() {
        const summary = {};

        this.results.benchmarks.forEach(benchmark => {
            if (!summary[benchmark.name]) {
                summary[benchmark.name] = [];
            }
            if (benchmark.success) {
                summary[benchmark.name].push(benchmark.duration);
            }
        });

        return Object.entries(summary).map(([name, durations]) => {
            const avg = durations.reduce((a, b) => a + b, 0) / durations.length;
            const variance = durations.reduce((sum, d) => sum + Math.pow(d - avg, 2), 0) / durations.length;
            const stdDev = Math.sqrt(variance);

            return {
                name,
                avgDuration: avg,
                stdDev,
                sampleCount: durations.length
            };
        });
    }

    generateRecommendations() {
        const recommendations = [];

        const summary = this.generateSummary();

        // Analyze Rust build times
        const rustBuilds = summary.filter(s => s.name.includes('Rust'));
        if (rustBuilds.length > 1) {
            const coldBuild = rustBuilds.find(b => b.name.includes('Cold'));
            const warmBuild = rustBuilds.find(b => b.name.includes('Warm'));

            if (coldBuild && warmBuild) {
                const speedup = coldBuild.avgDuration / warmBuild.avgDuration;
                if (speedup > 2) {
                    recommendations.push(`Rust caching is effective (${speedup.toFixed(1)}x speedup)`);
                } else {
                    recommendations.push('Consider optimizing Rust cache configuration');
                }
            }
        }

        // Analyze Node.js build times
        const nodeBuilds = summary.filter(s => s.name.includes('Node'));
        if (nodeBuilds.length > 1) {
            const coldBuild = nodeBuilds.find(b => b.name.includes('Cold'));
            const warmBuild = nodeBuilds.find(b => b.name.includes('Warm'));

            if (coldBuild && warmBuild) {
                const speedup = coldBuild.avgDuration / warmBuild.avgDuration;
                if (speedup > 1.5) {
                    recommendations.push(`Node.js caching is effective (${speedup.toFixed(1)}x speedup)`);
                }
            }
        }

        // Check for slow operations
        const slowOperations = summary.filter(s => s.avgDuration > 30000); // 30 seconds
        if (slowOperations.length > 0) {
            recommendations.push(`Consider optimizing slow operations: ${slowOperations.map(s => s.name).join(', ')}`);
        }

        if (recommendations.length === 0) {
            recommendations.push('All builds are performing well! Consider enabling distributed caching for team benefits.');
        }

        return recommendations;
    }

    async runFullBenchmark(options = {}) {
        const { clean = false, skipCacheAnalysis = false } = options;

        this.log('🚀 Starting comprehensive build performance benchmark');
        this.startTime = new Date();

        try {
            // Run all benchmarks
            await this.benchmarkRustBuild(clean);
            await this.benchmarkNodeBuild(clean);
            await this.benchmarkPythonBuild(clean);

            if (!skipCacheAnalysis) {
                await this.benchmarkCacheEffectiveness();
            }

            this.endTime = new Date();

            // Generate report
            return this.generateReport();

        } catch (error) {
            this.log(`Benchmark failed: ${error.message}`, 'error');
            throw error;
        }
    }
}

// CLI interface
async function main() {
    const args = process.argv.slice(2);
    const options = {
        clean: args.includes('--clean'),
        skipCacheAnalysis: args.includes('--no-cache'),
        help: args.includes('--help')
    };

    if (options.help) {
        console.log(`
Build Performance Benchmark Script

Usage: node benchmark-build-performance.js [options]

Options:
  --clean              Clean caches before benchmarking
  --no-cache           Skip cache effectiveness analysis
  --help               Show this help message

This script benchmarks build performance across Rust, Node.js, and Python
environments and provides optimization recommendations.
        `);
        return;
    }

    const benchmarker = new BuildBenchmarker();
    await benchmarker.runFullBenchmark(options);
}

if (require.main === module) {
    main().catch(console.error);
}

module.exports = BuildBenchmarker;
