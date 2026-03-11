# posthog-cli

[![CI](https://github.com/osodevops/posthog-cli/actions/workflows/test.yml/badge.svg)](https://github.com/osodevops/posthog-cli/actions/workflows/test.yml)
[![Release](https://github.com/osodevops/posthog-cli/actions/workflows/release.yml/badge.svg)](https://github.com/osodevops/posthog-cli/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Agent-first CLI for the [PostHog](https://posthog.com) analytics API. Designed for both human operators and AI agents, with structured JSON output, deterministic exit codes, and full API coverage.

## Install

### Homebrew (macOS / Linux)

```bash
brew install osodevops/tap/posthog-cli
```

### Shell installer (macOS / Linux)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/osodevops/posthog-cli/releases/latest/download/posthog-cli-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/osodevops/posthog-cli/releases/latest/download/posthog-cli-installer.ps1 | iex"
```

### GitHub Releases

Download pre-built binaries from [Releases](https://github.com/osodevops/posthog-cli/releases).

### Build from source

```bash
cargo install --git https://github.com/osodevops/posthog-cli.git
```

## Setup

### 1. Authenticate

```bash
# Interactive login — stores credentials in your OS keychain
posthog auth login

# Or use environment variables
export POSTHOG_TOKEN="phx_your_personal_api_key"
export POSTHOG_PROJECT_ID="12345"
```

### 2. Verify

```bash
posthog auth status
posthog auth whoami
```

### Auth chain

Credentials are resolved in order:

1. CLI flags (`--token`, `--host`, `--project`)
2. Environment variables (`POSTHOG_TOKEN`, `POSTHOG_HOST`, `POSTHOG_PROJECT_ID`)
3. OS keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service)
4. Config file (`~/.config/posthog/config.toml`)
5. Project file (`.posthog.toml` in current directory)

## Quick start

```bash
# List feature flags
posthog flags list

# Run a HogQL query
posthog query sql "SELECT count() FROM events WHERE timestamp > now() - INTERVAL 7 DAY"

# Get trends for an event
posthog query trends --event '$pageview' --interval day --date-from -7d

# Capture an event
posthog capture event --event "cli_test" --distinct-id "user-123"

# List experiments
posthog experiments list --status running

# Get dashboard as table
posthog dashboards get 42 -f table
```

## AI agent integration

`posthog-cli` is designed as a tool for AI agents and automation pipelines. Every command produces machine-readable output suitable for programmatic consumption.

### Structured output

All responses use a consistent JSON envelope:

```json
{
  "ok": true,
  "data": { ... },
  "meta": {
    "cached": false,
    "duration_ms": 142
  }
}
```

Errors follow the same structure:

```json
{
  "ok": false,
  "error": {
    "code": "auth_failed",
    "message": "Authentication failed: invalid token"
  }
}
```

### Deterministic exit codes

| Code | Meaning | Agent action |
|------|---------|-------------|
| 0 | Success | Parse `data` from stdout |
| 1 | General error | Log and report |
| 2 | Auth failed | Re-authenticate or check token |
| 3 | Rate limited | Wait and retry (see `retry_after_secs`) |
| 4 | Not found | Verify resource ID |
| 5 | Validation error | Fix input parameters |
| 6 | Query timeout | Increase timeout or simplify query |
| 7 | Server error | Retry later |

### Agent workflow example

```bash
#!/bin/bash
# Agent: check flag status, create if missing, evaluate for user

FLAG_KEY="new-onboarding"
DISTINCT_ID="user-456"

# Check if flag exists
result=$(posthog flags get "$FLAG_KEY" -f json 2>/dev/null)
if [ $? -eq 4 ]; then
  # Not found — create it
  posthog flags create --key "$FLAG_KEY" --name "New Onboarding Flow" --rollout 50 --active
fi

# Evaluate for a specific user
posthog flags evaluate "$FLAG_KEY" --distinct-id "$DISTINCT_ID" -f json
```

### Batch operations

```bash
# Export all feature flags
posthog flags list --all-pages -f json > flags.json

# Batch capture events from JSONL
posthog capture batch --file events.jsonl

# Query and pipe to jq
posthog query sql "SELECT event, count() FROM events GROUP BY event ORDER BY count() DESC LIMIT 10" -f json | jq '.data.results'
```

## Commands

| Command | Description |
|---------|-------------|
| `auth` | Manage authentication — login, logout, status, whoami, switch |
| `query` | Run HogQL queries — sql, trends, funnels, retention, status, cancel |
| `flags` | Feature flags — list, get, create, update, delete, evaluate |
| `insights` | Saved insights — list, get, create, update, delete |
| `dashboards` | Dashboards — list, get, create, update, delete |
| `annotations` | Annotations — list, get, create, update, delete |
| `capture` | Send events — event, batch, identify, group, alias |
| `experiments` | A/B tests — list, get, create, update, start, stop, results, delete |
| `surveys` | Surveys — list, get, create, update, launch, stop, archive, delete |
| `persons` | Persons — list, get, update, delete, delete-with-data, split, activity |
| `cohorts` | Cohorts — list, get, create, update, delete |
| `errors` | Error tracking — list, get, resolve, ignore, reopen |
| `actions` | Actions — list, get, create, update, delete |
| `definitions` | Event & property definitions — events (list/get/update/delete), properties (list/get/update) |
| `cache` | Local cache — stats, clear |
| `completions` | Shell completions — bash, zsh, fish, powershell |

Run `posthog <command> --help` for detailed usage of any command.

## Global options

| Flag | Short | Env var | Description |
|------|-------|---------|-------------|
| `--host` | `-H` | `POSTHOG_HOST` | PostHog instance URL |
| `--token` | `-t` | `POSTHOG_TOKEN` | Personal API key (`phx_...`) |
| `--project` | `-p` | `POSTHOG_PROJECT_ID` | Project/environment ID |
| `--format` | `-f` | `POSTHOG_FORMAT` | Output format: `json`, `table`, `csv`, `jsonl` |
| `--quiet` | `-q` | | Suppress non-data output |
| `--verbose` | `-v` | | Show HTTP request/response debug info |
| `--no-cache` | | | Bypass local response cache |
| `--timeout` | | `POSTHOG_TIMEOUT` | HTTP timeout in seconds (default: 30) |
| `--retry` | | | Retries on transient failures (default: 3) |
| `--page-size` | | | Pagination page size (default: 100) |
| `--all-pages` | | | Auto-paginate and return all results |

## Environment variables

| Variable | Description |
|----------|-------------|
| `POSTHOG_TOKEN` | Personal API key (`phx_...`) |
| `POSTHOG_HOST` | PostHog instance URL (default: `https://us.posthog.com`) |
| `POSTHOG_PROJECT_ID` | Project/environment ID |
| `POSTHOG_FORMAT` | Default output format |
| `POSTHOG_TIMEOUT` | HTTP timeout in seconds |

## Output formats

| Format | Flag | Description |
|--------|------|-------------|
| JSON | `-f json` | Default. Structured JSON envelope — best for agents and piping |
| Table | `-f table` | Human-readable table (auto-detected when stdout is a TTY) |
| CSV | `-f csv` | Comma-separated values for spreadsheet import |
| JSONL | `-f jsonl` | One JSON object per line — best for streaming and `jq` |

## Configuration

### Config file

`~/.config/posthog/config.toml`:

```toml
host = "https://us.posthog.com"
project_id = "12345"
```

### Project file

`.posthog.toml` in your project root:

```toml
host = "https://eu.posthog.com"
project_id = "67890"
```

## Shell completions

```bash
# Bash
posthog completions bash > ~/.local/share/bash-completion/completions/posthog

# Zsh
posthog completions zsh > ~/.zfunc/_posthog

# Fish
posthog completions fish > ~/.config/fish/completions/posthog.fish

# PowerShell
posthog completions powershell > posthog.ps1
```

## Man pages

Generate man pages locally:

```bash
cargo run --bin gen-man
man man/posthog.1
```

Man pages are generated for every command and subcommand (105 pages total).

## Building from source

```bash
git clone https://github.com/osodevops/posthog-cli.git
cd posthog-cli
cargo build --release
```

The binary is at `target/release/posthog`.

### Run tests

```bash
cargo test
```

### Run lints

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## License

[MIT](LICENSE)
