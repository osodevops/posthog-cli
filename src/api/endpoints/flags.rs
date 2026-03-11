use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List feature flags.
pub async fn list(
    client: &PostHogClient,
    active: bool,
    search: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    let active_str;
    if active {
        active_str = "true".to_string();
        params.push(("active", &active_str));
    }
    if let Some(s) = search {
        params.push(("search", s));
    }
    client.get_with_params("feature_flags/", &params).await
}

/// Get a feature flag by ID or key.
pub async fn get(client: &PostHogClient, key: &str) -> Result<serde_json::Value, AppError> {
    // Try numeric ID first, then search by key
    if key.parse::<u64>().is_ok() {
        client.get(&format!("feature_flags/{}/", key)).await
    } else {
        // Search by key and return the first match
        let result: serde_json::Value = client
            .get_with_params("feature_flags/", &[("search", key)])
            .await?;
        let results = result
            .get("results")
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        results
            .into_iter()
            .find(|f| f.get("key").and_then(|k| k.as_str()) == Some(key))
            .ok_or_else(|| AppError::NotFound {
                message: format!("Feature flag '{key}' not found"),
            })
    }
}

/// Create a new feature flag.
pub async fn create(
    client: &PostHogClient,
    key: &str,
    name: Option<&str>,
    rollout: Option<u8>,
    active: bool,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({
        "key": key,
        "active": active,
    });

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    if let Some(pct) = rollout {
        body["filters"] = json!({
            "groups": [{
                "rollout_percentage": pct,
            }]
        });
    }

    client.post("feature_flags/", &body).await
}

/// Update an existing feature flag.
pub async fn update(
    client: &PostHogClient,
    key: &str,
    rollout: Option<u8>,
    active: Option<bool>,
    name: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let flag = get(client, key).await?;
    let id = flag
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AppError::General {
            message: "Flag has no ID".into(),
        })?;

    let mut body = json!({});

    if let Some(pct) = rollout {
        body["filters"] = json!({
            "groups": [{
                "rollout_percentage": pct,
            }]
        });
    }

    if let Some(a) = active {
        body["active"] = json!(a);
    }

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    client.patch(&format!("feature_flags/{}/", id), &body).await
}

/// Delete a feature flag.
pub async fn delete(client: &PostHogClient, key: &str) -> Result<(), AppError> {
    let flag = get(client, key).await?;
    let id = flag
        .get("id")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AppError::General {
            message: "Flag has no ID".into(),
        })?;

    client.delete(&format!("feature_flags/{}/", id)).await
}

/// Evaluate a single flag for a distinct ID.
pub async fn evaluate(
    client: &PostHogClient,
    key: &str,
    distinct_id: &str,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "distinct_id": distinct_id,
    });
    client
        .post(&format!("feature_flags/{}/evaluate/", key), &body)
        .await
}

/// Evaluate all flags for a distinct ID.
pub async fn evaluate_all(
    client: &PostHogClient,
    distinct_id: &str,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "distinct_id": distinct_id,
    });
    client.post("feature_flags/evaluation/", &body).await
}
