#!/usr/bin/env bash
# Concurrency Gates Setup
# @darianrosebrook

set -euo pipefail

set_concurrency_gates() {
    # Detect CPU count
    local cpu_count
    cpu_count=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    # Detect memory (in GB)
    local mem_gb
    if [[ "$OSTYPE" == "darwin"* ]]; then
        mem_gb=$(($(sysctl -n hw.memsize) / 1024 / 1024 / 1024))
    else
        mem_gb=$(($(grep MemTotal /proc/meminfo | awk '{print $2}') / 1024 / 1024))
    fi

    # Compilation: CPU-bound, scale with cores
    export COMPILE_JOBS="$cpu_count"

    # Linking: Memory-bound, conservative scaling
    if [[ $mem_gb -gt 32 ]]; then
        export LINK_JOBS="6"
    elif [[ $mem_gb -gt 16 ]]; then
        export LINK_JOBS="4"
    elif [[ $mem_gb -gt 8 ]]; then
        export LINK_JOBS="2"
    else
        export LINK_JOBS="1"
    fi

    # Testing: I/O-bound, scale with cores but leave headroom
    if [[ $cpu_count -gt 8 ]]; then
        export TEST_JOBS=$((cpu_count - 2))
    elif [[ $cpu_count -gt 2 ]]; then
        export TEST_JOBS=$((cpu_count - 1))
    else
        export TEST_JOBS="$cpu_count"
    fi

    # File operations: Sequential to avoid contention
    export FILE_JOBS="1"

    # Build jobs for Cargo (if applicable)
    export CARGO_BUILD_JOBS="$COMPILE_JOBS"
}

# Run setup if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    set_concurrency_gates
    echo "COMPILE_JOBS=$COMPILE_JOBS"
    echo "LINK_JOBS=$LINK_JOBS"
    echo "TEST_JOBS=$TEST_JOBS"
    echo "FILE_JOBS=$FILE_JOBS"
    echo "CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS"
fi
