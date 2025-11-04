#!/bin/bash
echo "Detailed Error Analysis by Worker Assignment"
echo "============================================"

# Worker 1: Core Infrastructure
echo ""
echo "WORKER 1 - Core Infrastructure:"
echo "-------------------------------"
echo "system-federated-ml (532 errors):"
cargo check -p system-federated-ml 2>&1 | grep "^error" | head -10
echo ""
echo "data-interfaces (516 errors):"
cargo check -p data-interfaces 2>&1 | grep "^error" | head -10

# Worker 2: Testing & Orchestration  
echo ""
echo "WORKER 2 - Testing & Orchestration:"
echo "-----------------------------------"
echo "testing-validation (470 errors):"
cargo check -p testing-validation 2>&1 | grep "^error" | head -10
echo ""
echo "agent-orchestration (356 errors):"
cargo check -p agent-orchestration 2>&1 | grep "^error" | head -10

# Worker 3: Data & Research
echo ""
echo "WORKER 3 - Data & Research:"
echo "---------------------------"
echo "agent-research (139 errors):"
cargo check -p agent-research 2>&1 | grep "^error" | head -10
echo ""
echo "data-infrastructure (139 errors):"
cargo check -p data-infrastructure 2>&1 | grep "^error" | head -10
echo ""
echo "agent-workers (185 errors):"
cargo check -p agent-workers 2>&1 | grep "^error" | head -10

# Worker 4: Tools & Acceleration
echo ""
echo "WORKER 4 - Tools & Acceleration:"
echo "--------------------------------"
echo "agent-mcp (46 errors):"
cargo check -p agent-mcp 2>&1 | grep "^error" | head -10
echo ""
echo "development-tools (46 errors):"
cargo check -p development-tools 2>&1 | grep "^error" | head -10
echo ""
echo "system-acceleration (17 errors):"
cargo check -p system-acceleration 2>&1 | grep "^error" | head -10
echo ""
echo "engine-coreml (17 errors):"
cargo check -p engine-coreml 2>&1 | grep "^error" | head -10
echo ""
echo "agent-data-processing (33 errors):"
cargo check -p agent-data-processing 2>&1 | grep "^error" | head -10
