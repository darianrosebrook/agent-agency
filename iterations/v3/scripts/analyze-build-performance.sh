#!/usr/bin/env bash
# Build performance analysis script
# @darianrosebrook
#
# This script analyzes build performance bottlenecks and provides
# optimization recommendations.

set -euo pipefail

echo "🔍 Analyzing Rust build performance..."

# Check if we're using nightly for advanced analysis
if ! rustc -vV | grep -q nightly; then
    echo "⚠️  Some analysis features require nightly toolchain"
    echo "   Install with: rustup toolchain install nightly"
    echo "   Use with: rustup override set nightly"
fi

# Create analysis directory
mkdir -p target/analysis

echo "📊 Running build timing analysis..."

# Generate timing report
if rustc -vV | grep -q nightly; then
    echo "   Generating detailed timing report..."
    CARGO_PROFILE_TIMINGS=html cargo build --workspace 2>&1 | tee target/analysis/build-timings.log
    echo "   📈 Timing report saved to target/analysis/"
else
    echo "   Running basic timing analysis..."
    time cargo build --workspace 2>&1 | tee target/analysis/build-timings.log
fi

echo "🔍 Analyzing monomorphization (requires nightly)..."

if rustc -vV | grep -q nightly; then
    echo "   Generating monomorphization report..."
    cargo rustc --workspace -- -Zprint-mono-items=lazy 2>&1 | tee target/analysis/mono-items.log
    
    # Count unique monomorphizations
    echo "   📊 Monomorphization summary:"
    grep -c "MONO_ITEM" target/analysis/mono-items.log || echo "   No mono items found"
    
    # Find most common generic instantiations
    echo "   🔍 Top generic instantiations:"
    grep "MONO_ITEM" target/analysis/mono-items.log | \
        sed 's/.*MONO_ITEM //' | \
        sort | uniq -c | sort -nr | head -10 || echo "   No data available"
else
    echo "   ⚠️  Skipping monomorphization analysis (requires nightly)"
fi

echo "📦 Analyzing crate dependencies..."

# Generate dependency graph
if command -v cargo-tree >/dev/null 2>&1; then
    echo "   Generating dependency tree..."
    cargo tree --workspace > target/analysis/dependency-tree.txt
else
    echo "   Installing cargo-tree for dependency analysis..."
    cargo install cargo-tree
    cargo tree --workspace > target/analysis/dependency-tree.txt
fi

# Analyze dependency depth
echo "   📊 Dependency analysis:"
echo "   Total crates: $(cargo tree --workspace | wc -l)"
echo "   Unique dependencies: $(cargo tree --workspace | grep -v "└──" | grep -v "├──" | wc -l)"

echo "🔧 Checking for optimization opportunities..."

# Check for unused dependencies
if command -v cargo-machete >/dev/null 2>&1; then
    echo "   Checking for unused dependencies..."
    cargo machete --workspace > target/analysis/unused-deps.txt || echo "   No unused dependencies found"
fi

# Check for duplicate dependencies
echo "   Checking for duplicate dependencies..."
cargo tree --workspace --duplicates > target/analysis/duplicate-deps.txt || echo "   No duplicate dependencies found"

echo "📋 Performance recommendations:"

# Check cache usage
if [[ -n "${SCCACHE_STATS:-}" ]]; then
    echo "   🗄️  Compiler cache stats:"
    sccache --show-stats || echo "   No cache stats available"
fi

# Check target directory size
TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1 || echo "unknown")
echo "   📁 Target directory size: $TARGET_SIZE"

# Check for large object files
echo "   🔍 Large object files (>10MB):"
find target -name "*.rlib" -o -name "*.rmeta" | xargs ls -lh 2>/dev/null | awk '$5 ~ /[0-9]+M/ && $5+0 > 10' || echo "   No large object files found"

echo ""
echo "✅ Analysis complete!"
echo "📊 Results saved to target/analysis/"
echo ""
echo "🎯 Optimization priorities:"
echo "   1. Review timing report for slowest crates"
echo "   2. Check monomorphization report for generic hotspots"
echo "   3. Remove unused dependencies"
echo "   4. Consider splitting large crates"
echo "   5. Optimize feature flag combinations"
echo ""
echo "💡 Next steps:"
echo "   - Review target/analysis/build-timings.log"
echo "   - Check target/analysis/mono-items.log for generic hotspots"
echo "   - Run 'cargo machete' to remove unused dependencies"
echo "   - Consider using trait objects at crate boundaries"
