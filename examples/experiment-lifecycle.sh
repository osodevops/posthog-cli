#!/bin/bash
# Experiment lifecycle management
#
# Creates, starts, monitors, and stops an A/B test experiment.
# Usage: ./experiment-lifecycle.sh <experiment-name> <flag-key>

set -euo pipefail

NAME="${1:?Usage: $0 <experiment-name> <flag-key>}"
FLAG_KEY="${2:?Usage: $0 <experiment-name> <flag-key>}"

# Create the experiment
echo "Creating experiment: ${NAME}"
RESULT=$(posthog experiments create \
  --name "$NAME" \
  --feature-flag-key "$FLAG_KEY" \
  --description "A/B test created via CLI" \
  -f json)

EXP_ID=$(echo "$RESULT" | jq -r '.data.id')
echo "Created experiment ID: ${EXP_ID}"

# Start the experiment
read -rp "Press enter to start the experiment..."
posthog experiments start "$EXP_ID" -f json | jq '{id: .data.id, name: .data.name, start_date: .data.start_date}'
echo "Experiment started!"

# Monitor results
echo ""
echo "Checking results (run again anytime with: posthog experiments results ${EXP_ID})"
posthog experiments results "$EXP_ID" -f json | jq '.data'

# Stop when ready
read -rp "Press enter to stop the experiment..."
posthog experiments stop "$EXP_ID" -f json | jq '{id: .data.id, name: .data.name, end_date: .data.end_date}'
echo "Experiment stopped!"
