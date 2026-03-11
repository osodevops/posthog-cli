# Command reference

Complete reference for all `posthog` CLI commands.

## auth

Manage authentication and credentials.

```bash
# Interactive login — prompts for host, token, project ID
posthog auth login

# Check current auth status
posthog auth status
posthog auth status -f json

# Show current user details
posthog auth whoami

# Switch active project
posthog auth switch --project 67890

# Remove stored credentials
posthog auth logout
```

## query

Run HogQL and structured analytics queries.

### sql

```bash
# Inline query
posthog query sql "SELECT count() FROM events WHERE timestamp > now() - INTERVAL 7 DAY"

# From file
posthog query sql --file query.sql

# With parameters
posthog query sql "SELECT count() FROM events WHERE event = {event:String}" --param event=page_viewed

# Async mode — returns query_id immediately
posthog query sql "SELECT * FROM events LIMIT 1000000" --async

# Wait mode — submits async, polls until complete
posthog query sql "SELECT * FROM events LIMIT 1000000" --wait

# Output as CSV
posthog query sql "SELECT event, count() FROM events GROUP BY event" -f csv
```

### trends

```bash
posthog query trends --event '$pageview' --interval day --date-from -7d
posthog query trends --event sign_up --interval week --date-from -30d --date-to -1d
```

### funnels

```bash
posthog query funnels --steps sign_up onboarding_complete first_purchase --date-from -30d
```

### retention

```bash
posthog query retention --target-event sign_up --return-event '$pageview' --period week --date-from -90d
```

### status / result / cancel

```bash
# Check async query status
posthog query status <query-id>

# Fetch completed results
posthog query result <query-id>

# Cancel a running query
posthog query cancel <query-id>
```

## flags

Manage feature flags.

```bash
# List all flags
posthog flags list
posthog flags list --active
posthog flags list --search "onboarding"
posthog flags list --all-pages -f jsonl

# Get a flag
posthog flags get my-flag-key
posthog flags get 12345

# Create
posthog flags create --key new-feature --name "New Feature" --rollout 50 --active

# Update
posthog flags update my-flag --rollout 100
posthog flags update my-flag --active false
posthog flags update my-flag --name "Updated Name"

# Delete
posthog flags delete my-flag

# Evaluate for a user
posthog flags evaluate my-flag --distinct-id user-123
posthog flags evaluate-all --distinct-id user-123
```

## insights

Manage saved insights.

```bash
posthog insights list
posthog insights list --search "revenue" --saved
posthog insights get 42
posthog insights get 42 --refresh blocking
posthog insights create --name "Daily Active Users" --query '{"kind": "TrendsQuery", ...}'
posthog insights update 42 --name "New Name" --tags '["important"]'
posthog insights delete 42
```

## dashboards

Manage dashboards.

```bash
posthog dashboards list
posthog dashboards list --pinned
posthog dashboards get 10
posthog dashboards create --name "KPIs" --description "Key metrics" --pinned
posthog dashboards update 10 --name "Updated KPIs"
posthog dashboards delete 10
```

## annotations

Manage annotations on charts.

```bash
posthog annotations list
posthog annotations list --search "deploy"
posthog annotations get 5
posthog annotations create --content "Deployed v2.0" --date "2025-06-01T12:00:00Z" --scope project
posthog annotations update 5 --content "Updated annotation"
posthog annotations delete 5
```

## capture

Send events, identify users, and manage groups.

```bash
# Single event
posthog capture event --event page_viewed --distinct-id user-123 --properties '{"page": "/home"}'

# Batch from JSONL file
posthog capture batch --file events.jsonl

# Identify user (set person properties)
posthog capture identify --distinct-id user-123 --set '{"name": "Alice", "plan": "pro"}'

# Set group properties
posthog capture group --group-type company --group-key acme --set '{"name": "Acme Inc"}'

# Create alias
posthog capture alias --distinct-id user-123 --alias legacy-user-456
```

## experiments

Manage A/B test experiments.

