//! Error handling for the FHIRPath HTTP server

use crate::cli::server::models::{OperationOutcome, RequestError};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

/// Server-specific errors
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("FHIRPath evaluation error: {0}")]
    Evaluation(#[from] octofhir_fhirpath::FhirPathError),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Model error: {0}")]
    Model(#[from] octofhir_fhir_model::ModelError),

    #[error("Invalid FHIR version: {version}")]
    InvalidFhirVersion { version: String },

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("File not found: {filename}")]
    FileNotFound { filename: String },

    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid request: {message}")]
    BadRequest { message: String },

    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        // Map errors to FHIR OperationOutcome issue codes:
        // - "invalid" - Content invalid against the specification
        // - "not-found" - Resource not found
        // - "not-supported" - Feature not supported
        // - "processing" - Processing issues
        // - "exception" - Internal error/exception
        let (status, issue_code, message) = match self {
            ServerError::Evaluation(ref e) => (
                StatusCode::BAD_REQUEST,
                "processing",
                format!("FHIRPath evaluation failed: {}", e),
            ),
            ServerError::Analysis(ref e) => (
                StatusCode::BAD_REQUEST,
                "invalid",
                format!("Expression analysis failed: {}", e),
            ),
            ServerError::Model(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "exception",
                format!("Model provider error: {}", e),
            ),
            ServerError::InvalidFhirVersion { ref version } => (
                StatusCode::BAD_REQUEST,
                "not-supported",
                format!(
                    "Unsupported FHIR version: {}. Supported versions: r4, r4b, r5, r6",
                    version
                ),
            ),
            ServerError::NotSupported(ref msg) => {
                (StatusCode::NOT_IMPLEMENTED, "not-supported", msg.clone())
            }
            ServerError::FileNotFound { ref filename } => (
                StatusCode::NOT_FOUND,
                "not-found",
                format!("File not found: {}", filename),
            ),
            ServerError::InvalidJson(ref e) => (
                StatusCode::BAD_REQUEST,
                "invalid",
                format!("Invalid JSON format: {}", e),
            ),
            ServerError::BadRequest { ref message } => {
                (StatusCode::BAD_REQUEST, "invalid", message.clone())
            }
            ServerError::Io(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "exception",
                format!("File system error: {}", e),
            ),
            ServerError::Internal(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "exception",
                format!("Internal server error: {}", e),
            ),
        };

        // Return FHIR-compliant OperationOutcome
        let outcome = OperationOutcome::error(issue_code, &message, Some(format!("{:?}", self)));
        (status, Json(outcome)).into_response()
    }
}

/// Result type for server operations
pub type ServerResult<T> = Result<T, ServerError>;

impl From<RequestError> for ServerError {
    fn from(error: RequestError) -> Self {
        ServerError::BadRequest {
            message: error.to_string(),
        }
    }
}
