#!/bin/bash
# Feature flag progressive rollout
#
# Creates a feature flag and gradually increases rollout from 0% to 100%.
# Usage: ./feature-flag-rollout.sh <flag-key> <flag-name>

set -euo pipefail

FLAG_KEY="${1:?Usage: $0 <flag-key> <flag-name>}"
FLAG_NAME="${2:?Usage: $0 <flag-key> <flag-name>}"

echo "Creating feature flag: ${FLAG_KEY}"
posthog flags create --key "$FLAG_KEY" --name "$FLAG_NAME" --rollout 0 --active -f json

for pct in 10 25 50 75 100; do
  read -rp "Press enter to increase rollout to ${pct}%..."
  posthog flags update "$FLAG_KEY" --rollout "$pct" -f json | jq '{key: .data.key, rollout: .data.rollout_percentage}'
  echo "Rollout at ${pct}%"
done

echo "Flag ${FLAG_KEY} is now at 100% rollout"
