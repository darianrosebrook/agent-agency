#!/usr/bin/env bash
# Python Environment Setup
# @darianrosebrook

set -euo pipefail

setup_python() {
    if ! command -v python3 >/dev/null 2>&1; then
        echo "Python not found, skipping setup"
        return 0
    fi

    # Virtual environment path
    export PYTHON_VENV_PATH=".venv/${BUILD_NAMESPACE:-unknown}"
    export PYTHONPATH="${PYTHON_VENV_PATH}/lib/python$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')/site-packages"

    # Wheel cache (shared)
    export PIP_CACHE_DIR="${PIP_CACHE_DIR:-$HOME/.cache/pip}"
    export PIP_WHEEL_CACHE="${PIP_WHEEL_CACHE:-$HOME/.cache/pip/wheels}"

    # pytest cache (per-agent)
    export PYTEST_CACHE_DIR=".pytest_cache/${BUILD_NAMESPACE:-unknown}"
    export MYPY_CACHE_DIR=".mypy_cache/${BUILD_NAMESPACE:-unknown}"

    # Ensure cache directories exist
    mkdir -p "$PIP_CACHE_DIR" "$PIP_WHEEL_CACHE" "$PYTEST_CACHE_DIR" "$MYPY_CACHE_DIR"
}

# Run setup if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    setup_python
fi
