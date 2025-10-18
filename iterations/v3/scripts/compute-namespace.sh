#!/usr/bin/env bash
# Namespace Computation Script
# @darianrosebrook
#
# Computes deterministic namespace for agent isolation

set -euo pipefail

# Detect platform and architecture
detect_platform() {
    local os arch platform

    case "$(uname -s)" in
        Darwin)
            os="darwin"
            case "$(uname -m)" in
                arm64) arch="aarch64" ;;
                x86_64) arch="x86_64" ;;
                *) arch="unknown" ;;
            esac
            ;;
        Linux)
            os="linux"
            case "$(uname -m)" in
                x86_64) arch="x86_64" ;;
                aarch64) arch="aarch64" ;;
                *) arch="unknown" ;;
            esac
            ;;
        *)
            os="unknown"
            arch="unknown" ;;
    esac

    platform="${arch}-${os}"
    echo "$platform"
}

# Main namespace computation
compute_build_namespace() {
    local platform toolchain_hash project_hash

    platform="$(detect_platform)"

    # Toolchain hash (language-specific)
    toolchain_hash=""
    if command -v rustc >/dev/null 2>&1; then
        toolchain_hash="rust-$(rustc -vV | grep '^release:' | sed 's/release: //')"
    fi
    if command -v python3 >/dev/null 2>&1; then
        toolchain_hash="${toolchain_hash}_python-$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
    fi
    if command -v node >/dev/null 2>&1; then
        toolchain_hash="${toolchain_hash}_node-$(node --version | sed 's/v//')"
    fi

    # Project hash (lockfiles + source)
    if [[ -f "Cargo.lock" ]]; then
        project_hash="cargo-$(sha1sum Cargo.lock 2>/dev/null | cut -c1-8 || echo 'nolock')"
    elif [[ -f "package-lock.json" ]] || [[ -f "yarn.lock" ]] || [[ -f "pnpm-lock.yaml" ]]; then
        project_hash="node-$(find . -maxdepth 1 -name "*lock*" -type f -exec sha1sum {} \; 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'nolock')"
    elif [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]]; then
        project_hash="python-$(find . -maxdepth 1 -name "requirements*.txt" -o -name "pyproject.toml" | xargs sha1sum 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'noreq')"
    else
        project_hash="unknown-$(find . -name "*.rs" -o -name "*.py" -o -name "*.js" -o -name "*.ts" | head -10 | xargs sha1sum 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'nosrc')"
    fi

    # Clean and combine
    toolchain_hash="${toolchain_hash#_}"  # Remove leading underscore
    echo "${platform}_${toolchain_hash}_${project_hash}_${AGENT_ID:-unknown}"
}

# Export the namespace
export BUILD_NAMESPACE="$(compute_build_namespace)"
export AGENT_PLATFORM="$(detect_platform)"

# For sourcing this script
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    # Script was executed directly
    echo "$BUILD_NAMESPACE"
else
    # Script was sourced
    echo "BUILD_NAMESPACE=$BUILD_NAMESPACE"
    echo "AGENT_PLATFORM=$AGENT_PLATFORM"
fi

