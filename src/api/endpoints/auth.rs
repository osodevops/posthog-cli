use crate::api::PostHogClient;
use crate::error::AppError;

/// Fetch current user info via GET /api/users/@me/
pub async fn whoami(client: &PostHogClient) -> Result<serde_json::Value, AppError> {
    client.get_me().await
}
