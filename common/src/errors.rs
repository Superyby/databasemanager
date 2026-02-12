//! Application error types.
//!
//! Defines custom error types with automatic HTTP response conversion.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use tracing::{error, warn};

/// Application error enumeration.
///
/// Each variant automatically converts to an appropriate HTTP status code
/// and error response when returned from a handler.
#[derive(Debug, Error)]
pub enum AppError {
    // ============== Client Errors (4xx) ==============

    /// Invalid user input or request parameters.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Validation error with field details.
    #[error("validation failed: {0}")]
    Validation(String),

    /// Resource not found.
    #[error("resource not found: {0}")]
    NotFound(String),

    /// Database connection not found.
    #[error("connection not found: {0}")]
    ConnectionNotFound(String),

    /// Unauthorized access.
    #[error("unauthorized")]
    Unauthorized,

    /// Forbidden access.
    #[error("forbidden: {0}")]
    Forbidden(String),

    /// Conflict with existing resource.
    #[error("conflict: {0}")]
    Conflict(String),

    /// Unsafe SQL operation.
    #[error("unsafe SQL: {0}")]
    UnsafeSql(String),

    // ============== Server Errors (5xx) ==============

    /// Database connection error.
    #[error("database connection failed: {0}")]
    DatabaseConnection(String),

    /// Database query error.
    #[error("database query failed: {0}")]
    DatabaseQuery(String),

    /// Redis connection error.
    #[error("redis connection failed: {0}")]
    RedisConnection(String),

    /// Redis operation error.
    #[error("redis operation failed: {0}")]
    RedisOperation(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Configuration(String),

    /// External service error.
    #[error("external service error: {0}")]
    ExternalService(String),

    /// Timeout error.
    #[error("operation timeout: {0}")]
    Timeout(String),

    /// Service unavailable.
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Unsupported database type.
    #[error("unsupported database type: {0}")]
    UnsupportedDatabaseType(String),
}

impl AppError {
    /// Returns the error code string for this error type.
    pub fn code(&self) -> &'static str {
        match self {
            // Client errors
            AppError::InvalidInput(_) => "INVALID_INPUT",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::ConnectionNotFound(_) => "CONNECTION_NOT_FOUND",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::Conflict(_) => "CONFLICT",
            AppError::UnsafeSql(_) => "UNSAFE_SQL",
            // Server errors
            AppError::DatabaseConnection(_) => "DATABASE_CONNECTION_ERROR",
            AppError::DatabaseQuery(_) => "DATABASE_QUERY_ERROR",
            AppError::RedisConnection(_) => "REDIS_CONNECTION_ERROR",
            AppError::RedisOperation(_) => "REDIS_OPERATION_ERROR",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Configuration(_) => "CONFIGURATION_ERROR",
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR",
            AppError::Timeout(_) => "TIMEOUT",
            AppError::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            AppError::UnsupportedDatabaseType(_) => "UNSUPPORTED_DATABASE_TYPE",
        }
    }

