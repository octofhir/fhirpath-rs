//! HTTP request handlers for the FHIRPath server

pub mod files_handler;

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    version::ServerFhirVersion,
};
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use octofhir_fhirpath::{Collection, FhirPathValue};
// use axum_macros::debug_handler;
// Analysis types - will be added when analyzer is properly integrated

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
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
            "POST / - FHIRPath Lab API (auto-detect version)",
            "POST /r4 - FHIRPath Lab API (R4)",
            "POST /r4b - FHIRPath Lab API (R4B)",
            "POST /r5 - FHIRPath Lab API (R5)",
            "POST /r6 - FHIRPath Lab API (R6)",
            "GET / - Web UI root"
        ],
        "fhir_versions_supported": ["r4", "r4b", "r5", "r6"]
    });

    Ok(Json(version_response))
}

/// FHIRPath Lab API endpoint - auto-detect FHIR version
pub async fn fhirpath_lab_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> Json<FhirPathLabResponse> {
    // Create test response with Json extractor
    let mut response = FhirPathLabResponse::new();
    response.add_string_parameter("status", "test working WITH Json extractor".to_string());
    Json(response)
}

/// FHIRPath Lab API endpoint - R4  
pub async fn fhirpath_lab_r4_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> Json<FhirPathLabResponse> {
    // Create test response for R4
    let mut response = FhirPathLabResponse::new();
    response.add_string_parameter("status", "R4 handler working".to_string());
    Json(response)
}

/// FHIRPath Lab API endpoint - R4B
pub async fn fhirpath_lab_r4b_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> Json<FhirPathLabResponse> {
    // Create test response for R4B
    let mut response = FhirPathLabResponse::new();
    response.add_string_parameter("status", "R4B handler working".to_string());
    Json(response)
}

/// FHIRPath Lab API endpoint - R5
pub async fn fhirpath_lab_r5_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> Json<FhirPathLabResponse> {
    // Create test response for R5
    let mut response = FhirPathLabResponse::new();
    response.add_string_parameter("status", "R5 handler working".to_string());
    Json(response)
}

/// FHIRPath Lab API endpoint - R6
pub async fn fhirpath_lab_r6_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> Json<FhirPathLabResponse> {
    // Create test response for R6
    let mut response = FhirPathLabResponse::new();
    response.add_string_parameter("status", "R6 handler working".to_string());
    Json(response)
}

/// Core FHIRPath Lab API implementation using shared engines
async fn fhirpath_lab_handler_impl(
    registry: &ServerRegistry,
    request: FhirPathLabRequest,
    version: ServerFhirVersion,
) -> ServerResult<Json<FhirPathLabResponse>> {
    let start_time = Instant::now();

    info!("🔍 FHIRPath Lab API request for FHIR {}", version);

    // Parse the FHIR Parameters request
    let parsed_request = request
        .parse()
        .map_err(|e| ServerError::BadRequest { message: e })?;

    // Get the evaluation engine for the specified version
    let engine_arc =
        registry
            .get_evaluation_engine(version)
            .ok_or_else(|| ServerError::BadRequest {
                message: format!("FHIR version {} not supported", version),
            })?;

    let mut response = FhirPathLabResponse::new();

    // Add evaluator information
    response.add_string_parameter(
        "evaluator",
        format!("octofhir-fhirpath-{}", env!("CARGO_PKG_VERSION")),
    );

    // Add input parameters echo
    response.add_string_parameter("expression", parsed_request.expression.clone());
    response.add_resource_parameter("resource", parsed_request.resource.clone());

    if let Some(context) = &parsed_request.context {
        response.add_string_parameter("context", context.clone());
    }

    // Perform validation if requested
    if parsed_request.validate {
        // TODO: Add proper validation when analyzer is integrated
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
    }

    // Add AST representation as parseDebugTree
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            // Convert AST to JSON for debug tree representation
            let ast_json = serde_json::to_string_pretty(ast).unwrap_or_else(|_| "{}".to_string());
            response.add_string_parameter("parseDebugTree", ast_json);
        } else {
            response.add_string_parameter("parseDebugTree", "{}".to_string());
        }
    } else {
        response.add_string_parameter("parseDebugTree", "{}".to_string());
    }

    // Evaluate the expression - get engine and evaluate
    let result = {
        let mut engine = engine_arc.lock().unwrap();
        // Create a new tokio runtime handle for the async evaluation
        let handle = tokio::runtime::Handle::current();
        handle.block_on(evaluate_fhirpath_expression(&mut engine, &parsed_request))
    };

    match result {
        Ok(result) => {
            // Convert result to FHIR Parameters format
            let result_json = collection_to_json(result);

            // Create result parameter
            let mut result_parts = Vec::new();
            result_parts.push(FhirPathLabResponseParameter {
                name: "trace".to_string(),
                value_string: Some(format!(
                    "Evaluated expression: {}",
                    parsed_request.expression
                )),
                resource: None,
                part: None,
            });

            result_parts.push(FhirPathLabResponseParameter {
                name: "result".to_string(),
                value_string: None,
                resource: Some(result_json),
                part: None,
            });

            response.add_complex_parameter("result", result_parts);

            info!(
                "✅ FHIRPath Lab evaluation completed in {:?}",
                start_time.elapsed()
            );
        }
        Err(e) => {
            let error_msg = format!("Evaluation failed: {}", e);
            response.add_string_parameter("error", error_msg);
            info!("❌ FHIRPath Lab evaluation failed: {}", e);
        }
    }

    Ok(Json(response))
}

