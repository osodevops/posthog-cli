use std::process;

/// Exit codes per PRD specification.
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERAL: i32 = 1;
pub const EXIT_AUTH: i32 = 2;
pub const EXIT_RATE_LIMITED: i32 = 3;
pub const EXIT_NOT_FOUND: i32 = 4;
pub const EXIT_VALIDATION: i32 = 5;
pub const EXIT_QUERY_TIMEOUT: i32 = 6;
pub const EXIT_SERVER: i32 = 7;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("General error: {message}")]
    General { message: String },

    #[error("Authentication failed: {message}")]
    Auth { message: String },

    #[error("Rate limited: retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("Not found: {message}")]
    NotFound { message: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Query timeout")]
    QueryTimeout,

    #[error("Server error ({status_code}): {message}")]
    Server { status_code: u16, message: String },

    #[error("Network error: {message}")]
    Network { message: String },

    #[error("Configuration error: {message}")]
    Config { message: String },
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::General { .. } => EXIT_GENERAL,
            AppError::Auth { .. } => EXIT_AUTH,
            AppError::RateLimited { .. } => EXIT_RATE_LIMITED,
            AppError::NotFound { .. } => EXIT_NOT_FOUND,
            AppError::Validation { .. } => EXIT_VALIDATION,
            AppError::QueryTimeout => EXIT_QUERY_TIMEOUT,
            AppError::Server { .. } => EXIT_SERVER,
            AppError::Network { .. } => EXIT_GENERAL,
            AppError::Config { .. } => EXIT_GENERAL,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::General { .. } => "general_error",
            AppError::Auth { .. } => "auth_failed",
            AppError::RateLimited { .. } => "rate_limited",
            AppError::NotFound { .. } => "not_found",
            AppError::Validation { .. } => "validation_error",
            AppError::QueryTimeout => "query_timeout",
            AppError::Server { .. } => "server_error",
            AppError::Network { .. } => "network_error",
            AppError::Config { .. } => "config_error",
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let mut error = serde_json::json!({
            "code": self.error_code(),
            "message": self.to_string(),
        });

        if let AppError::RateLimited { retry_after_secs } = self {
            error["retry_after_secs"] = serde_json::json!(retry_after_secs);
        }

        serde_json::json!({
            "ok": false,
            "error": error,
        })
    }

    pub fn print_and_exit(self) -> ! {
        let code = self.exit_code();
        eprintln!("{}", self.to_json());
        process::exit(code);
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Network {
                message: format!("Request timed out: {err}"),
            }
        } else if err.is_connect() {
            AppError::Network {
                message: format!("Connection failed: {err}"),
            }
        } else {
            AppError::Network {
                message: err.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::General {
            message: format!("JSON error: {err}"),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::General {
            message: format!("IO error: {err}"),
        }
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        AppError::Validation {
            message: format!("Invalid URL: {err}"),
        }
    }
}
