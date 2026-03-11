use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List surveys, optionally filtered by status.
pub async fn list(
    client: &PostHogClient,
    status: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = status {
        params.push(("status", s));
    }
    client.get_with_params("surveys/", &params).await
}

/// Get a survey by ID.
pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("surveys/{}/", id)).await
}

/// Create a new survey.
pub async fn create(
    client: &PostHogClient,
    name: &str,
    questions: &str,
    targeting: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let questions_value: serde_json::Value =
        serde_json::from_str(questions).map_err(|e| AppError::Validation {
            message: format!("Invalid questions JSON: {e}"),
        })?;

    let mut body = json!({
        "name": name,
        "questions": questions_value,
    });

    if let Some(t) = targeting {
        let targeting_value: serde_json::Value =
            serde_json::from_str(t).map_err(|e| AppError::Validation {
                message: format!("Invalid targeting JSON: {e}"),
            })?;
        body["targeting_flag_filters"] = targeting_value;
    }

    client.post("surveys/", &body).await
}

/// Update an existing survey.
pub async fn update(
    client: &PostHogClient,
    id: u64,
    end_date: Option<&str>,
    name: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(d) = end_date {
        body["end_date"] = json!(d);
    }

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    client.patch(&format!("surveys/{}/", id), &body).await
}

/// Launch a survey by setting its start_date to now.
pub async fn launch(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "start_date": chrono::Utc::now().to_rfc3339(),
    });
    client.patch(&format!("surveys/{}/", id), &body).await
}

/// Stop a survey by setting its end_date to now.
pub async fn stop(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "end_date": chrono::Utc::now().to_rfc3339(),
    });
    client.patch(&format!("surveys/{}/", id), &body).await
}

/// Archive a survey.
pub async fn archive(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "archived": true,
    });
    client.patch(&format!("surveys/{}/", id), &body).await
}

/// Delete a survey.
pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("surveys/{}/", id)).await
}
