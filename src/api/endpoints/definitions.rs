use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

// ---------------------------------------------------------------------------
// Event definitions
// ---------------------------------------------------------------------------

/// List event definitions, with an optional search filter.
pub async fn list_events(
    client: &PostHogClient,
    search: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    client.get_with_params("event_definitions/", &params).await
}

/// Get a single event definition by ID.
pub async fn get_event(client: &PostHogClient, id: &str) -> Result<serde_json::Value, AppError> {
    client.get(&format!("event_definitions/{}/", id)).await
}

/// Update an event definition.
///
/// Only the supplied fields are included in the PATCH body. If `tags` is
/// provided it is parsed as a `serde_json::Value` (e.g. `["tag1","tag2"]`).
pub async fn update_event(
    client: &PostHogClient,
    id: &str,
    description: Option<&str>,
    tags: Option<&str>,
    verified: Option<bool>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(d) = description {
        body["description"] = json!(d);
    }

    if let Some(t) = tags {
        let parsed: serde_json::Value =
            serde_json::from_str(t).map_err(|e| AppError::Validation {
                message: format!("Invalid JSON for tags: {e}"),
            })?;
        body["tags"] = parsed;
    }

    if let Some(v) = verified {
        body["verified"] = json!(v);
    }

    client
        .patch(&format!("event_definitions/{}/", id), &body)
        .await
}

/// Delete an event definition.
pub async fn delete_event(client: &PostHogClient, id: &str) -> Result<(), AppError> {
    client.delete(&format!("event_definitions/{}/", id)).await
}

// ---------------------------------------------------------------------------
// Property definitions
// ---------------------------------------------------------------------------

/// List property definitions, with optional search and event-name filters.
pub async fn list_properties(
    client: &PostHogClient,
    search: Option<&str>,
    event_names: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    if let Some(en) = event_names {
        params.push(("event_names", en));
    }
    client
        .get_with_params("property_definitions/", &params)
        .await
}

/// Get a single property definition by ID.
pub async fn get_property(client: &PostHogClient, id: &str) -> Result<serde_json::Value, AppError> {
    client.get(&format!("property_definitions/{}/", id)).await
}

/// Update a property definition.
pub async fn update_property(
    client: &PostHogClient,
    id: &str,
    description: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(d) = description {
        body["description"] = json!(d);
    }

    client
        .patch(&format!("property_definitions/{}/", id), &body)
        .await
}
