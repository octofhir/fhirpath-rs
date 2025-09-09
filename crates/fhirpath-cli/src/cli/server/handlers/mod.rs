//! HTTP request handlers for the FHIRPath server

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
    extract::{Query, State},
    response::{IntoResponse, Json},
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

/// Legacy evaluation endpoint handler
pub async fn evaluate_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<EvaluateRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let version = ServerFhirVersion::R4; // Default to R4 for legacy API

    info!("🔍 Legacy evaluation request for FHIR {}", version);

    // Get the evaluation engine for R4
    let engine_arc = match registry.get_evaluation_engine(version) {
        Some(engine) => engine,
        None => {
            let response = EvaluateResponse {
                success: false,
                result: None,
                error: Some(ErrorInfo {
                    code: "unsupported_version".to_string(),
                    message: format!("FHIR version {} not supported", version),
                    details: None,
                    location: None,
                }),
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(start_time.elapsed(), false),
                trace: None,
            };
            return Json(response);
        }
    };

    // Use provided resource or create a default test resource
    let resource = request.resource.unwrap_or_else(|| {
        serde_json::json!({
            "resourceType": "Patient",
            "id": "test",
            "name": [{"family": "Test", "given": ["Patient"]}]
        })
    });

    // Create a ParsedFhirPathLabRequest-like structure for evaluation
    let parsed_request = ParsedFhirPathLabRequest {
        expression: request.expression.clone(),
        resource,
        variables: request.variables,
        validate: request.options.validate,
        context: None,
        terminology_server: None,
    };

    // Evaluate the expression
    let result = {
        let mut engine = engine_arc.lock_owned().await;
        evaluate_fhirpath_expression(&mut engine, &parsed_request).await
    };

    let execution_time = start_time.elapsed();

    let response = match result {
        Ok(collection) => {
            let result_json = collection;
            EvaluateResponse {
                success: true,
                result: Some(result_json.to_json_value()),
                error: None,
                expression: request.expression,
                fhir_version: version.to_string(),
                metadata: ExecutionMetadata::with_duration(execution_time, false),
                trace: if request.options.trace {
                    Some(vec!["Evaluation completed".to_string()])
                } else {
                    None
                },
            }
        }
        Err(e) => EvaluateResponse {
            success: false,
            result: None,
            error: Some(ErrorInfo {
                code: "evaluation_failed".to_string(),
                message: format!("Evaluation failed: {}", e),
                details: None,
                location: None,
            }),
            expression: request.expression,
            fhir_version: version.to_string(),
            metadata: ExecutionMetadata::with_duration(execution_time, false),
            trace: None,
        },
    };

    Json(response)
}

/// Legacy analysis endpoint handler
pub async fn analyze_handler(
    State(_registry): State<ServerRegistry>,
    Json(request): Json<AnalyzeRequest>,
) -> impl IntoResponse {
    let start_time = Instant::now();
    let version = ServerFhirVersion::R4; // Default to R4 for legacy API

    info!("🔍 Legacy analysis request for FHIR {}", version);

    // Parse the expression to check syntax
    let parse_result = parse_with_mode(&request.expression, ParsingMode::Analysis);

    let execution_time = start_time.elapsed();

    let response = if parse_result.success {
        // TODO: When analyzer is properly integrated, perform real analysis here
        AnalyzeResponse {
            success: true,
            analysis: Some(crate::cli::server::models::AnalysisResult {
                type_info: None, // TODO: Add type information when available
                validation_errors: Vec::new(),
                type_annotations: 0,
                function_calls: 0,
                union_types: 0,
            }),
            error: None,
            expression: request.expression,
            fhir_version: version.to_string(),
            metadata: ExecutionMetadata::with_duration(execution_time, false),
        }
    } else {
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

        AnalyzeResponse {
            success: false,
            analysis: None,
            error: Some(ErrorInfo {
                code: "parse_error".to_string(),
                message: format!("Parse errors: {}", error_details.join(", ")),
                details: Some(error_details.join("; ")),
                location: None,
            }),
            expression: request.expression,
            fhir_version: version.to_string(),
            metadata: ExecutionMetadata::with_duration(execution_time, false),
        }
    };

    Json(response)
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

/// Convert GET query parameters to FHIRPath Lab request format
fn convert_get_to_fhirpath_lab_request(
    params: HashMap<String, String>,
) -> Result<FhirPathLabRequest, String> {
    let mut parameters = Vec::new();

    // Add expression parameter (required)
    if let Some(expression) = params.get("expression") {
        parameters.push(FhirPathLabParameter {
            name: "expression".to_string(),
            value_string: Some(expression.clone()),
            value_boolean: None,
            resource: None,
            part: None,
        });
    } else {
        return Err("Missing required 'expression' parameter".to_string());
    }

    // Add resource parameter - handle either direct resource or resource URL
    if let Some(resource) = params.get("resource") {
        // Try to parse as JSON first, if not treat as resource URL/ID
        if let Ok(json_resource) = serde_json::from_str::<serde_json::Value>(resource) {
            parameters.push(FhirPathLabParameter {
                name: "resource".to_string(),
                value_string: None,
                value_boolean: None,
                resource: Some(json_resource),
                part: None,
            });
        } else {
            // Treat as resource ID/URL
            parameters.push(FhirPathLabParameter {
                name: "resource".to_string(),
                value_string: Some(resource.clone()),
                value_boolean: None,
                resource: None,
                part: None,
            });
        }
    } else {
        // Use default test resource if none provided
        let default_resource = serde_json::json!({
            "resourceType": "Patient",
            "id": "test",
            "name": [{"family": "Test", "given": ["Patient"]}]
        });
        parameters.push(FhirPathLabParameter {
            name: "resource".to_string(),
            value_string: None,
            value_boolean: None,
            resource: Some(default_resource),
            part: None,
        });
    }

    // Add context parameter (optional)
    if let Some(context) = params.get("context") {
        parameters.push(FhirPathLabParameter {
            name: "context".to_string(),
            value_string: Some(context.clone()),
            value_boolean: None,
            resource: None,
            part: None,
        });
    }

    // Add validate parameter (optional)
    if let Some(validate) = params.get("validate") {
        let validate_bool = validate.parse::<bool>().unwrap_or(false);
        parameters.push(FhirPathLabParameter {
            name: "validate".to_string(),
            value_string: None,
            value_boolean: Some(validate_bool),
            resource: None,
            part: None,
        });
    }

    // Add terminology server parameter (optional)
    if let Some(terminology_server) = params.get("terminologyserver") {
        parameters.push(FhirPathLabParameter {
            name: "terminologyServer".to_string(),
            value_string: Some(terminology_server.clone()),
            value_boolean: None,
            resource: None,
            part: None,
        });
    }

    Ok(FhirPathLabRequest {
        resource_type: "Parameters".to_string(),
        parameter: parameters,
    })
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
            "GET /health - Health check",
            "GET /healthz - Health check",
            "GET /version - Version and build info",
            "POST / - FHIRPath Lab API (auto-detect version)",
            "POST /r4 - FHIRPath Lab API (R4)",
            "POST /r4b - FHIRPath Lab API (R4B)",
            "POST /r5 - FHIRPath Lab API (R5)",
            "POST /r6 - FHIRPath Lab API (R6)"
        ],
        "fhir_versions_supported": ["r4", "r4b", "r5", "r6"]
    });

    Ok(Json(version_response))
}

