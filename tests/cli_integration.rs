use assert_cmd::Command;
use predicates::prelude::*;

fn posthog_cmd() -> Command {
    Command::cargo_bin("posthog").unwrap()
}

#[test]
fn test_help_output() {
    posthog_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent-first CLI for the PostHog analytics API"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("query"))
        .stdout(predicate::str::contains("flags"))
        .stdout(predicate::str::contains("insights"))
        .stdout(predicate::str::contains("dashboards"))
        .stdout(predicate::str::contains("annotations"))
        .stdout(predicate::str::contains("capture"))
        .stdout(predicate::str::contains("cache"))
        .stdout(predicate::str::contains("completions"));
}

#[test]
fn test_version_output() {
    posthog_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("posthog 0.1.0"));
}

#[test]
fn test_auth_help() {
    posthog_cmd()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("logout"))
        .stdout(predicate::str::contains("whoami"))
        .stdout(predicate::str::contains("switch"));
}

#[test]
fn test_query_help() {
    posthog_cmd()
        .args(["query", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sql"))
        .stdout(predicate::str::contains("trends"))
        .stdout(predicate::str::contains("funnels"))
        .stdout(predicate::str::contains("retention"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("cancel"));
}

#[test]
fn test_flags_help() {
    posthog_cmd()
        .args(["flags", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("evaluate"))
        .stdout(predicate::str::contains("evaluate-all"));
}

#[test]
fn test_auth_status_unauthenticated_json() {
    posthog_cmd()
        .args(["auth", "status", "-f", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"authenticated\": false"));
}

#[test]
fn test_cache_stats() {
    posthog_cmd()
        .args(["cache", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cache entries:"))
        .stdout(predicate::str::contains("Cache size:"))
        .stdout(predicate::str::contains("Cache dir:"));
}

#[test]
fn test_cache_clear() {
    posthog_cmd()
        .args(["cache", "clear"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Cleared"));
}

#[test]
fn test_completions_bash() {
    posthog_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("posthog"));
}

#[test]
fn test_completions_zsh() {
    posthog_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("posthog"));
}

#[test]
fn test_query_sql_no_auth_exits_with_error() {
    // Without any auth configured, should fail with auth error (exit 2)
    // or config error (exit 1) for missing project
    posthog_cmd()
        .args(["query", "sql", "SELECT 1"])
        .env_remove("POSTHOG_TOKEN")
        .env_remove("POSTHOG_PROJECT_ID")
        .assert()
        .failure();
}

#[test]
fn test_query_sql_missing_query() {
    // Providing auth but no query string or file should fail with validation error
    posthog_cmd()
        .args(["query", "sql", "-t", "phx_test", "-p", "123"])
        .assert()
        .failure();
}

#[test]
fn test_experiments_help() {
    posthog_cmd()
        .args(["experiments", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("results"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_surveys_help() {
    posthog_cmd()
        .args(["surveys", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("launch"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("archive"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_persons_help() {
    posthog_cmd()
        .args(["persons", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("split"))
        .stdout(predicate::str::contains("activity"));
}

#[test]
fn test_cohorts_help() {
    posthog_cmd()
        .args(["cohorts", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_errors_help() {
    posthog_cmd()
        .args(["errors", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("resolve"))
        .stdout(predicate::str::contains("ignore"))
        .stdout(predicate::str::contains("reopen"));
}

#[test]
fn test_actions_help() {
    posthog_cmd()
        .args(["actions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_definitions_help() {
    posthog_cmd()
        .args(["definitions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("events"))
        .stdout(predicate::str::contains("properties"));
}

#[test]
fn test_help_includes_phase2_commands() {
    posthog_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("experiments"))
        .stdout(predicate::str::contains("surveys"))
        .stdout(predicate::str::contains("persons"))
        .stdout(predicate::str::contains("cohorts"))
        .stdout(predicate::str::contains("errors"))
        .stdout(predicate::str::contains("actions"))
        .stdout(predicate::str::contains("definitions"));
}

#[test]
fn test_global_flags_propagate() {
    // Verify global flags show up in subcommand help
    posthog_cmd()
        .args(["flags", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--token"))
        .stdout(predicate::str::contains("--host"))
        .stdout(predicate::str::contains("--project"))
        .stdout(predicate::str::contains("--format"))
        .stdout(predicate::str::contains("--all-pages"));
}
