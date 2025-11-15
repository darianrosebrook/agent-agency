#!/usr/bin/env bash
# M1 Max MacBook Pro Build Environment Setup
# Ensures CoreML and torch-sys compile correctly with ARM64 Python
# @author: @darianrosebrook
# Target: M1 Max 64GB MacBook Pro

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_DIR="$ROOT_DIR/iterations/v3"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  M1 Max Build Environment Setup${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# 1. Verify we're on Apple Silicon
ARCH=$(uname -m)
if [ "$ARCH" != "arm64" ]; then
    echo -e "${YELLOW}⚠️  Warning: Not on ARM64 architecture (detected: $ARCH)${NC}"
    echo -e "${YELLOW}   This script is optimized for M1 Max MacBook Pro${NC}"
fi

# 2. Verify ARM64 Python
echo -e "${BLUE}[1/6] Verifying ARM64 Python installation...${NC}"
PYTHON3_PATH=$(which python3)
PYTHON_ARCH=$(python3 -c "import platform; print(platform.machine())" 2>/dev/null || echo "unknown")

if [ "$PYTHON_ARCH" != "arm64" ]; then
    echo -e "${RED}❌ Python is not ARM64 (detected: $PYTHON_ARCH)${NC}"
    echo -e "${YELLOW}   Current Python: $PYTHON3_PATH${NC}"
    echo -e "${YELLOW}   Please install ARM64 Python via Homebrew:${NC}"
    echo -e "${YELLOW}   brew install python@3.13${NC}"
    exit 1
fi
echo -e "${GREEN}✅ ARM64 Python confirmed: $PYTHON3_PATH ($PYTHON_ARCH)${NC}"

# 3. Verify libtorch-cpu exists
echo -e "${BLUE}[2/6] Verifying libtorch-cpu installation...${NC}"
LIBTORCH_CPU="$ROOT_DIR/libtorch-cpu"
if [ ! -d "$LIBTORCH_CPU" ]; then
    echo -e "${RED}❌ libtorch-cpu not found at: $LIBTORCH_CPU${NC}"
    exit 1
fi
if [ ! -f "$LIBTORCH_CPU/lib/libtorch.dylib" ]; then
    echo -e "${RED}❌ libtorch.dylib not found in $LIBTORCH_CPU/lib/${NC}"
    exit 1
fi
echo -e "${GREEN}✅ libtorch-cpu found: $LIBTORCH_CPU${NC}"

# 4. Verify Xcode Command Line Tools
echo -e "${BLUE}[3/6] Verifying Xcode Command Line Tools...${NC}"
if ! xcode-select -p &>/dev/null; then
    echo -e "${RED}❌ Xcode Command Line Tools not installed${NC}"
    echo -e "${YELLOW}   Install with: xcode-select --install${NC}"
    exit 1
fi
XCODE_PATH=$(xcode-select -p)
echo -e "${GREEN}✅ Xcode Command Line Tools: $XCODE_PATH${NC}"

# 5. Verify C++ compiler supports C++17
echo -e "${BLUE}[4/6] Verifying C++17 compiler support...${NC}"
CXX_VERSION=$(clang++ --version | head -1)
if ! clang++ -std=c++17 -x c++ - -o /dev/null <<< "int main() { return 0; }" 2>/dev/null; then
    echo -e "${RED}❌ C++ compiler does not support C++17${NC}"
    exit 1
fi
echo -e "${GREEN}✅ C++17 support confirmed: $CXX_VERSION${NC}"

# 6. Create .env file with all required environment variables
echo -e "${BLUE}[5/6] Creating .env file with build environment...${NC}"
ENV_FILE="$ROOT_DIR/.env.build"
cat > "$ENV_FILE" <<EOF
# M1 Max Build Environment Configuration
# Generated: $(date)
# Target: M1 Max 64GB MacBook Pro
# @author: @darianrosebrook

# LibTorch Configuration (CPU-only for macOS compatibility)
export LIBTORCH="$LIBTORCH_CPU"
export LIBTORCH_CXX11_ABI=0
export CMAKE_PREFIX_PATH="$LIBTORCH_CPU"
export DYLD_LIBRARY_PATH="$LIBTORCH_CPU/lib:\${DYLD_LIBRARY_PATH:-}"

# C++17 Compiler Flags (required for torch-sys)
export CXXFLAGS="-std=c++17 -stdlib=libc++"
export CXX="clang++"
export CC="clang"

# Python Configuration (ARM64)
export PYTHON3_PATH="$PYTHON3_PATH"
export PYTHON_ARCH="$PYTHON_ARCH"

# Rust Build Configuration
export RUSTFLAGS="-C link-arg=-fuse-ld=/usr/local/bin/ld64.lld"
export CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER="clang++"

# CoreML Swift Bridge Path
export COREML_BRIDGE_PATH="$ROOT_DIR/models/languages/swift/coreml-bridge"

# macOS Deployment Target (for compatibility)
export MACOSX_DEPLOYMENT_TARGET="11.0"

# Disable IPHONEOS_DEPLOYMENT_TARGET (can interfere with macOS builds)
unset IPHONEOS_DEPLOYMENT_TARGET
EOF

echo -e "${GREEN}✅ Environment file created: $ENV_FILE${NC}"

# 7. Update .cargo/config.toml with C++17 flags
echo -e "${BLUE}[6/6] Updating Cargo configuration for C++17...${NC}"
CARGO_CONFIG="$V3_DIR/.cargo/config.toml"
mkdir -p "$(dirname "$CARGO_CONFIG")"

# Check if config exists, if not create it
if [ ! -f "$CARGO_CONFIG" ]; then
    cat > "$CARGO_CONFIG" <<EOF
[build]
target = "aarch64-apple-darwin"

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "codegen-units=16",
  "-C", "incremental=true",
]

[profile.dev]
opt-level = 1
debug = true
split-debuginfo = "packed"
EOF
fi

# Add C++17 configuration if not present
if ! grep -q "CXXFLAGS" "$CARGO_CONFIG"; then
    # Add environment variable passthrough for C++ flags
    cat >> "$CARGO_CONFIG" <<EOF

# C++17 support for torch-sys and CoreML
[env]
CXXFLAGS = "-std=c++17 -stdlib=libc++"
CXX = "clang++"
CC = "clang"
EOF
fi

echo -e "${GREEN}✅ Cargo configuration updated${NC}"

# Summary
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  Setup Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Environment variables configured:"
echo "  • LIBTORCH=$LIBTORCH_CPU"
echo "  • LIBTORCH_CXX11_ABI=0"
echo "  • CXXFLAGS=-std=c++17 -stdlib=libc++"
echo "  • Python: $PYTHON3_PATH ($PYTHON_ARCH)"
echo ""
echo "Next steps:"
echo "  1. Source the environment:"
echo "     ${YELLOW}source $ENV_FILE${NC}"
echo ""
echo "  2. Or add to your shell profile (~/.zshrc):"
echo "     ${YELLOW}source $ROOT_DIR/.env.build${NC}"
echo ""
echo "  3. Test the build:"
echo "     ${YELLOW}cd $V3_DIR${NC}"
echo "     ${YELLOW}cargo test --workspace --all-features --lib --no-run${NC}"
echo ""
echo "  4. Run readiness assessment:"
echo "     ${YELLOW}bash $ROOT_DIR/scripts/v3/assess/readiness-assessment.sh${NC}"
echo ""

