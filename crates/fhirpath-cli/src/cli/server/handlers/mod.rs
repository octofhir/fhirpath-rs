//! HTTP request handlers for the FHIRPath server

use crate::cli::server::{
    error::{ServerError, ServerResult},
    models::*,
    registry::ServerRegistry,
    response::create_error_response,
    trace::ServerApiTraceProvider,
    version::ServerFhirVersion,
};
use octofhir_fhir_model::{HttpTerminologyProvider, TerminologyProvider, TypeInfo};
use octofhir_fhirpath::diagnostics::DiagnosticSeverity;
use octofhir_fhirpath::parser::{ParsingMode, parse_with_mode, parse_with_semantic_analysis};
use octofhir_fhirpath::{Collection, FhirPathValue};
use serde_json::Value as JsonValue;
use octofhir_fhirpath::evaluator::EvaluationContext;

use crate::cli::ast::{add_type_information, convert_ast_to_lab_format, extract_resource_type};
use axum::{
    extract::State,
    response::{IntoResponse, Json},
};

use octofhir_fhirpath::core::trace::SharedTraceProvider;
use serde::Deserialize;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// Query parameters for evaluation endpoints
#[derive(Debug, Deserialize)]
pub struct EvaluateQuery {
    /// Optional file to load as resource
    #[allow(dead_code)]
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

/// Convert JSON value to FhirPathValue
fn json_to_fhirpath_value(json: serde_json::Value) -> FhirPathValue {
    let default_type_info = TypeInfo {
        type_name: "System.Any".to_string(),
        singleton: Some(true),
        is_empty: Some(false),
        namespace: Some("System".to_string()),
        name: Some("Any".to_string()),
    };

    match json {
        serde_json::Value::Bool(b) => FhirPathValue::Boolean(b, default_type_info.clone(), None),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FhirPathValue::Integer(i, default_type_info.clone(), None)
            } else if let Some(f) = n.as_f64() {
                use rust_decimal::Decimal;
                FhirPathValue::Decimal(
                    Decimal::from_f64_retain(f).unwrap_or_default(),
                    default_type_info.clone(),
                    None,
                )
            } else {
                FhirPathValue::String(n.to_string(), default_type_info.clone(), None)
            }
        }
        serde_json::Value::String(s) => FhirPathValue::String(s, default_type_info.clone(), None),
        serde_json::Value::Array(arr) => {
            let values: Vec<FhirPathValue> = arr.into_iter().map(json_to_fhirpath_value).collect();
            FhirPathValue::Collection(values.into())
        }
        serde_json::Value::Object(_) => {
            // For objects, convert to Resource type or string representation
            FhirPathValue::String(json.to_string(), default_type_info.clone(), None)
        }
        serde_json::Value::Null => FhirPathValue::Empty,
    }
}

fn resolve_terminology_provider(
    engine: &octofhir_fhirpath::FhirPathEngine,
    override_url: Option<&str>,
) -> Option<Arc<dyn TerminologyProvider>> {
    if let Some(url) = override_url {
        match HttpTerminologyProvider::new(url.to_string()) {
            Ok(provider) => Some(Arc::new(provider) as Arc<dyn TerminologyProvider>),
            Err(error) => {
                warn!(
                    "Failed to initialize terminology provider from {}: {}",
                    url, error
                );
                engine.get_terminology_provider()
            }
        }
    } else {
        engine.get_terminology_provider()
    }
}

