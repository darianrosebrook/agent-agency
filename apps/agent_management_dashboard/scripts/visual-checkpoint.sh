#!/bin/bash

# Visual Regression Checkpoint Script
# Captures baseline screenshots at migration checkpoints

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
CHECKPOINT_NAME="${1:-checkpoint-$(date +%Y%m%d-%H%M%S)}"
CHECKPOINT_DIR="$PROJECT_DIR/tests/visual-regression/checkpoints/$CHECKPOINT_NAME"

echo "Creating visual regression checkpoint: $CHECKPOINT_NAME"
echo "Checkpoint directory: $CHECKPOINT_DIR"

# Create checkpoint directory
mkdir -p "$CHECKPOINT_DIR/baselines"

# Check if dev server is running
if ! curl -s http://localhost:3000 > /dev/null; then
  echo "Starting dev server..."
  cd "$PROJECT_DIR"
  npm run dev &
  DEV_PID=$!
  echo "Waiting for server to start..."
  sleep 10
  
  # Wait for server to be ready
  for i in {1..30}; do
    if curl -s http://localhost:3000 > /dev/null; then
      break
    fi
    sleep 1
  done
fi

# Run Playwright tests to capture screenshots
cd "$PROJECT_DIR"
echo "Running visual regression tests..."
npx playwright test tests/visual-regression/visual.spec.ts --update-snapshots || true

# Copy screenshots to checkpoint directory
if [ -d "$PROJECT_DIR/tests/visual-regression/visual.spec.ts-snapshots" ]; then
  cp -r "$PROJECT_DIR/tests/visual-regression/visual.spec.ts-snapshots"/* "$CHECKPOINT_DIR/baselines/" 2>/dev/null || true
fi

# Create checkpoint metadata
cat > "$CHECKPOINT_DIR/metadata.json" << EOF
{
  "checkpoint_name": "$CHECKPOINT_NAME",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "git_commit": "$(git rev-parse HEAD)",
  "git_branch": "$(git rev-parse --abbrev-ref HEAD)",
  "migration_phase": "${MIGRATION_PHASE:-unknown}"
}
EOF

echo "Checkpoint created successfully!"
echo "Metadata: $CHECKPOINT_DIR/metadata.json"
echo "Screenshots: $CHECKPOINT_DIR/baselines/"

# Cleanup dev server if we started it
if [ ! -z "$DEV_PID" ]; then
  echo "Stopping dev server..."
  kill $DEV_PID 2>/dev/null || true
fi

