use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] mongodb::error::Error),

    #[error("PostgreSQL error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid ObjectId: {0}")]
    InvalidObjectId(#[from] mongodb::bson::oid::Error),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(#[from] uuid::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Internal error")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<ValidationError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, error_message, details) = match self {
            AppError::Database(ref e) => {
                tracing::error!("MongoDB error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database error occurred".to_string(),
                    None,
                )
            }
            AppError::Sqlx(ref e) => {
                tracing::error!("PostgreSQL error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "Database error occurred".to_string(),
                    None,
                )
            }
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg,
                None,
            ),
            AppError::InvalidObjectId(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_ID",
                "Invalid ID format".to_string(),
                None,
            ),
            AppError::InvalidUuid(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_UUID",
                "Invalid UUID format".to_string(),
                None,
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg,
                None,
            ),
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg,
                None,
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                msg,
                None,
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg,
                None,
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                msg,
                None,
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Too many requests. Please try again later.".to_string(),
                None,
            ),
            AppError::ServiceUnavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "Service temporarily unavailable".to_string(),
                None,
            ),
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    msg,
                    None,
                )
            }
            AppError::Internal(ref e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorDetail {
                code: error_code.to_string(),
                message: error_message,
                details,
                request_id: None, // TODO: Adicionar request ID do tracing
            },
        });

        (status, body).into_response()
    }
}

// Helper para criar erros de validação
impl AppError {
    pub fn validation_error(message: impl Into<String>) -> Self {
        AppError::Validation(message.into())
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        AppError::NotFound(format!("{} not found", resource.into()))
    }

    pub fn unauthorized() -> Self {
        AppError::Unauthorized("Authentication required".to_string())
    }

    pub fn forbidden() -> Self {
        AppError::Forbidden("You don't have permission to access this resource".to_string())
    }
}