    /// Returns the HTTP status code for this error type.
    pub fn status_code(&self) -> StatusCode {
        match self {
            // Client errors (4xx)
            AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::ConnectionNotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::UnsafeSql(_) => StatusCode::BAD_REQUEST,
            AppError::UnsupportedDatabaseType(_) => StatusCode::BAD_REQUEST,
            // Server errors (5xx)
            AppError::DatabaseConnection(_) => StatusCode::BAD_GATEWAY,
            AppError::DatabaseQuery(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RedisConnection(_) => StatusCode::BAD_GATEWAY,
            AppError::RedisOperation(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ExternalService(_) => StatusCode::BAD_GATEWAY,
            AppError::Timeout(_) => StatusCode::GATEWAY_TIMEOUT,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Returns the response code (200=success, 4xx=client error, 5xx=server error, 7xx=business, 8xx=database, 9xx=external).
    pub fn response_code(&self) -> i32 {
        use crate::response::code;
        match self {
            // 客户端错误 (4xx)
            AppError::InvalidInput(_) => code::BAD_REQUEST,
            AppError::Validation(_) => code::VALIDATION_ERROR,
            AppError::Unauthorized => code::UNAUTHORIZED,
            AppError::Forbidden(_) => code::FORBIDDEN,
            
            // 业务异常 (7xx)
            AppError::NotFound(_) => code::DATA_NOT_FOUND,
            AppError::Conflict(_) => code::DATA_ALREADY_EXISTS,
            
            // 数据库相关 (8xx)
            AppError::ConnectionNotFound(_) => code::DB_CONNECTION_NOT_FOUND,
            AppError::UnsupportedDatabaseType(_) => code::DB_UNSUPPORTED_TYPE,
            AppError::UnsafeSql(_) => code::DB_UNSAFE_SQL,
            AppError::DatabaseConnection(_) => code::DB_CONNECTION_ERROR,
            AppError::DatabaseQuery(_) => code::DB_QUERY_ERROR,
            AppError::RedisConnection(_) => code::REDIS_CONNECTION_ERROR,
            AppError::RedisOperation(_) => code::REDIS_OPERATION_ERROR,
            
            // 服务器错误 (5xx)
            AppError::Internal(_) => code::INTERNAL_ERROR,
            AppError::Configuration(_) => code::INTERNAL_ERROR,
            AppError::Timeout(_) => code::GATEWAY_TIMEOUT,
            AppError::ServiceUnavailable(_) => code::SERVICE_UNAVAILABLE,
            
            // 外部服务 (9xx)
            AppError::ExternalService(_) => code::EXTERNAL_SERVICE_ERROR,
        }
    }

    /// Returns whether this error should be logged as an error or warning.
    fn is_server_error(&self) -> bool {
        self.status_code().is_server_error()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the error appropriately
        if self.is_server_error() {
            error!(error_code = %self.code(), error = %self, "Server error occurred");
        } else {
            warn!(error_code = %self.code(), error = %self, "Client error occurred");
        }

        // Don't expose internal error details to clients
        let message = match &self {
            AppError::Internal(_) => "服务器内部错误".to_string(),
            AppError::Configuration(_) => "配置错误".to_string(),
            e => e.to_string(),
        };

        let body = Json(json!({
            "code": self.response_code(),
            "message": message,
            "success": false,
            "error": {
                "code": self.code(),
                "message": message
            },
            "meta": {
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        }));

        (self.status_code(), body).into_response()
    }
}

// ============== Error Conversions ==============

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("Database record not found".into()),
            sqlx::Error::PoolTimedOut => {
                AppError::Timeout("Database connection pool timeout".into())
            }
            sqlx::Error::Configuration(e) => AppError::Configuration(e.to_string()),
            _ => AppError::DatabaseQuery(err.to_string()),
        }
    }
}

impl From<redis::RedisError> for AppError {
    fn from(err: redis::RedisError) -> Self {
        if err.is_connection_dropped() || err.is_io_error() {
            AppError::RedisConnection(err.to_string())
        } else {
            AppError::RedisOperation(err.to_string())
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::InvalidInput(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal(format!("IO error: {}", err))
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let errors: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!(
                        "{}: {}",
                        field,
                        e.message.as_ref().map(|m| m.to_string()).unwrap_or_default()
                    )
                })
            })
            .collect();
        AppError::Validation(errors.join("; "))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Timeout(format!("HTTP request timeout: {}", err))
        } else if err.is_connect() {
            AppError::ExternalService(format!("Connection failed: {}", err))
        } else {
            AppError::ExternalService(format!("HTTP request failed: {}", err))
        }
    }
}

impl From<std::env::VarError> for AppError {
    fn from(err: std::env::VarError) -> Self {
        AppError::Configuration(format!("Environment variable error: {}", err))
    }
}

/// Result type alias for AppError.
pub type AppResult<T> = Result<T, AppError>;
