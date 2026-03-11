use crate::error::AppError;

const KEYRING_SERVICE: &str = "posthog-cli";
const KEYRING_USER_TOKEN: &str = "api_token";

#[derive(Debug)]
pub struct ResolvedAuth {
    pub token: String,
    pub host: String,
    pub project_id: String,
    pub token_source: String,
}

impl ResolvedAuth {
    /// Resolve credentials from: CLI flag > env var > keyring > config file.
    pub fn resolve(
        cli_token: Option<&str>,
        cli_host: Option<&str>,
        cli_project: Option<&str>,
    ) -> Result<Self, AppError> {
        let config = super::AppConfig::load()?;

        let (token, token_source) = Self::resolve_token(cli_token)?;
        let host = Self::resolve_host(cli_host, &config);
        let project_id = Self::resolve_project(cli_project, &config)?;

        Ok(ResolvedAuth {
            token,
            host,
            project_id,
            token_source,
        })
    }

    fn resolve_token(cli_token: Option<&str>) -> Result<(String, String), AppError> {
        // 1. CLI flag
        if let Some(token) = cli_token {
            if !token.is_empty() {
                return Ok((token.to_string(), "cli flag".into()));
            }
        }

        // 2. Environment variable
        if let Ok(token) = std::env::var("POSTHOG_TOKEN") {
            if !token.is_empty() {
                return Ok((token, "environment variable".into()));
            }
        }

        // 3. OS keyring
        if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_TOKEN) {
            if let Ok(token) = entry.get_password() {
                if !token.is_empty() {
                    return Ok((token, "keyring".into()));
                }
            }
        }

        Err(AppError::Auth {
            message:
                "No API token found. Set POSTHOG_TOKEN, use --token, or run 'posthog auth login'"
                    .into(),
        })
    }

    fn resolve_host(cli_host: Option<&str>, config: &super::AppConfig) -> String {
        if let Some(host) = cli_host {
            return host.to_string();
        }

        // env var already handled by clap `env` attribute, so check config
        if let Some(ref host) = config.host {
            return host.clone();
        }

        // Check project-level config
        if let Ok(proj) = super::ProjectConfig::load() {
            if let Some(ref host) = proj.host {
                return host.clone();
            }
        }

        "https://us.posthog.com".to_string()
    }

    fn resolve_project(
        cli_project: Option<&str>,
        config: &super::AppConfig,
    ) -> Result<String, AppError> {
        if let Some(id) = cli_project {
            return Ok(id.to_string());
        }

        if let Ok(id) = std::env::var("POSTHOG_PROJECT_ID") {
            if !id.is_empty() {
                return Ok(id);
            }
        }

        if let Some(ref id) = config.project_id {
            return Ok(id.clone());
        }

        // Check project-level config
        if let Ok(proj) = super::ProjectConfig::load() {
            if let Some(ref id) = proj.project_id {
                return Ok(id.clone());
            }
        }

        Err(AppError::Config {
            message: "No project ID configured. Set via --project, POSTHOG_PROJECT_ID, or run 'posthog auth login'".into(),
        })
    }

    pub fn store_token(token: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_TOKEN).map_err(|e| {
            AppError::Config {
                message: format!("Keyring unavailable: {e}. Use POSTHOG_TOKEN env var instead."),
            }
        })?;
        entry.set_password(token).map_err(|e| AppError::Config {
            message: format!(
                "Failed to store token in keyring: {e}. Use POSTHOG_TOKEN env var instead."
            ),
        })?;
        Ok(())
    }

    pub fn delete_token() -> Result<(), AppError> {
        let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER_TOKEN).map_err(|e| {
            AppError::Config {
                message: format!("Keyring unavailable: {e}"),
            }
        })?;
        entry.delete_credential().map_err(|e| AppError::Config {
            message: format!("Failed to delete token from keyring: {e}"),
        })?;
        Ok(())
    }
}
