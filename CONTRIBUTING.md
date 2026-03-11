# Contributing to posthog-cli

Thank you for your interest in contributing to `posthog-cli`.

## Getting started

```bash
git clone https://github.com/osodevops/posthog-cli.git
cd posthog-cli
cargo build
cargo test
```

## Development workflow

### Prerequisites

- Rust 1.80.0+ (see `rust-version` in Cargo.toml)
- A PostHog account with a personal API key for integration testing

### Running tests

```bash
# Unit and integration tests
cargo test

# Integration tests only
cargo test --test cli_integration

# With output
cargo test -- --nocapture
```

### Linting

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

### Generating man pages

```bash
cargo run --example gen-man
ls man/
```

## Project structure

```
src/
  main.rs           # CLI entry point and command dispatch
  lib.rs            # Library re-exports
  cli/              # Clap command definitions
    mod.rs          # All command and subcommand enums
    auth.rs         # Auth subcommand definitions
  api/
    client.rs       # PostHogClient HTTP methods
    endpoints/      # One file per API resource (flags.rs, query.rs, etc.)
  config/           # Config file and credential loading
  cache/            # Disk cache with SHA256 keys
  output/           # JSON, table, CSV, JSONL formatters
  models/           # Shared data structures
  error.rs          # AppError enum with exit codes
tests/
  cli_integration.rs  # CLI --help and basic invocation tests
  cache_test.rs       # Cache unit tests
  error_test.rs       # Error formatting tests
  output_test.rs      # Output formatting tests
examples/
  gen-man.rs          # Man page generator
docs/
  agent-guide.md      # AI agent integration guide
```

## Adding a new command

1. Add the API endpoint in `src/api/endpoints/`
2. Register the module in `src/api/endpoints/mod.rs`
3. Add the clap subcommand enum in `src/cli/mod.rs`
4. Wire the dispatch in `src/main.rs`
5. Add integration tests in `tests/cli_integration.rs`
6. Update the commands table in `README.md`

## Code conventions

- `thiserror` for error types, `AppError::Validation` for user input errors
- `serde_json::Value` for API responses (no typed response structs)
- Tracing to stderr, data to stdout
- Global options propagate via clap's `global = true`
- Cache keys include host + project ID to avoid cross-project collisions
- Write operations (`create`, `update`, `delete`, `capture`) are never cached

## Pull requests

- Run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` before pushing
- Include tests for new commands
- Keep PRs focused on a single change

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
