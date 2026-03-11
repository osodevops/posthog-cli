use posthog::error::AppError;

#[test]
fn test_exit_codes() {
    assert_eq!(
        AppError::General {
            message: "test".into()
        }
        .exit_code(),
        1
    );
    assert_eq!(
        AppError::Auth {
            message: "test".into()
        }
        .exit_code(),
        2
    );
    assert_eq!(
        AppError::RateLimited {
            retry_after_secs: 5
        }
        .exit_code(),
        3
    );
    assert_eq!(
        AppError::NotFound {
            message: "test".into()
        }
        .exit_code(),
        4
    );
    assert_eq!(
        AppError::Validation {
            message: "test".into()
        }
        .exit_code(),
        5
    );
    assert_eq!(AppError::QueryTimeout.exit_code(), 6);
    assert_eq!(
        AppError::Server {
            status_code: 500,
            message: "test".into()
        }
        .exit_code(),
        7
    );
}

#[test]
fn test_error_codes() {
    assert_eq!(
        AppError::Auth {
            message: "test".into()
        }
        .error_code(),
        "auth_failed"
    );
    assert_eq!(
        AppError::RateLimited {
            retry_after_secs: 5
        }
        .error_code(),
        "rate_limited"
    );
    assert_eq!(
        AppError::NotFound {
            message: "test".into()
        }
        .error_code(),
        "not_found"
    );
    assert_eq!(AppError::QueryTimeout.error_code(), "query_timeout");
}

#[test]
fn test_error_json_envelope() {
    let err = AppError::Auth {
        message: "Invalid token".into(),
    };
    let json = err.to_json();

    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "auth_failed");
    assert!(json["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Invalid token"));
}

#[test]
fn test_rate_limited_json_includes_retry_after() {
    let err = AppError::RateLimited {
        retry_after_secs: 12,
    };
    let json = err.to_json();

    assert_eq!(json["ok"], false);
    assert_eq!(json["error"]["code"], "rate_limited");
    assert_eq!(json["error"]["retry_after_secs"], 12);
}

#[test]
fn test_error_display() {
    let err = AppError::Validation {
        message: "bad input".into(),
    };
    assert_eq!(err.to_string(), "Validation error: bad input");
}

#[test]
fn test_from_reqwest_timeout() {
    // We can't easily create a real reqwest timeout error, but we can test
    // the io error conversion
    let io_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
    let app_err = AppError::from(io_err);
    assert_eq!(app_err.exit_code(), 1);
}

#[test]
fn test_from_serde_error() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let app_err = AppError::from(json_err);
    assert!(app_err.to_string().contains("JSON error"));
}
