# PostHog CLI (`posthog-rs`) — Product Requirements Document

## Executive Summary

This PRD defines `posthog-rs`, a comprehensive Rust-based CLI for PostHog that provides full programmatic access to PostHog's private API surface. The primary design goal is **agent-first interaction**: every command produces structured, machine-parseable output (JSON by default) suitable for AI agents operating from the command line, while remaining ergonomic for human operators. This CLI replaces the need for PostHog's MCP server in agent workflows, offering broader API coverage, lower latency (native binary vs. Node.js MCP proxy), and deterministic invocation semantics that agents can reliably compose into multi-step workflows.

### Why This Exists

PostHog's current official CLI (`@posthog/cli` v0.4.x) only supports three commands: `login`, `query`, and `sourcemap`. The official PostHog MCP server exposes 27 tools across 7 categories but still misses critical endpoints: surveys, experiments, cohorts, persons, session recordings, batch exports, annotations (partially), and data warehouse management. Community users report frustration with PostHog's UI navigation complexity, API rate limiting, and the inability to automate workflows programmatically. This CLI fills every gap with a single, zero-dependency binary.[^1][^2][^3][^4][^5]

### Design Principles

- **Agent-first, human-friendly**: JSON output by default; `--format table` for humans; `--format csv` for piping
- **Composable**: Every command is a Unix-style building block. Agents chain commands via stdout/stdin
- **Complete**: Every private API endpoint has a corresponding CLI command
- **Predictable**: Deterministic exit codes, structured errors, no interactive prompts in non-TTY mode
- **Fast**: Native Rust binary, async HTTP, connection pooling, local response caching

***

## Architecture

### Binary Name

```
posthog
```

### Global Flags

| Flag | Short | Env Var | Description |
|------|-------|---------|-------------|
| `--host <URL>` | `-H` | `POSTHOG_HOST` | PostHog instance URL (default: `https://us.posthog.com`) |
| `--token <KEY>` | `-t` | `POSTHOG_TOKEN` | Personal API key (`phx_...`) |
| `--project <ID>` | `-p` | `POSTHOG_PROJECT_ID` | Project/environment ID |
| `--format <FMT>` | `-f` | `POSTHOG_FORMAT` | Output format: `json` (default), `table`, `csv`, `jsonl` |
| `--quiet` | `-q` | | Suppress non-data output (progress, hints) |
| `--verbose` | `-v` | | Show HTTP request/response debug info |
| `--no-cache` | | | Bypass local response cache |
| `--timeout <SECS>` | | `POSTHOG_TIMEOUT` | HTTP timeout in seconds (default: 30) |
| `--retry <N>` | | | Number of retries on transient failures (default: 3) |
| `--page-size <N>` | | | Pagination page size (default: 100, max: 1000) |
| `--all-pages` | | | Automatically paginate and return all results |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication failure |
| 3 | Rate limited (includes `Retry-After` in stderr) |
| 4 | Resource not found |
| 5 | Validation error (bad input) |
| 6 | Query timeout |
| 7 | Server error (5xx) |

### Output Contract

All commands output a JSON envelope by default:

```json
{
  "ok": true,
  "data": { ... },
  "meta": {
    "request_id": "...",
    "cached": false,
    "duration_ms": 142,
    "pagination": { "next": "...", "count": 500 }
  }
}
```

Error responses:

```json
{
  "ok": false,
  "error": {
    "code": "rate_limited",
    "message": "Rate limit exceeded. Retry after 12s.",
    "retry_after_secs": 12
  }
}
```

***

## Authentication (`auth`)

PostHog uses personal API keys with scoped permissions for private endpoints. The CLI supports interactive login, environment variable auth, and keychain storage.[^6]

```
posthog auth login            # Interactive browser-based OAuth flow
posthog auth login --token    # Direct token input (non-interactive)
posthog auth status           # Show current auth status, scopes, project
posthog auth logout           # Remove stored credentials
posthog auth switch           # Switch active project/environment
posthog auth whoami           # Show current user details (/api/users/@me/)
```

### Auth Storage

- Credentials stored in OS keychain (macOS Keychain, Linux Secret Service, Windows Credential Manager)
- Fallback to `~/.config/posthog/credentials.json` (600 permissions)
- Environment variables (`POSTHOG_TOKEN`, `POSTHOG_HOST`, `POSTHOG_PROJECT_ID`) always take precedence[^1]
- Non-TTY detection: If stdin is not a terminal, all commands run non-interactively and require env vars or flags

***

## Query Engine (`query`)

