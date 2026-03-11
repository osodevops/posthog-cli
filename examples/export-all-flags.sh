#!/bin/bash
# Export all feature flags to JSON and JSONL
#
# Useful for backup, migration, or audit purposes.
# Usage: ./export-all-flags.sh [output-dir]

set -euo pipefail

OUTPUT_DIR="${1:-.}"

echo "Exporting feature flags..."

# Full JSON export
posthog flags list --all-pages -f json > "${OUTPUT_DIR}/flags-export.json"
echo "Saved ${OUTPUT_DIR}/flags-export.json"

# JSONL for streaming/processing
posthog flags list --all-pages -f jsonl > "${OUTPUT_DIR}/flags-export.jsonl"
echo "Saved ${OUTPUT_DIR}/flags-export.jsonl"

# Summary
COUNT=$(posthog flags list --all-pages -f json | jq '.data.results | length')
echo "Exported ${COUNT} flags"

# Active flags report
echo ""
echo "Active flags:"
posthog flags list --active --all-pages -f json | jq -r '.data.results[] | "  \(.key) — rollout: \(.rollout_percentage // "N/A")%"'
