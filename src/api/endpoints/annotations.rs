use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

pub async fn list(
    client: &PostHogClient,
    search: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    client.get_with_params("annotations/", &params).await
}

pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("annotations/{}/", id)).await
}

pub async fn create(
    client: &PostHogClient,
    content: &str,
    date: &str,
    scope: &str,
) -> Result<serde_json::Value, AppError> {
    let body = json!({
        "content": content,
        "date_marker": date,
        "scope": scope,
    });
    client.post("annotations/", &body).await
}

pub async fn update(
    client: &PostHogClient,
    id: u64,
    content: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(c) = content {
        body["content"] = json!(c);
    }

    client.patch(&format!("annotations/{}/", id), &body).await
}

pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("annotations/{}/", id)).await
}
