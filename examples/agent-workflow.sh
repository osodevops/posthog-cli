#!/bin/bash
# AI agent workflow example
#
# Demonstrates how an AI agent would use posthog-cli as a tool:
# - Check authentication
# - Read data with error handling
# - Create resources conditionally
# - Use exit codes for control flow
#
# Usage: ./agent-workflow.sh

set -euo pipefail

echo "=== Agent Workflow Demo ==="

# Step 1: Verify authentication
echo ""
echo "Step 1: Checking authentication..."
AUTH=$(posthog auth status -f json 2>/dev/null) || {
  echo "ERROR: Not authenticated. Run: posthog auth login"
  exit 2
}
echo "$AUTH" | jq '{authenticated: .data.authenticated, host: .data.host}'

# Step 2: Read-check-create pattern for feature flags
echo ""
echo "Step 2: Ensuring feature flag exists..."
FLAG_KEY="agent-demo-flag"

if FLAG=$(posthog flags get "$FLAG_KEY" -f json 2>/dev/null); then
  echo "Flag already exists:"
  echo "$FLAG" | jq '{key: .data.key, active: .data.active, rollout: .data.rollout_percentage}'
else
  EXIT_CODE=$?
  if [ $EXIT_CODE -eq 4 ]; then
    echo "Flag not found, creating..."
    posthog flags create --key "$FLAG_KEY" --name "Agent Demo Flag" --rollout 50 --active -f json \
      | jq '{key: .data.key, active: .data.active, rollout: .data.rollout_percentage}'
  else
    echo "Unexpected error (exit ${EXIT_CODE})"
    exit $EXIT_CODE
  fi
fi

# Step 3: Query with retry on rate limit
echo ""
echo "Step 3: Running analytics query..."
MAX_RETRIES=3
for i in $(seq 1 $MAX_RETRIES); do
  if RESULT=$(posthog query sql "SELECT count() AS total_events FROM events WHERE timestamp > now() - INTERVAL 7 DAY" -f json 2>&1); then
    echo "$RESULT" | jq '.data.results'
    break
  else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 3 ]; then
      WAIT=$(echo "$RESULT" | jq -r '.error.retry_after_secs // 5')
      echo "Rate limited, waiting ${WAIT}s (attempt ${i}/${MAX_RETRIES})..."
      sleep "$WAIT"
    else
      echo "Query failed (exit ${EXIT_CODE}): $RESULT"
      break
    fi
  fi
done

# Step 4: Export data for processing
echo ""
echo "Step 4: Exporting active flags as JSONL..."
posthog flags list --active --all-pages -f jsonl | while read -r line; do
  KEY=$(echo "$line" | jq -r '.key')
  ROLLOUT=$(echo "$line" | jq -r '.rollout_percentage // "N/A"')
  echo "  ${KEY}: ${ROLLOUT}%"
done

echo ""
echo "=== Agent workflow complete ==="
