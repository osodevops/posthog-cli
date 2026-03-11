# Changelog

All notable changes to this project will be documented in this file.

## [0.8.0] - 2026-03-11

### Added

- **16 command groups** covering the full PostHog API surface:
  - `auth` — login, logout, status, whoami, switch
  - `query` — sql (HogQL), trends, funnels, retention, async status/result/cancel
  - `flags` — list, get, create, update, delete, evaluate, evaluate-all
  - `insights` — list, get, create, update, delete
  - `dashboards` — list, get, create, update, delete
  - `annotations` — list, get, create, update, delete
  - `capture` — event, batch, identify, group, alias
  - `experiments` — list, get, create, update, start, stop, results, delete
  - `surveys` — list, get, create, update, launch, stop, archive, delete
  - `persons` — list, get, update, delete, delete-with-data, split, activity
  - `cohorts` — list, get, create, update, delete
  - `errors` — list, get, resolve, ignore, reopen
  - `actions` — list, get, create, update, delete
  - `definitions` — events (list/get/update/delete), properties (list/get/update)
  - `cache` — stats, clear
  - `completions` — bash, zsh, fish, powershell
- **Agent-first design** — structured JSON envelope output with `ok`, `data`, `meta` fields
- **Deterministic exit codes** (0-7) per error category for programmatic handling
- **Auth chain** — CLI flags > env vars > OS keyring > config file > project file
- **OS keyring integration** — macOS Keychain, Windows Credential Manager, Linux Secret Service
- **Local disk cache** with SHA256 keys, TTL expiration, per-host/per-project isolation
- **Auto-pagination** with `--all-pages` flag
- **Async query polling** with spinner and exponential backoff
- **4 output formats** — json (default), table, csv, jsonl
- **Global options** — `--host`, `--token`, `--project`, `--format`, `--quiet`, `--verbose`, `--no-cache`, `--timeout`, `--retry`, `--page-size`, `--all-pages`
- **Shell completions** for bash, zsh, fish, powershell
- **105 man pages** generated from clap definitions
- **Cross-platform releases** — macOS (ARM + x86), Linux, Windows
- **Homebrew tap** at `osodevops/tap/posthog-cli`
- **CI/CD** — GitHub Actions for testing (multi-platform), auto-tagging, and cargo-dist releases

### Documentation

- README with install, setup, quick start, agent integration, commands reference
- Agent integration guide (`docs/agent-guide.md`)
- Example scripts for common workflows
- 105 auto-generated man pages