/// Alternative FHIRPath Lab API implementation using per-request engine creation
async fn fhirpath_lab_handler_impl_per_request(
    registry: ServerRegistry,
    request: FhirPathLabRequest,
    version: ServerFhirVersion,
) -> Json<FhirPathLabResponse> {
    let start_time = Instant::now();

    info!(
        "🔍 FHIRPath Lab API request for FHIR {} (per-request engine)",
        version
    );

    // Parse the FHIR Parameters request
    let parsed_request = match request.parse() {
        Ok(req) => req,
        Err(e) => {
            let mut error_response = FhirPathLabResponse::new();
            error_response.add_string_parameter("error", format!("Invalid request: {}", e));
            return Json(error_response);
        }
    };

    // Create a new engine for this request
    let (mut engine, engine_creation_time) = match registry.create_engine_for_version(version).await
    {
        Ok((engine, time)) => (engine, time),
        Err(e) => {
            let mut error_response = FhirPathLabResponse::new();
            error_response.add_string_parameter("error", format!("Engine creation failed: {}", e));
            return Json(error_response);
        }
    };
    info!(
        "📊 Per-request engine created in {:?}",
        engine_creation_time
    );

    let mut response = FhirPathLabResponse::new();

    // Add evaluator information
    response.add_string_parameter(
        "evaluator",
        format!("octofhir-fhirpath-{}", env!("CARGO_PKG_VERSION")),
    );

    // Add input parameters echo
    response.add_string_parameter("expression", parsed_request.expression.clone());
    response.add_resource_parameter("resource", parsed_request.resource.clone());

    if let Some(context) = &parsed_request.context {
        response.add_string_parameter("context", context.clone());
    }

    // Perform validation if requested
    if parsed_request.validate {
        // TODO: Add proper validation when analyzer is integrated
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
    }

    // Add AST representation as parseDebugTree
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            // Convert AST to JSON for debug tree representation
            let ast_json = serde_json::to_string_pretty(ast).unwrap_or_else(|_| "{}".to_string());
            response.add_string_parameter("parseDebugTree", ast_json);
        } else {
            response.add_string_parameter("parseDebugTree", "{}".to_string());
        }
    } else {
        response.add_string_parameter("parseDebugTree", "{}".to_string());
    }

    // Evaluate the expression directly with the per-request engine
    let evaluation_start = Instant::now();
    let result = evaluate_fhirpath_expression(&mut engine, &parsed_request).await;
    let evaluation_time = evaluation_start.elapsed();
    info!(
        "📊 Per-request evaluation completed in {:?}",
        evaluation_time
    );

    match result {
        Ok(result) => {
            // Convert result to FHIR Parameters format
            let result_json = collection_to_json(result);

            // Create result parameter
            let mut result_parts = Vec::new();
            result_parts.push(FhirPathLabResponseParameter {
                name: "trace".to_string(),
                value_string: Some(format!(
                    "Evaluated expression: {} (engine creation: {:?}, evaluation: {:?})",
                    parsed_request.expression, engine_creation_time, evaluation_time
                )),
                resource: None,
                part: None,
            });

            result_parts.push(FhirPathLabResponseParameter {
                name: "result".to_string(),
                value_string: None,
                resource: Some(result_json),
                part: None,
            });

            response.add_complex_parameter("result", result_parts);

            let total_time = start_time.elapsed();
            info!(
                "✅ FHIRPath Lab per-request evaluation completed in {:?} (engine: {:?}, eval: {:?})",
                total_time, engine_creation_time, evaluation_time
            );
        }
        Err(e) => {
            let error_msg = format!("Evaluation failed: {}", e);
            response.add_string_parameter("error", error_msg);
            info!("❌ FHIRPath Lab per-request evaluation failed: {}", e);
        }
    }

    Json(response)
}

/// Detect FHIR version from the request resource
fn detect_fhir_version(_request: &FhirPathLabRequest) -> Option<ServerFhirVersion> {
    // TODO: Implement actual FHIR version detection from resource
    // For now, default to R4
    Some(ServerFhirVersion::R4)
}

/// Evaluate FHIRPath expression using the engine
async fn evaluate_fhirpath_expression(
    engine: &mut octofhir_fhirpath::evaluator::FhirPathEngine,
    request: &ParsedFhirPathLabRequest,
) -> Result<Collection, ServerError> {
    use octofhir_fhirpath::evaluator::EvaluationContext;

    // Convert resource to FhirPathValue and create initial collection
    let resource_value = json_to_fhirpath_value(request.resource.clone());
    let context_collection = Collection::single(resource_value);

    // Create evaluation context
    let mut eval_context = EvaluationContext::new(context_collection);

    // Set variables
    for (name, value) in &request.variables {
        let fhir_value = json_to_fhirpath_value(value.clone());
        eval_context.set_variable(name.to_string(), fhir_value);
    }

    // First parse the expression to get the AST
    let parse_result = parse_with_mode(&request.expression, ParsingMode::Analysis);

    if !parse_result.success {
        let error_details: Vec<String> = parse_result
            .diagnostics
            .iter()
            .map(|d| {
                if let Some(location) = &d.location {
                    format!("{} at {}:{}", d.code.code, location.line, location.column)
                } else {
                    d.code.code.clone()
                }
            })
            .collect();

        let error_message = if error_details.is_empty() {
            "Parse failed - unknown error".to_string()
        } else {
            format!("Parse failed: {}", error_details.join(", "))
        };

        return Err(ServerError::BadRequest {
            message: error_message,
        });
    }

    // Parse successful - now evaluate using the AST
    let ast = parse_result.ast.unwrap();

    let result = engine.evaluate_ast(&ast, &eval_context).await?;

    // Convert the result to a Collection
    Ok(result.into())
}
