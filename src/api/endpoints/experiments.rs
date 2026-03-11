use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List experiments.
pub async fn list(
    client: &PostHogClient,
    status: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = status {
        params.push(("status", s));
    }
    client.get_with_params("experiments/", &params).await
}

/// Get an experiment by ID.
pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("experiments/{}/", id)).await
}

/// Create a new experiment.
pub async fn create(
    client: &PostHogClient,
    name: &str,
    feature_flag_key: &str,
    description: Option<&str>,
    metrics: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({
        "name": name,
        "feature_flag_key": feature_flag_key,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }

    if let Some(m) = metrics {
        let metrics_value: serde_json::Value =
            serde_json::from_str(m).map_err(|e| AppError::Validation {
                message: format!("Invalid metrics JSON: {e}"),
            })?;
        body["metrics"] = metrics_value;
    }

    client.post("experiments/", &body).await
}

/// Update an existing experiment.
pub async fn update(
    client: &PostHogClient,
    id: u64,
    description: Option<&str>,
    name: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(d) = description {
        body["description"] = json!(d);
    }

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    client.patch(&format!("experiments/{}/", id), &body).await
}

/// Start an experiment by setting the start_date to the current UTC time.
pub async fn start(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "start_date": chrono::Utc::now().to_rfc3339(),
    });

    client.patch(&format!("experiments/{}/", id), &body).await
}

/// Stop an experiment by setting the end_date to the current UTC time.
pub async fn stop(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "end_date": chrono::Utc::now().to_rfc3339(),
    });

    client.patch(&format!("experiments/{}/", id), &body).await
}

/// Get experiment results.
pub async fn results(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("experiments/{}/results/", id)).await
}

/// Delete an experiment.
pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("experiments/{}/", id)).await
}