/// Version endpoint - required by task specification
pub async fn version_handler() -> Result<Json<serde_json::Value>, ServerError> {
    info!("🔖 Version info requested");

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
    println!("{:#?}", parsed_request);
    info!("🧪 Expression: {}", parsed_request.expression);

    // Create a trace provider scoped to this request so traces never leak across calls
    let trace_provider = ServerApiTraceProvider::create_shared();

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


    let engine = engine_arc.lock_owned().await;
    let model_provider_arc = engine.get_model_provider();

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
        value_string: Some(parsed_request.expression.clone()),
        ..Default::default()
    });

    // Add resource to parameters part
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        resource: Some(parsed_request.resource.clone()),
        ..Default::default()
    });

    // Add context if present
    if let Some(ref context_value) = parsed_request.context {
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "context".to_string(),
            value_string: Some(context_value.clone()),
            ..Default::default()
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
        value_string: Some(parse_debug_tree), // JSON string for FHIR compliance and UI compatibility
        ..Default::default()
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
        value_string: Some(parse_debug_text),
        ..Default::default()
    });

    // Add parseDebugTree (AST as JSON string) - must be valueString, according to API spec
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
        value_string: Some(parse_debug_tree), // JSON string for FHIR compliance and UI compatibility
        ..Default::default()
    });

    // Add expression directly to response
    response.parameter.push(FhirPathLabResponseParameter {
        name: "expression".to_string(),
        value_string: Some(parsed_request.expression.clone()),
        ..Default::default()
    });

    // Add resource directly to response
    response.parameter.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        resource: Some(parsed_request.resource.clone()),
        ..Default::default()
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
    if parse_result.has_errors() {
        println!("{:#?}", parse_result);
        println!("{:#?}", &parsed_request.expression);
    }

    if parse_result.success {
        if let Some(_ast) = parse_result.ast {
            let context_type =
                if let Some(resource_type) = extract_resource_type(&parsed_request.resource) {
                    model_provider_arc
                        .get_type(&resource_type)
                        .await
                        .ok()
                        .flatten()
                } else {
                    None
                };

            let semantic_result = parse_with_semantic_analysis(
                    &parsed_request.expression,
                    model_provider_arc,
                    context_type,
            ).await;

            let semantic_errors: Vec<_> = semantic_result
                .analysis
                .diagnostics
                .iter()
                .filter(|diag| matches!(diag.severity, DiagnosticSeverity::Error))
                .collect();

            if !semantic_errors.is_empty() {
                let primary_message = semantic_errors[0].message.clone();
                let diagnostics_text = semantic_errors
                    .iter()
                    .map(|diag| diag.message.clone())
                    .collect::<Vec<_>>()
                    .join("; ");

                let operation_outcome =
                    OperationOutcome::error("invalid", &primary_message, Some(diagnostics_text));

                return Ok(Json(serde_json::to_value(operation_outcome).unwrap()));
            }

            let terminology_provider =
                resolve_terminology_provider(&engine, parsed_request.terminology_server.as_deref());
            let validation_provider = engine.get_validation_provider();
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
                let embedded_provider = crate::EmbeddedModelProvider::r4();
                let context_eval_context = EvaluationContext::new(
                    initial_context_collection,
                    std::sync::Arc::new(embedded_provider),
                    terminology_provider.clone(),
                    validation_provider.clone(),
                    Some(trace_provider.clone()),
                )
                .await;

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
                        let context_collection = context_collection_with_metadata.result.value;
                        if context_collection.is_empty() {
                            // Context expression returned empty - no results to evaluate main expression against
                            vec![]
                        } else {
                            // Convert each context result into a separate evaluation context
                            context_collection.iter().cloned().collect()
                        }
                    }
                    Err(e) => {
                        let _ = eval_start.elapsed();
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
                let resource_value = FhirPathValue::resource(parsed_request.resource.clone());
                vec![resource_value]
            };

            // Step 2: Evaluate main expression against each context result
            let mut all_results = Vec::new();

            for context_value in context_results {
                let context_collection = Collection::single(context_value);
                let embedded_provider = engine.get_model_provider();
                let eval_context = EvaluationContext::new(
                    context_collection,
                    embedded_provider,
                    terminology_provider.clone(),
                    validation_provider.clone(),
                    Some(trace_provider.clone()),
                )
                .await;

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
                        all_results.extend(collection_with_metadata.result.value.iter().cloned());
                    }
                    Err(e) => {
                        let _ = eval_start.elapsed();
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

            // Create a combined Collection from all results
            let combined_collection = Collection::from_values(all_results);

            // Format results
            format_fhirpath_results(
                &mut response,
                combined_collection,
                &parsed_request.expression,
            );
        }
    } else {
        response.parameter.push(FhirPathLabResponseParameter {
            name: "result".to_string(),
            value_string: Some("evaluation".to_string()),
            ..Default::default()
        });
    }

    // Structure response: separate top-level parameters
    // 1. First parameter: "parameters" with metadata
    let mut metadata_params = Vec::new();

    // Add evaluator info
    metadata_params.push(FhirPathLabResponseParameter {
        name: "evaluator".to_string(),
        value_string: Some(format!(
            "octofhir-fhirpath-{} (R4)",
            env!("CARGO_PKG_VERSION")
        )),
        ..Default::default()
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
        part: Some(metadata_params),
        ..Default::default()
    });

    // 2. Add result parameters as separate top-level parameters (not nested in "parameters")
    // The result parameters are already in response.parameter, so they will be added separately

    let collected_traces = trace_provider.collect_traces();
    if !collected_traces.is_empty() {
        response.add_trace_from_collection(collected_traces);
    }

    trace_provider.clear_traces();

    Ok(Json(serde_json::to_value(response).unwrap()))
}

