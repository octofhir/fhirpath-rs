//! HTTP request handlers for the FHIRPath server

pub mod files_handler;

use crate::FhirPathValue;
use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    version::{ServerFhirVersion, extract_version_from_path},
};
use octofhir_fhirpath_analyzer::error::SourceLocation as AnalyzerSourceLocation;
use octofhir_fhirpath_analyzer::{AnalysisResult as AnalyzerResult, ValidationError};

use axum::{
    extract::{Query, State},
    http::Uri,
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{error, info, warn};

/// Query parameters for evaluation endpoints
#[derive(Debug, Deserialize)]
pub struct EvaluateQuery {
    /// Optional file to load as resource
    file: Option<String>,
}

/// Health check endpoint
pub async fn health_handler(
    State(registry): State<ServerRegistry>,
) -> ServerResult<Json<HealthResponse>> {
    let supported_versions: Vec<String> = registry
        .supported_versions()
        .into_iter()
        .map(|v| v.to_string())
        .collect();

    let mut engines = HashMap::new();
    for version in ServerFhirVersion::all() {
        let status = EngineStatus {
            available: registry.supports_version(*version),
            analysis_available: registry.supports_analysis(*version),
            initialized_at: "server_start".to_string(), // TODO: Track actual init time
        };
        engines.insert(version.to_string(), status);
    }

    let response = HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds: 0, // TODO: Track actual uptime
        fhir_versions: supported_versions,
        engines,
        memory: MemoryInfo {
            used_bytes: 0,      // TODO: Track actual memory usage
            total_bytes: 0,     // TODO: Track total memory
            usage_percent: 0.0, // TODO: Calculate percentage
        },
    };

    Ok(Json(response))
}

/// Versioned evaluation endpoint handler
pub async fn evaluate_handler(
    uri: Uri,
    Query(query): Query<EvaluateQuery>,
    State(registry): State<ServerRegistry>,
    Json(request): Json<EvaluateRequest>,
) -> ServerResult<Json<EvaluateResponse>> {
    let start_time = Instant::now();

    // Extract FHIR version from path
    let version = extract_version_from_path(uri.path())?;
    info!(
        "🔍 Evaluating expression for FHIR {}: {}",
        version, request.expression
    );

    // Get the pre-initialized engine for this version
    let engine =
        registry
            .get_evaluation_engine(version)
            .ok_or_else(|| ServerError::InvalidFhirVersion {
                version: version.to_string(),
            })?;

    // Determine resource to evaluate against
    let resource = if let Some(filename) = &query.file {
        // Load resource from file
        load_resource_from_file(filename).await?
    } else if let Some(resource) = request.resource {
        resource
    } else {
        return Err(ServerError::BadRequest {
            message: "Either 'resource' field or 'file' query parameter must be provided"
                .to_string(),
        });
    };

    // Convert variables from JSON to FhirPathValue
    let variables: HashMap<String, FhirPathValue> = request
        .variables
        .into_iter()
        .map(|(k, v)| (k, json_to_fhirpath_value(v)))
        .collect();

    // Perform evaluation
    let result = if variables.is_empty() {
        engine.evaluate(&request.expression, resource).await
    } else {
        engine
            .evaluate_with_variables(&request.expression, resource, variables)
            .await
    };

    let execution_time = start_time.elapsed();

    // Build response
    let response = match result {
        Ok(fhir_value) => {
            info!("✅ Expression evaluated successfully");
            EvaluateResponse {
                success: true,
                result: Some(crate::cli::server::models::fhir_value_to_json(fhir_value)),
                error: None,
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, true),
                trace: None, // TODO: Implement trace support
            }
        }
        Err(e) => {
            error!("❌ Expression evaluation failed: {}", e);
            EvaluateResponse {
                success: false,
                result: None,
                error: Some(ErrorInfo {
                    code: "EVALUATION_ERROR".to_string(),
                    message: e.to_string(),
                    details: Some(format!("{:?}", e)),
                    location: None, // TODO: Extract location from error
                }),
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, true),
                trace: None,
            }
        }
    };

    Ok(Json(response))
}

