#!/bin/bash
# Error tracking triage workflow
#
# Lists recent errors, shows details, and lets you resolve/ignore them.
# Usage: ./error-tracking-triage.sh

set -euo pipefail

echo "=== Error Triage ==="
echo ""

# List active errors from the last 7 days
echo "Active errors (last 7 days):"
ERRORS=$(posthog errors list --status active --date-from -7d --order-by last_seen -f json)
COUNT=$(echo "$ERRORS" | jq '.data.results | length')

if [ "$COUNT" -eq 0 ]; then
  echo "  No active errors. All clear!"
  exit 0
fi

echo "$ERRORS" | jq -r '.data.results[] | "  [\(.id)] \(.title // .type) — \(.occurrences) occurrences"'

echo ""
echo "Found ${COUNT} active errors."
echo ""

# Interactive triage
echo "$ERRORS" | jq -r '.data.results[].id' | while read -r ERROR_ID; do
  echo "---"
  echo "Error: ${ERROR_ID}"
  posthog errors get "$ERROR_ID" -f json | jq '{
    type: .data.type,
    message: .data.title,
    occurrences: .data.occurrences,
    first_seen: .data.first_seen,
    last_seen: .data.last_seen
  }'

  echo "Action? [r]esolve / [i]gnore / [s]kip"
  read -rp "> " ACTION
  case "$ACTION" in
    r) posthog errors resolve "$ERROR_ID" -q && echo "Resolved." ;;
    i) posthog errors ignore "$ERROR_ID" -q && echo "Ignored." ;;
    *) echo "Skipped." ;;
  esac
  echo ""
done