/// Format FHIRPath evaluation results as a result parameter with parts
fn format_fhirpath_results(
    response: &mut FhirPathLabResponse,
    collection: Collection,
    expression: &str,
) {
    // Access the individual results
    let results: Vec<&FhirPathValue> = collection.iter().collect();

    // Create result parts for each evaluation result
    let mut result_parts = Vec::new();
    for (index, result) in results.iter().enumerate() {
        let result_param = create_single_result_parameter_simple(result, index, expression);
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
        value_string: Some("evaluation".to_string()),
        part: parts,
        ..Default::default()
    });

    // Add debug trace information
    add_debug_trace_info(response, expression, results.len());
}

// Removed hardcoded type correction - will investigate ModelProvider issue instead

fn sanitize_temporal_prefix(raw: &str) -> String {
    if let Some(rest) = raw.strip_prefix("@T") {
        rest.to_string()
    } else if let Some(rest) = raw.strip_prefix('@') {
        rest.to_string()
    } else if raw.starts_with('T') && raw.chars().nth(1).is_some_and(|c| c.is_ascii_digit()) {
        raw[1..].to_string()
    } else {
        raw.to_string()
    }
}

/// Resolve the most specific type name from TypeInfo metadata
fn resolve_type_name(type_info: &octofhir_fhir_model::TypeInfo) -> String {
    if let Some(name) = &type_info.name {
        if !name.is_empty() {
            return name.clone();
        }
    }

    let raw = &type_info.type_name;
    raw.split(['.', ':']).last().unwrap_or(raw).to_string()
}

