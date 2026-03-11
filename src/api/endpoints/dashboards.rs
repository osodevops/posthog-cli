use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

pub async fn list(
    client: &PostHogClient,
    search: Option<&str>,
    pinned: bool,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    let pinned_str;
    if pinned {
        pinned_str = "true".to_string();
        params.push(("pinned", &pinned_str));
    }
    client.get_with_params("dashboards/", &params).await
}

pub async fn get(client: &PostHogClient, id: u64) -> Result<serde_json::Value, AppError> {
    client.get(&format!("dashboards/{}/", id)).await
}

pub async fn create(
    client: &PostHogClient,
    name: &str,
    description: Option<&str>,
    pinned: bool,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({
        "name": name,
        "pinned": pinned,
    });

    if let Some(d) = description {
        body["description"] = json!(d);
    }

    client.post("dashboards/", &body).await
}

pub async fn update(
    client: &PostHogClient,
    id: u64,
    name: Option<&str>,
    tags: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut body = json!({});

    if let Some(n) = name {
        body["name"] = json!(n);
    }

    if let Some(t) = tags {
        let tags_value: serde_json::Value =
            serde_json::from_str(t).map_err(|e| AppError::Validation {
                message: format!("Invalid tags JSON: {e}"),
            })?;
        body["tags"] = tags_value;
    }

    client.patch(&format!("dashboards/{}/", id), &body).await
}

pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("dashboards/{}/", id)).await
}
