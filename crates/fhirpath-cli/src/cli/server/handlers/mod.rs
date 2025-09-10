//! HTTP request handlers for the FHIRPath server

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    version::ServerFhirVersion,
};
use octofhir_fhirpath::core::CollectionWithMetadata;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode};
use octofhir_fhirpath::{Collection, FhirPathValue};
use serde_json::Value as JsonValue;
// use axum_macros::debug_handler;
// Analysis types - will be added when analyzer is properly integrated

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
            // Return OperationOutcome for errors
            let operation_outcome = OperationOutcome::error(
                "processing",
                &format!("Request processing failed: {}", error),
                None,
            );
            Json(operation_outcome).into_response()
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
            let operation_outcome = OperationOutcome::error(
                "processing",
                &format!("R4 request processing failed: {}", error),
                None,
            );
            Json(operation_outcome).into_response()
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
            let operation_outcome = OperationOutcome::error(
                "processing",
                &format!("R4B request processing failed: {}", error),
                None,
            );
            Json(operation_outcome).into_response()
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
            let operation_outcome = OperationOutcome::error(
                "processing",
                &format!("R5 request processing failed: {}", error),
                None,
            );
            Json(operation_outcome).into_response()
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
            let operation_outcome = OperationOutcome::error(
                "processing",
                &format!("R6 request processing failed: {}", error),
                None,
            );
            Json(operation_outcome).into_response()
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
            // Return OperationOutcome for parse errors
            let operation_outcome = OperationOutcome::error(
                "structure",
                &format!("Invalid request format: {}", e),
                None,
            );
            return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
        }
    };

    // Get the evaluation engine for the specified version
    let engine_arc = match registry.get_evaluation_engine(version) {
        Some(engine) => engine,
        None => {
            // Return OperationOutcome for unsupported version
            let operation_outcome = OperationOutcome::error(
                "not-supported",
                &format!("FHIR version {} not supported", version),
                None,
            );
            return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
        }
    };

    let mut response = FhirPathLabResponse::new();

    // Parse expression and collect diagnostics
    let parse_start = Instant::now();
    let parse_result = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    let parse_time = parse_start.elapsed();

    // Create the main "parameters" section with metadata
    let mut parameters_parts = Vec::new();

    // Add evaluator information
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "evaluator".to_string(),
        extension: None,
        value_string: Some(format!(
            "octofhir-fhirpath-{} ({:?})",
            env!("CARGO_PKG_VERSION"),
            version
        )),
        value_code: None,
        value_decimal: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add parseDebugTree to parameters section using FHIRPath Lab format
    let ast_json = if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            let model_provider = registry.get_model_provider(version);
            let mut fhirpath_lab_ast = convert_rust_ast_to_fhirpath_lab_format_with_registry(
                ast, 
                Some(registry.get_function_registry().as_ref()),
                model_provider.as_ref().map(|p| p.as_ref())
            );
            
            // Enhance with proper FHIR type information using ModelProvider
            if let Some(provider) = model_provider {
                if let Ok(enhanced_ast) = enhance_ast_with_type_information(
                    fhirpath_lab_ast.clone(),
                    ast,
                    provider.as_ref(),
                    Some("Patient")  // Default context type
                ).await {
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
        value_string: Some(ast_json),
        value_code: None,
        value_decimal: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

    // Add parseDebug (simple text representation)
    let parse_debug_text = if parse_result.success {
        if let Some(ref ast) = parse_result.ast {
            format!("{} : string[]", parsed_request.expression)
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
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: Some(parsed_request.resource.clone()),
        part: None,
    });

    // Add the top-level "parameters" parameter with all the metadata parts
    response.add_complex_parameter("parameters", parameters_parts);

    // Check for parsing errors and return OperationOutcome if any
    let has_parse_errors = parse_result.diagnostics.iter().any(|diag| {
        matches!(
            diag.severity,
            octofhir_fhirpath::diagnostics::DiagnosticSeverity::Error
        )
    });

    if has_parse_errors {
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

        let error_msg = error_messages.join("; ");
        let operation_outcome = OperationOutcome::error(
            "invalid",
            &error_msg,
            Some(format!(
                "Expression parsing failed: {}",
                parsed_request.expression
            )),
        );
        return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
    }
    // Add basic validation result if requested
    if parsed_request.validate && !parse_result.has_errors() {
        response.add_string_parameter("validation", "Expression syntax is valid".to_string());
    }

    // If parsing failed, skip evaluation but still report timing
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
    }

    // Timing metrics
    let total_time = total_start.elapsed();
    let mut timing_parts = Vec::new();
    timing_parts.push(FhirPathLabResponseParameter {
        name: "total".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: Some(total_time.as_secs_f64() * 1000.0),
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });
    timing_parts.push(FhirPathLabResponseParameter {
        name: "parse".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: Some(parse_time.as_secs_f64() * 1000.0),
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });
    timing_parts.push(FhirPathLabResponseParameter {
        name: "evaluation".to_string(),
        extension: None,
        value_string: None,
        value_code: None,
        value_decimal: Some(eval_time.as_secs_f64() * 1000.0),
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });
    response.add_complex_parameter("timing", timing_parts);

    Ok(Json(serde_json::to_value(response).unwrap()))
}

