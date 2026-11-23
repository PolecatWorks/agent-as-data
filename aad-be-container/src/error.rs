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

    /// Semantic validation or upstream capability error (422 Unprocessable Entity).
    #[error("Unprocessable Entity `{0}`")]
    UnprocessableEntity(String),

    /// Upstream service communication error (502 Bad Gateway).
    #[error("Bad Gateway `{0}`")]
    BadGateway(String),

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

    /// Timeout error for async operations.
    #[error("Timeout error `{0}`")]
    Timeout(#[from] tokio::time::error::Elapsed),

    /// LLM Completion Engine Error
    #[error("LLM error `{0}`")]
    CompletionError(#[from] rig_core::completion::CompletionError),

    /// HTTP Client build error
    #[error("LLM Client Build error `{0}`")]
    LlmClientError(#[from] rig_core::http_client::Error),

    /// Action was cancelled
    #[error("Action cancelled `{0}`")]
    Cancelled(String),

    /// Prompt error
    #[error("Prompt error `{0}`")]
    PromptError(#[from] rig::completion::PromptError),

    /// Model leaked a tool call as raw text
    #[error("Tool hallucination `{0}`")]
    ToolHallucination(String),

    /// LLM output could not be parsed into expected schema
    #[error("LLM Extraction failed `{0}`")]
    LlmExtractionError(String),
}

impl AppError {
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            AppError::Message(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => axum::http::StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => axum::http::StatusCode::FORBIDDEN,
            AppError::UnprocessableEntity(_) => axum::http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::BadGateway(_) => axum::http::StatusCode::BAD_GATEWAY,
            AppError::Serde(_) => axum::http::StatusCode::BAD_REQUEST,
            AppError::Io(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::JsonRejection(rejection) => rejection.status(),
            AppError::FigmentError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DatabaseError(sqlx::Error::RowNotFound) => axum::http::StatusCode::NOT_FOUND,
            AppError::DatabaseError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Timeout(_) => axum::http::StatusCode::GATEWAY_TIMEOUT,
            AppError::CompletionError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::LlmClientError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PromptError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ToolHallucination(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::LlmExtractionError(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Cancelled(_) => axum::http::StatusCode::CONFLICT,
        }
    }
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
            AppError::UnprocessableEntity(msg) => {
                (axum::http::StatusCode::UNPROCESSABLE_ENTITY, msg)
            }
            AppError::BadGateway(msg) => (axum::http::StatusCode::BAD_GATEWAY, msg),
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
            AppError::DatabaseError(error) => match error {
                sqlx::Error::RowNotFound => {
                    tracing::debug!("Database resource not found");
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        "Resource not found".to_string(),
                    )
                }
                _ => {
                    tracing::error!("Database error: {}", error);
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Database Error".to_string(),
                    )
                }
            },

            AppError::Timeout(error) => (
                axum::http::StatusCode::GATEWAY_TIMEOUT,
                format!("Request Timed Out: {}", error),
            ),
            AppError::CompletionError(error) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM Engine Error: {}", error),
            ),
            AppError::LlmClientError(error) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("LLM Client Error: {}", error),
            ),
            AppError::PromptError(error) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Prompt Error: {}", error),
            ),
            AppError::ToolHallucination(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
            AppError::LlmExtractionError(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                msg,
            ),
            AppError::Cancelled(msg) => (
                axum::http::StatusCode::CONFLICT,
                msg,
            ),
        };

        (status, axum::Json(ErrorResponse { message })).into_response()
    }
}
