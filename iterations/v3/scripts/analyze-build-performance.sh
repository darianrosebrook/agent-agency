#!/usr/bin/env bash
set -euo pipefail

# Build performance analysis script
# Helps identify bottlenecks in Rust compilation

echo "🔍 Rust Build Performance Analysis"
echo "=================================="

# Check if we're in a Rust project
if [[ ! -f "Cargo.toml" ]]; then
    echo "Error: Not in a Rust project directory"
    exit 1
fi

# Function to run cargo with timing
run_with_timing() {
    local cmd="$1"
    local description="$2"
    
    echo ""
    echo "📊 $description"
    echo "Command: $cmd"
    echo "----------------------------------------"
    
    # Clear any existing timing data
    rm -rf target/.rustc-timing
    
    # Run with timing if nightly is available
    if rustc -vV | grep -q nightly; then
        echo "Using nightly rustc with timing..."
        time CARGO_PROFILE_TIMINGS=html "$cmd"
        
        # Check if timing report was generated
        if [[ -d "target/.rustc-timing" ]]; then
            echo "✅ Timing report generated in target/.rustc-timing/"
            echo "   Open target/.rustc-timing/index.html in your browser"
        fi
    else
        echo "Using stable rustc (no detailed timing available)..."
        time "$cmd"
    fi
}

# Function to analyze crate graph
analyze_crate_graph() {
    echo ""
    echo "📈 Crate Graph Analysis"
    echo "----------------------"
    
    if command -v cargo-tree >/dev/null 2>&1; then
        echo "Crate dependency tree:"
        cargo tree --duplicates
    else
        echo "Install cargo-tree for dependency analysis:"
        echo "  cargo install cargo-tree"
    fi
    
    echo ""
    echo "Workspace members:"
    cargo metadata --format-version 1 | jq -r '.workspace_members[]' | sort
}

# Function to check for common performance issues
check_performance_issues() {
    echo ""
    echo "⚠️  Performance Issue Checks"
    echo "---------------------------"
    
    # Check for heavy proc-macros
    echo "Checking for heavy proc-macros..."
    if grep -r "proc-macro = true" . --include="*.toml" >/dev/null 2>&1; then
        echo "Found proc-macros in:"
        grep -r "proc-macro = true" . --include="*.toml" | cut -d: -f1
    else
        echo "✅ No proc-macros found"
    fi
    
    # Check for large feature sets
    echo ""
    echo "Checking feature flags..."
    if grep -r "features = \[" . --include="*.toml" >/dev/null 2>&1; then
        echo "Found feature flags in:"
        grep -r "features = \[" . --include="*.toml" | head -5
    fi
    
    # Check for inline attributes
    echo ""
    echo "Checking for inline attributes..."
    if find . -name "*.rs" -exec grep -l "#\[inline" {} \; >/dev/null 2>&1; then
        echo "Found inline attributes in:"
        find . -name "*.rs" -exec grep -l "#\[inline" {} \; | head -5
    else
        echo "✅ No inline attributes found"
    fi
}

# Function to suggest optimizations
suggest_optimizations() {
    echo ""
    echo "💡 Optimization Suggestions"
    echo "-------------------------"
    
    echo "1. Enable sccache for compiler caching:"
    echo "   ./scripts/setup-sccache.sh"
    echo ""
    
    echo "2. Use fast linkers:"
    echo "   - Linux: lld or mold"
    echo "   - macOS: ld64.lld or zld"
    echo ""
    
    echo "3. Use Cranelift for dev builds (nightly only):"
    echo "   RUSTFLAGS='-Zcodegen-backend=cranelift' cargo build"
    echo ""
    
    echo "4. Use cargo-nextest for faster testing:"
    echo "   cargo install cargo-nextest"
    echo "   cargo nextest run"
    echo ""
    
    echo "5. Profile specific crates:"
    echo "   cargo build -p <crate-name>"
    echo ""
    
    echo "6. Use unique target directories for agents:"
    echo "   ./scripts/cargo-agent-wrapper.sh dev"
}

# Main analysis
echo "Starting build performance analysis..."

# Check basic setup
echo ""
echo "🔧 Build Environment"
echo "-------------------"
echo "Rust version: $(rustc --version)"
echo "Cargo version: $(cargo --version)"
echo "Platform: $(rustc -vV | sed -n 's/^host: //p')"

# Check for sccache
if command -v sccache >/dev/null 2>&1; then
    echo "✅ sccache available: $(sccache --version | head -1)"
    sccache --show-stats | head -10
else
    echo "❌ sccache not available (recommended for faster builds)"
fi

# Check for fast linkers
echo ""
echo "🔗 Linker Configuration"
echo "----------------------"
case "$(rustc -vV | sed -n 's/^host: //p')" in
    *linux*)
        if command -v lld >/dev/null 2>&1; then
            echo "✅ lld available"
        else
            echo "❌ lld not available (recommended for Linux)"
        fi
        ;;
    *darwin*)
        if command -v ld64.lld >/dev/null 2>&1; then
            echo "✅ ld64.lld available"
        elif command -v zld >/dev/null 2>&1; then
            echo "✅ zld available"
        else
            echo "❌ Fast linker not available (ld64.lld or zld recommended)"
        fi
        ;;
esac

# Run analysis
analyze_crate_graph
check_performance_issues

# Run build with timing
run_with_timing "cargo check" "Full workspace check"

# Run package-specific builds
echo ""
echo "📦 Package-specific builds"
echo "-------------------------"
for package in $(cargo metadata --format-version 1 | jq -r '.workspace_members[]' | sed 's/.* //'); do
    if [[ -d "$package" ]]; then
        run_with_timing "cargo check -p $package" "Check $package"
    fi
done

# Suggest optimizations
suggest_optimizations

echo ""
echo "🎯 Analysis complete!"
echo "Check the timing reports in target/.rustc-timing/ for detailed breakdowns"