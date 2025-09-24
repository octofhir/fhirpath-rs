//! Request and response models for the FHIRPath HTTP server

use octofhir_fhirpath::FhirPathValue;
use serde::ser::{Serialize as SerSerialize, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

// ===== EVALUATION MODELS =====

/// Request model for FHIRPath expression evaluation
#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    /// FHIRPath expression to evaluate
    pub expression: String,
    /// FHIR resource to evaluate against (optional if using file parameter)
    pub resource: Option<JsonValue>,
    /// Variables to set in the evaluation context
    #[serde(default)]
    pub variables: HashMap<String, JsonValue>,
    /// Evaluation options
    #[serde(default)]
    pub options: EvaluationOptions,
}

/// Options for FHIRPath evaluation
#[derive(Debug, Deserialize)]
pub struct EvaluationOptions {
    /// Validate expression syntax before evaluation
    #[serde(default)]
    pub validate: bool,
    /// Enable trace output
    #[serde(default)]
    pub trace: bool,
}

impl Default for EvaluationOptions {
    fn default() -> Self {
        Self {
            validate: true,
            trace: false,
        }
    }
}

/// Response model for FHIRPath expression evaluation
#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    /// Whether the evaluation was successful
    pub success: bool,
    /// Evaluation result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JsonValue>,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// The expression that was evaluated
    pub expression: String,
    /// FHIR version used
    pub fhir_version: String,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
    /// Trace output (if requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<Vec<String>>,
}

// ===== ANALYSIS MODELS =====

/// Request model for FHIRPath expression analysis
#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    /// FHIRPath expression to analyze
    pub expression: String,
    /// Analysis options
    #[serde(default)]
    pub options: AnalysisOptions,
}

/// Options for FHIRPath analysis
#[derive(Debug, Deserialize)]
pub struct AnalysisOptions {
    /// Include optimization suggestions
    #[serde(default = "default_true")]
    pub include_optimizations: bool,
    /// Include diagnostic information
    #[serde(default = "default_true")]
    pub include_diagnostics: bool,
    /// Include validation results
    #[serde(default = "default_true")]
    pub include_validation: bool,
    /// Include parsing information
    #[serde(default = "default_true")]
    pub include_parsing: bool,
}

fn default_true() -> bool {
    true
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            include_optimizations: true,
            include_diagnostics: true,
            include_validation: true,
            include_parsing: true,
        }
    }
}

/// Response model for FHIRPath expression analysis
#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    /// Whether the analysis was successful
    pub success: bool,
    /// Analysis results (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisResult>,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
    /// The expression that was analyzed
    pub expression: String,
    /// FHIR version used
    pub fhir_version: String,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Analysis results for the API
#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    /// Expression type information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_info: Option<TypeInfo>,
    /// Validation errors
    pub validation_errors: Vec<ValidationErrorInfo>,
    /// Type annotations from analysis
    pub type_annotations: usize,
    /// Function calls detected
    pub function_calls: usize,
    /// Union types found
    pub union_types: usize,
}

// ===== FILE MANAGEMENT MODELS =====

/// Response model for file listing
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    /// List of available files
    pub files: Vec<FileInfo>,
    /// Storage directory path
    pub storage_path: String,
}

/// Information about a stored file
#[derive(Debug, Serialize)]
pub struct FileInfo {
    /// Filename
    pub name: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp (ISO 8601)
    pub modified: String,
    /// File type (detected from content)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_type: Option<String>,
}

/// Response model for file upload
#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    /// Whether the upload was successful
    pub success: bool,
    /// Uploaded filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// Error information (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorInfo>,
}

// ===== FHIRPATH LAB API MODELS =====

/// FHIRPath Lab API request in FHIR Parameters format
#[derive(Debug, Deserialize)]
pub struct FhirPathLabRequest {
    /// FHIR resource type (should be "Parameters")
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    /// Parameters array
    pub parameter: Vec<FhirPathLabParameter>,
}