/// Create a simple result parameter from FhirPathValue
fn create_single_result_parameter_simple(
    result: &FhirPathValue,
    index: usize,
    _expression: &str,
) -> FhirPathLabResponseParameter {
    let type_info = result.type_info();
    let type_name = resolve_type_name(type_info);
    let type_lower = type_name.to_ascii_lowercase();

    let mut param = FhirPathLabResponseParameter {
        name: if type_lower.is_empty() {
            format!("item{}", index)
        } else {
            type_lower.clone()
        },
        ..Default::default()
    };

    match result {
        FhirPathValue::Boolean(value, _, _) => {
            param.value_boolean = Some(*value);
        }
        FhirPathValue::Integer(value, _, _) => {
            if let Ok(integer) = i32::try_from(*value) {
                param.value_integer = Some(integer);
            } else {
                param.value_string = Some(value.to_string());
            }
        }
        FhirPathValue::Decimal(decimal, _, _) => {
            param.value_decimal = Some(DecimalRepresentation::from_decimal(decimal));
        }
        FhirPathValue::String(text, _, _) => match type_lower.as_str() {
            "code" => param.value_code = Some(text.clone()),
            "id" => param.value_id = Some(text.clone()),
            "oid" => param.value_oid = Some(text.clone()),
            "uuid" => param.value_uuid = Some(text.clone()),
            "markdown" => param.value_markdown = Some(text.clone()),
            "canonical" => param.value_canonical = Some(text.clone()),
            "uri" => param.value_uri = Some(text.clone()),
            "url" => param.value_url = Some(text.clone()),
            "date" => {
                param.value_date = Some(sanitize_temporal_prefix(text));
            }
            "datetime" | "instant" => {
                param.value_date_time = Some(sanitize_temporal_prefix(text));
            }
            "time" => {
                param.value_time = Some(sanitize_temporal_prefix(text));
            }
            _ => param.value_string = Some(text.clone()),
        },
        FhirPathValue::Date(date, _, _) => {
            param.value_date = Some(sanitize_temporal_prefix(&date.to_string()));
        }
        FhirPathValue::DateTime(date_time, _, _) => {
            param.value_date_time = Some(sanitize_temporal_prefix(&date_time.to_string()));
        }
        FhirPathValue::Time(time, _, _) => {
            param.value_time = Some(sanitize_temporal_prefix(&time.to_string()));
        }
        FhirPathValue::Quantity {
            value,
            unit,
            code,
            system,
            ucum_unit,
            ..
        } => {
            let mut quantity_map = serde_json::Map::new();
            quantity_map.insert("value".to_string(), decimal_to_json_value(value));

            if let Some(unit_str) = unit.clone() {
                quantity_map.insert("unit".to_string(), JsonValue::String(unit_str));
            }

            if let Some(code_str) = code.clone() {
                quantity_map.insert("code".to_string(), JsonValue::String(code_str));
            }

            if let Some(system_str) = system.clone() {
                quantity_map.insert("system".to_string(), JsonValue::String(system_str));
            }

            if let Some(ucum) = ucum_unit {
                if !quantity_map.contains_key("system") {
                    quantity_map.insert(
                        "system".to_string(),
                        JsonValue::String("http://unitsofmeasure.org".to_string()),
                    );
                }
                if !quantity_map.contains_key("code") {
                    quantity_map
                        .insert("code".to_string(), JsonValue::String(ucum.code.to_string()));
                }
                if !quantity_map.contains_key("unit") {
                    quantity_map.insert(
                        "unit".to_string(),
                        JsonValue::String(ucum.display_name.to_string()),
                    );
                }
            }

            param.value_quantity = Some(JsonValue::Object(quantity_map));
        }
        FhirPathValue::Resource(resource, _, _) => {
            let json = resource.as_ref().clone();
            match type_lower.as_str() {
                "humanname" => param.value_human_name = Some(json),
                "identifier" => param.value_identifier = Some(json),
                "address" => param.value_address = Some(json),
                "contactpoint" => param.value_contact_point = Some(json),
                "coding" => param.value_coding = Some(json),
                "codeableconcept" => param.value_codeable_concept = Some(json),
                "period" => param.value_period = Some(json),
                "reference" => param.value_reference = Some(json),
                _ => param.resource = Some(json),
            }
        }
        FhirPathValue::Collection(collection) => {
            param.resource = Some(collection.to_json_value());
        }
        FhirPathValue::Empty => param.resource = Some(JsonValue::Array(Vec::new())),
    }

    param
}

