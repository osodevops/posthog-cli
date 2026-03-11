#!/bin/bash
# Daily metrics report
#
# Queries key metrics from PostHog and outputs a summary.
# Usage: ./daily-metrics-report.sh
# Requires: POSTHOG_TOKEN, POSTHOG_PROJECT_ID

set -euo pipefail

DATE_FROM="-1d"

echo "=== Daily Metrics Report ==="
echo "Date: $(date -u +%Y-%m-%d)"
echo ""

echo "--- Pageviews ---"
posthog query trends --event '$pageview' --interval hour --date-from "$DATE_FROM" -f table

echo ""
echo "--- Unique Users ---"
posthog query sql "SELECT uniq(distinct_id) AS unique_users FROM events WHERE timestamp > now() - INTERVAL 1 DAY" -f table

echo ""
echo "--- Top Events ---"
posthog query sql "SELECT event, count() AS count FROM events WHERE timestamp > now() - INTERVAL 1 DAY GROUP BY event ORDER BY count DESC LIMIT 10" -f table

echo ""
echo "--- Active Feature Flags ---"
posthog flags list --active -f table

echo ""
echo "--- Active Experiments ---"
posthog experiments list --status running -f table