/// Individual parameter in FHIR Parameters resource
#[derive(Debug, Deserialize)]
pub struct FhirPathLabParameter {
    /// Parameter name
    pub name: String,
    /// String value (for expression, context, etc.)
    #[serde(rename = "valueString")]
    pub value_string: Option<String>,
    /// Boolean value (for validate flag)
    #[serde(rename = "valueBoolean")]
    pub value_boolean: Option<bool>,
    /// Resource value (for the resource to evaluate against)
    #[serde(rename = "resource")]
    pub resource: Option<JsonValue>,
    /// Nested parameters (for variables)
    pub part: Option<Vec<FhirPathLabParameter>>,
}

/// FHIRPath Lab API response in FHIR Parameters format
#[derive(Debug, Serialize)]
pub struct FhirPathLabResponse {
    /// Resource id (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// FHIR resource type (always "Parameters")
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    /// Parameters array containing results
    pub parameter: Vec<FhirPathLabResponseParameter>,
}

/// Response parameter in FHIR Parameters resource
#[derive(Debug, Serialize, Default)]
pub struct FhirPathLabResponseParameter {
    /// Parameter name
    pub name: String,
    /// Extensions (for resource paths, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension: Option<Vec<JsonValue>>,
    /// String value (for simple results)
    #[serde(rename = "valueString", skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
    /// Code value (for issue severity/code fields)
    #[serde(rename = "valueCode", skip_serializing_if = "Option::is_none")]
    pub value_code: Option<String>,
    /// Decimal value (for timing metrics)
    #[serde(rename = "valueDecimal", skip_serializing_if = "Option::is_none")]
    pub value_decimal: Option<DecimalRepresentation>,
    /// Boolean value (for boolean results)
    #[serde(rename = "valueBoolean", skip_serializing_if = "Option::is_none")]
    pub value_boolean: Option<bool>,
    /// Integer value (for integer results)
    #[serde(rename = "valueInteger", skip_serializing_if = "Option::is_none")]
    pub value_integer: Option<i32>,
    /// URI value (for uri type results)
    #[serde(rename = "valueUri", skip_serializing_if = "Option::is_none")]
    pub value_uri: Option<String>,
    /// URL value (for url type results)
    #[serde(rename = "valueUrl", skip_serializing_if = "Option::is_none")]
    pub value_url: Option<String>,
    /// Canonical value (for canonical type results)
    #[serde(rename = "valueCanonical", skip_serializing_if = "Option::is_none")]
    pub value_canonical: Option<String>,
    /// Id value (for id type results)
    #[serde(rename = "valueId", skip_serializing_if = "Option::is_none")]
    pub value_id: Option<String>,
    /// OID value (for oid type results)
    #[serde(rename = "valueOid", skip_serializing_if = "Option::is_none")]
    pub value_oid: Option<String>,
    /// UUID value (for uuid type results)
    #[serde(rename = "valueUuid", skip_serializing_if = "Option::is_none")]
    pub value_uuid: Option<String>,
    /// Markdown value (for markdown type results)
    #[serde(rename = "valueMarkdown", skip_serializing_if = "Option::is_none")]
    pub value_markdown: Option<String>,
    /// Time value (for time results)
    #[serde(rename = "valueTime", skip_serializing_if = "Option::is_none")]
    pub value_time: Option<String>,
    /// DateTime value (for dateTime results)
    #[serde(rename = "valueDateTime", skip_serializing_if = "Option::is_none")]
    pub value_date_time: Option<String>,
    /// Date value (for date results)
    #[serde(rename = "valueDate", skip_serializing_if = "Option::is_none")]
    pub value_date: Option<String>,
    /// Quantity value
    #[serde(rename = "valueQuantity", skip_serializing_if = "Option::is_none")]
    pub value_quantity: Option<JsonValue>,
    /// Coding value
    #[serde(rename = "valueCoding", skip_serializing_if = "Option::is_none")]
    pub value_coding: Option<JsonValue>,
    /// CodeableConcept value
    #[serde(
        rename = "valueCodeableConcept",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_codeable_concept: Option<JsonValue>,
    /// Period value
    #[serde(rename = "valuePeriod", skip_serializing_if = "Option::is_none")]
    pub value_period: Option<JsonValue>,
    /// Reference value
    #[serde(rename = "valueReference", skip_serializing_if = "Option::is_none")]
    pub value_reference: Option<JsonValue>,
    /// HumanName value (for HumanName results)
    #[serde(rename = "valueHumanName", skip_serializing_if = "Option::is_none")]
    pub value_human_name: Option<JsonValue>,
    /// Identifier value (for Identifier results)
    #[serde(rename = "valueIdentifier", skip_serializing_if = "Option::is_none")]
    pub value_identifier: Option<JsonValue>,
    /// Address value (for Address results)
    #[serde(rename = "valueAddress", skip_serializing_if = "Option::is_none")]
    pub value_address: Option<JsonValue>,
    /// ContactPoint value (for telecom results)
    #[serde(rename = "valueContactPoint", skip_serializing_if = "Option::is_none")]
    pub value_contact_point: Option<JsonValue>,
    /// Resource value (for complex results)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<JsonValue>,
    /// Nested parameters (for structured data)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part: Option<Vec<FhirPathLabResponseParameter>>,
}

