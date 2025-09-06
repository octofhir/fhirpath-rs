//! HTTP request handlers for the FHIRPath server

pub mod files_handler;

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    version::ServerFhirVersion,
};
use octofhir_fhirpath::FhirPathValue;
// Analysis types - will be added when analyzer is properly integrated

use axum::{
    extract::State,
    response::Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;
use tracing::info;

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

/// Simple test evaluation endpoint handler
pub async fn evaluate_handler() -> Result<Json<EvaluateResponse>, ServerError> {
    let start_time = Instant::now();
    let version = ServerFhirVersion::R4;

    info!("🔍 Simple test evaluation for FHIR {}", version);

    let execution_time = start_time.elapsed();

    // Return a simple test response
    let response = EvaluateResponse {
        success: true,
        result: Some(serde_json::json!({"test": "value"})),
        error: None,
        expression: "test expression".to_string(),
        fhir_version: version.to_string(),
        metadata: ExecutionMetadata::with_duration(execution_time, false),
        trace: None,
    };

    Ok(Json(response))
}

/// Simple test analysis endpoint handler
pub async fn analyze_handler() -> Result<Json<AnalyzeResponse>, ServerError> {
    let start_time = Instant::now();
    let version = ServerFhirVersion::R4;

    info!("🔍 Simple test analysis for FHIR {}", version);

    let execution_time = start_time.elapsed();

    // Return a simple test response
    let response = AnalyzeResponse {
        success: true,
        analysis: Some(crate::cli::server::models::AnalysisResult {
            type_info: None,
            validation_errors: Vec::new(),
            type_annotations: 0,
            function_calls: 0,
            union_types: 0,
        }),
        error: None,
        expression: "test expression".to_string(),
        fhir_version: version.to_string(),
        metadata: ExecutionMetadata::with_duration(execution_time, false),
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

/// Convert Collection to JSON for API response
fn collection_to_json(collection: octofhir_fhirpath::Collection) -> serde_json::Value {
    let values: Vec<serde_json::Value> = collection
        .iter()
        .map(|v| crate::cli::server::models::fhir_value_to_json(v.clone()))
        .collect();

    // If single value, return it directly; otherwise return as array
    if values.len() == 1 {
        values.into_iter().next().unwrap()
    } else {
        serde_json::Value::Array(values)
    }
}

/// Convert analysis result to API format
fn convert_analysis_result(
    _analysis: String, // Simplified for now
    _options: &AnalysisOptions,
) -> crate::cli::server::models::AnalysisResult {
    crate::cli::server::models::AnalysisResult {
        type_info: None,
        validation_errors: Vec::new(), // TODO: Add real validation errors when analyzer is integrated
        type_annotations: 0,
        function_calls: 0,
        union_types: 0,
    }
}

/// Version endpoint - required by task specification
pub async fn version_handler() -> Result<Json<serde_json::Value>, ServerError> {
    tracing::info!("🔖 Version info requested");

    let version_response = serde_json::json!({
        "service": "octofhir-fhirpath-server",
        "version": env!("CARGO_PKG_VERSION"),
        "build": {
            "date": "unknown", // TODO: Add build timestamp when available
            "commit": "unknown", // TODO: Add git commit info
        },
        "routes": [
            "GET /healthz - Health check",
            "GET /version - Version and build info",
            "POST /test/evaluate - Test evaluation endpoint",
            "POST /test/analyze - Test analysis endpoint",
            "GET / - Web UI root"
        ],
        "fhir_versions_supported": ["r4", "r4b", "r5"]
    });

    Ok(Json(version_response))
}