/// Format FHIRPath evaluation results
fn format_fhirpath_results(
    response: &mut FhirPathLabResponse,
    collection_with_metadata: CollectionWithMetadata,
    expression: &str,
) {
    // Create the nested "result" parameter with individual results as parts
    let mut result_parts = Vec::new();

    // Access the individual results with their metadata
    let results = collection_with_metadata.results();

    for (index, result) in results.iter().enumerate() {
        let result_param = create_single_result_parameter_with_metadata(result, index, expression);
        result_parts.push(result_param);
    }

    // Add the top-level "result" parameter with all individual results as parts
    response.add_complex_parameter("result", result_parts);

    // Add debug trace information
    add_debug_trace_info(response, expression, results.len());
}

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
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    };

    // Set the appropriate value field based on detected type
    match fhir_type.as_str() {
        "HumanName" => param.value_human_name = Some(item),
        "Identifier" => param.value_identifier = Some(item),
        "Address" => param.value_address = Some(item),
        "ContactPoint" => param.value_contact_point = Some(item),
        "string" | "code" | "id" | "uri" | "url" | "oid" | "uuid" => {
            if fhir_type == "code" {
                param.value_code = Some(item.as_str().unwrap_or("").to_string());
            } else {
                param.value_string = Some(item.as_str().unwrap_or("").to_string());
            }
        }
        "decimal" | "integer" | "positiveInt" | "unsignedInt" => {
            param.value_decimal = Some(item.as_f64().unwrap_or(0.0));
        }
        _ => {
            // For complex objects or unknown types, use resource field
            param.resource = Some(item);
        }
    }

    param
}