/// Parsed FHIRPath Lab request for easier handling
#[derive(Debug)]
pub struct ParsedFhirPathLabRequest {
    /// FHIRPath expression to evaluate
    pub expression: String,
    /// Resource to evaluate against
    pub resource: JsonValue,
    /// Optional context
    pub context: Option<String>,
    /// Validate expression flag
    pub validate: bool,
    /// Variables for the expression
    pub variables: HashMap<String, JsonValue>,
    /// Terminology server URL
    pub terminology_server: Option<String>,
}

impl FhirPathLabRequest {
    /// Parse the FHIR Parameters request into a more usable format
    pub fn parse(self) -> Result<ParsedFhirPathLabRequest, String> {
        let mut expression = None;
        let mut resource = None;
        let mut context = None;
        let mut validate = false;
        let mut variables = HashMap::new();
        let mut terminology_server = None;

        for param in self.parameter {
            match param.name.as_str() {
                "expression" => {
                    expression = param.value_string;
                }
                "resource" => {
                    resource = param.resource;
                }
                "context" => {
                    context = param.value_string;
                }
                "validate" => {
                    validate = param.value_boolean.unwrap_or(false);
                }
                "terminologyServer" => {
                    terminology_server = param.value_string;
                }
                "variables" => {
                    if let Some(parts) = param.part {
                        for part in parts {
                            if let Some(value) = part.value_string {
                                variables.insert(part.name, JsonValue::String(value));
                            } else if let Some(res) = part.resource {
                                variables.insert(part.name, res);
                            }
                        }
                    }
                }
                _ => {} // Ignore unknown parameters
            }
        }

        let expression = expression
            .ok_or("Missing required 'expression' parameter")?
            .trim()
            .to_string();
        let resource = resource.ok_or("Missing required 'resource' parameter")?;

        Ok(ParsedFhirPathLabRequest {
            expression,
            resource,
            context,
            validate,
            variables,
            terminology_server,
        })
    }
}

impl FhirPathLabResponse {
    /// Create a new FHIRPath Lab response
    pub fn new() -> Self {
        Self {
            id: None,
            resource_type: "Parameters".to_string(),
            parameter: Vec::new(),
        }
    }

    /// Add a string parameter to the response
    pub fn add_string_parameter(&mut self, name: &str, value: String) {
        self.parameter.push(FhirPathLabResponseParameter {
            name: name.to_string(),
            extension: None,
            value_string: Some(value),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });
    }

    /// Add a resource parameter to the response
    pub fn add_resource_parameter(&mut self, name: &str, resource: JsonValue) {
        self.parameter.push(FhirPathLabResponseParameter {
            name: name.to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: Some(resource),
            part: None,
        });
    }

    /// Add a parameter with JSON extension for complex types (FHIRPath Lab API compatibility)
    pub fn add_json_extension_parameter(&mut self, name: &str, value: JsonValue) {
        // Create a parameter with the json-value extension for complex types
        let param = FhirPathLabResponseParameter {
            name: name.to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: Some(serde_json::json!({
                "extension": [{
                    "url": "http://fhir.forms-lab.com/StructureDefinition/json-value",
                    "valueString": value.to_string()
                }]
            })),
            part: None,
        };
        self.parameter.push(param);
    }

