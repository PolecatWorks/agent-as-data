//! Custom application error types and Axum HTTP response implementations.

use axum::extract::rejection::JsonRejection;
use thiserror::Error;

/// Root error type for backend operations and API responses.
#[derive(Error, Debug)]
pub enum AppError {
    /// General internal server error message.
    #[error("General error `{0}`")]
    Message(String),

    /// Client request invalid error (400 Bad Request).
    #[error("Bad Request `{0}`")]
    BadRequest(String),

    /// Requested resource was not found (404 Not Found).
    #[error("Not Found: `{0}`")]
    NotFound(String),

    /// Missing or invalid authentication credentials (401 Unauthorized).
    #[error("Unauthorized `{0}`")]
    Unauthorized(String),

    /// Insufficient role or access permissions (403 Forbidden).
    #[error("Forbidden `{0}`")]
    Forbidden(String),

    /// JSON serialization or deserialization error.
    #[error("Serde error `{0}`")]
    Serde(#[from] serde_json::Error),

    /// I/O operation error.
    #[error("IO error `{0}`")]
    Io(#[from] std::io::Error),

    /// Axum JSON extractor rejection error.
    #[error("Json Rejection `{0}`")]
    JsonRejection(#[from] JsonRejection),

    /// Configuration parsing error from Figment.
    #[error("Figment error `{0}`")]
    FigmentError(#[from] Box<figment::error::Error>),

    /// PostgreSQL database query or connection pool error.
    #[error("Database error `{0}`")]
    DatabaseError(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        #[derive(serde::Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::Message(msg) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::BadRequest(msg) => (axum::http::StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (axum::http::StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized(msg) => (axum::http::StatusCode::UNAUTHORIZED, msg),
            AppError::Forbidden(msg) => (axum::http::StatusCode::FORBIDDEN, msg),
            AppError::Serde(error) => (
                axum::http::StatusCode::BAD_REQUEST,
                format!("JSON Error: {}", error),
            ),
            AppError::Io(error) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO Error: {}", error),
            ),
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            AppError::FigmentError(error) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Config Error: {}", error),
            ),
            AppError::DatabaseError(error) => {
                tracing::error!("Database error: {}", error);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error".to_string(),
                )
            }
        };

        (status, axum::Json(ErrorResponse { message })).into_response()
    }
}
