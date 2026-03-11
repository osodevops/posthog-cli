use serde_json::json;

use crate::api::PostHogClient;
use crate::error::AppError;

pub async fn list(
    client: &PostHogClient,
    search: Option<&str>,
    saved: bool,
    favorited: bool,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(s) = search {
        params.push(("search", s));
    }
    let saved_str;
    if saved {
        saved_str = "true".to_string();
        params.push(("saved", &saved_str));
    }
    let fav_str;
    if favorited {
        fav_str = "true".to_string();
        params.push(("favorited", &fav_str));
    }
    client.get_with_params("insights/", &params).await
}

pub async fn get(
    client: &PostHogClient,
    id: u64,
    refresh: Option<&str>,
) -> Result<serde_json::Value, AppError> {
    let mut params: Vec<(&str, &str)> = Vec::new();
    if let Some(r) = refresh {
        params.push(("refresh", r));
    }
    client
        .get_with_params(&format!("insights/{}/", id), &params)
        .await
}

pub async fn create(
    client: &PostHogClient,
    name: &str,
    query_json: &str,
) -> Result<serde_json::Value, AppError> {
    let query: serde_json::Value =
        serde_json::from_str(query_json).map_err(|e| AppError::Validation {
            message: format!("Invalid query JSON: {e}"),
        })?;

    let body = json!({
        "name": name,
        "query": query,
    });

    client.post("insights/", &body).await
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

    client.patch(&format!("insights/{}/", id), &body).await
}

pub async fn delete(client: &PostHogClient, id: u64) -> Result<(), AppError> {
    client.delete(&format!("insights/{}/", id)).await
}
