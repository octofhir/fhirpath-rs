//! Type definitions for the FHIRPath analyzer

use crate::error::ValidationError;
use octofhir_fhirpath_model::FhirPathValue;
use octofhir_fhirpath_model::types::TypeInfo;
use std::collections::HashMap;

/// Semantic information attached to AST nodes for FHIRPath specification compliance
#[derive(Debug, Clone, PartialEq)]
pub struct SemanticInfo {
    /// Resolved FHIRPath type (String, Integer, etc.)
    pub fhir_path_type: Option<String>,
    /// Original FHIR model type (Patient, HumanName, etc.)
    pub model_type: Option<String>,
    /// Cardinality information (0..1, 0..*, 1..1, etc.)
    pub cardinality: Cardinality,
    /// Type inference confidence level
    pub confidence: ConfidenceLevel,
    /// Variable scope information for lambda expressions
    pub scope_info: Option<ScopeInfo>,
    /// Function signature validation info
    pub function_info: Option<FunctionSignature>,
}

/// Cardinality constraints for FHIRPath values
#[derive(Debug, Clone, PartialEq)]
pub enum Cardinality {
    /// Zero to one occurrence (0..1)
    ZeroToOne,
    /// Zero to many occurrences (0..*)
    ZeroToMany,
    /// Exactly one occurrence (1..1)
    OneToOne,
    /// One to many occurrences (1..*)
    OneToMany,
    /// Exact occurrence count
    Exactly(u32),
}

/// Confidence level of type inference
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceLevel {
    /// High confidence (95%+ certainty)
    High,
    /// Medium confidence (70-95% certainty)
    Medium,
    /// Low confidence (<70% certainty)
    Low,
}

/// Variable scope information for lambda expressions
#[derive(Debug, Clone, PartialEq)]
pub struct ScopeInfo {
    /// Variable bindings in current scope
    pub variable_bindings: HashMap<String, TypeInfo>,
    /// Lambda parameter name if in lambda context
    pub lambda_parameter: Option<String>,
}

/// Function signature information for validation
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,
    /// Function parameters with type constraints
    pub parameters: Vec<ParameterInfo>,
    /// Expected return type
    pub return_type: TypeInfo,
    /// Whether this is an aggregate function
    pub is_aggregate: bool,
    /// Human-readable function description
    pub description: String,
}

/// Parameter information for function validation
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Type constraint for this parameter
    pub type_constraint: TypeConstraint,
    /// Cardinality constraint for this parameter
    pub cardinality: Cardinality,
    /// Whether this parameter is optional
    pub is_optional: bool,
}

/// Type constraints for function parameters
#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    /// Must be exactly this type
    Exact(String),
    /// Must be one of the specified types
    OneOf(Vec<String>),
    /// Any type is allowed
    Any,
    /// Must be numeric (Integer or Decimal)
    Numeric,
    /// Must be temporal (Date, DateTime, or Time)
    Temporal,
    /// Collection of values with inner type constraint
    Collection(Box<TypeConstraint>),
}

/// Union type information for children() function and choice types
#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeInfo {
    /// Types that make up this union
    pub constituent_types: Vec<TypeInfo>,
    /// Whether this union represents a collection
    pub is_collection: bool,
    /// Model context mapping for type resolution
    pub model_context: HashMap<String, String>,
}

/// Node ID type for tracking AST nodes without modifying them
pub type NodeId = u64;

/// Content-based hash for AST nodes (preserves existing structure)  
pub type ExpressionHash = u64;

/// Analysis result with rich semantic information
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Validation errors found during analysis
    pub validation_errors: Vec<ValidationError>,
    /// Type annotations for AST nodes
    pub type_annotations: HashMap<NodeId, SemanticInfo>,
    /// Function call analysis results
    pub function_calls: Vec<FunctionCallAnalysis>,
    /// Union type information for nodes
    pub union_types: HashMap<NodeId, UnionTypeInfo>,
}

/// Analysis result for a function call
#[derive(Debug, Clone)]
pub struct FunctionCallAnalysis {
    /// AST node ID of the function call
    pub node_id: NodeId,
    /// Name of the called function
    pub function_name: String,
    /// Function signature information
    pub signature: FunctionSignature,
    /// Actual parameter types passed to function
    pub parameter_types: Vec<TypeInfo>,
    /// Inferred return type
    pub return_type: TypeInfo,
    /// Validation errors for this function call
    pub validation_errors: Vec<ValidationError>,
}

/// Context for analysis operations
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Root resource type being analyzed against
    pub root_type: Option<String>,
    /// Current scope variables
    pub variables: HashMap<String, TypeInfo>,
    /// Available environment variables
    pub environment: HashMap<String, FhirPathValue>,
    /// Analysis settings
    pub settings: AnalysisSettings,
}

/// Settings for analysis operations
#[derive(Debug, Clone)]
pub struct AnalysisSettings {
    /// Enable type inference
    pub enable_type_inference: bool,
    /// Enable function signature validation
    pub enable_function_validation: bool,
    /// Enable union type analysis
    pub enable_union_analysis: bool,
    /// Maximum analysis depth to prevent infinite recursion
    pub max_analysis_depth: u32,
}

impl Default for AnalysisSettings {
    fn default() -> Self {
        Self {
            enable_type_inference: true,
            enable_function_validation: true,
            enable_union_analysis: true,
            max_analysis_depth: 100,
        }
    }
}