    /// Add a result parameter with enhanced type information (FHIRPath Lab API format)
    pub fn add_result_with_metadata(&mut self, result_json: JsonValue) {
        let mut result_parts = Vec::new();

        // Add trace information
        result_parts.push(FhirPathLabResponseParameter {
            name: "trace".to_string(),
            extension: None,
            value_string: Some("Evaluation completed with type metadata".to_string()),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });

        // For FHIRPath Lab API, we should return the structured metadata directly
        // The JSON extension is only needed for values that can't be represented in FHIR
        result_parts.push(FhirPathLabResponseParameter {
            name: "result".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: Some(result_json),
            part: None,
        });

        self.add_complex_parameter("result", result_parts);
    }

    /// Add a result with JSON extension for truly complex non-FHIR data
    pub fn add_result_with_json_extension(&mut self, result_json: JsonValue) {
        let mut result_parts = Vec::new();

        result_parts.push(FhirPathLabResponseParameter {
            name: "trace".to_string(),
            extension: None,
            value_string: Some("Evaluation completed with JSON extension".to_string()),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: None,
        });

        // Use JSON extension only for complex data that doesn't fit FHIR structure
        result_parts.push(FhirPathLabResponseParameter {
            name: "result".to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: Some(serde_json::json!({
                "extension": [{
                    "url": "http://fhir.forms-lab.com/StructureDefinition/json-value",
                    "valueString": result_json.to_string()
                }]
            })),
            part: None,
        });

        self.add_complex_parameter("result", result_parts);
    }

    /// Add a complex parameter with nested parts
    pub fn add_complex_parameter(&mut self, name: &str, parts: Vec<FhirPathLabResponseParameter>) {
        self.parameter.push(FhirPathLabResponseParameter {
            name: name.to_string(),
            extension: None,
            value_string: None,
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: Some(parts),
        });
    }

    /// Add trace information from collected traces
    pub fn add_trace_from_collection(&mut self, traces: Vec<String>) {
        if traces.is_empty() {
            return;
        }

        // Combine all traces into a single trace string
        let combined_trace = traces.join("\n");
        self.add_string_parameter("trace", combined_trace);
    }

    /// Add AST debug information with proper structure
    pub fn add_ast_debug_info(&mut self, parse_debug: String, parse_debug_tree: serde_json::Value) {
        self.add_string_parameter("parseDebug", parse_debug);
        // parseDebugTree should be a JSON string, not a resource parameter according to API spec
        let tree_json =
            serde_json::to_string_pretty(&parse_debug_tree).unwrap_or_else(|_| "{}".to_string());
        self.add_string_parameter("parseDebugTree", tree_json);
    }

    pub fn add_structured_trace_parameter(&mut self, name: &str, values: Vec<JsonValue>) {
        let mut trace_parts = Vec::new();

        // Convert each trace value to a parameter
        for (index, value) in values.iter().enumerate() {
            trace_parts.push(FhirPathLabResponseParameter {
                name: format!("value{}", index),
                extension: None,
                value_string: if value.is_string() {
                    Some(value.as_str().unwrap_or("").to_string())
                } else {
                    None
                },
                value_code: None,
                value_decimal: if value.is_number() {
                    value.as_f64().map(DecimalRepresentation::from_f64)
                } else {
                    None
                },
                value_boolean: None,
                value_integer: None,
                value_uri: None,
                value_url: None,
                value_canonical: None,
                value_id: None,
                value_oid: None,
                value_uuid: None,
                value_markdown: None,
                value_time: None,
                value_date_time: None,
                value_date: None,
                value_quantity: None,
                value_coding: None,
                value_codeable_concept: None,
                value_period: None,
                value_reference: None,
                value_human_name: None,
                value_identifier: None,
                value_address: None,
                value_contact_point: None,
                resource: if value.is_object() || value.is_array() {
                    Some(value.clone())
                } else {
                    None
                },
                part: None,
            });
        }

        self.parameter.push(FhirPathLabResponseParameter {
            name: "trace".to_string(),
            extension: None,
            value_string: Some(name.to_string()),
            value_code: None,
            value_decimal: None,
            value_boolean: None,
            value_integer: None,
            value_uri: None,
            value_url: None,
            value_canonical: None,
            value_id: None,
            value_oid: None,
            value_uuid: None,
            value_markdown: None,
            value_time: None,
            value_date_time: None,
            value_date: None,
            value_quantity: None,
            value_coding: None,
            value_codeable_concept: None,
            value_period: None,
            value_reference: None,
            value_human_name: None,
            value_identifier: None,
            value_address: None,
            value_contact_point: None,
            resource: None,
            part: Some(trace_parts),
        });
    }
}

