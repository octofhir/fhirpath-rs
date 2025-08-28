//! Request and response models for the FHIRPath HTTP server

use crate::FhirPathValue;
use octofhir_fhirpath_analyzer::{AnalysisResult as AnalyzerResult, ValidationError};
use octofhir_ucum::precision::NumericOps;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
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
        FhirPathValue::Boolean(b) => JsonValue::Bool(b),
        FhirPathValue::String(s) => JsonValue::String(s.to_string()),
        FhirPathValue::Integer(i) => JsonValue::Number(serde_json::Number::from(i)),
        FhirPathValue::Decimal(d) => {
            // Convert decimal to JSON number, handling precision
            if let Ok(f) = d.to_string().parse::<f64>() {
                JsonValue::Number(
                    serde_json::Number::from_f64(f).unwrap_or_else(|| serde_json::Number::from(0)),
                )
            } else {
                JsonValue::String(d.to_string())
            }
        }
        FhirPathValue::DateTime(dt) => JsonValue::String(dt.to_string()),
        FhirPathValue::Date(d) => JsonValue::String(d.to_string()),
        FhirPathValue::Time(t) => JsonValue::String(t.to_string()),
        FhirPathValue::Quantity(q) => {
            // Serialize quantity as object
            serde_json::json!({
                "value": q.value.to_f64(),
                "unit": q.unit.as_ref().map(|u| u.to_string()),
            })
        }
        FhirPathValue::Collection(collection) => {
            let items: Vec<JsonValue> = collection.into_iter().map(fhir_value_to_json).collect();
            JsonValue::Array(items)
        }
        FhirPathValue::Resource(resource) => {
            // Convert FHIR resource back to JSON using Debug trait
            JsonValue::String(format!("{:?}", resource))
        }
        FhirPathValue::JsonValue(json_val) => json_val.into_inner(),
        FhirPathValue::TypeInfoObject { namespace, name } => {
            serde_json::json!({
                "namespace": namespace.to_string(),
                "name": name.to_string()
            })
        }
        FhirPathValue::Empty => JsonValue::Array(vec![]), // Empty collection
    }
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