```bash
posthog experiments list
posthog experiments list --status running
posthog experiments get 7
posthog experiments create --name "Checkout Flow" --feature-flag-key checkout-v2 --description "Test new checkout"
posthog experiments update 7 --description "Updated description"
posthog experiments start 7
posthog experiments results 7
posthog experiments stop 7
posthog experiments delete 7
```

## surveys

Manage surveys.

```bash
posthog surveys list
posthog surveys list --status active
posthog surveys get 3
posthog surveys create --name "NPS Survey" --questions '[{"type": "rating", "question": "How likely are you to recommend us?", "scale": 10}]'
posthog surveys update 3 --name "Updated Survey"
posthog surveys launch 3
posthog surveys stop 3
posthog surveys archive 3
posthog surveys delete 3
```

## persons

Manage persons.

```bash
posthog persons list
posthog persons list --search "alice@example.com"
posthog persons list --properties '{"email": "alice@example.com"}'

# Get by UUID or distinct ID
posthog persons get <uuid>
posthog persons get --distinct-id user-123

# Update properties
posthog persons update <uuid> --set '{"plan": "enterprise"}'
posthog persons update <uuid> --unset '["old_property"]'

# Delete
posthog persons delete <uuid>
posthog persons delete-with-data <uuid>    # GDPR full deletion

# Split merged person
posthog persons split <uuid>

# Activity log
posthog persons activity <uuid>
```

## cohorts

Manage cohorts.

```bash
posthog cohorts list
posthog cohorts get 15
posthog cohorts create --name "Power Users" --filters '{"properties": {"type": "AND", "values": [...]}}'
posthog cohorts create --name "Static Cohort" --filters '{}' --is-static
posthog cohorts update 15 --name "Updated Name"
posthog cohorts delete 15
```

## errors

Track and manage error issues.

```bash
posthog errors list
posthog errors list --status active --date-from -7d --order-by last_seen
posthog errors get <issue-id>
posthog errors get <issue-id> --date-from -24h
posthog errors resolve <issue-id>
posthog errors ignore <issue-id>
posthog errors reopen <issue-id>
```

## actions

Manage actions (reusable event groups).

```bash
posthog actions list
posthog actions get 20
posthog actions create --name "Signed Up" --steps '[{"event": "sign_up"}]'
posthog actions update 20 --name "Updated Action"
posthog actions delete 20
```

## definitions

Manage event and property definitions.

```bash
# Event definitions
posthog definitions events list
posthog definitions events list --search "purchase"
posthog definitions events get <id>
posthog definitions events update <id> --description "User completed purchase" --verified true
posthog definitions events delete <id>

# Property definitions
posthog definitions properties list
posthog definitions properties list --search "email" --event-names '["sign_up"]'
posthog definitions properties get <id>
posthog definitions properties update <id> --description "User email address"
```

## cache

Manage local response cache.

```bash
# Show cache stats
posthog cache stats

# Clear all cached responses
posthog cache clear
```

## completions

Generate shell completions.

```bash
posthog completions bash > ~/.local/share/bash-completion/completions/posthog
posthog completions zsh > ~/.zfunc/_posthog
posthog completions fish > ~/.config/fish/completions/posthog.fish
posthog completions powershell > posthog.ps1
```

## Global options

These options work with every command:

| Flag | Short | Description |
|------|-------|-------------|
| `--host <URL>` | `-H` | PostHog instance URL |
| `--token <KEY>` | `-t` | Personal API key (`phx_...`) |
| `--project <ID>` | `-p` | Project/environment ID |
| `--format <FMT>` | `-f` | Output format: `json`, `table`, `csv`, `jsonl` |
| `--quiet` | `-q` | Suppress non-data output |
| `--verbose` | `-v` | Show HTTP debug info |
| `--no-cache` | | Bypass local cache |
| `--timeout <SECS>` | | HTTP timeout (default: 30) |
| `--retry <N>` | | Retries on transient failures (default: 3) |
| `--page-size <N>` | | Pagination page size (default: 100) |
| `--all-pages` | | Fetch all pages automatically |
