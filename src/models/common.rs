use serde::{Deserialize, Serialize};

/// Standard paginated list response from PostHog API.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub count: Option<u64>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}

/// Standard PostHog CLI output envelope (used in JSON output mode).
#[derive(Debug, Serialize)]
pub struct ApiEnvelope<T: Serialize> {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

#[derive(Debug, Serialize)]
pub struct ResponseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub cached: bool,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub next: Option<String>,
    pub count: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_secs: Option<u64>,
}
