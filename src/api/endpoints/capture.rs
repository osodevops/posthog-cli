use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// Capture a single event via POST /capture/
pub async fn event(
    client: &PostHogClient,
    event_name: &str,
    distinct_id: &str,
    properties: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut props = if let Some(p) = properties {
        serde_json::from_str(p).map_err(|e| AppError::Validation {
            message: format!("Invalid properties JSON: {e}"),
        })?
    } else {
        json!({})
    };

    if let Some(obj) = props.as_object_mut() {
        obj.insert("token".to_string(), json!(client.token));
    }

    let body = json!({
        "event": event_name,
        "distinct_id": distinct_id,
        "properties": props,
    });

    client.post_capture("capture/", &body).await?;
    Ok(json!({"status": "ok", "event": event_name}))
}

/// Batch capture events from a JSONL file.
pub async fn batch(
    client: &PostHogClient,
    file_path: &str,
) -> Result<serde_json::Value, AppError> {
    let contents = std::fs::read_to_string(file_path).map_err(|e| AppError::Validation {
        message: format!("Failed to read file {file_path}: {e}"),
    })?;

    let mut events: Vec<serde_json::Value> = Vec::new();
    for line in contents.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let mut event: serde_json::Value =
            serde_json::from_str(line).map_err(|e| AppError::Validation {
                message: format!("Invalid JSONL line: {e}"),
            })?;
        if let Some(props) = event.get_mut("properties") {
            if let Some(obj) = props.as_object_mut() {
                obj.entry("token".to_string())
                    .or_insert_with(|| json!(client.token));
            }
        }
        events.push(event);
    }

    let event_count = events.len();
    let body = json!({
        "batch": events,
    });

    client.post_capture("batch/", &body).await?;
    Ok(json!({"status": "ok", "events_sent": event_count}))
}

/// Identify a user (set person properties).
pub async fn identify(
    client: &PostHogClient,
    distinct_id: &str,
    set_json: &str,
) -> Result<serde_json::Value, AppError> {
    let set_props: serde_json::Value =
        serde_json::from_str(set_json).map_err(|e| AppError::Validation {
            message: format!("Invalid properties JSON: {e}"),
        })?;

    let body = json!({
        "event": "$identify",
        "distinct_id": distinct_id,
        "properties": {
            "token": client.token,
            "$set": set_props,
        },
    });

    client.post_capture("capture/", &body).await?;
    Ok(json!({"status": "ok", "distinct_id": distinct_id}))
}

/// Set group properties.
pub async fn group(
    client: &PostHogClient,
    group_type: &str,
    group_key: &str,
    set_json: &str,
) -> Result<serde_json::Value, AppError> {
    let set_props: serde_json::Value =
        serde_json::from_str(set_json).map_err(|e| AppError::Validation {
            message: format!("Invalid properties JSON: {e}"),
        })?;

    let body = json!({
        "event": "$groupidentify",
        "distinct_id": format!("${}_{}", group_type, group_key),
        "properties": {
            "token": client.token,
            "$group_type": group_type,
            "$group_key": group_key,
            "$group_set": set_props,
        },
    });

    client.post_capture("capture/", &body).await?;
    Ok(json!({"status": "ok", "group_type": group_type, "group_key": group_key}))
}

/// Create a user alias.
pub async fn alias(
    client: &PostHogClient,
    distinct_id: &str,
    alias_id: &str,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "event": "$create_alias",
        "distinct_id": distinct_id,
        "properties": {
            "token": client.token,
            "alias": alias_id,
        },
    });

    client.post_capture("capture/", &body).await?;
    Ok(json!({"status": "ok", "distinct_id": distinct_id, "alias": alias_id}))
}
