# PostHog CLI — Agent integration guide

This guide explains how to use `posthog-cli` as a tool for AI agents, automation scripts, and CI/CD pipelines.

## Design principles

- **JSON by default** — All output is structured JSON when stdout is not a TTY (i.e., when piped or called from a script).
- **Deterministic exit codes** — Every error category maps to a specific exit code, enabling programmatic error handling without parsing error messages.
- **Consistent envelope** — Success and error responses share the same top-level structure (`ok`, `data`/`error`, `meta`).
- **Idempotent reads** — GET operations are cached locally with automatic TTL expiration. Use `--no-cache` to bypass.
- **Auto-pagination** — Use `--all-pages` to automatically fetch all pages of paginated results.

## Authentication for agents

Set environment variables for non-interactive use:

```bash
export POSTHOG_TOKEN="phx_your_personal_api_key"
export POSTHOG_HOST="https://us.posthog.com"    # or https://eu.posthog.com
export POSTHOG_PROJECT_ID="12345"
```

Or pass credentials inline:

```bash
posthog flags list -t phx_key -p 12345
```

Verify authentication programmatically:

```bash
posthog auth status -f json
# Returns: {"ok": true, "data": {"authenticated": true, "host": "...", "project_id": "..."}}
```

## Response format

### Success

```json
{
  "ok": true,
  "data": {
    "results": [...],
    "count": 42
  },
  "meta": {
    "cached": false,
    "duration_ms": 187
  }
}
```

### Error

```json
{
  "ok": false,
  "error": {
    "code": "not_found",
    "message": "Not found: Feature flag 'nonexistent' not found"
  }
}
```

### Rate limit error

```json
{
  "ok": false,
  "error": {
    "code": "rate_limited",
    "message": "Rate limited: retry after 5s",
    "retry_after_secs": 5
  }
}
```

## Exit codes

| Code | Constant | When | Recommended agent action |
|------|----------|------|--------------------------|
| 0 | `EXIT_SUCCESS` | Command succeeded | Parse stdout JSON |
| 1 | `EXIT_GENERAL` | Unexpected error, IO error, config error | Log error, alert operator |
| 2 | `EXIT_AUTH` | Invalid/expired token, no credentials | Refresh token, check env vars |
| 3 | `EXIT_RATE_LIMITED` | API rate limit hit | Sleep for `retry_after_secs`, then retry |
| 4 | `EXIT_NOT_FOUND` | Resource doesn't exist | Verify resource ID, create if needed |
| 5 | `EXIT_VALIDATION` | Invalid input (bad JSON, missing field) | Fix input parameters |
| 6 | `EXIT_QUERY_TIMEOUT` | Async query exceeded timeout | Increase `--timeout`, simplify query |
| 7 | `EXIT_SERVER` | PostHog API 5xx error | Exponential backoff and retry |

## Common agent patterns

### Read-check-create pattern

```bash
#!/bin/bash
set -euo pipefail

FLAG_KEY="feature-rollout"

# Try to get the flag
if ! result=$(posthog flags get "$FLAG_KEY" -f json 2>/dev/null); then
  exit_code=$?
  if [ $exit_code -eq 4 ]; then
    # Not found — create it
    result=$(posthog flags create --key "$FLAG_KEY" --name "Feature Rollout" --rollout 0)
  else
    echo "Unexpected error (exit $exit_code)" >&2
    exit $exit_code
  fi
fi

echo "$result" | jq '.data'
```

### Polling async queries

```bash
# Submit an async query
query_id=$(posthog query sql "SELECT count() FROM events" --async -f json | jq -r '.data.query_status.id')

# Poll until complete
while true; do
  status=$(posthog query status "$query_id" -f json)
  complete=$(echo "$status" | jq -r '.data.query_status.complete')
  if [ "$complete" = "true" ]; then
    posthog query result "$query_id" -f json
    break
  fi
  sleep 2
done
```

Or use the built-in `--wait` flag which handles polling automatically:

```bash
posthog query sql "SELECT count() FROM events" --wait -f json
```

### Bulk operations with JSONL

```bash
# Stream events from a file
posthog capture batch --file events.jsonl

# Export all flags as JSONL for processing
posthog flags list --all-pages -f jsonl | while read -r line; do
  key=$(echo "$line" | jq -r '.key')
  active=$(echo "$line" | jq -r '.active')
  echo "$key: $active"
done
```

### Error handling with retry

```bash
#!/bin/bash
max_retries=3
retry_count=0

while [ $retry_count -lt $max_retries ]; do
  if result=$(posthog flags list -f json 2>&1); then
    echo "$result"
    exit 0
  fi

  exit_code=$?
  case $exit_code in
    3)  # Rate limited
      wait_secs=$(echo "$result" | jq -r '.error.retry_after_secs // 5')
      sleep "$wait_secs"
      ;;
    7)  # Server error
      sleep $((2 ** retry_count))
      ;;
    *)  # Non-retryable
      echo "$result" >&2
      exit $exit_code
      ;;
  esac

  retry_count=$((retry_count + 1))
done

echo "Max retries exceeded" >&2
exit 1
```

### Multi-project workflows

```bash
# Compare flag states across projects
for project_id in 12345 67890; do
  echo "=== Project $project_id ==="
  posthog flags list -p "$project_id" --all-pages -f json | jq '.data.results[] | {key, active}'
done
```

### CI/CD integration

```bash
# In your deployment pipeline: toggle a flag after deploy
posthog flags update "new-feature" --active true -t "$POSTHOG_TOKEN" -p "$POSTHOG_PROJECT_ID"

# Create an annotation marking the deploy
posthog annotations create \
  --content "Deployed v2.3.1" \
  --date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --scope project
```

## Caching behaviour

Read operations are cached locally to reduce API calls. Cache keys include the host, project ID, and full command arguments.

| Behaviour | Flag |
|-----------|------|
| Use cache (default) | _(no flag)_ |
| Bypass cache | `--no-cache` |
| View cache stats | `posthog cache stats` |
| Clear all cache | `posthog cache clear` |

Write operations (`create`, `update`, `delete`, `capture`) are never cached.

## Performance tips

- Use `--quiet` (`-q`) to suppress spinners and hints — reduces stderr noise.
- Use `-f json` explicitly to avoid TTY detection overhead.
- Use `--all-pages` with `--page-size 250` for large exports.
- Use `--timeout 120` for long-running HogQL queries.
- Cache is per-host, per-project — safe to use across multiple projects.

## Tool definition for AI agents

If you're building an AI agent that uses `posthog-cli` as a tool, here's a tool definition template:

```json
{
  "name": "posthog",
  "description": "Interact with the PostHog analytics API. Returns structured JSON. Use for feature flags, analytics queries, experiments, persons, and event capture.",
  "parameters": {
    "command": {
      "type": "string",
      "description": "The full posthog CLI command to run (e.g., 'flags list --all-pages -f json')"
    }
  }
}
```

Key facts for the agent system prompt:

- Output is always JSON when piped (no TTY)
- Exit code 0 means success — parse `data` from stdout
- Exit codes 1-7 indicate specific error types — parse `error` from stderr
- Use `--all-pages` to get complete paginated results
- Use `-f json` to ensure JSON output
- The `meta.cached` field indicates whether the response came from local cache