/// Create a single result parameter with metadata from ResultWithMetadata
#[allow(dead_code)]
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
        ..Default::default()
    };

    // Set the appropriate value field based on detected type
    match fhir_type.as_str() {
        "Quantity" | "SimpleQuantity" => {
            if let FhirPathValue::Quantity {
                value,
                unit,
                ucum_unit,
                ..
            } = &result.value
            {
                let mut quantity_map = serde_json::Map::new();
                quantity_map.insert("value".to_string(), decimal_to_json_value(value));

                if let Some(unit_str) = unit.clone() {
                    quantity_map.insert("unit".to_string(), JsonValue::String(unit_str));
                }

                if let Some(ucum) = ucum_unit {
                    quantity_map.insert(
                        "system".to_string(),
                        JsonValue::String("http://unitsofmeasure.org".to_string()),
                    );
                    quantity_map
                        .insert("code".to_string(), JsonValue::String(ucum.code.to_string()));

                    if !quantity_map.contains_key("unit") {
                        quantity_map.insert(
                            "unit".to_string(),
                            JsonValue::String(ucum.display_name.to_string()),
                        );
                    }
                }

                param.value_quantity = Some(JsonValue::Object(quantity_map));
                return param;
            }
        }
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
            param.value_decimal = Some(DecimalRepresentation::from_f64(
                item.as_f64().unwrap_or(0.0),
            ));
        }

        // Date/time types
        "dateTime" | "instant" => {
            let value = item
                .as_str()
                .map(sanitize_temporal_prefix)
                .unwrap_or_else(|| item.to_string());
            param.value_date_time = Some(value);
        }
        "date" => {
            let value = item
                .as_str()
                .map(sanitize_temporal_prefix)
                .unwrap_or_else(|| item.to_string());
            param.value_date = Some(value);
        }
        "time" => {
            let value = item
                .as_str()
                .map(sanitize_temporal_prefix)
                .unwrap_or_else(|| item.to_string());
            param.value_time = Some(value);
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
                        param.value_decimal =
                            Some(DecimalRepresentation::from_f64(n.as_f64().unwrap_or(0.0)));
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

/// Determine the appropriate parameter name based on FHIR type (from metadata)
#[allow(dead_code)]
fn determine_fhir_type_name_from_string(
    _item: &JsonValue,
    fhir_type: &str,
    _index: usize,
) -> String {
    // Use FHIR type name directly from metadata - don't inspect JSON properties for arrays
    match fhir_type {
        // Primitive types - map to standard FHIR parameter names
        "string" | "id" | "oid" | "uuid" | "markdown" | "xhtml" => "string".to_string(),
        "integer" | "positiveInt" | "unsignedInt" => "integer".to_string(),
        "decimal" => "decimal".to_string(),
        "boolean" => "boolean".to_string(),
        "uri" | "url" | "canonical" => "uri".to_string(),
        "code" => "code".to_string(),
        "dateTime" | "instant" => "dateTime".to_string(),
        "date" => "date".to_string(),
        "time" => "time".to_string(),

        // Complex types - use exact type name
        "HumanName" => "HumanName".to_string(),
        "Identifier" => "Identifier".to_string(),
        "Address" => "Address".to_string(),
        "ContactPoint" => "ContactPoint".to_string(),
        "Quantity" => "Quantity".to_string(),

        // Use the metadata type directly for any other types
        other => other.to_string(),
    }
}

/// Create a resource path string for trace information
#[allow(dead_code)]
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
#[allow(dead_code)]
fn items_count(value: &JsonValue) -> usize {
    match value {
        JsonValue::Array(items) => items.len(),
        _ => 1,
    }
}

/// Alternative FHIRPath Lab API using per-request engine creation
#[allow(dead_code)]
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

    info!("🧪 Expression: {}", parsed_request.expression);

    let trace_provider = ServerApiTraceProvider::create_shared();

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
        value_string: Some(format!(
            "octofhir-fhirpath-{} (R4)",
            env!("CARGO_PKG_VERSION")
        )),
        ..Default::default()
    });

    // Add expression
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "expression".to_string(),
        value_string: Some(parsed_request.expression.clone()),
        ..Default::default()
    });

    // Add resource
    parameters_parts.push(FhirPathLabResponseParameter {
        name: "resource".to_string(),
        resource: Some(parsed_request.resource.clone()),
        ..Default::default()
    });

    if let Some(terminology_server) = &parsed_request.terminology_server {
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "terminologyServerUrl".to_string(),
            value_string: Some(terminology_server.clone()),
            ..Default::default()
        });
    }

    if !parsed_request.variables.is_empty() {
        // Add variables parameter (empty for now to match structure)
        parameters_parts.push(FhirPathLabResponseParameter {
            name: "variables".to_string(),
            ..Default::default()
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
    let result =
        evaluate_fhirpath_expression(&mut engine, &parsed_request, Some(trace_provider.clone()))
            .await;
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
                ..Default::default()
            });

            result_parts.push(FhirPathLabResponseParameter {
                name: "result".to_string(),
                resource: Some(collection_to_json(&result_json.value)),
                ..Default::default()
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

    let collected_traces = trace_provider.collect_traces();
    if !collected_traces.is_empty() {
        response.add_trace_from_collection(collected_traces);
    }

    trace_provider.clear_traces();

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
    trace_provider: Option<SharedTraceProvider>,
) -> Result<octofhir_fhirpath::EvaluationResult, Box<dyn std::error::Error>> {
    // Create an evaluation context with the resource
    let resource_value = FhirPathValue::resource(request.resource.clone());
    let collection = Collection::single(resource_value);
    let model_provider = engine.get_model_provider();
    let terminology_provider =
        resolve_terminology_provider(engine, request.terminology_server.as_deref());
    let validation_provider = engine.get_validation_provider();
    let context = octofhir_fhirpath::EvaluationContext::new(
        collection,
        model_provider,
        terminology_provider,
        validation_provider,
        trace_provider.clone(),
    )
    .await;

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

    Ok(result.result)
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
            value_decimal: Some(DecimalRepresentation::from_f64(
                parse_time.as_secs_f64() * 1000.0,
            )),
            ..Default::default()
        },
        FhirPathLabResponseParameter {
            name: "evaluationTime".to_string(),
            value_decimal: Some(DecimalRepresentation::from_f64(
                eval_time.as_secs_f64() * 1000.0,
            )),
            ..Default::default()
        },
        FhirPathLabResponseParameter {
            name: "totalTime".to_string(),
            value_decimal: Some(DecimalRepresentation::from_f64(
                total_time.as_secs_f64() * 1000.0,
            )),
            ..Default::default()
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath::core::temporal::{PrecisionDate, PrecisionDateTime, PrecisionTime};

    #[test]
    fn time_values_emit_value_time_parameter() {
        let precision_time = PrecisionTime::parse("00:30:00").expect("valid time");
        let value = FhirPathValue::time(precision_time);

        let param = create_single_result_parameter_simple(&value, 0, "");

        assert_eq!(param.name, "time");
        assert_eq!(param.value_time.as_deref(), Some("00:30:00"));
        assert!(param.value_string.is_none());
    }

    #[test]
    fn boolean_values_emit_value_boolean_parameter() {
        let value = FhirPathValue::boolean(true);

        let param = create_single_result_parameter_simple(&value, 0, "");

        assert_eq!(param.name, "boolean");
        assert_eq!(param.value_boolean, Some(true));
        assert!(param.value_string.is_none());
    }

    #[test]
    fn format_results_include_boolean_item() {
        let mut response = FhirPathLabResponse::new();
        let collection = Collection::single(FhirPathValue::boolean(true));

        format_fhirpath_results(&mut response, collection, "true");

        let result_param = response
            .parameter
            .iter()
            .find(|p| p.name == "result")
            .expect("result parameter");
        let parts = result_param.part.as_ref().expect("result parts");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].name, "boolean");
        assert_eq!(parts[0].value_boolean, Some(true));
    }

    #[test]
    fn date_values_strip_prefix() {
        let precision_date = PrecisionDate::parse("2014-01-01").expect("valid date");
        let value = FhirPathValue::date(precision_date);

        let param = create_single_result_parameter_simple(&value, 0, "");

        assert_eq!(param.value_date.as_deref(), Some("2014-01-01"));
        assert!(param.value_string.is_none());
    }

    #[test]
    fn datetime_values_strip_prefix() {
        let precision_datetime =
            PrecisionDateTime::parse("2014-01-01T08:05:00-05:00").expect("valid datetime");
        let value = FhirPathValue::datetime(precision_datetime);

        let param = create_single_result_parameter_simple(&value, 0, "");

        assert_eq!(
            param.value_date_time.as_deref(),
            Some("2014-01-01T08:05:00-05:00")
        );
        assert!(param.value_string.is_none());
    }
}
