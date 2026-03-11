use std::io::IsTerminal;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// Execute a HogQL query via POST /api/projects/:id/query/
pub async fn hogql(
    client: &PostHogClient,
    sql: &str,
    refresh: &str,
    async_mode: bool,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "query": {
            "kind": "HogQLQuery",
            "query": sql,
        },
        "refresh": refresh,
        "async": async_mode,
    });
    client.post("query/", &body).await
}

/// Execute a HogQL query with --wait: submit async, poll until complete.
pub async fn hogql_wait(
    client: &PostHogClient,
    sql: &str,
    refresh: &str,
    quiet: bool,
) -> Result<serde_json::Value, AppError> {
    // Submit as async
    let body = json!({
        "query": {
            "kind": "HogQLQuery",
            "query": sql,
        },
        "refresh": refresh,
        "async": true,
    });

    let result: serde_json::Value = client.post("query/", &body).await?;

    // If already complete, return immediately
    if is_complete(&result) {
        return Ok(result);
    }

    // Extract query_id and poll
    let query_id = result
        .get("query_status")
        .or_else(|| result.get("id"))
        .and_then(|v| {
            v.get("id")
                .and_then(|id| id.as_str())
                .or_else(|| v.as_str())
        })
        .ok_or_else(|| AppError::General {
            message: "No query_id returned from async query submission".into(),
        })?
        .to_string();

    poll_query(client, &query_id, quiet).await
}

/// Poll an async query until completion with a spinner.
pub async fn poll_query(
    client: &PostHogClient,
    query_id: &str,
    quiet: bool,
) -> Result<serde_json::Value, AppError> {
    let show_spinner = !quiet && std::io::stderr().is_terminal();
    let spinner = if show_spinner {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("Query {query_id} running..."));
        pb.enable_steady_tick(Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let mut poll_interval = Duration::from_millis(500);
    let max_poll_interval = Duration::from_secs(5);
    let max_attempts = 120; // 10 minutes max with increasing intervals

    for _ in 0..max_attempts {
        tokio::time::sleep(poll_interval).await;

        let status: serde_json::Value =
            match client.get(&format!("query/{}/status/", query_id)).await {
                Ok(s) => s,
                Err(AppError::NotFound { .. }) => {
                    // Query may not be registered yet, keep polling
                    continue;
                }
                Err(e) => {
                    if let Some(ref pb) = spinner {
                        pb.finish_and_clear();
                    }
                    return Err(e);
                }
            };

        // Check if complete
        let query_status = status.get("query_status").unwrap_or(&status);

        let is_running = query_status
            .get("complete")
            .and_then(|v| v.as_bool())
            .map(|c| !c)
            .unwrap_or(true);

        if !is_running {
            if let Some(ref pb) = spinner {
                pb.finish_and_clear();
            }

            // Check for errors
            if let Some(err) = query_status.get("error") {
                if !err.is_null() {
                    let msg = if err.is_string() {
                        err.as_str().unwrap_or("Unknown error").to_string()
                    } else {
                        err.get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or(&err.to_string())
                            .to_string()
                    };
                    return Err(AppError::General {
                        message: format!("Query failed: {msg}"),
                    });
                }
            }

            // Fetch full results
            return query_result(client, query_id).await;
        }

        // Increase poll interval with backoff, capped at max
        poll_interval = (poll_interval * 2).min(max_poll_interval);
    }

    if let Some(ref pb) = spinner {
        pb.finish_and_clear();
    }

    Err(AppError::QueryTimeout)
}

/// Check if a query response indicates completion.
fn is_complete(result: &serde_json::Value) -> bool {
    // If there's a "results" key at top level, it's complete
    if result.get("results").is_some() {
        return true;
    }
    // Check query_status.complete
    result
        .get("query_status")
        .and_then(|qs| qs.get("complete"))
        .and_then(|c| c.as_bool())
        .unwrap_or(false)
}

/// Execute a trends query.
pub async fn trends(
    client: &PostHogClient,
    event: &str,
    interval: &str,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut query = json!({
        "kind": "TrendsQuery",
        "series": [{
            "kind": "EventsNode",
            "event": event,
        }],
        "interval": interval,
    });

    if let Some(from) = date_from {
        query["dateRange"] = json!({"date_from": from});
    }
    if let Some(to) = date_to {
        if let Some(range) = query.get_mut("dateRange") {
            range["date_to"] = json!(to);
        } else {
            query["dateRange"] = json!({"date_to": to});
        }
    }

    let body = json!({ "query": query });
    client.post("query/", &body).await
}

/// Execute a funnel query.
pub async fn funnels(
    client: &PostHogClient,
    steps: &[String],
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let series: Vec<serde_json::Value> = steps
        .iter()
        .map(|e| json!({"kind": "EventsNode", "event": e}))
        .collect();

    let mut query = json!({
        "kind": "FunnelsQuery",
        "series": series,
    });

    if let Some(from) = date_from {
        query["dateRange"] = json!({"date_from": from});
    }
    if let Some(to) = date_to {
        if let Some(range) = query.get_mut("dateRange") {
            range["date_to"] = json!(to);
        } else {
            query["dateRange"] = json!({"date_to": to});
        }
    }

    let body = json!({ "query": query });
    client.post("query/", &body).await
}

/// Execute a retention query.
pub async fn retention(
    client: &PostHogClient,
    target_event: &str,
    return_event: &str,
    period: &str,
    date_from: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut query = json!({
        "kind": "RetentionQuery",
        "retentionFilter": {
            "targetEntity": {"id": target_event, "type": "events"},
            "returningEntity": {"id": return_event, "type": "events"},
            "period": period,
        },
    });

    if let Some(from) = date_from {
        query["dateRange"] = json!({"date_from": from});
    }

    let body = json!({ "query": query });
    client.post("query/", &body).await
}

/// Check async query status.
pub async fn query_status(
    client: &PostHogClient,
    query_id: &str,
) -> Result<serde_json::Value, AppError> {
    client.get(&format!("query/{}/status/", query_id)).await
}

/// Fetch completed async query result.
pub async fn query_result(
    client: &PostHogClient,
    query_id: &str,
) -> Result<serde_json::Value, AppError> {
    client.get(&format!("query/{}/", query_id)).await
}

/// Cancel a running query (DELETE).
pub async fn query_cancel(client: &PostHogClient, query_id: &str) -> Result<(), AppError> {
    client.delete(&format!("query/{}/", query_id)).await
}
