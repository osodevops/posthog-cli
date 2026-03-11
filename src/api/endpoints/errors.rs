use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

/// List error tracking issues.
pub async fn list(
    client: &PostHogClient,
    status: Option<&str>,
    date_from: Option<&str>,
    order_by: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = status {
        params.push(("status", s));
    }
    if let Some(d) = date_from {
        params.push(("date_from", d));
    }
    if let Some(o) = order_by {
        params.push(("order_by", o));
    }
    client
        .get_with_params("error_tracking/issue/", &params)
        .await
}

/// Get an error tracking issue by ID.
pub async fn get(
    client: &PostHogClient,
    id: &str,
    date_from: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(d) = date_from {
        params.push(("date_from", d));
    }
    client
        .get_with_params(&format!("error_tracking/issue/{}/", id), &params)
        .await
}

/// Update the status of an error tracking issue.
pub async fn update_status(
    client: &PostHogClient,
    id: &str,
    status: &str,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "status": status,
    });
    client
        .patch(&format!("error_tracking/issue/{}/", id), &body)
        .await
}
