#!/bin/bash
# Batch capture events from various sources
#
# Demonstrates different ways to send events to PostHog.
# Usage: ./batch-capture-events.sh

set -euo pipefail

# Single event
echo "Capturing single event..."
posthog capture event \
  --event "cli_demo" \
  --distinct-id "user-demo-123" \
  --properties '{"source": "example_script", "version": "1.0"}'

# Identify a user
echo "Identifying user..."
posthog capture identify \
  --distinct-id "user-demo-123" \
  --set '{"name": "Demo User", "email": "demo@example.com", "plan": "pro"}'

# Set group properties
echo "Setting group properties..."
posthog capture group \
  --group-type "company" \
  --group-key "acme-inc" \
  --set '{"name": "Acme Inc", "industry": "Technology", "employees": 150}'

# Create alias
echo "Creating alias..."
posthog capture alias \
  --distinct-id "user-demo-123" \
  --alias "user-legacy-456"

# Batch from JSONL file
echo "Creating sample JSONL file..."
cat > /tmp/posthog-events.jsonl << 'EVENTS'
{"event": "page_viewed", "distinct_id": "user-1", "properties": {"page": "/home"}}
{"event": "page_viewed", "distinct_id": "user-2", "properties": {"page": "/pricing"}}
{"event": "button_clicked", "distinct_id": "user-1", "properties": {"button": "signup"}}
{"event": "form_submitted", "distinct_id": "user-2", "properties": {"form": "contact"}}
EVENTS

echo "Batch capturing from JSONL..."
posthog capture batch --file /tmp/posthog-events.jsonl

echo "Done! All events captured."