// ===== COMMON MODELS =====

/// Error information
#[derive(Debug, Serialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    /// Source location (for parsing errors)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

/// Source location for errors
#[derive(Debug, Serialize)]
pub struct SourceLocation {
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Character offset
    pub offset: usize,
}

/// Execution metadata
#[derive(Debug, Serialize)]
pub struct ExecutionMetadata {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Cache hits (if available)
    pub cache_hits: u64,
    /// AST node count (if available)
    pub ast_nodes: u64,
    /// Memory usage in bytes (if available)
    pub memory_used: u64,
    /// Engine reuse (indicates if pre-initialized engine was used)
    pub engine_reused: bool,
}

/// Type information for expressions
#[derive(Debug, Serialize)]
pub struct TypeInfo {
    /// Expected return type
    pub return_type: String,
    /// Type constraints
    pub constraints: Vec<String>,
    /// Cardinality information
    pub cardinality: String,
}

/// Validation error information for the API
#[derive(Debug, Serialize)]
pub struct ValidationErrorInfo {
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: String,
    /// Source location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<SourceLocation>,
}

/// Optimization suggestion
#[derive(Debug, Serialize)]
pub struct OptimizationSuggestion {
    /// Suggestion type
    pub suggestion_type: String,
    /// Human-readable suggestion
    pub message: String,
    /// Suggested replacement (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    /// Impact level (low, medium, high)
    pub impact: String,
}

/// Diagnostic information
#[derive(Debug, Serialize)]
pub struct DiagnosticInfo {
    /// Diagnostic type
    pub diagnostic_type: String,
    /// Message
    pub message: String,
    /// Additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Parsing information
#[derive(Debug, Serialize)]
pub struct ParsingInfo {
    /// Number of tokens
    pub token_count: usize,
    /// Number of AST nodes
    pub ast_node_count: usize,
    /// Parsing time in microseconds
    pub parse_time_us: u64,
    /// Grammar features used
    pub features_used: Vec<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Supported FHIR versions
    pub fhir_versions: Vec<String>,
    /// Engine status for each version
    pub engines: HashMap<String, EngineStatus>,
    /// Memory usage information
    pub memory: MemoryInfo,
}

/// Engine status information
#[derive(Debug, Serialize)]
pub struct EngineStatus {
    /// Whether the engine is available
    pub available: bool,
    /// Whether analysis is supported
    pub analysis_available: bool,
    /// Last initialization time
    pub initialized_at: String,
}

/// Memory usage information
#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    /// Used memory in bytes
    pub used_bytes: u64,
    /// Total memory in bytes
    pub total_bytes: u64,
    /// Memory usage percentage
    pub usage_percent: f64,
}

// ===== CONVERSION IMPLEMENTATIONS =====

/// Convert FhirPathValue to JsonValue for API responses
pub fn fhir_value_to_json(value: FhirPathValue) -> JsonValue {
    match value {
        FhirPathValue::Boolean(b, _, _) => JsonValue::Bool(b),
        FhirPathValue::String(s, _, _) => JsonValue::String(s),
        FhirPathValue::Integer(i, _, _) => JsonValue::Number(serde_json::Number::from(i)),
        FhirPathValue::Decimal(d, _, _) => decimal_to_json_value(&d),
        FhirPathValue::DateTime(dt, _, _) => JsonValue::String(dt.to_string()),
        FhirPathValue::Date(d, _, _) => JsonValue::String(d.to_string()),
        FhirPathValue::Time(t, _, _) => JsonValue::String(t.to_string()),
        FhirPathValue::Quantity {
            value,
            unit,
            code,
            system,
            ucum_unit,
            calendar_unit: _,
            type_info: _,
            primitive_element: _,
        } => {
            let mut obj = JsonMap::new();
            obj.insert("value".to_string(), decimal_to_json_value(&value));

            if let Some(unit_str) = unit {
                obj.insert("unit".to_string(), JsonValue::String(unit_str));
            }

            if let Some(system_str) = system {
                obj.insert("system".to_string(), JsonValue::String(system_str));
            }

            if let Some(code_str) = code {
                obj.insert("code".to_string(), JsonValue::String(code_str));
            }

            if let Some(ucum) = ucum_unit {
                if !obj.contains_key("system") {
                    obj.insert(
                        "system".to_string(),
                        JsonValue::String("http://unitsofmeasure.org".to_string()),
                    );
                }
                if !obj.contains_key("code") {
                    obj.insert("code".to_string(), JsonValue::String(ucum.code.to_string()));
                }
                if !obj.contains_key("unit") {
                    obj.insert(
                        "unit".to_string(),
                        JsonValue::String(ucum.display_name.to_string()),
                    );
                }
            }

            JsonValue::Object(obj)
        }
        FhirPathValue::Collection(collection) => {
            let items: Vec<JsonValue> = collection
                .iter()
                .map(|v| fhir_value_to_json(v.clone()))
                .collect();
            JsonValue::Array(items)
        }
        FhirPathValue::Resource(resource, _, _) => {
            // Convert FHIR resource back to original JSON value
            resource.as_ref().clone()
        }
        FhirPathValue::Empty => JsonValue::Array(vec![]), // Empty collection
    }
}

