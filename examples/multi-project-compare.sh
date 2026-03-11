#!/bin/bash
# Multi-project comparison
#
# Compares feature flag states across multiple PostHog projects.
# Useful for verifying staging vs production flag consistency.
#
# Usage: ./multi-project-compare.sh <project-id-1> <project-id-2> [project-id-3...]

set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <project-id-1> <project-id-2> [project-id-3...]"
  echo "Example: $0 12345 67890"
  exit 1
fi

PROJECTS=("$@")

echo "=== Feature Flag Comparison ==="
echo "Projects: ${PROJECTS[*]}"
echo ""

# Collect flags from each project
declare -A ALL_FLAGS

for PROJECT_ID in "${PROJECTS[@]}"; do
  echo "--- Project ${PROJECT_ID} ---"
  FLAGS=$(posthog flags list -p "$PROJECT_ID" --all-pages -f json 2>/dev/null) || {
    echo "  Error fetching flags (check auth for project ${PROJECT_ID})"
    continue
  }

  echo "$FLAGS" | jq -r '.data.results[] | "  \(.key): active=\(.active), rollout=\(.rollout_percentage // "N/A")%"'
  echo ""
done

# Find differences
echo "=== Differences ==="
FIRST_PROJECT="${PROJECTS[0]}"
FIRST_FLAGS=$(posthog flags list -p "$FIRST_PROJECT" --all-pages -f json 2>/dev/null | jq -r '.data.results[].key' | sort)

for PROJECT_ID in "${PROJECTS[@]:1}"; do
  OTHER_FLAGS=$(posthog flags list -p "$PROJECT_ID" --all-pages -f json 2>/dev/null | jq -r '.data.results[].key' | sort)

  ONLY_FIRST=$(comm -23 <(echo "$FIRST_FLAGS") <(echo "$OTHER_FLAGS"))
  ONLY_OTHER=$(comm -13 <(echo "$FIRST_FLAGS") <(echo "$OTHER_FLAGS"))

  if [ -n "$ONLY_FIRST" ]; then
    echo "Only in project ${FIRST_PROJECT}:"
    echo "$ONLY_FIRST" | sed 's/^/  /'
  fi

  if [ -n "$ONLY_OTHER" ]; then
    echo "Only in project ${PROJECT_ID}:"
    echo "$ONLY_OTHER" | sed 's/^/  /'
  fi

  if [ -z "$ONLY_FIRST" ] && [ -z "$ONLY_OTHER" ]; then
    echo "Projects ${FIRST_PROJECT} and ${PROJECT_ID}: flags match"
  fi
done
