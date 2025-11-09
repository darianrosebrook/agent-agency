#!/bin/bash
# Investigation script for worktree creation issue
# Checks worktree directory, git status, and execution logs

set -euo pipefail

WORKTREE_DIR="${WORKTREE_DIR:-/tmp/agent-agency-worktrees}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "=========================================="
echo "Worktree Creation Investigation"
echo "=========================================="
echo ""

echo "1. Checking worktree directory..."
if [ -d "$WORKTREE_DIR" ]; then
    echo "   ✅ Directory exists: $WORKTREE_DIR"
    WORKTREE_COUNT=$(find "$WORKTREE_DIR" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l | tr -d ' ')
    echo "   Found $WORKTREE_COUNT worktree(s)"
    
    if [ "$WORKTREE_COUNT" -gt 0 ]; then
        echo ""
        echo "   Worktrees:"
        find "$WORKTREE_DIR" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | while read -r wt; do
            echo "   - $(basename "$wt")"
            if [ -d "$wt/.git" ]; then
                echo "     ✅ Git repository found"
                cd "$wt" && git status --short 2>/dev/null | head -5 || echo "     ⚠️  No changes detected"
            else
                echo "     ❌ No git repository"
            fi
        done
    else
        echo "   ⚠️  Directory exists but is empty"
    fi
else
    echo "   ❌ Directory does not exist: $WORKTREE_DIR"
    echo "   This indicates worktrees are not being created"
fi

echo ""
echo "2. Checking git worktree list..."
cd "$PROJECT_ROOT" || exit 1
if git worktree list 2>/dev/null | grep -q "$WORKTREE_DIR"; then
    echo "   ✅ Git worktrees found:"
    git worktree list 2>/dev/null | grep "$WORKTREE_DIR" || true
else
    echo "   ❌ No git worktrees found in $WORKTREE_DIR"
fi

echo ""
echo "3. Checking for worktree creation in code..."
echo "   Searching for create_worktree calls..."
if grep -r "create_worktree" "$PROJECT_ROOT/iterations/v3/agent-orchestration/src" 2>/dev/null | grep -v "test" | head -5; then
    echo "   ✅ Found create_worktree calls"
else
    echo "   ⚠️  No create_worktree calls found"
fi

echo ""
echo "4. Checking execution flow..."
echo "   Checking if ParallelCoordinator is used..."
if grep -r "ParallelCoordinator" "$PROJECT_ROOT/iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs" 2>/dev/null | grep -q "execute_plan_parallel"; then
    echo "   ✅ ParallelCoordinator.execute_plan_parallel() is called"
else
    echo "   ⚠️  ParallelCoordinator may not be used"
fi

echo ""
echo "5. Checking PlanExecutor worktree handling..."
if grep -A 10 "get_worktree_path" "$PROJECT_ROOT/iterations/v3/agent-orchestration/src/planning/plan_executor.rs" 2>/dev/null | grep -q "TODO"; then
    echo "   ⚠️  Found TODO for worktree creation in PlanExecutor"
    echo "   This is the likely issue - PlanExecutor doesn't create worktrees"
else
    echo "   ✅ No TODO found (or already fixed)"
fi

echo ""
echo "=========================================="
echo "Investigation Complete"
echo "=========================================="

