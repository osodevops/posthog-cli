use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List actions.
pub async fn list(client: &PostHogClient) -> Result<serde_json::Value, AppError> {
    client.get("actions/").await
}

/// Get an action by ID.
pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("actions/{}/", id)).await
}

/// Create a new action.
pub async fn create(
    client: &PostHogClient,
    name: &str,
    steps: &str,
) -> Result<serde_json::Value, AppError> {
    let parsed_steps: serde_json::Value = serde_json::from_str(steps)?;

    let body = json!({
        "name": name,
        "steps": parsed_steps,
    });

    client.post("actions/", &body).await
}

/// Update an existing action.
pub async fn update(
    client: &PostHogClient,
    id: u64,
    name: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    client.patch(&format!("actions/{}/", id), &body).await
}

/// Delete an action.
pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("actions/{}/", id)).await
}
