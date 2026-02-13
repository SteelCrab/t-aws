#!/usr/bin/env bash
# Description:
#   Coverage gate for large-scale tests.
#   - Scenario gate: 20 total / 17 automated
#   - Global workspace line coverage gate: cargo llvm-cov >= 85%
set -euo pipefail

SCENARIO_CATALOG="${SCENARIO_CATALOG:-phase2-test-scenarios.csv}"
MIN_LINES="${MIN_LINES:-85}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found."
  exit 1
fi

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not found. Install with:"
  echo "  cargo install cargo-llvm-cov --locked"
  exit 1
fi

echo "=== phase2 scenario gate ==="
bash ./scripts/check-phase2-scenarios.sh "$SCENARIO_CATALOG"

echo "=== global coverage gate ==="
cargo llvm-cov \
  --workspace \
  --all-features \
  --all-targets \
  --fail-under-lines "$MIN_LINES"

echo "Global coverage gate passed."
