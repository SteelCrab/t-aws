#!/usr/bin/env bash
# Description:
#   Phase 2 scenario catalog validation script.
#   Rules:
#   - total scenarios: >= 20
#   - automated=yes scenarios: >= 17
#   - every automated scenario must have a real test_ref function in src/
set -euo pipefail

CATALOG="${1:-phase2-test-scenarios.csv}"
MIN_TOTAL="${MIN_TOTAL:-20}"
MIN_AUTOMATED="${MIN_AUTOMATED:-17}"

if [[ ! -f "$CATALOG" ]]; then
  echo "Scenario catalog not found: $CATALOG"
  exit 1
fi

total="$(awk -F, 'NR > 1 && $1 != "" { count++ } END { print count + 0 }' "$CATALOG")"
automated="$(awk -F, 'NR > 1 && tolower($5) == "yes" { count++ } END { print count + 0 }' "$CATALOG")"

echo "Scenario catalog: $CATALOG"
echo "Total scenarios: $total (required: >= $MIN_TOTAL)"
echo "Automated scenarios: $automated (required: >= $MIN_AUTOMATED)"

if (( total < MIN_TOTAL )); then
  echo "Scenario total gate failed."
  exit 1
fi

if (( automated < MIN_AUTOMATED )); then
  echo "Automated scenario gate failed."
  exit 1
fi

find_test_ref() {
  local test_ref="$1"
  if command -v rg >/dev/null 2>&1; then
    rg -n "fn[[:space:]]+${test_ref}[[:space:]]*\\(" src >/dev/null
  else
    grep -RsnE "fn[[:space:]]+${test_ref}[[:space:]]*\\(" src >/dev/null
  fi
}

missing_refs=()
while IFS=, read -r id service priority test_type automated_flag test_ref description; do
  [[ "$id" == "id" ]] && continue

  automated_flag_lower="$(printf '%s' "$automated_flag" | tr '[:upper:]' '[:lower:]')"
  if [[ "$automated_flag_lower" != "yes" ]]; then
    continue
  fi

  if [[ -z "$test_ref" ]]; then
    missing_refs+=("${id}:missing_test_ref")
    continue
  fi

  if ! find_test_ref "$test_ref"; then
    missing_refs+=("${id}:${test_ref}")
  fi
done < "$CATALOG"

if (( ${#missing_refs[@]} > 0 )); then
  echo "Missing automated scenario test refs:"
  for missing in "${missing_refs[@]}"; do
    echo "  - $missing"
  done
  exit 1
fi

echo "Scenario gate passed."
