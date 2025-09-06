//! Error handling for the FHIRPath HTTP server

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
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
        let (status, error_code, message) = match self {
            ServerError::Evaluation(ref e) => (
                StatusCode::BAD_REQUEST,
                "EVALUATION_ERROR",
                format!("FHIRPath evaluation failed: {}", e),
            ),
            ServerError::Analysis(ref e) => (
                StatusCode::BAD_REQUEST,
                "ANALYSIS_ERROR",
                format!("Expression analysis failed: {}", e),
            ),
            ServerError::Model(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "MODEL_ERROR",
                format!("Model provider error: {}", e),
            ),
            ServerError::InvalidFhirVersion { ref version } => (
                StatusCode::BAD_REQUEST,
                "INVALID_FHIR_VERSION",
                format!(
                    "Unsupported FHIR version: {}. Supported versions: r4, r4b, r5, r6",
                    version
                ),
            ),
            ServerError::FileNotFound { ref filename } => (
                StatusCode::NOT_FOUND,
                "FILE_NOT_FOUND",
                format!("File not found: {}", filename),
            ),
            ServerError::InvalidJson(ref e) => (
                StatusCode::BAD_REQUEST,
                "INVALID_JSON",
                format!("Invalid JSON format: {}", e),
            ),
            ServerError::BadRequest { ref message } => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", message.clone())
            }
            ServerError::Io(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                format!("File system error: {}", e),
            ),
            ServerError::Internal(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                format!("Internal server error: {}", e),
            ),
        };

        let body = Json(json!({
            "error": {
                "code": error_code,
                "message": message,
                "details": format!("{:?}", self),
            },
            "success": false,
        }));

        (status, body).into_response()
    }
}

/// Result type for server operations
pub type ServerResult<T> = Result<T, ServerError>;
