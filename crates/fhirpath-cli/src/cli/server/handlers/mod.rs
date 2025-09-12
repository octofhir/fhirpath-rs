//! HTTP request handlers for the FHIRPath server

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    response::create_error_response,
    version::ServerFhirVersion,
};
use octofhir_fhirpath::core::CollectionWithMetadata;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use octofhir_fhirpath::{Collection, FhirPathValue};
use serde_json::Value as JsonValue;
// use axum_macros::debug_handler;
// Analysis types - will be added when analyzer is properly integrated

use crate::cli::ast::{add_type_information, convert_ast_to_lab_format, extract_resource_type};
use axum::{
    extract::State,
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
    let version = ServerFhirVersion::R4;

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
                result: Some(result_json.to_json_parts()),
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
    let version = ServerFhirVersion::R4;

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
        Err(error) => {
            // Return standardized error response
            let error_response = create_error_response(
                "processing",
                &format!("Request processing failed: {}", error),
                None,
            )
            .build();
            Json(serde_json::to_value(error_response).unwrap()).into_response()
        }
    }
}

// GET handler removed for simplicity - focus on POST compatibility first

/// FHIRPath Lab API endpoint - R4  
pub async fn fhirpath_lab_r4_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    match fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R4).await {
        Ok(response) => response.into_response(),
        Err(error) => {
            let error_response = create_error_response(
                "processing",
                &format!("R4 request processing failed: {}", error),
                None,
            )
            .build();
            Json(serde_json::to_value(error_response).unwrap()).into_response()
        }
    }
}

/// FHIRPath Lab API endpoint - R4B
pub async fn fhirpath_lab_r4b_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    match fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R4B).await {
        Ok(response) => response.into_response(),
        Err(error) => {
            let error_response = create_error_response(
                "processing",
                &format!("R4B request processing failed: {}", error),
                None,
            )
            .build();
            Json(serde_json::to_value(error_response).unwrap()).into_response()
        }
    }
}

/// FHIRPath Lab API endpoint - R5
pub async fn fhirpath_lab_r5_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    match fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R5).await {
        Ok(response) => response.into_response(),
        Err(error) => {
            let error_response = create_error_response(
                "processing",
                &format!("R5 request processing failed: {}", error),
                None,
            )
            .build();
            Json(serde_json::to_value(error_response).unwrap()).into_response()
        }
    }
}

/// FHIRPath Lab API endpoint - R6
pub async fn fhirpath_lab_r6_handler(
    State(registry): State<ServerRegistry>,
    Json(request): Json<FhirPathLabRequest>,
) -> impl IntoResponse {
    match fhirpath_lab_handler_impl(&registry, request, ServerFhirVersion::R6).await {
        Ok(response) => response.into_response(),
        Err(error) => {
            let error_response = create_error_response(
                "processing",
                &format!("R6 request processing failed: {}", error),
                None,
            )
            .build();
            Json(serde_json::to_value(error_response).unwrap()).into_response()
        }
    }
}

