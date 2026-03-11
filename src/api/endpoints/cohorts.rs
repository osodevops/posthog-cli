use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List cohorts.
pub async fn list(client: &PostHogClient) -> Result<serde_json::Value, AppError> {
    client.get("cohorts/").await
}

/// Get a cohort by ID.
pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("cohorts/{}/", id)).await
}

/// Create a new cohort.
pub async fn create(
    client: &PostHogClient,
    name: &str,
    filters: &str,
    is_static: bool,
) -> Result<serde_json::Value, AppError> {
    let filters_value: serde_json::Value =
        serde_json::from_str(filters).map_err(|e| AppError::Validation {
            message: format!("Invalid filters JSON: {e}"),
        })?;

    let body = json!({
        "name": name,
        "filters": filters_value,
        "is_static": is_static,
    });

    client.post("cohorts/", &body).await
}

/// Update an existing cohort.
pub async fn update(
    client: &PostHogClient,
    id: u64,
    name: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    client.patch(&format!("cohorts/{}/", id), &body).await
}

/// Delete a cohort.
pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("cohorts/{}/", id)).await
}
