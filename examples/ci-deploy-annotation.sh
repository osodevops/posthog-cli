#!/bin/bash
# CI/CD deployment annotation
#
# Creates a PostHog annotation marking a deployment. Add this to your
# CI/CD pipeline after a successful deploy.
#
# Usage: ./ci-deploy-annotation.sh <version> [environment]
# Requires: POSTHOG_TOKEN, POSTHOG_PROJECT_ID

set -euo pipefail

VERSION="${1:?Usage: $0 <version> [environment]}"
ENVIRONMENT="${2:-production}"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

CONTENT="Deployed ${VERSION} (${GIT_SHA}) to ${ENVIRONMENT}"

echo "Creating annotation: ${CONTENT}"
posthog annotations create \
  --content "$CONTENT" \
  --date "$TIMESTAMP" \
  --scope project \
  -f json | jq '{id: .data.id, content: .data.content, date_marker: .data.date_marker}'

echo "Annotation created at ${TIMESTAMP}"

# Optionally toggle a feature flag after deploy
if [ "${ENABLE_FLAG:-}" != "" ]; then
  echo "Enabling flag: ${ENABLE_FLAG}"
  posthog flags update "$ENABLE_FLAG" --active true -f json | jq '{key: .data.key, active: .data.active}'
fi
