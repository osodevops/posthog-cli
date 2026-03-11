use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List persons with optional search and properties filters.
pub async fn list(
    client: &PostHogClient,
    search: Option<&str>,
    properties: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    if let Some(p) = properties {
        params.push(("properties", p));
    }
    client.get_with_params("persons/", &params).await
}

/// Get a person by ID (UUID).
pub async fn get(client: &PostHogClient, id: &str) -> Result<serde_json::Value, AppError> {
    client.get(&format!("persons/{}/", id)).await
}

/// Get a person by distinct ID. Returns the first matching result or a NotFound error.
pub async fn get_by_distinct_id(
    client: &PostHogClient,
    distinct_id: &str,
) -> Result<serde_json::Value, AppError> {
    let result: serde_json::Value = client
        .get_with_params("persons/", &[("distinct_id", distinct_id)])
        .await?;
    let results = result
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    results.into_iter().next().ok_or_else(|| AppError::NotFound {
        message: format!("Person with distinct_id '{distinct_id}' not found"),
    })
}

/// Update a person's properties.
pub async fn update(
    client: &PostHogClient,
    id: &str,
    set: Option<&str>,
    unset: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(s) = set {
        let properties: serde_json::Value =
            serde_json::from_str(s).map_err(|e| AppError::Validation {
                message: format!("Invalid JSON for set properties: {e}"),
            })?;
        body["properties"] = properties;
    }

    if let Some(u) = unset {
        let keys: Vec<String> = serde_json::from_str(u).map_err(|e| AppError::Validation {
            message: format!("Invalid JSON for unset properties: {e}"),
        })?;
        body["unset"] = json!(keys);
    }

    client.patch(&format!("persons/{}/", id), &body).await
}

/// Delete a person.
pub async fn delete(client: &PostHogClient, id: &str) -> Result<(), AppError> {
    client.delete(&format!("persons/{}/", id)).await
}

/// Delete a person and all associated data.
pub async fn delete_with_data(
    client: &PostHogClient,
    id: &str,
) -> Result<serde_json::Value, AppError> {
    client
        .post(&format!("persons/{}/delete_with_data/", id), &json!({}))
        .await
}

/// Split a person into separate persons.
pub async fn split(client: &PostHogClient, id: &str) -> Result<serde_json::Value, AppError> {
    client
        .post(&format!("persons/{}/split/", id), &json!({}))
        .await
}

/// Get activity log for a person.
pub async fn activity(client: &PostHogClient, id: &str) -> Result<serde_json::Value, AppError> {
    client.get(&format!("persons/{}/activity/", id)).await
}
