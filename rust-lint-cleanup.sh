#!/usr/bin/env bash
# Description:
#   Rust 프로젝트 정리 스크립트.
#   순서: cargo clean -> cargo fmt -> cargo clippy --fix -> cargo clippy check
#   필요 시 테스트까지 연계.
set -euo pipefail

ALLOW_DIRTY="${ALLOW_DIRTY:-1}"
RUN_CLEAN="${RUN_CLEAN:-1}"
RUN_TESTS="${RUN_TESTS:-0}"
CLIPPY_ALL_FEATURES="${CLIPPY_ALL_FEATURES:-0}"
CLIPPY_ALL_TARGETS="${CLIPPY_ALL_TARGETS:-0}"
RUN_DEADCODE="${RUN_DEADCODE:-1}"
TARGETS="${TARGETS:-emd:emd}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust toolchain first."
  exit 1
fi

run_for_target() {
  local package="$1"
  local bin="$2"

  local extra_fix_flags=""
  local extra_check_flags=""
  if [[ "$ALLOW_DIRTY" == "1" ]]; then
    extra_fix_flags="--allow-dirty"
  fi
  if [[ "$CLIPPY_ALL_TARGETS" == "1" ]]; then
    extra_fix_flags="${extra_fix_flags} --all-targets"
    extra_check_flags="--all-targets"
  fi
  if [[ "$CLIPPY_ALL_FEATURES" == "1" ]]; then
    extra_fix_flags="${extra_fix_flags} --all-features"
    extra_check_flags="${extra_check_flags} --all-features"
  fi

  echo "=== rust lint cleanup start (${package}:${bin}) ==="
  echo "=== cargo clippy --fix ==="
  # shellcheck disable=SC2086
  cargo clippy --fix --bin "$bin" -p "$package" $extra_fix_flags

  echo "=== cargo clippy ==="
  # shellcheck disable=SC2086
  cargo clippy --bin "$bin" -p "$package" $extra_check_flags

  if [[ "$RUN_DEADCODE" == "1" ]]; then
    echo "=== cargo clippy (dead-code) ==="
    # shellcheck disable=SC2086
    cargo clippy --bin "$bin" -p "$package" $extra_check_flags -- -W dead-code
  fi
}

echo "=== rust lint cleanup start ==="
echo "Targets: ${TARGETS}"

echo "=== target split ==="
for target in ${TARGETS//,/ }; do
  [[ -z "$target" ]] && continue
  if [[ "$target" == *:* ]]; then
    target_package="${target%%:*}"
    target_bin="${target##*:}"
  else
    target_package="$target"
    target_bin="$target"
  fi
  run_for_target "$target_package" "$target_bin"
done

if [[ "$RUN_TESTS" == "1" ]]; then
  echo "=== cargo test ==="
  cargo test --all-targets --locked
fi

echo "=== rust lint cleanup done ==="