The query endpoint (`POST /api/projects/:project_id/query/`) is the core data access layer, supporting HogQL (PostHog's SQL dialect), plus structured query types for trends, funnels, retention, and paths.[^6]

### HogQL Queries

```
posthog query sql "SELECT count() FROM events WHERE timestamp > now() - INTERVAL 7 DAY"
posthog query sql --file ./my_query.sql
posthog query sql --file ./my_query.sql --param start_date=2025-01-01 --param end_date=2025-03-01
posthog query sql "SELECT * FROM events LIMIT 1000" --name "daily_event_count"
```

### Structured Queries

```
posthog query trends --event pageview --interval day --date-from -7d
posthog query funnels --steps '$pageview' 'sign_up' 'purchase' --date-from -30d
posthog query retention --target-event sign_up --return-event purchase --period week
posthog query paths --start '$pageview' --end 'purchase' --date-from -14d
posthog query events --event '$pageview' --limit 500 --properties '{"$current_url": {"contains": "blog"}}'
```

### Async Query Support

PostHog queries can run asynchronously with polling. The CLI handles this transparently:[^6]

```
posthog query sql "..." --async                   # Submit and return query_id immediately
posthog query status <query_id>                    # Poll query status
posthog query result <query_id>                    # Fetch completed result
posthog query cancel <query_id>                    # Cancel running query (DELETE)
posthog query sql "..." --wait                     # Submit async, poll until complete (default behavior)
```

### Caching Control

The `refresh` parameter controls execution modes:[^6]

```
posthog query sql "..." --refresh blocking         # Default: sync unless cached
posthog query sql "..." --refresh force_cache      # Only return cached results
posthog query sql "..." --refresh force_blocking   # Always execute synchronously
posthog query sql "..." --refresh async            # Async unless cached
```

### Query Log Inspection

Every query is logged in PostHog's `query_log` table:[^6]

```
posthog query log --limit 50                       # Recent queries
posthog query log --name "daily_active_users"      # Filter by name
```

***

## Feature Flags (`flags`)

Full CRUD for feature flags plus evaluation. PostHog's feature flag API supports boolean flags, multivariate flags with variants, rollout percentages, and property-based targeting.[^2][^7]

```
posthog flags list                                                # List all flags
posthog flags list --active                                       # Only active flags
posthog flags list --search "beta"                                # Search by key/name
posthog flags get <key_or_id>                                     # Get flag definition
posthog flags create --key new-feature --name "New Feature" \
  --rollout 50 --active                                           # Create with 50% rollout
posthog flags update <key> --rollout 100                          # Update rollout to 100%
posthog flags update <key> --active false                         # Disable flag
posthog flags delete <key>                                        # Delete flag
posthog flags evaluate <key> --distinct-id user@example.com       # Evaluate for a user
posthog flags evaluate-all --distinct-id user@example.com         # Evaluate all flags for user
```

### Bulk Operations (Agent-Optimised)

```
posthog flags bulk-update --file flags.json        # Bulk update from JSON file
posthog flags export --output flags_backup.json    # Export all flag definitions
posthog flags import --file flags_backup.json      # Import flags (idempotent upsert)
posthog flags diff <key> --from-env staging --to-env production   # Compare flag across environments
```

***

## Experiments (`experiments`)

Full lifecycle management for A/B tests and experiments. The experiments API supports holdouts, metrics configuration, scheduling, and results analysis.[^8]

```
posthog experiments list                                          # List all experiments
posthog experiments list --status running                         # Filter by status
posthog experiments get <id>                                      # Get experiment details + results
posthog experiments create --name "Checkout Flow v2" \
  --feature-flag-key checkout-v2 \
  --description "Testing new checkout" \
  --metrics '{"primary": [{"kind": "ExperimentTrendsQuery", ...}]}'
posthog experiments update <id> --description "Updated desc"
posthog experiments start <id>                                    # Set start_date to now
posthog experiments stop <id>                                     # Set end_date to now
posthog experiments conclude <id> --winner variant-a \
  --comment "Variant A showed 12% lift"
posthog experiments archive <id>
posthog experiments delete <id>
posthog experiments results <id>                                  # Fetch computed results
```

### Holdouts

```
posthog experiments holdouts list
posthog experiments holdouts create --name "Global Holdout" --percentage 10
posthog experiments holdouts get <id>
posthog experiments holdouts update <id> --percentage 5
posthog experiments holdouts delete <id>
```

***

## Surveys (`surveys`)

Complete survey management. The surveys API supports multiple question types (open text, rating, single/multiple choice, NPS, link), targeting rules, scheduling, iteration, and response sampling.[^9][^10]

```
posthog surveys list
posthog surveys list --status active
posthog surveys get <id>
posthog surveys create --name "NPS Survey" \
  --questions '[{"type": "rating", "question": "How likely are you to recommend us?", "scale": 10}]' \
  --targeting '{"url_contains": "/dashboard"}'
posthog surveys update <id> --end-date 2026-04-01
posthog surveys launch <id>                                       # Set start_date
posthog surveys stop <id>                                         # Set end_date
posthog surveys archive <id>
posthog surveys delete <id>
posthog surveys responses <id> --date-from -30d                   # Fetch survey responses via HogQL
```

***

## Insights (`insights`)

Full insight management including creation, querying, sharing, and alert thresholds. Insights are PostHog's saved analytical queries.[^11][^2]

```
posthog insights list                                             # List all insights
posthog insights list --saved --search "revenue"                  # Search saved insights
posthog insights list --favorited                                 # List favourited insights
posthog insights get <id>                                         # Get insight definition + cached result
posthog insights get <id> --refresh blocking                      # Force fresh computation
posthog insights create --name "Daily Active Users" \
  --query '{"kind": "TrendsQuery", ...}'
posthog insights update <id> --name "Updated Name" --tags '["production"]'
posthog insights delete <id>
posthog insights activity <id>                                    # Audit log for insight
posthog insights my-last-viewed                                   # Last viewed insights
```

### Insight Sharing

```
posthog insights sharing get <id>                                 # Get sharing config
posthog insights sharing enable <id>                              # Enable public sharing
posthog insights sharing disable <id>
posthog insights sharing set-password <id> --password "secret"
posthog insights sharing refresh <id>                             # Refresh shared data
```

### Insight Alerts/Thresholds

```
posthog insights thresholds list <insight_id>
posthog insights thresholds get <insight_id> <threshold_id>
posthog insights thresholds create <insight_id> \
  --name "High bounce rate" \
  --type absolute --upper 0.7
```

***

## Dashboards (`dashboards`)

Full dashboard lifecycle management.[^12][^2]

```
posthog dashboards list
posthog dashboards list --pinned --search "product"
posthog dashboards get <id>
posthog dashboards create --name "Product KPIs" \
  --description "Core product metrics" --pinned
posthog dashboards update <id> --name "Updated KPIs" --tags '["team-alpha"]'
posthog dashboards delete <id>
posthog dashboards add-insight <dashboard_id> <insight_id>
posthog dashboards remove-insight <dashboard_id> <tile_id>
```

### Dashboard Sharing

```
posthog dashboards sharing get <id>
posthog dashboards sharing enable <id>
posthog dashboards sharing disable <id>
posthog dashboards sharing refresh <id>
```

***

## Persons (`persons`)

Person management for reading, updating properties, splitting, and deleting. Person creation is done via the capture API.[^13][^14]

```
posthog persons list                                              # List persons (paginated)
posthog persons list --search "john@example.com"
posthog persons list --properties '{"plan": "enterprise"}'
posthog persons get <id>                                          # Get person by ID
posthog persons get --distinct-id "user@example.com"              # Get by distinct_id
posthog persons update <id> --set '{"role": "admin"}'             # Set properties
posthog persons update <id> --unset '["temp_flag"]'               # Unset properties
posthog persons delete <id>                                       # Delete person (GDPR)
posthog persons delete-with-data <id>                             # Delete person + all events
posthog persons split <id>                                        # Split a merged person
posthog persons merge <id1> <id2>                                 # Merge two persons
posthog persons activity <id>                                     # Activity log
```

***

## Cohorts (`cohorts`)

Cohort management with support for both dynamic and static cohorts.[^15][^16]

```
posthog cohorts list
posthog cohorts get <id>
posthog cohorts create --name "Power Users" \
  --filters '{"properties": {"type": "AND", "values": [...]}}'
posthog cohorts create-static --name "Beta Testers" \
  --distinct-ids '["user1@test.com", "user2@test.com"]'
posthog cohorts update <id> --name "Updated Cohort"
posthog cohorts delete <id>
posthog cohorts add-persons <id> --distinct-ids '["user3@test.com"]'     # Static cohorts
posthog cohorts remove-person <id> --distinct-id "user3@test.com"
posthog cohorts calculation-history <id>                                  # View calculation runs
posthog cohorts export <id> --format csv                                  # Export cohort members
```

***

## Annotations (`annotations`)

Annotations mark significant events on PostHog charts (deploys, launches, incidents).[^17][^18]

```
posthog annotations list
posthog annotations list --search "deploy"
posthog annotations get <id>
posthog annotations create --content "v2.1.0 deployed" \
  --date "2026-03-11T09:00:00Z" --scope project
posthog annotations create --content "Incident: DB failover" \
  --date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --scope organization
posthog annotations update <id> --content "v2.1.0 deployed (hotfix)"
posthog annotations delete <id>
```

### Agent-Friendly: Auto-Annotate on Deploy

```
posthog annotations create \
  --content "Deploy $(git rev-parse --short HEAD): $(git log -1 --pretty=%s)" \
  --date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --scope project
```

***

## Session Recordings (`recordings`)

Session recording management. The API provides metadata and list access but not raw JSON replay data.[^19]

```
posthog recordings list                                           # List recordings
posthog recordings list --date-from -7d --date-to -1d
posthog recordings list --person-id <person_id>
posthog recordings list --min-duration 30 --has-errors
posthog recordings get <recording_id>                             # Get recording metadata
posthog recordings update <recording_id> --pinned true
posthog recordings delete <recording_id>
```

### Recording Playlists

```
posthog recordings playlists list
posthog recordings playlists get <short_id>
posthog recordings playlists create --name "Onboarding Issues" \
  --filters '{"events": [{"id": "rage_click"}]}'
posthog recordings playlists update <short_id> --pinned true
posthog recordings playlists delete <short_id>
posthog recordings playlists add <short_id> <recording_id>
posthog recordings playlists remove <short_id> <recording_id>
```

### Recording Sharing

```
posthog recordings sharing get <recording_id>
posthog recordings sharing enable <recording_id>
posthog recordings sharing disable <recording_id>
```

***

## Error Tracking (`errors`)

Error tracking for listing, inspecting, and managing error issues.[^2]

```
posthog errors list                                               # List error issues
posthog errors list --status active --date-from -7d
posthog errors list --order-by last_seen --order desc
posthog errors get <issue_id>                                     # Detailed error info
posthog errors get <issue_id> --date-from -24h                    # Scoped to time range
posthog errors resolve <issue_id>                                 # Mark as resolved
posthog errors ignore <issue_id>                                  # Mark as ignored
posthog errors reopen <issue_id>                                  # Reopen issue
```

***

## Actions (`actions`)

Actions combine related events into reusable analytics units.[^20]

```
posthog actions list
posthog actions get <id>
posthog actions create --name "Signed Up" \
  --steps '[{"event": "user_created"}, {"event": "$pageview", "url": "/welcome"}]'
posthog actions update <id> --name "User Signed Up"
posthog actions delete <id>
```

***

## Batch Exports (`exports`)

Full management of scheduled data exports to external destinations (S3, BigQuery, Snowflake, Postgres, Redshift).[^21][^22]

```
posthog exports list                                              # List batch exports
posthog exports get <id>
posthog exports create --name "Events to S3" \
  --destination '{"type": "S3", "config": {"bucket_name": "my-bucket", ...}}' \
  --interval "hour" \
  --model events
posthog exports update <id> --paused true
posthog exports delete <id>
posthog exports pause <id>
posthog exports unpause <id>
posthog exports backfill <id> --start "2026-03-01" --end "2026-03-10"
```

### Export Runs

```
posthog exports runs list <export_id>                             # List runs
posthog exports runs get <export_id> <run_id>                     # Run details
posthog exports runs cancel <export_id> <run_id>
posthog exports runs retry <export_id> <run_id>
posthog exports runs logs <export_id> <run_id>                    # View run logs
```

***

## Custom Endpoints (`endpoints`)

PostHog's Endpoints feature lets users create predefined queries exposed as API endpoints with optional materialization.[^23][^24]

```
posthog endpoints list
posthog endpoints get <name>
posthog endpoints create --name "daily_active_users" \
  --query '{"kind": "HogQLQuery", "query": "SELECT count(DISTINCT person_id) ..."}' \
  --cache-age 3600 --materialized
posthog endpoints update <name> --cache-age 7200
posthog endpoints delete <name>
posthog endpoints run <name>                                      # Execute endpoint
posthog endpoints run <name> --variables '{"start_date": "2026-03-01"}'
posthog endpoints openapi <name>                                  # Get OpenAPI spec
```

***

## Event & Property Definitions (`definitions`)

Manage event and property definitions for data governance.[^25]

```
posthog definitions events list
posthog definitions events list --search "purchase"
posthog definitions events get <name_or_id>
posthog definitions events update <id> --description "User completed purchase" \
  --tags '["revenue"]' --verified true
posthog definitions events hide <id>
posthog definitions events delete <id>

posthog definitions properties list
posthog definitions properties list --event-names '["$pageview"]'
posthog definitions properties get <name_or_id>
posthog definitions properties update <id> --description "User's plan tier"
```

***

## Organizations & Projects (`org`, `project`)

Organization and project management.[^26][^2]

```
posthog org list                                                  # List organisations
posthog org get                                                   # Get current org details
posthog org members                                               # List org members

posthog project list                                              # List projects in org
posthog project get                                               # Get current project details
posthog project get <id>
posthog project create --name "My New Project"
posthog project update <id> --name "Renamed Project"
posthog project delete <id>
posthog project switch <id>                                       # Switch active project
posthog project activity                                          # Project activity log
```

***

## Event Capture (`capture`)

Public POST-only endpoints for sending events, identifying users, and managing groups. These use the project API key (token), not the personal API key.[^6]

```
posthog capture event --event "purchase" --distinct-id "user123" \
  --properties '{"amount": 49.99, "currency": "GBP"}'
posthog capture batch --file events.jsonl                         # Batch capture from JSONL
posthog capture identify --distinct-id "user123" \
  --set '{"email": "john@example.com", "plan": "pro"}'
posthog capture group --group-type "company" --group-key "acme-inc" \
  --set '{"name": "Acme Inc", "employees": 150}'
posthog capture alias --distinct-id "user123" --alias "anon-abc"
```

***

## Source Maps (`sourcemap`)

Source map upload for error tracking (existing functionality from official CLI).[^27][^1]

```
posthog sourcemap inject <directory>                              # Inject PostHog metadata
posthog sourcemap upload <directory>                              # Upload source maps
posthog sourcemap inject-and-upload <directory>                   # Combined operation
posthog sourcemap upload <directory> \
  --release-name "web-app" --release-version "$(git rev-parse HEAD)"
```

***

## Data Warehouse (`warehouse`)

Materialized views and data warehouse table management.

```
posthog warehouse tables list                                     # List warehouse tables
posthog warehouse tables get <name>
posthog warehouse views list                                      # List materialized views
posthog warehouse views create --name "daily_metrics" \
  --query "SELECT toDate(timestamp) as day, count() as events FROM events GROUP BY day" \
  --schedule "daily"
posthog warehouse views refresh <name>                            # Trigger refresh
posthog warehouse views delete <name>
```

***

## Agent-Specific Features

These features specifically target AI agent workflows and address gaps in the existing MCP server.[^28][^2]

### Structured Discovery

Agents need to discover available resources before operating on them:

```
posthog discover                                                  # Full project inventory
posthog discover --resources flags,insights,experiments           # Specific resources
posthog discover schema events                                    # Show schema for events table
posthog discover schema persons                                   # Show schema for persons table
posthog discover properties --event '$pageview'                   # Properties for an event
```

### Watch Mode

Long-running observation for agent monitoring loops:

```
posthog watch events --filter '$event = "error"' --interval 30    # Poll every 30s
posthog watch errors --status active --interval 60
posthog watch flag <key> --interval 10                            # Monitor flag changes
posthog watch query "SELECT count() FROM events WHERE timestamp > now() - INTERVAL 5 MINUTE" \
  --interval 60 --alert-above 1000                                # Alert on threshold
```

### Pipeline Commands

Agents composing multi-step workflows:

```
# Get users from a cohort and check their flag values
posthog cohorts export 42 --format jsonl | \
  posthog flags evaluate-all --distinct-ids-from-stdin

# Find errors, get affected persons, check their recordings
posthog errors list --status active --date-from -24h -f jsonl | \
  jq -r '.distinct_id' | \
  posthog persons get --distinct-ids-from-stdin -f jsonl | \
  jq -r '.id' | \
  posthog recordings list --person-ids-from-stdin

# Export insight results to CSV for external analysis  
posthog insights get 123 --refresh blocking -f csv > report.csv
```

### Batch Operations

```
posthog batch --file operations.json                              # Execute multiple API calls
posthog batch --stdin                                             # Read operations from stdin
```

Operations file format:

```json
[
  {"command": "flags.create", "args": {"key": "feature-a", "rollout": 50}},
  {"command": "flags.create", "args": {"key": "feature-b", "rollout": 25}},
  {"command": "annotations.create", "args": {"content": "Batch deploy", "date": "now"}}
]
```

### Rate Limit Awareness

The CLI handles PostHog's rate limits (2400/hr for queries, 240/min for analytics, 480/min for CRUD) transparently:[^29][^6]

- Automatic backoff with jitter on HTTP 429
- `--respect-rate-limits` flag (default: true) queues requests to stay within limits
- `posthog rate-limits status` shows current usage against limits
- Stderr warnings when approaching rate limits

### Context File (`.posthog.toml`)

Project-level configuration for agents to discover:

```toml
[project]
host = "https://eu.posthog.com"
project_id = "12345"
default_date_range = "-30d"

[aliases]
dau = "query sql 'SELECT count(DISTINCT person_id) FROM events WHERE timestamp > now() - INTERVAL 1 DAY'"
weekly_retention = "query retention --target-event sign_up --return-event any --period week --date-from -90d"

[agent]
max_concurrent_queries = 2
output_format = "json"
cache_ttl_seconds = 300
```

***

## Configuration & Profiles

```
posthog config init                                               # Create .posthog.toml
posthog config show                                               # Show resolved config
posthog config set <key> <value>                                  # Set config value
posthog config profiles list                                      # List named profiles
posthog config profiles create staging --host https://eu.posthog.com --project 999
posthog config profiles use staging                               # Switch profile
```

***

## Completions & Help

```
posthog completion bash                                           # Generate shell completions
posthog completion zsh
posthog completion fish
posthog help mmand>                                            # Detailed help
posthog docs <topic>                                              # Open PostHog docs in browser
```

***

## Implementation Priorities

### Phase 1: Core (MVP)

- `auth` (login, status, logout, whoami)
- `query` (sql, trends, funnels, async, cancel)
- `flags` (full CRUD + evaluate)
- `insights` (list, get, create, update, delete)
- `dashboards` (list, get, create, update, delete)
- `annotations` (full CRUD)
- `capture` (event, identify, batch)
- Global flags, output formatting, exit codes
- `.posthog.toml` config file support

### Phase 2: Analytics & Management

- `experiments` (full CRUD + results + holdouts)
- `surveys` (full CRUD + responses)
- `persons` (list, get, update, delete, merge, split)
- `cohorts` (full CRUD + static management + export)
- `errors` (list, get, resolve, ignore)
- `actions` (full CRUD)
- `definitions` (events + properties)
- `sourcemap` (inject, upload)
- Shell completions

### Phase 3: Advanced & Agent Features

- `recordings` (list, get, playlists, sharing)
- `exports` (batch export management + runs)
- `endpoints` (custom endpoints CRUD)
- `warehouse` (tables, materialized views)
- `discover` (schema introspection)
- `watch` (polling/monitoring mode)
- `batch` (multi-operation execution)
- Pipeline stdin/stdout composition
- Rate limit awareness dashboard
- Profiles & multi-environment support

***

## Technical Requirements

### Rust Dependencies (Recommended Crates)

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing with derive macros |
| `reqwest` | Async HTTP client |
| `tokio` | Async runtime |
| `serde` / `serde_json` | JSON serialization |
| `tabled` | Table output formatting |
| `csv` | CSV output |
| `keyring` | OS keychain integration |
| `toml` | Config file parsing |
| `indicatif` | Progress bars (TTY mode) |
| `tracing` | Structured logging |
| `dirs` | XDG directory resolution |
| `chrono` | Date/time handling |

### API Client Architecture

```
src/
├── main.rs                    # Entry point, clap setup
├── cli/                       # Command definitions (one file per resource)
│   ├── auth.rs
│   ├── query.rs
│   ├── flags.rs
│   ├── experiments.rs
│   ├── surveys.rs
│   ├── insights.rs
│   ├── dashboards.rs
│   ├── persons.rs
│   ├── cohorts.rs
│   ├── annotations.rs
│   ├── recordings.rs
│   ├── errors.rs
│   ├── actions.rs
│   ├── exports.rs
│   ├── endpoints.rs
│   ├── definitions.rs
│   ├── capture.rs
│   ├── sourcemap.rs
│   ├── warehouse.rs
│   ├── discover.rs
│   ├── watch.rs
│   └── batch.rs
├── client/                    # HTTP client layer
│   ├── mod.rs                 # PostHogClient struct
│   ├── auth.rs                # Auth handling
│   ├── pagination.rs          # Auto-pagination
│   ├── rate_limit.rs          # Rate limit tracking
│   ├── cache.rs               # Local response cache
│   └── retry.rs               # Retry with backoff
├── models/                    # API request/response types
│   ├── query.rs
│   ├── flag.rs
│   ├── insight.rs
│   ├── experiment.rs
│   ├── survey.rs
│   ├── person.rs
│   ├── cohort.rs
│   ├── annotation.rs
│   ├── recording.rs
│   ├── error_tracking.rs
│   ├── action.rs
│   ├── export.rs
│   ├── endpoint.rs
│   └── common.rs              # Shared types (pagination, envelope)
├── output/                    # Output formatters
│   ├── json.rs
│   ├── table.rs
│   ├── csv.rs
│   └── jsonl.rs
└── config/                    # Configuration
    ├── mod.rs
    ├── credentials.rs
    ├── profiles.rs
    └── toml.rs
```

### Testing Strategy

- **Unit tests**: Model serialization, output formatting, config parsing
- **Integration tests**: Against PostHog Cloud sandbox project (env-gated)
- **Snapshot tests**: CLI output format regression testing with `insta`
- **CI**: GitHub Actions with cross-compilation (Linux x86_64, macOS ARM64, Windows x86_64)

### Distribution

- GitHub Releases with pre-built binaries for Linux, macOS (Intel + ARM), Windows
- Homebrew tap: `brew install posthog-rs/tap/posthog`
- Cargo: `cargo install posthog-cli`
- NPM wrapper (optional): `npx @posthog-rs/cli`
- Docker: `ghcr.io/posthog-rs/cli:latest`

***

## Competitive Comparison

| Capability | Official CLI (`@posthog/cli`) | PostHog MCP Server | `posthog-rs` (This PRD) |
|---|---|---|---|
| Language | Rust (NPM wrapper) | TypeScript (Cloudflare Worker) | Rust |
| Commands | 3 (`login`, `query`, `sourcemap`)[^1] | 27 tools[^2] | 100+ commands |
| Feature Flags | ❌ | ✅ CRUD[^2] | ✅ CRUD + evaluate + bulk + diff |
| Insights | ❌ | ✅ CRUD + SQL[^2] | ✅ CRUD + sharing + alerts |
| Dashboards | ❌ | ✅ CRUD[^2] | ✅ CRUD + sharing + tile mgmt |
| Experiments | ❌ | ❌ | ✅ Full lifecycle + holdouts |
| Surveys | ❌ | ❌ | ✅ Full CRUD + responses |
| Persons | ❌ | ❌ | ✅ Search + update + merge + GDPR delete |
| Cohorts | ❌ | ❌ | ✅ CRUD + static mgmt + export |
| Session Recordings | ❌ | ❌ | ✅ List + playlists + sharing |
| Error Tracking | ❌ | ✅ Basic (2 tools)[^2] | ✅ Full + resolve/ignore/reopen |
| Annotations | ❌ | via MCP | ✅ Full CRUD |
| Batch Exports | ❌ | ❌ | ✅ Full + runs + logs |
| Custom Endpoints | ❌ | ❌ | ✅ Full CRUD + execute |
| Data Warehouse | ❌ | ❌ | ✅ Tables + materialized views |
| Event Capture | ❌ | ❌ | ✅ Single + batch + identify |
| Agent Mode | ❌ | Implicit (MCP) | ✅ Explicit (JSON output, discover, watch, batch) |
| Async Queries | ❌ | ❌ | ✅ Submit + poll + cancel |
| Rate Limit Handling | ❌ | ❌ | ✅ Auto-backoff + status |
| Multi-Environment | ❌ | ❌ | ✅ Profiles + switch |
| Offline Config | ❌ | ❌ | ✅ `.posthog.toml` + aliases |
| Stdin Piping | ❌ | ❌ | ✅ Full pipeline support |

***

## Non-Goals

- **Real-time event streaming**: Use PostHog's real-time destinations for webhooks/Kafka
- **Embedded analytics rendering**: The CLI returns data; rendering is the agent's/user's responsibility
- **Self-hosted deployment management**: This is an API client, not a PostHog server management tool
- **Raw session replay JSON export**: PostHog's API does not expose this; the CLI works within API limitations[^19]
- **UI replacement**: The CLI complements the web UI, not replaces it

---

## References

1. [posthog/cli](https://www.npmjs.com/package/@posthog/cli) - The command line interface for PostHog 🦔. Latest version: 0.4.6, last published: 5 days ago. Start u...

2. [Master Your Data with PostHog MCP Server Insights | Growth Method](https://growthmethod.com/posthog-mcp-server/) - Discover how the PostHog MCP Server enables instant data insights and seamless conversations with yo...

3. [Limiting data pulled from Posthog in Airbyte Cloud](https://discuss.airbyte.io/t/limiting-data-pulled-from-posthog-in-airbyte-cloud/6999) - Summary Exploring options to limit data pulled from Posthog in Airbyte Cloud to avoid throttling by ...

4. [The UX is just terrible. New user rant!](https://www.reddit.com/r/posthog/comments/1qg7n2n/the_ux_is_just_terrible_new_user_rant/) - The UX is just terrible. New user rant!

5. [Any PM using Posthog?](https://www.reddit.com/r/ProductManagement/comments/1544s9t/any_pm_using_posthog/)

6. [API overview - Docs - PostHog](https://posthog.com/docs/api) - PostHog has a powerful API that enables you to capture, evaluate, create, update, and delete nearly ...

7. [Feature flags - Posthog Docs](https://scaling-devrel--posthog.netlify.app/docs/api/feature-flags) - For instructions on how to authenticate to use this endpoint, see API overview . PostHog provides yo...

8. [Experiments API Reference - PostHog](https://posthog.com/docs/api/experiments) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

9. [Creating surveys - Docs - PostHog](https://posthog-com-eight.vercel.app/docs/surveys/creating-surveys) - To create a new survey, go to the surveys tab in the PostHog app, and click on the "New survey" butt...

10. [Survey API Reference - PostHog](https://posthog.com/docs/api/surveys) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

11. [Insights API Reference - PostHog](https://posthog.com/docs/api/insights) - List all environments insights · Required API key scopes · Path parameters · Query parameters · Resp...

12. [Dashboards - Docs - PostHog](https://posthog.com/docs/product-analytics/dashboards) - Dashboards are the easiest way to track all your most important product and performance metrics. Unl...

13. [Persons API Reference - PostHog](https://posthog.com/docs/api/persons) - This endpoint is meant for reading and deleting persons. To create or update persons, we recommend u...

14. [Persons-4 API Reference - PostHog](https://posthog.com/docs/api/persons-4) - This endpoint is meant for reading and deleting persons. To create or update persons, we recommend u...

15. [PostHog - Introduction](https://docs.sim.ai/tools/posthog) - Product analytics and feature management

16. [Cohorts API Reference - PostHog](https://posthog.com/docs/api/cohorts) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

17. [Automating PostHog Annotations - Brian Morrison II](https://brianmorrison.me/blog/automating-posthog-annotations) - Personal blog of Brian Morrison II, full stack developer & content creator.

18. [Annotations API Reference - PostHog](https://posthog.com/docs/api/annotations) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

19. [Session recordings API Reference - PostHog](https://posthog.com/docs/api/session-recordings) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

20. [Actions - Docs - PostHog](https://posthog.com/docs/data/actions) - What is an action? Actions are a way of combining several related events into one, which you can the...

21. [Batch exports API Reference - PostHog](https://posthog.com/docs/api/batch-exports) - Batch exports. For instructions on how to authenticate to use this endpoint, see API overview. Endpo...

22. [Batch exports - Docs - PostHog](https://posthog.com/docs/cdp/batch-exports) - Batch exports give you a platform to schedule data exports to supported destinations. Batch exports ...

23. [Endpoints API Reference - PostHog](https://posthog.com/docs/api/endpoints) - The single platform for engineers to analyze, test, observe, and deploy new features. Product analyt...

24. [Endpoints - Docs - PostHog](https://posthog.com/docs/endpoints) - Overview. Endpoints enable you to create predefined queries from PostHog insights or SQL queries and...

25. [Event definitions API Reference - PostHog](https://posthog.com/docs/api/event-definitions) - Event definitions · List all event definitions · Create event definitions · Retrieve event definitio...

26. [Projects API Reference - PostHog](https://posthog.com/docs/api/projects) - Projects · Retrieve list · Create create · Retrieve retrieve · Update partial update · Delete destro...

27. [Upload source maps with CLI - Docs - PostHog](https://posthog.com/docs/error-tracking/upload-source-maps/cli) - Once you've built your application and have bundled assets, inject the context required by PostHog t...

28. [An AI Engineer's Guide to the Official PostHog MCP Server](https://skywork.ai/skypage/en/mastering-model-context-ai-engineer-guide-posthog-mcp-server/1978313891499077632) - Unlock the potential of your AI agents with the PostHog MCP Server! Streamline product analytics, en...

29. [Announcing our PostHog Integration - Growth Method](https://growthmethod.com/posthog-integration/) - Unlock the power of your data with PostHog integration—gain real-time insights, optimise growth, and...

