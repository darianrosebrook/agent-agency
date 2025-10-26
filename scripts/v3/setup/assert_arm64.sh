#!/usr/bin/env bash
set -euo pipefail
arch | grep -q "arm64" || { echo "ERROR: Not an arm64 shell (Rosetta?)."; exit 1; }
if rustc -vV | grep -q "x86_64-apple-darwin"; then
  echo "ERROR: Rust host is x86_64. Set target=aarch64-apple-darwin or switch toolchains."; exit 1;
fi
if which python >/dev/null 2>&1; then
  pyinfo=$(python - <<'PY'
import platform, struct
print(platform.machine(), struct.calcsize("P")*8)
PY
)
  [ "$pyinfo" = "arm64 64" ] || echo "WARN: Python is not arm64. OK if unused."
fi
echo "OK: arm64 environment."
