#!/usr/bin/env bash
set -euo pipefail

# Summarize Rust tarpaulin XMLs and optional JS lcov into a simple JSON.
# Usage: coverage-summary.sh <rust_glob_root> <js_lcov_dir> > summary.json

RUST_ROOT=${1:-artifacts/rust}
JS_DIR=${2:-artifacts/js}

to_int() { awk '{printf "%d\n", $1}'; }

rust_total=0
rust_covered=0
shopt -s globstar nullglob
for f in ${RUST_ROOT}/**/tarpaulin-report.xml; do
  lines=$(xmllint --xpath 'count(//line)' "$f" 2>/dev/null || echo 0)
  hits=$(xmllint --xpath 'count(//line[@hits>0])' "$f" 2>/dev/null || echo 0)
  rust_total=$(echo "$rust_total + $lines" | bc)
  rust_covered=$(echo "$rust_covered + $hits" | bc)
done

js_total=0
js_covered=0
if [ -f "$JS_DIR/lcov.info" ]; then
  while IFS= read -r line; do
    case "$line" in
      DA:*)
        IFS="," read -r _ rest <<< "$line"
        IFS="," read -r lnum hits <<< "${line#DA:}"
        js_total=$((js_total+1))
        if [ "$hits" -gt 0 ]; then js_covered=$((js_covered+1)); fi
        ;;
    esac
  done < "$JS_DIR/lcov.info"
fi

rust_pct=0
js_pct=0
if [ "$rust_total" != "0" ]; then rust_pct=$(echo "scale=2; 100*$rust_covered/$rust_total" | bc); fi
if [ "$js_total" != "0" ]; then js_pct=$(echo "scale=2; 100*$js_covered/$js_total" | bc); fi

cat <<JSON
{
  "rust": {"covered": $rust_covered, "total": $rust_total, "percent": $rust_pct},
  "js": {"covered": $js_covered, "total": $js_total, "percent": $js_pct}
}
JSON