/// Versioned analysis endpoint handler
pub async fn analyze_handler(
    uri: Uri,
    State(registry): State<ServerRegistry>,
    Json(request): Json<AnalyzeRequest>,
) -> ServerResult<Json<AnalyzeResponse>> {
    let start_time = Instant::now();

    // Extract FHIR version from path
    let version = extract_version_from_path(uri.path())?;
    info!(
        "🔍 Analyzing expression for FHIR {}: {}",
        version, request.expression
    );

    // Get the pre-initialized analysis engine for this version
    let engine =
        registry
            .get_analysis_engine(version)
            .ok_or_else(|| ServerError::InvalidFhirVersion {
                version: version.to_string(),
            })?;

    // Perform analysis
    let result = engine.analyze_expression(&request.expression).await;
    let execution_time = start_time.elapsed();

    // Build response
    let response = match result {
        Ok(Some(analysis)) => {
            info!("✅ Expression analyzed successfully");
            AnalyzeResponse {
                success: analysis.validation_errors.is_empty(),
                analysis: Some(convert_analysis_result(analysis, &request.options)),
                error: None,
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, true),
            }
        }
        Ok(None) => {
            warn!("⚠️ No analysis result returned");
            AnalyzeResponse {
                success: false,
                analysis: None,
                error: Some(ErrorInfo {
                    code: "NO_ANALYZER".to_string(),
                    message: "Analyzer is not available for this version".to_string(),
                    details: None,
                    location: None,
                }),
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, true),
            }
        }
        Err(e) => {
            error!("❌ Expression analysis failed: {}", e);
            AnalyzeResponse {
                success: false,
                analysis: None,
                error: Some(ErrorInfo {
                    code: "ANALYSIS_ERROR".to_string(),
                    message: e.to_string(),
                    details: Some(format!("{:?}", e)),
                    location: None,
                }),
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, true),
            }
        }
    };

    Ok(Json(response))
}

// Helper functions

/// Load a FHIR resource from a file
async fn load_resource_from_file(filename: &str) -> ServerResult<serde_json::Value> {
    use std::path::PathBuf;
    use tokio::fs;

    let storage_dir = PathBuf::from("./storage");
    let file_path = storage_dir.join(filename);

    // Security check: ensure the resolved path is still within storage directory
    if !file_path.starts_with(&storage_dir) {
        return Err(ServerError::BadRequest {
            message: "Invalid file path".to_string(),
        });
    }

    if !file_path.exists() {
        return Err(ServerError::BadRequest {
            message: format!("File '{}' not found", filename),
        });
    }

    let content = fs::read_to_string(&file_path)
        .await
        .map_err(|e| ServerError::BadRequest {
            message: format!("Failed to read file '{}': {}", filename, e),
        })?;

    let json: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| ServerError::BadRequest {
            message: format!("Invalid JSON in file '{}': {}", filename, e),
        })?;

    Ok(json)
}

/// Convert JSON value to FhirPathValue
fn json_to_fhirpath_value(json: serde_json::Value) -> FhirPathValue {
    match json {
        serde_json::Value::Bool(b) => FhirPathValue::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FhirPathValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                use rust_decimal::Decimal;
                FhirPathValue::Decimal(Decimal::from_f64_retain(f).unwrap_or_default())
            } else {
                FhirPathValue::String(n.to_string().into())
            }
        }
        serde_json::Value::String(s) => FhirPathValue::String(s.into()),
        serde_json::Value::Array(arr) => {
            let values: Vec<FhirPathValue> = arr.into_iter().map(json_to_fhirpath_value).collect();
            FhirPathValue::Collection(values.into())
        }
        serde_json::Value::Object(_) => {
            // For now, convert objects to JSON strings
            // TODO: Properly handle FHIR nodes
            FhirPathValue::String(json.to_string().into())
        }
        serde_json::Value::Null => FhirPathValue::Empty,
    }
}

/// Convert analysis result to API format
fn convert_analysis_result(
    analysis: AnalyzerResult,
    _options: &AnalysisOptions,
) -> crate::cli::server::models::AnalysisResult {
    crate::cli::server::models::AnalysisResult {
        type_info: None, // TODO: Convert type information
        validation_errors: analysis
            .validation_errors
            .into_iter()
            .map(convert_validation_error)
            .collect(),
        type_annotations: analysis.type_annotations.len(),
        function_calls: analysis.function_calls.len(),
        union_types: analysis.union_types.len(),
    }
}

/// Convert validation error to API format
fn convert_validation_error(error: ValidationError) -> ValidationErrorInfo {
    ValidationErrorInfo {
        message: error.message,
        severity: format!("{:?}", error.error_type), // Convert error type to string
        location: error.location.map(|loc| SourceLocation {
            line: loc.line as usize,
            column: loc.column as usize,
            offset: loc.start,
        }),
    }
}