/// Determine the appropriate parameter name based on FHIR type (from string)
fn determine_fhir_type_name_from_string(
    item: &JsonValue,
    fhir_type: &str,
    _index: usize,
) -> String {
    // Use the provided FHIR type directly
    if !fhir_type.is_empty() && fhir_type != "unknown" {
        return fhir_type.to_string();
    }

    // Fallback: infer type from JSON structure
    if let JsonValue::Object(obj) = item {
        // Check for common FHIR types based on properties
        if obj.contains_key("family") || obj.contains_key("given") {
            return "HumanName".to_string();
        }
        if obj.contains_key("system") && obj.contains_key("value") {
            if obj.contains_key("use") {
                return "ContactPoint".to_string();
            } else {
                return "Identifier".to_string();
            }
        }
        if obj.contains_key("line")
            || obj.contains_key("city")
            || obj.contains_key("state")
            || obj.contains_key("postalCode")
        {
            return "Address".to_string();
        }
        if obj.contains_key("resourceType") {
            if let Some(resource_type) = obj.get("resourceType").and_then(|rt| rt.as_str()) {
                return resource_type.to_string();
            }
        }
    } else if item.is_string() {
        return "string".to_string();
    } else if item.is_number() {
        return "decimal".to_string();
    } else if item.is_boolean() {
        return "boolean".to_string();
    }

    "result".to_string()
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
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    // Add parseDebugTree to parameters section using FHIRPath Lab format
    let parse_result_for_ast = parse_with_mode(&parsed_request.expression, ParsingMode::Analysis);
    let ast_json = if parse_result_for_ast.success {
        if let Some(ref ast) = parse_result_for_ast.ast {
            let model_provider = registry.get_model_provider(version);
            let mut fhirpath_lab_ast = convert_rust_ast_to_fhirpath_lab_format_with_registry(
                ast, 
                Some(registry.get_function_registry().as_ref()),
                model_provider.as_ref().map(|p| p.as_ref())
            );
            
            // Enhance with proper FHIR type information using ModelProvider
            if let Some(provider) = model_provider {
                if let Ok(enhanced_ast) = enhance_ast_with_type_information(
                    fhirpath_lab_ast.clone(),
                    ast,
                    provider.as_ref(),
                    Some("Patient")  // Default context type
                ).await {
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
        value_string: Some(ast_json),
        value_code: None,
        value_decimal: None,
        value_human_name: None,
        value_identifier: None,
        value_address: None,
        value_contact_point: None,
        resource: None,
        part: None,
    });

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
                value_human_name: None,
                value_identifier: None,
                value_address: None,
                value_contact_point: None,
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

/// FHIRPath Lab AST node format
#[derive(serde::Serialize, Clone)]
struct FhirPathLabAstNode {
    #[serde(rename = "ExpressionType")]
    expression_type: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Arguments", skip_serializing_if = "Option::is_none")]
    arguments: Option<Vec<FhirPathLabAstNode>>,
    #[serde(rename = "ReturnType", skip_serializing_if = "Option::is_none")]
    return_type: Option<String>,
    #[serde(rename = "Position", skip_serializing_if = "Option::is_none")]
    position: Option<usize>,
    #[serde(rename = "Length", skip_serializing_if = "Option::is_none")]
    length: Option<usize>,
}

/// Helper function to infer FHIR type for property access using ModelProvider
async fn infer_property_access_type_async(
    object_type: &str,
    property_name: &str,
    model_provider: &crate::EmbeddedModelProvider,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    use octofhir_fhir_model::reflection::TypeReflectionInfo;
    use octofhir_fhirpath::ModelProvider;
    
    // Get type reflection for the parent type
    match model_provider.get_type_reflection(object_type).await? {
        Some(TypeReflectionInfo::ClassInfo { elements, .. }) => {
            // Look for the property in the elements
            for element in &elements {
                if element.name == property_name {
                    let base_type = element.type_info.name().to_string();
                    
                    // Determine cardinality based on element information
                    // FHIR uses max cardinality to determine if it's an array
                    let is_array = element.max_cardinality
                        .as_ref()
                        .map(|max| *max != 1)
                        .unwrap_or(false);
                    
                    let type_with_cardinality = if is_array {
                        format!("{}[]", base_type)
                    } else {
                        base_type
                    };
                    
                    return Ok(Some(type_with_cardinality));
                }
            }
            
            // Property not found
            Ok(None)
        }
        _ => {
            // Type not found or not a class type
            Ok(None)
        }
    }
}

/// Synchronous wrapper that tries basic type inference for common cases
fn infer_property_access_type_sync(
    _object: &octofhir_fhirpath::ast::ExpressionNode,
    property_name: &str,
    _model_provider: &crate::EmbeddedModelProvider,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // For now, return None to indicate we need async resolution
    // This will be used as a fallback when async resolution isn't possible
    match property_name {
        // Only handle very basic cases synchronously
        "resourceType" => Ok(Some("code".to_string())),
        "id" => Ok(Some("id".to_string())),
        _ => Ok(None),
    }
}

/// Enhanced AST converter that uses ModelProvider for proper FHIR type inference
fn enhance_ast_with_type_information<'a>(
    mut ast_node: FhirPathLabAstNode,
    original_ast: &'a octofhir_fhirpath::ast::ExpressionNode,
    model_provider: &'a crate::EmbeddedModelProvider,
    base_type: Option<&'a str>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<FhirPathLabAstNode, Box<dyn std::error::Error>>> + Send + 'a>> {
    Box::pin(async move {
    use octofhir_fhirpath::ast::*;
    
    match original_ast {
        ExpressionNode::PropertyAccess(node) => {
            // First enhance the object recursively
            if let Some(ref mut args) = ast_node.arguments {
                if let Some(object_arg) = args.get_mut(0) {
                    let enhanced_object = enhance_ast_with_type_information(
                        object_arg.clone(),
                        &node.object,
                        model_provider,
                        base_type,
                    ).await?;
                    
                    // Get the object's return type to use as the base for property lookup
                    let object_type = enhanced_object.return_type.as_ref()
                        .map(|t| t.trim_end_matches("[]"))  // Remove array notation if present
                        .unwrap_or(base_type.unwrap_or("Patient"));  // Default to Patient if unknown
                    
                    // Now infer the property type using ModelProvider
                    let property_type = infer_property_access_type_async(
                        object_type,
                        &node.property,
                        model_provider,
                    ).await?;
                    
                    // Update the object and set return type
                    args[0] = enhanced_object;
                    ast_node.return_type = property_type;
                }
            }
        }
        ExpressionNode::FunctionCall(node) => {
            // Enhance function call arguments recursively
            if let Some(ref mut args) = ast_node.arguments {
                for (i, arg_ast) in node.arguments.iter().enumerate() {
                    if let Some(ast_arg) = args.get_mut(i) {
                        let enhanced_arg = enhance_ast_with_type_information(
                            ast_arg.clone(),
                            arg_ast,
                            model_provider,
                            base_type,
                        ).await?;
                        args[i] = enhanced_arg;
                    }
                }
            }
        }
        ExpressionNode::MethodCall(node) => {
            // Enhance method call object and arguments recursively
            if let Some(ref mut args) = ast_node.arguments {
                if let Some(object_arg) = args.get_mut(0) {
                    let enhanced_object = enhance_ast_with_type_information(
                        object_arg.clone(),
                        &node.object,
                        model_provider,
                        base_type,
                    ).await?;
                    args[0] = enhanced_object;
                }
                
                // Enhance method arguments
                for (i, method_arg) in node.arguments.iter().enumerate() {
                    if let Some(ast_arg) = args.get_mut(i + 1) {  // +1 because object is first
                        let enhanced_arg = enhance_ast_with_type_information(
                            ast_arg.clone(),
                            method_arg,
                            model_provider,
                            base_type,
                        ).await?;
                        args[i + 1] = enhanced_arg;
                    }
                }
            }
        }
        ExpressionNode::BinaryOperation(node) => {
            // Enhance binary operation operands
            if let Some(ref mut args) = ast_node.arguments {
                if args.len() >= 2 {
                    let enhanced_left = enhance_ast_with_type_information(
                        args[0].clone(),
                        &node.left,
                        model_provider,
                        base_type,
                    ).await?;
                    let enhanced_right = enhance_ast_with_type_information(
                        args[1].clone(),
                        &node.right,
                        model_provider,
                        base_type,
                    ).await?;
                    args[0] = enhanced_left;
                    args[1] = enhanced_right;
                }
            }
        }
        _ => {
            // For other node types, recursively enhance children if they exist
            if let Some(ref mut args) = ast_node.arguments {
                // This is a generic recursive enhancement - specific node types handled above
                // would need more specific logic for their child nodes
            }
        }
    }
    
    Ok(ast_node)
    })
}


/// Convert Rust AST to FHIRPath Lab format with enhanced type information
fn convert_rust_ast_to_fhirpath_lab_format_with_registry(
    ast: &octofhir_fhirpath::ast::ExpressionNode,
    function_registry: Option<&octofhir_fhirpath::FunctionRegistry>,
    model_provider: Option<&crate::EmbeddedModelProvider>,
) -> FhirPathLabAstNode {
    use octofhir_fhirpath::ast::*;

    match ast {
        ExpressionNode::Identifier(node) => {
            // For simple identifiers, use AxisExpression with "builtin.that"
            FhirPathLabAstNode {
                expression_type: "AxisExpression".to_string(),
                name: "builtin.that".to_string(),
                arguments: None,
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::PropertyAccess(node) => {
            let object_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.object, function_registry, model_provider);
            
            // Try to infer type using ModelProvider if available
            let return_type = if let Some(provider) = model_provider {
                infer_property_access_type_sync(&node.object, &node.property, provider).unwrap_or(None)
            } else {
                None
            };
            
            FhirPathLabAstNode {
                expression_type: "ChildExpression".to_string(),
                name: node.property.clone(),
                arguments: Some(vec![object_arg]),
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::FunctionCall(node) => {
            let mut args = vec![];
            for arg in &node.arguments {
                args.push(convert_rust_ast_to_fhirpath_lab_format_with_registry(arg, function_registry, model_provider));
            }
            
            // Get return type from function registry if available
            let return_type = if let Some(registry) = function_registry {
                // Query the function registry for the return type
                if let Some(function_info) = registry.get_function_metadata(&node.name) {
                    // Convert the function's return type to FHIRPath Lab format
                    function_info.return_type.clone()
                } else {
                    None
                }
            } else {
                None
            };
            
            FhirPathLabAstNode {
                expression_type: "FunctionCallExpression".to_string(),
                name: node.name.clone(),
                arguments: if args.is_empty() { None } else { Some(args) },
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Literal(node) => {
            use octofhir_fhirpath::ast::LiteralValue;
            let (name, return_type) = match &node.value {
                LiteralValue::String(s) => (s.clone(), Some("string".to_string())),
                LiteralValue::Integer(i) => (i.to_string(), Some("integer".to_string())),
                LiteralValue::Decimal(d) => (d.to_string(), Some("decimal".to_string())),
                LiteralValue::Boolean(b) => (b.to_string(), Some("boolean".to_string())),
                LiteralValue::Date(d) => (d.to_string(), Some("date".to_string())),
                LiteralValue::DateTime(dt) => (dt.to_string(), Some("dateTime".to_string())),
                LiteralValue::Time(t) => (t.to_string(), Some("time".to_string())),
                LiteralValue::Quantity { value, unit } => {
                    let unit_str = unit.as_ref().map(|u| format!(" {}", u)).unwrap_or_default();
                    (
                        format!("{}{}", value, unit_str),
                        Some("Quantity".to_string()),
                    )
                }
            };

            FhirPathLabAstNode {
                expression_type: "ConstantExpression".to_string(),
                name,
                arguments: None,
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::BinaryOperation(node) => {
            let left_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.left, function_registry, model_provider);
            let right_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.right, function_registry, model_provider);

            let operator_name = match node.operator {
                BinaryOperator::Equal => "=",
                BinaryOperator::NotEqual => "!=",
                BinaryOperator::LessThan => "<",
                BinaryOperator::LessThanOrEqual => "<=",
                BinaryOperator::GreaterThan => ">",
                BinaryOperator::GreaterThanOrEqual => ">=",
                BinaryOperator::Add => "+",
                BinaryOperator::Subtract => "-",
                BinaryOperator::Multiply => "*",
                BinaryOperator::Divide => "/",
                BinaryOperator::And => "and",
                BinaryOperator::Or => "or",
                BinaryOperator::Union => "|",
                BinaryOperator::In => "in",
                BinaryOperator::Contains => "contains",
                _ => "unknown",
            };

            FhirPathLabAstNode {
                expression_type: "BinaryExpression".to_string(),
                name: operator_name.to_string(),
                arguments: Some(vec![left_arg, right_arg]),
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Variable(node) => FhirPathLabAstNode {
            expression_type: "VariableRefExpression".to_string(),
            name: format!("${}", node.name),
            arguments: None,
            return_type: None,
            position: node.location.as_ref().map(|l| l.offset),
            length: node.location.as_ref().map(|l| l.length),
        },

        ExpressionNode::MethodCall(node) => {
            // Convert object argument first
            let object_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.object, function_registry, model_provider);
            
            // Convert method arguments
            let mut args = vec![object_arg];
            for arg in &node.arguments {
                args.push(convert_rust_ast_to_fhirpath_lab_format_with_registry(arg, function_registry, model_provider));
            }
            
            // Get return type from function registry for method calls
            let return_type = if let Some(registry) = function_registry {
                if let Some(function_info) = registry.get_function_metadata(&node.method) {
                    function_info.return_type.clone()
                } else {
                    None
                }
            } else {
                None
            };
            
            FhirPathLabAstNode {
                expression_type: "FunctionCallExpression".to_string(),
                name: node.method.clone(),
                arguments: Some(args),
                return_type,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::IndexAccess(node) => {
            let object_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.object, function_registry, model_provider);
            let index_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.index, function_registry, model_provider);
            
            FhirPathLabAstNode {
                expression_type: "IndexerExpression".to_string(),
                name: "[]".to_string(),
                arguments: Some(vec![object_arg, index_arg]),
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::UnaryOperation(node) => {
            let operand_arg = convert_rust_ast_to_fhirpath_lab_format_with_registry(&node.operand, function_registry, model_provider);
            
            let operator_name = match node.operator {
                UnaryOperator::Not => "not",
                UnaryOperator::Negate => "-",
                UnaryOperator::Positive => "+",
            };
            
            FhirPathLabAstNode {
                expression_type: "UnaryExpression".to_string(),
                name: operator_name.to_string(),
                arguments: Some(vec![operand_arg]),
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Collection(node) => {
            let mut args = vec![];
            for item in &node.elements {
                args.push(convert_rust_ast_to_fhirpath_lab_format_with_registry(item, function_registry, model_provider));
            }
            
            FhirPathLabAstNode {
                expression_type: "CollectionExpression".to_string(),
                name: "{}".to_string(),
                arguments: if args.is_empty() { None } else { Some(args) },
                return_type: None,
                position: node.location.as_ref().map(|l| l.offset),
                length: node.location.as_ref().map(|l| l.length),
            }
        }

        ExpressionNode::Parenthesized(expr) => {
            // For parenthesized expressions, just convert the inner expression
            convert_rust_ast_to_fhirpath_lab_format_with_registry(expr, function_registry, model_provider)
        }

        // Add more conversions as needed for other node types
        _ => {
            // Fallback for unsupported node types - use a more descriptive approach
            let node_type = match ast {
                ExpressionNode::TypeCast(_) => "TypeCastExpression",
                ExpressionNode::Filter(_) => "FilterExpression",
                ExpressionNode::Union(_) => "UnionExpression", 
                ExpressionNode::TypeCheck(_) => "TypeCheckExpression",
                ExpressionNode::Path(_) => "PathExpression",
                ExpressionNode::Lambda(_) => "LambdaExpression",
                _ => "UnsupportedExpression",
            };
            
            FhirPathLabAstNode {
                expression_type: node_type.to_string(),
                name: "unsupported".to_string(),
                arguments: None,
                return_type: None,
                position: None,
                length: None,
            }
        }
    }
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