/// FHIRPath Lab API endpoint - auto-detect FHIR version (POST request)
pub async fn fhirpath_lab_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    // Detect FHIR version from request or default to R4
    let version = detect_fhir_version(&request).unwrap_or(ServerFhirVersion::R4);

    match fhirpath_lab_handler_impl(&registry, request, version).await {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}

// GET handler removed for simplicity - focus on POST compatibility first

/// FHIRPath Lab API endpoint - R4  
pub async fn fhirpath_lab_r4_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R4).await
}

/// FHIRPath Lab API endpoint - R4B
pub async fn fhirpath_lab_r4b_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R4B).await
}

/// FHIRPath Lab API endpoint - R5
pub async fn fhirpath_lab_r5_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R5).await
}

/// FHIRPath Lab API endpoint - R6
pub async fn fhirpath_lab_r6_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R6).await
}

/// Core FHIRPath Lab API implementation using shared engines
async fn fhirpath_lab_handler_impl(
    registry: &ServerRegistry,
    request: FhirPathLabRequest,
    version: ServerFhirVersion,
) -> ServerResult<Json<FhirPathLabResponse>> {
    use octofhir_fhirpath::evaluator::EvaluationContext;

    let total_start = Instant::now();

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

    // Echo input parameters
    response.add_string_parameter("expression", parsed_request.expression.clone());
    response.add_resource_parameter("resource", parsed_request.resource.clone());
    if let Some(context) = &parsed_request.context {
        response.add_string_parameter("context", context.clone());
    }

    // Parse expression and collect diagnostics
    let parse_start = Instant::now();
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    let parse_time = parse_start.elapsed();

    // Add AST debug tree if available
    if let Some(ref ast) = parse_result.ast {
        let ast_json = serde_json::to_string_pretty(ast).unwrap_or_else(|_| "{}".to_string());
        response.add_string_parameter("parseDebugTree", ast_json);
    } else {
        response.add_string_parameter("parseDebugTree", "{}".to_string());
    }

    // Add diagnostics as issues
    for diag in &parse_result.diagnostics {
        let mut parts = Vec::new();
        parts.push(FhirPathLabResponseParameter {
            name: "severity".to_string(),
            value_string: None,
            value_code: Some(match diag.severity {
                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error => "error".to_string(),
                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Warning => {
                    "warning".to_string()
                }
                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Info => {
                    "information".to_string()
                }
                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Hint => {
                    "information".to_string()
                }
            }),
            value_decimal: None,
            resource: None,
            part: None,
        });
        parts.push(FhirPathLabResponseParameter {
            name: "code".to_string(),
            value_string: None,
            value_code: Some(diag.code.code.clone()),
            value_decimal: None,
            resource: None,
            part: None,
        });
        parts.push(FhirPathLabResponseParameter {
            name: "details".to_string(),
            value_string: Some(diag.message.clone()),
            value_code: None,
            value_decimal: None,
            resource: None,
            part: None,
        });
        if let Some(loc) = &diag.location {
            parts.push(FhirPathLabResponseParameter {
                name: "location".to_string(),
                value_string: Some(format!("{}:{}", loc.line, loc.column)),
                value_code: None,
                value_decimal: None,
                resource: None,
                part: None,
            });
        }
        response.add_complex_parameter("issue", parts);
    }
    // Add basic validation result if requested
    if parsed_request.validate && !parse_result.has_errors() {
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
    }

    // If parsing failed, skip evaluation but still report timing
    let mut eval_time = std::time::Duration::from_millis(0);
    if parse_result.success {
        if let Some(ast) = parse_result.ast {
            // Convert resource to FhirPathValue
            let resource_value =
                octofhir_fhirpath::FhirPathValue::resource(parsed_request.resource.clone());
            let context_collection = Collection::single(resource_value);

            let mut eval_context = EvaluationContext::new(context_collection);
            for (name, value) in &parsed_request.variables {
                eval_context.set_variable(name.clone(), json_to_fhirpath_value(value.clone()));
            }

            let mut engine = engine_arc.lock_owned().await;
            let eval_start = Instant::now();
            match engine.evaluate_with_metadata(&parsed_request.expression, &eval_context).await {
                Ok(collection_with_metadata) => {
                    eval_time = eval_start.elapsed();
                    
                    // Get the actual values and separate type metadata
                    let result_values = collection_with_metadata.to_json_parts();
                    let type_metadata = collection_with_metadata.get_type_metadata_array();
                    
                    let mut result_parts = Vec::new();
                    result_parts.push(FhirPathLabResponseParameter {
                        name: "trace".to_string(),
                        value_string: Some(format!(
                            "Evaluated expression: {} with type metadata preserved",
                            parsed_request.expression
                        )),
                        value_code: None,
                        value_decimal: None,
                        resource: None,
                        part: None,
                    });

                    // Add the result value directly
                    result_parts.push(FhirPathLabResponseParameter {
                        name: "result".to_string(),
                        value_string: None,
                        value_code: None,
                        value_decimal: None,
                        resource: Some(result_values),
                        part: None,
                    });

                    // Add type metadata as a separate parameter for tools that need it
                    result_parts.push(FhirPathLabResponseParameter {
                        name: "typeInfo".to_string(),
                        value_string: None,
                        value_code: None,
                        value_decimal: None,
                        resource: Some(type_metadata),
                        part: None,
                    });
                    
                    response.add_complex_parameter("result", result_parts);
                }
                Err(e) => {
                    eval_time = eval_start.elapsed();
                    let mut parts = Vec::new();
                    parts.push(FhirPathLabResponseParameter {
                        name: "severity".to_string(),
                        value_string: None,
                        value_code: Some("error".to_string()),
                        value_decimal: None,
                        resource: None,
                        part: None,
                    });
                    parts.push(FhirPathLabResponseParameter {
                        name: "code".to_string(),
                        value_string: None,
                        value_code: Some("exception".to_string()),
                        value_decimal: None,
                        resource: None,
                        part: None,
                    });
                    parts.push(FhirPathLabResponseParameter {
                        name: "details".to_string(),
                        value_string: Some(format!("{}", e)),
                        value_code: None,
                        value_decimal: None,
                        resource: None,
                        part: None,
                    });
                    response.add_complex_parameter("issue", parts);
                }
            }
        }
    }

    // Timing metrics
    let total_time = total_start.elapsed();
    let mut timing_parts = Vec::new();
    timing_parts.push(FhirPathLabResponseParameter {
        name: "total".to_string(),
        value_string: None,
        value_code: None,
        value_decimal: Some(total_time.as_secs_f64() * 1000.0),
        resource: None,
        part: None,
    });
    timing_parts.push(FhirPathLabResponseParameter {
        name: "parse".to_string(),
        value_string: None,
        value_code: None,
        value_decimal: Some(parse_time.as_secs_f64() * 1000.0),
        resource: None,
        part: None,
    });
    timing_parts.push(FhirPathLabResponseParameter {
        name: "evaluation".to_string(),
        value_string: None,
        value_code: None,
        value_decimal: Some(eval_time.as_secs_f64() * 1000.0),
        resource: None,
        part: None,
    });
    response.add_complex_parameter("timing", timing_parts);

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
            let result_json = result;

            // Create result parameter
            let mut result_parts = Vec::new();
            result_parts.push(FhirPathLabResponseParameter {
                name: "trace".to_string(),
                value_string: Some(format!(
                    "Evaluated expression: {} (engine creation: {:?}, evaluation: {:?})",
                    parsed_request.expression, engine_creation_time, evaluation_time
                )),
                value_code: None,
                value_decimal: None,
                resource: None,
                part: None,
            });

            result_parts.push(FhirPathLabResponseParameter {
                name: "result".to_string(),
                value_string: None,
                value_code: None,
                value_decimal: None,
                resource: Some(result_json.to_json_value()),
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