pub fn decimal_to_json_value(decimal: &rust_decimal::Decimal) -> JsonValue {
    let decimal_str = canonical_decimal_string(decimal);

    match serde_json::Number::from_str(&decimal_str) {
        Ok(number) => JsonValue::Number(number),
        Err(_) => JsonValue::String(decimal_str),
    }
}

fn canonical_decimal_string(decimal: &rust_decimal::Decimal) -> String {
    let mut decimal_str = decimal.to_string();
    if decimal_str.contains(',') {
        decimal_str = decimal_str.replace(',', ".");
    }
    decimal_str
}

/// Representation for FHIR decimal values allowing precise string preservation
#[derive(Debug, Clone)]
pub enum DecimalRepresentation {
    Float(f64),
    Canonical(String),
}

impl DecimalRepresentation {
    pub fn from_f64(value: f64) -> Self {
        Self::Float(value)
    }

    pub fn from_decimal(decimal: &rust_decimal::Decimal) -> Self {
        Self::Canonical(canonical_decimal_string(decimal))
    }
}

impl SerSerialize for DecimalRepresentation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Float(value) => serializer.serialize_f64(*value),
            Self::Canonical(text) => serializer.serialize_str(text),
        }
    }
}

/// Convert Collection to JsonValue array for API responses
pub fn collection_to_json(collection: &octofhir_fhirpath::Collection) -> JsonValue {
    JsonValue::Array(
        collection
            .iter()
            .map(|v| fhir_value_to_json(v.clone()))
            .collect(),
    )
}

impl ExecutionMetadata {
    /// Create execution metadata with timing information
    pub fn with_duration(duration: Duration, engine_reused: bool) -> Self {
        Self {
            execution_time_ms: duration.as_millis() as u64,
            cache_hits: 0,
            ast_nodes: 0,
            memory_used: 0,
            engine_reused,
        }
    }
}

// ===== OPERATION OUTCOME MODELS =====

/// OperationOutcome response for errors
#[derive(Debug, Serialize)]
pub struct OperationOutcome {
    /// FHIR resource type (always "OperationOutcome")
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    /// Issues array
    pub issue: Vec<OperationOutcomeIssue>,
}

/// Issue within OperationOutcome
#[derive(Debug, Serialize)]
pub struct OperationOutcomeIssue {
    /// Severity of the issue
    pub severity: String, // "error", "warning", "information"
    /// Issue type code
    pub code: String, // "exception", "invalid", etc.
    /// Details with text message
    pub details: OperationOutcomeDetails,
    /// Optional diagnostics info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<String>,
}

/// Details structure for OperationOutcome issues
#[derive(Debug, Serialize)]
pub struct OperationOutcomeDetails {
    /// Text description of the issue
    pub text: String,
}

impl OperationOutcome {
    /// Create a new OperationOutcome with a single error issue
    pub fn error(code: &str, message: &str, diagnostics: Option<String>) -> Self {
        Self {
            resource_type: "OperationOutcome".to_string(),
            issue: vec![OperationOutcomeIssue {
                severity: "error".to_string(),
                code: code.to_string(),
                details: OperationOutcomeDetails {
                    text: message.to_string(),
                },
                diagnostics,
            }],
        }
    }
}
