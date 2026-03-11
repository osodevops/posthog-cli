use std::time::Duration;

use reqwest::{Client, Response, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AppError;

pub struct PostHogClient {
    http: Client,
    pub base_url: String,
    pub token: String,
    pub project_id: String,
    max_retries: u32,
}

impl PostHogClient {
    pub fn new(
        base_url: String,
        token: String,
        project_id: String,
        timeout_secs: u64,
        max_retries: u32,
    ) -> Result<Self, AppError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .gzip(true)
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| AppError::Network {
                message: format!("Failed to create HTTP client: {e}"),
            })?;

        Ok(PostHogClient {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            token,
            project_id,
            max_retries,
        })
    }

    /// Build the full URL for a project-scoped API path.
    pub fn project_url(&self, path: &str) -> String {
        format!(
            "{}/api/projects/{}/{}",
            self.base_url,
            self.project_id,
            path.trim_start_matches('/')
        )
    }

    /// Build the full URL for a non-project-scoped API path.
    pub fn api_url(&self, path: &str) -> String {
        format!("{}/api/{}", self.base_url, path.trim_start_matches('/'))
    }

    /// GET request to a project-scoped endpoint.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let url = self.project_url(path);
        self.get_url(&url).await
    }

    /// GET request to an arbitrary URL.
    pub async fn get_url<T: DeserializeOwned>(&self, url: &str) -> Result<T, AppError> {
        let resp = self.request_with_retry(|| {
            self.http
                .get(url)
                .header("Authorization", format!("Bearer {}", self.token))
        }).await?;
        self.handle_json_response(resp).await
    }

    /// POST request to a project-scoped endpoint.
    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = self.project_url(path);
        let resp = self.request_with_retry(|| {
            self.http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .json(body)
        }).await?;
        self.handle_json_response(resp).await
    }

    /// PATCH request to a project-scoped endpoint.
    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = self.project_url(path);
        let resp = self.request_with_retry(|| {
            self.http
                .patch(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .json(body)
        }).await?;
        self.handle_json_response(resp).await
    }

    /// DELETE request to a project-scoped endpoint.
    pub async fn delete(&self, path: &str) -> Result<(), AppError> {
        let url = self.project_url(path);
        let resp = self.request_with_retry(|| {
            self.http
                .delete(&url)
                .header("Authorization", format!("Bearer {}", self.token))
        }).await?;
        self.handle_status(resp).await
    }

    /// GET with query parameters.
    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<T, AppError> {
        let url = self.project_url(path);
        let resp = self.request_with_retry(|| {
            self.http
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .query(params)
        }).await?;
        self.handle_json_response(resp).await
    }

    /// POST to a non-project-scoped endpoint (e.g. capture).
    pub async fn post_api<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, AppError> {
        let url = self.api_url(path);
        let resp = self.request_with_retry(|| {
            self.http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .json(body)
        }).await?;
        self.handle_json_response(resp).await
    }

    /// POST to the /capture/ endpoint (no auth header, token in body).
    /// Used for event capture, identify, group, alias.
    pub async fn post_capture<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), AppError> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let resp = self.request_with_retry(|| {
            self.http.post(&url).json(body)
        }).await?;
        self.handle_status(resp).await
    }

    /// GET to the /api/users/@me/ endpoint.
    pub async fn get_me(&self) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/api/users/@me/", self.base_url);
        let resp = self.request_with_retry(|| {
            self.http
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.token))
        }).await?;
        self.handle_json_response(resp).await
    }

    /// GET with auto-pagination. Follows `next` URLs and collects all `results`
    /// into a single JSON array. Used when `--all-pages` is set.
    pub async fn get_all_pages(
        &self,
        path: &str,
        params: &[(&str, &str)],
        page_size: u32,
    ) -> Result<serde_json::Value, AppError> {
        let mut all_results: Vec<serde_json::Value> = Vec::new();
        let mut total_count: Option<u64> = None;

        // First page
        let url = self.project_url(path);
        let mut page_params: Vec<(&str, String)> = params
            .iter()
            .map(|(k, v)| (*k, v.to_string()))
            .collect();
        page_params.push(("limit", page_size.to_string()));

        let query_pairs: Vec<(&str, &str)> = page_params
            .iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let first_page: serde_json::Value = {
            let resp = self.request_with_retry(|| {
                self.http
                    .get(&url)
                    .header("Authorization", format!("Bearer {}", self.token))
                    .query(&query_pairs)
            }).await?;
            self.handle_json_response(resp).await?
        };

        if let Some(count) = first_page.get("count").and_then(|c| c.as_u64()) {
            total_count = Some(count);
        }

        if let Some(results) = first_page.get("results").and_then(|r| r.as_array()) {
            all_results.extend(results.clone());
        } else {
            // Not a paginated response, return as-is
            return Ok(first_page);
        }

        // Follow next pages
        let mut next_url = first_page
            .get("next")
            .and_then(|n| n.as_str())
            .map(|s| s.to_string());

        while let Some(ref url) = next_url {
            tracing::debug!("Fetching next page: {url}");
            let page: serde_json::Value = self.get_url(url).await?;

            if let Some(results) = page.get("results").and_then(|r| r.as_array()) {
                all_results.extend(results.clone());
            }

            next_url = page
                .get("next")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
        }

        Ok(serde_json::json!({
            "count": total_count.unwrap_or(all_results.len() as u64),
            "results": all_results,
            "next": null,
            "previous": null,
        }))
    }

    async fn request_with_retry<F>(&self, build: F) -> Result<Response, AppError>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let max = self.max_retries.max(1);
        let mut last_err = None;

        for attempt in 0..max {
            let resp = match build().send().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::debug!("Request attempt {attempt} failed: {e}");
                    last_err = Some(AppError::from(e));
                    if attempt + 1 < max {
                        let backoff = Duration::from_secs(2u64.pow(attempt));
                        tokio::time::sleep(backoff).await;
                    }
                    continue;
                }
            };

            let status = resp.status();

            // Rate limited — backoff and retry
            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(2u64.pow(attempt + 1));

                tracing::warn!("Rate limited (429). Retry after {retry_after}s");
                if attempt + 1 < max {
                    tokio::time::sleep(Duration::from_secs(retry_after)).await;
                    continue;
                }
                return Err(AppError::RateLimited {
                    retry_after_secs: retry_after,
                });
            }

            // Server errors — retry with backoff
            if status.is_server_error() {
                let body_text = resp.text().await.unwrap_or_default();
                tracing::debug!("Server error {status}: {body_text}");
                last_err = Some(AppError::Server {
                    status_code: status.as_u16(),
                    message: body_text,
                });
                if attempt + 1 < max {
                    let backoff = Duration::from_secs(2u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                    continue;
                }
                // Last attempt, return the error
                return Err(last_err.unwrap());
            }

            return Ok(resp);
        }

        Err(last_err.unwrap_or_else(|| AppError::Network {
            message: "All retries exhausted".into(),
        }))
    }

    async fn handle_json_response<T: DeserializeOwned>(
        &self,
        resp: Response,
    ) -> Result<T, AppError> {
        let status = resp.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Auth {
                message: format!("HTTP {status}: {body}"),
            });
        }

        if status == StatusCode::NOT_FOUND {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::NotFound {
                message: body,
            });
        }

        if status.is_client_error() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Validation {
                message: format!("HTTP {status}: {body}"),
            });
        }

        if status.is_server_error() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Server {
                status_code: status.as_u16(),
                message: body,
            });
        }

        resp.json::<T>().await.map_err(|e| AppError::General {
            message: format!("Failed to parse response: {e}"),
        })
    }

    async fn handle_status(&self, resp: Response) -> Result<(), AppError> {
        let status = resp.status();

        if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Auth {
                message: format!("HTTP {status}: {body}"),
            });
        }

        if status == StatusCode::NOT_FOUND {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::NotFound { message: body });
        }

        if status.is_client_error() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Validation {
                message: format!("HTTP {status}: {body}"),
            });
        }

        if status.is_server_error() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Server {
                status_code: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }
}
