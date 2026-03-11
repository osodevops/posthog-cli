use serde::Deserialize;
use std::path::PathBuf;

use crate::error::AppError;

/// Project-level configuration loaded from `.posthog.toml` in the current
/// directory or any parent directory.
#[derive(Debug, Default, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: Option<ProjectSection>,
    pub host: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSection {
    pub host: Option<String>,
    pub project_id: Option<String>,
    pub default_date_range: Option<String>,
}

impl ProjectConfig {
    pub fn load() -> Result<Self, AppError> {
        let path = match Self::find_config() {
            Some(p) => p,
            None => return Ok(Self::default()),
        };

        let contents = std::fs::read_to_string(&path).map_err(|e| AppError::Config {
            message: format!("Failed to read {}: {e}", path.display()),
        })?;

        let config: ProjectConfig =
            toml::from_str(&contents).map_err(|e| AppError::Config {
                message: format!("Failed to parse {}: {e}", path.display()),
            })?;

        // Flatten project section fields for easier access
        Ok(Self {
            host: config
                .host
                .or_else(|| config.project.as_ref().and_then(|p| p.host.clone())),
            project_id: config
                .project_id
                .or_else(|| config.project.as_ref().and_then(|p| p.project_id.clone())),
            project: config.project,
        })
    }

    /// Walk up from current directory looking for `.posthog.toml`.
    fn find_config() -> Option<PathBuf> {
        let mut dir = std::env::current_dir().ok()?;
        loop {
            let candidate = dir.join(".posthog.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            if !dir.pop() {
                return None;
            }
        }
    }
}
