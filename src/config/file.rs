use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::AppError;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: Option<String>,
    pub project_id: Option<String>,
    pub default_format: Option<String>,

    #[serde(default)]
    pub cache: CacheConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_cache_ttl")]
    pub ttl_secs: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl_secs: default_cache_ttl(),
            enabled: true,
        }
    }
}

fn default_cache_ttl() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    pub fn load() -> Result<Self, AppError> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path).map_err(|e| AppError::Config {
            message: format!("Failed to read config at {}: {e}", path.display()),
        })?;
        toml::from_str(&contents).map_err(|e| AppError::Config {
            message: format!("Failed to parse config: {e}"),
        })
    }

    pub fn save(&self) -> Result<(), AppError> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Config {
                message: format!("Failed to create config directory: {e}"),
            })?;
        }
        let toml_str = toml::to_string_pretty(self).map_err(|e| AppError::Config {
            message: format!("Failed to serialize config: {e}"),
        })?;
        std::fs::write(&path, toml_str).map_err(|e| AppError::Config {
            message: format!("Failed to write config to {}: {e}", path.display()),
        })?;
        Ok(())
    }

    pub fn config_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "posthog", "posthog-cli")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("."));
                home.join(".config").join("posthog-cli")
            })
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn cache_dir() -> PathBuf {
        directories::ProjectDirs::from("com", "posthog", "posthog-cli")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(|| Self::config_dir().join("cache"))
    }
}