/// Core FHIRPath Lab API using shared engines
async fn fhirpath_lab_handler_impl(
    registry: &ServerRegistry,
    request: FhirPathLabRequest,
    version: ServerFhirVersion,
) -> ServerResult<Json<serde_json::Value>> {
    use octofhir_fhirpath::evaluator::EvaluationContext;

    let total_start = Instant::now();

    info!("🔍 FHIRPath Lab API request for FHIR {}", version);

    // Parse the FHIR Parameters request
    let parsed_request = match request.parse() {
        Ok(req) => req,
        Err(e) => {
            // Return standardized error response
            let error_response =
                create_error_response("structure", &format!("Invalid request format: {}", e), None)
                    .build();
            return Ok(Json(serde_json::to_value(error_response).unwrap()));
        }
    };

    // Get the evaluation engine for the specified version
    let engine_arc = match registry.get_evaluation_engine(version) {
        Some(engine) => engine,
        None => {
            // Return standardized error response
            let error_response = create_error_response(
                "not-supported",
                &format!("FHIR version {} not supported", version),
                None,
            )
            .build();
            return Ok(Json(serde_json::to_value(error_response).unwrap()));
        }
    };

    let mut response = FhirPathLabResponse::new();

    response.id = Some("fhirpath".to_string());

    // Parse expression and collect diagnostics
    let parse_start = Instant::now();
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    let parse_time = parse_start.elapsed();

    // Create parameters part to hold all sub-parameters
    let mut parameters_parts: Vec<FhirPathLabResponseParameter> = Vec::new();

    // Evaluator information will be added later to avoid duplication

    // Add expression to parameters part
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "expression".to_string(),
        extension: None,
        value_string: Some(parsed_request.expression.clone()),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add resource to parameters part
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: Some(parsed_request.resource.clone()),
        part: None,
    });

    // Add context if present
    if let Some(ref context_value) = parsed_request.context {
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "context".to_string(),
            extension: None,
            value_string: Some(context_value.clone()),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    // Add parseDebugTree (AST as JSON string) to parameters part - must be valueString according to API spec
    let parse_debug_tree = if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            let model_provider = registry.get_model_provider(version);
            let mut fhirpath_lab_ast = convert_ast_to_lab_format(
                ast,
                Some(registry.get_function_registry().as_ref()),
                model_provider
                    .as_ref()
                    .map(|p| p.as_ref() as &dyn octofhir_fhirpath::ModelProvider),
            );

            // Enhance with proper FHIR type information using ModelProvider
            if let Some(provider) = model_provider {
                // Extract actual resource type from the parsed request
                let resource_type = extract_resource_type(&parsed_request.resource)
                    .unwrap_or("Patient".to_string());

                if let Ok(enhanced_ast) = add_type_information(
                    fhirpath_lab_ast.clone(),
                    ast,
                    provider.as_ref() as &dyn octofhir_fhirpath::ModelProvider,
                    Some(registry.get_function_registry().as_ref()),
                    Some(&resource_type),
                )
                .await
                {
                    fhirpath_lab_ast = enhanced_ast;
                }
            }

            serde_json::to_string(&fhirpath_lab_ast).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    } else {
        "{}".to_string()
    };

    parameters_parts.push(FhirPathLabResponseParameter {
        name: "parseDebugTree".to_string(),
        extension: None,
        value_string: Some(parse_debug_tree), // JSON string for FHIR compliance and UI compatibility
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None, // Don't use resource field - AST is not a FHIR resource
        part: None,
    });

    // Add parseDebug (simple text representation) to parameters part
    let parse_debug_text = if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            // Get the enhanced AST with type information
            let model_provider = registry.get_model_provider(version);
            let mut fhirpath_lab_ast = convert_ast_to_lab_format(
                ast,
                Some(registry.get_function_registry().as_ref()),
                model_provider
                    .as_ref()
                    .map(|p| p.as_ref() as &dyn octofhir_fhirpath::ModelProvider),
            );

            // Enhance with proper FHIR type information using ModelProvider
            let inferred_type = if let Some(provider) = model_provider {
                // Extract actual resource type from the parsed request
                let resource_type = extract_resource_type(&parsed_request.resource)
                    .unwrap_or("Patient".to_string());

                if let Ok(enhanced_ast) = add_type_information(
                    fhirpath_lab_ast.clone(),
                    ast,
                    provider.as_ref() as &dyn octofhir_fhirpath::ModelProvider,
                    Some(registry.get_function_registry().as_ref()),
                    Some(&resource_type),
                )
                .await
                {
                    fhirpath_lab_ast = enhanced_ast;
                    // Use the return type from enhanced AST, or fall back to a default
                    fhirpath_lab_ast
                        .return_type
                        .unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            format!("{} : {}", parsed_request.expression, inferred_type)
        } else {
            format!("{} : unknown", parsed_request.expression)
        }
    } else {
        format!("{} : error", parsed_request.expression)
    };

    parameters_parts.push(FhirPathLabResponseParameter {
        name: "parseDebug".to_string(),
        extension: None,
        value_string: Some(parse_debug_text),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add parseDebugTree (AST as JSON string) - must be valueString according to API spec
    let parse_debug_tree = if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            let model_provider = registry.get_model_provider(version);
            let mut fhirpath_lab_ast = convert_ast_to_lab_format(
                ast,
                Some(registry.get_function_registry().as_ref()),
                model_provider
                    .as_ref()
                    .map(|p| p.as_ref() as &dyn octofhir_fhirpath::ModelProvider),
            );

            // Enhance with proper FHIR type information using ModelProvider
            if let Some(provider) = model_provider {
                // Extract actual resource type from the parsed request
                let resource_type = extract_resource_type(&parsed_request.resource)
                    .unwrap_or("Patient".to_string());

                if let Ok(enhanced_ast) = add_type_information(
                    fhirpath_lab_ast.clone(),
                    ast,
                    provider.as_ref() as &dyn octofhir_fhirpath::ModelProvider,
                    Some(registry.get_function_registry().as_ref()),
                    Some(&resource_type),
                )
                .await
                {
                    fhirpath_lab_ast = enhanced_ast;
                }
            }

            serde_json::to_string_pretty(&fhirpath_lab_ast).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    } else {
        "{}".to_string()
    };

    response.parameter.push(FhirPathLabResponseParameter {
        name: "parseDebugTree".to_string(),
        extension: None,
        value_string: Some(parse_debug_tree), // JSON string for FHIR compliance and UI compatibility
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None, // Don't use resource field - AST is not a FHIR resource
        part: None,
    });

    // Add expression directly to response
    response.parameter.push(FhirPathLabResponseParameter {
        name: "expression".to_string(),
        extension: None,
        value_string: Some(parsed_request.expression.clone()),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add resource directly to response
    response.parameter.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: Some(parsed_request.resource.clone()),
        part: None,
    });

    // Parameters are now added directly to response - no nested structure needed

    // Check for parsing errors and return OperationOutcome if any
    let has_parse_errors = parse_result.diagnostics.iter().any(|diag| {
        matches!(
            diag.severity,
            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
        )
    });

    if has_parse_errors {
        // Add parsing error diagnostics to the response instead of returning early
        let error_messages: Vec<String> = parse_result
            .diagnostics
            .iter()
            .filter(|diag| {
                matches!(
                    diag.severity,
                    octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                )
            })
            .map(|diag| {
                if let Some(loc) = &diag.location {
                    format!(
                        "Invalid expression: {} at line {}:{}",
                        diag.message, loc.line, loc.column
                    )
                } else {
                    format!("Invalid expression: {}", diag.message)
                }
            })
            .collect();

        // Add each error as a diagnostic parameter
        for error_msg in &error_messages {
            response.add_string_parameter("diagnostic", error_msg.clone());
        }
    }
    // Add basic validation result if requested
    if parsed_request.validate && !parse_result.has_errors() {
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
    }

    // Always add a result parameter, even for parsing failures
    let mut eval_time = std::time::Duration::from_millis(0);
    if parse_result.success {
        if let Some(_ast) = parse_result.ast {
            let mut engine = engine_arc.lock_owned().await;
            let eval_start = Instant::now();

            // Handle optional start context evaluation
            let context_results = if let Some(ref context_expr) = parsed_request.context {
                // Parse and evaluate the context expression against the resource
                let context_parse_result = parse_with_mode(context_expr, ParsingMode::Analysis);

                if !context_parse_result.success {
                    let error_messages: Vec<String> = context_parse_result
                        .diagnostics
                        .iter()
                        .filter(|diag| {
                            matches!(
                                diag.severity,
                                octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
                            )
                        })
                        .map(|diag| {
                            if let Some(loc) = &diag.location {
                                format!(
                                    "Invalid context expression: {} at line {}:{}",
                                    diag.message, loc.line, loc.column
                                )
                            } else {
                                format!("Invalid context expression: {}", diag.message)
                            }
                        })
                        .collect();

                    let error_msg = error_messages.join("; ");
                    let operation_outcome = OperationOutcome::error(
                        "invalid",
                        &error_msg,
                        Some(format!(
                            "Context expression parsing failed: {}",
                            context_expr
                        )),
                    );
                    return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
                }

                // Create initial context for context expression evaluation
                let resource_value =
                    octofhir_fhirpath::FhirPathValue::resource(parsed_request.resource.clone());
                let initial_context_collection = Collection::single(resource_value);
                let mut context_eval_context = EvaluationContext::new(initial_context_collection);

                // Set variables for context evaluation
                for (name, value) in &parsed_request.variables {
                    context_eval_context
                        .set_variable(name.clone(), json_to_fhirpath_value(value.clone()));
                }

                // Evaluate context expression to get starting points
                match engine
                    .evaluate_with_metadata(context_expr, &context_eval_context)
                    .await
                {
                    Ok(context_collection_with_metadata) => {
                        let context_collection = context_collection_with_metadata.to_collection();
                        if context_collection.is_empty() {
                            // Context expression returned empty - no results to evaluate main expression against
                            vec![]
                        } else {
                            // Convert each context result into a separate evaluation context
                            context_collection.iter().cloned().collect()
                        }
                    }
                    Err(e) => {
                        eval_time = eval_start.elapsed();
                        let operation_outcome = OperationOutcome::error(
                            "processing",
                            &format!("Context expression evaluation error: {}", e),
                            Some(format!("Failed to evaluate context: {}", context_expr)),
                        );
                        return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
                    }
                }
            } else {
                // No context expression - evaluate against the full resource
                let resource_value =
                    octofhir_fhirpath::FhirPathValue::resource(parsed_request.resource.clone());
                vec![resource_value]
            };

            // Step 2: Evaluate main expression against each context result
            let mut all_results = Vec::new();

            for context_value in context_results {
                let context_collection = Collection::single(context_value);
                let mut eval_context = EvaluationContext::new(context_collection);

                // Set variables for main expression evaluation
                for (name, value) in &parsed_request.variables {
                    eval_context.set_variable(name.clone(), json_to_fhirpath_value(value.clone()));
                }

                // Evaluate main expression against this context
                match engine
                    .evaluate_with_metadata(&parsed_request.expression, &eval_context)
                    .await
                {
                    Ok(collection_with_metadata) => {
                        all_results.extend(collection_with_metadata.results().iter().cloned());
                    }
                    Err(e) => {
                        eval_time = eval_start.elapsed();
                        let operation_outcome = OperationOutcome::error(
                            "processing",
                            &format!("Expression evaluation error: {}", e),
                            Some(format!("Failed to evaluate: {}", parsed_request.expression)),
                        );
                        return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
                    }
                }
            }

            eval_time = eval_start.elapsed();

            // Create a combined CollectionWithMetadata from all results
            let combined_collection_with_metadata =
                CollectionWithMetadata::from_results(all_results);

            // Format results
            format_fhirpath_results(
                &mut response,
                combined_collection_with_metadata,
                &parsed_request.expression,
            );
        }
    } else {
        response.parameter.push(FhirPathLabResponseParameter {
            name: "result".to_string(),
            extension: None,
            value_string: Some("evaluation".to_string()), // Keep same format as successful evaluations
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    // Structure response: separate top-level parameters
    // 1. First parameter: "parameters" with metadata
    let mut metadata_params = Vec::new();

    // Add evaluator info
    metadata_params.push(FhirPathLabResponseParameter {
        name: "evaluator".to_string(),
        extension: None,
        value_string: Some(format!(
            "octofhir-fhirpath-{} (R4)",
            env!("CARGO_PKG_VERSION")
        )),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add collected parameters from parameters_parts to metadata
    for param in parameters_parts {
        metadata_params.push(param);
    }

    // Add timing parameters to metadata
    let total_time = total_start.elapsed();
    let timing_params = create_timing_parameters(parse_time, eval_time, total_time);
    for timing_param in timing_params {
        metadata_params.push(timing_param);
    }

    // Add the "parameters" parameter with all metadata
    response.parameter.push(FhirPathLabResponseParameter {
        name: "parameters".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: Some(metadata_params),
    });

    // 2. Add result parameters as separate top-level parameters (not nested in "parameters")
    // The result parameters are already in response.parameter, so they will be added separately

    Ok(Json(serde_json::to_value(response).unwrap()))
}

/// Format FHIRPath evaluation results as a result parameter with parts
fn format_fhirpath_results(
    response: &mut FhirPathLabResponse,
    collection_with_metadata: CollectionWithMetadata,
    expression: &str,
) {
    // Access the individual results with their metadata
    let results = collection_with_metadata.results();

    // Create result parts for each evaluation result
    let mut result_parts = Vec::new();
    for (index, result) in results.iter().enumerate() {
        let result_param = create_single_result_parameter_with_metadata(result, index, expression);
        result_parts.push(result_param);
    }

    // Add the main "result" parameter with all results as parts
    let parts = if result_parts.is_empty() {
        None
    } else {
        Some(result_parts)
    };

    response.parameter.push(FhirPathLabResponseParameter {
        name: "result".to_string(),
        extension: None,
        value_string: Some("evaluation".to_string()), // Context string
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: parts,
    });

    // Add debug trace information
    add_debug_trace_info(response, expression, results.len());
}

// Removed hardcoded type correction - will investigate ModelProvider issue instead

/// Create a single result parameter with metadata from ResultWithMetadata
fn create_single_result_parameter_with_metadata(
    result: &octofhir_fhirpath::core::ResultWithMetadata,
    index: usize,
    _expression: &str,
) -> FhirPathLabResponseParameter {
    let item = result.to_json_parts();
    let fhir_type = result
        .type_info
        .expected_return_type
        .as_ref()
        .unwrap_or(&result.type_info.type_name);

    // Extract path from metadata if available
    let resource_path = if let Some(metadata) = &result.metadata {
        metadata
            .get("path")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    // Determine the FHIR type name for parameter naming
    let param_name = determine_fhir_type_name_from_string(&item, fhir_type, index);

    // Create extension with resource path information
    let extension = if !resource_path.is_empty() {
        Some(vec![serde_json::json!({
            "url": "http://fhir.forms-lab.com/StructureDefinition/resource-path",
            "valueString": resource_path
        })])
    } else {
        None
    };

    // Create the parameter with appropriate value field based on FHIR type
    let mut param = FhirPathLabResponseParameter {
        name: param_name.clone(),
        extension,
        value_string: None,
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    };

    // Set the appropriate value field based on detected type
    match fhir_type.as_str() {
        // Complex FHIR types
        "HumanName" => param.value_human_name = Some(item),
        "Identifier" => param.value_identifier = Some(item),
        "Address" => param.value_address = Some(item),
        "ContactPoint" => param.value_contact_point = Some(item),

        // String-like types with specific mappings
        "uri" | "url" | "canonical" => {
            param.value_uri = Some(item.as_str().unwrap_or("").to_string());
        }
        "code" => {
            param.value_code = Some(item.as_str().unwrap_or("").to_string());
        }
        "string" | "id" | "oid" | "uuid" | "markdown" | "xhtml" => {
            param.value_string = Some(item.as_str().unwrap_or("").to_string());
        }

        // Boolean type
        "boolean" => {
            param.value_boolean = Some(item.as_bool().unwrap_or(false));
        }

        // Numeric types with specific mappings
        "integer" | "positiveInt" | "unsignedInt" => {
            param.value_integer = Some(item.as_i64().unwrap_or(0) as i32);
        }
        "decimal" => {
            param.value_decimal = Some(item.as_f64().unwrap_or(0.0));
        }

        // Date/time types
        "dateTime" | "instant" => {
            param.value_date_time = Some(item.as_str().unwrap_or("").to_string());
        }
        "date" => {
            param.value_date = Some(item.as_str().unwrap_or("").to_string());
        }
        "time" => {
            // For now, use string representation for time without dedicated field
            param.value_string = Some(item.as_str().unwrap_or("").to_string());
        }

        // Default handling for unknown types - try to detect from JSON value
        _ => {
            // First, try to detect the actual type from the JSON value
            match &item {
                serde_json::Value::String(s) => {
                    param.value_string = Some(s.clone());
                }
                serde_json::Value::Bool(b) => {
                    param.value_boolean = Some(*b);
                }
                serde_json::Value::Number(n) => {
                    if n.is_i64() {
                        param.value_integer = Some(n.as_i64().unwrap_or(0) as i32);
                    } else if n.is_f64() {
                        param.value_decimal = Some(n.as_f64().unwrap_or(0.0));
                    } else {
                        param.value_string = Some(n.to_string());
                    }
                }
                serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
                    // Complex objects go in resource field
                    param.resource = Some(item);
                }
                serde_json::Value::Null => {
                    // For null values, don't set any field (empty parameter)
                }
            }
        }
    }

    param
}

/// Determine the appropriate parameter name based on FHIR type (from string)
fn determine_fhir_type_name_from_string(
    _item: &JsonValue,
    _fhir_type: &str,
    index: usize,
) -> String {
    format!("item{}", index)
}

/// Create a resource path string for trace information
fn create_resource_path(expression: &str, index: usize) -> String {
    // For arrays, add index notation
    if index > 0 {
        format!("{}[{}]", expression, index)
    } else if expression.contains('.') {
        // For property access, this is the base path
        expression.to_string()
    } else {
        String::new()
    }
}

/// Add debug trace information
fn add_debug_trace_info(response: &mut FhirPathLabResponse, expression: &str, result_count: usize) {
    let trace_info = format!(
        "Expression '{}' evaluated successfully, returned {} item(s)",
        expression, result_count
    );

    response.add_string_parameter("trace", trace_info);
}

/// Count items in a JSON value (array length or 1 for non-arrays)
fn items_count(value: &JsonValue) -> usize {
    match value {
        JsonValue::Array(items) => items.len(),
        _ => 1,
    }
}

/// Alternative FHIRPath Lab API using per-request engine creation
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

    // Create the main "parameters" section with metadata
    let mut parameters_parts = Vec::new();

    // Add evaluator information
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "evaluator".to_string(),
        extension: None,
        value_string: Some(format!(
            "octofhir-fhirpath-{} (R4)",
            env!("CARGO_PKG_VERSION")
        )),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add expression
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "expression".to_string(),
        extension: None,
        value_string: Some(parsed_request.expression.clone()),
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add resource
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: None,
        value_boolean: None,
        value_integer: None,
        value_uri: None,
        value_date_time: None,
        value_date: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: Some(parsed_request.resource.clone()),
        part: None,
    });

    if let Some(terminology_server) = &parsed_request.terminology_server {
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "terminologyServerUrl".to_string(),
            extension: None,
            value_string: Some(terminology_server.clone()),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    if !parsed_request.variables.is_empty() {
        // Add variables parameter (empty for now to match structure)
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "variables".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    // Add the top-level "parameters" parameter with all the metadata parts
    response.add_complex_parameter("parameters", parameters_parts);

    // Perform validation if requested
    if parsed_request.validate {
        // TODO: Add proper validation when analyzer is integrated
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
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
                extension: None,
                value_string: Some(format!(
                    "Evaluated expression: {} (engine creation: {:?}, evaluation: {:?})",
                    parsed_request.expression, engine_creation_time, evaluation_time
                )),
                value_code: None,
                value_decimal: None,
                value_boolean: None,
                value_integer: None,
                value_uri: None,
                value_date_time: None,
                value_date: None,
                value_human_name: None,
                value_identifier: None,
                value_address: None,
                value_contact_point: None,
                resource: None,
                part: None,
            });

            result_parts.push(FhirPathLabResponseParameter {
                name: "result".to_string(),
                extension: None,
                value_string: None,
                value_code: None,
                value_decimal: None,
                value_boolean: None,
                value_integer: None,
                value_uri: None,
                value_date_time: None,
                value_date: None,
                value_human_name: None,
                value_identifier: None,
                value_address: None,
                value_contact_point: None,
                resource: Some(result_json.to_json_parts()),
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
    engine: &mut octofhir_fhirpath::FhirPathEngine,
    request: &ParsedFhirPathLabRequest,
) -> Result<octofhir_fhirpath::core::CollectionWithMetadata, Box<dyn std::error::Error>> {
    // Create evaluation context with the resource
    let resource_value = octofhir_fhirpath::FhirPathValue::resource(request.resource.clone());
    let collection = octofhir_fhirpath::Collection::single(resource_value);
    let mut context = octofhir_fhirpath::EvaluationContext::new(collection);

    // Set variables
    for (name, value) in &request.variables {
        let fhir_value = json_to_fhirpath_value(value.clone());
        context.set_variable(name.clone(), fhir_value);
    }

    // Evaluate the expression
    let result = engine
        .evaluate_with_metadata(&request.expression, &context)
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(result)
}

/// Create timing parameters for the response
fn create_timing_parameters(
    parse_time: std::time::Duration,
    eval_time: std::time::Duration,
    total_time: std::time::Duration,
) -> Vec<FhirPathLabResponseParameter> {
    vec![
        FhirPathLabResponseParameter {
            name: "parseTime".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: Some(parse_time.as_secs_f64() * 1000.0),
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        },
        FhirPathLabResponseParameter {
            name: "evaluationTime".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: Some(eval_time.as_secs_f64() * 1000.0),
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        },
        FhirPathLabResponseParameter {
            name: "totalTime".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: Some(total_time.as_secs_f64() * 1000.0),
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_date_time: None,
            value_date: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        },
    ]
}
