// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Unified metadata system for FHIRPath operations
//!
//! This module provides a comprehensive metadata system that describes
//! both functions and operators with unified type information, performance
//! characteristics, and LSP support.

use crate::enhanced_metadata::{PerformanceComplexity, PerformanceMetadata as LegacyPerformanceMetadata};
use crate::signature::FunctionSignature;
use octofhir_fhirpath_model::FhirPathValue;
use serde::{Deserialize, Serialize};

/// Unified metadata for operations (functions and operators)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationMetadata {
    /// Basic operation information
    pub basic: BasicOperationInfo,
    
    /// Type constraints and signatures
    pub types: TypeConstraints,
    
    /// Performance characteristics
    pub performance: PerformanceMetadata,
    
    /// LSP support information
    pub lsp: LspMetadata,
    
    /// Operation-specific metadata
    pub specific: OperationSpecificMetadata,
}

/// Basic information about an operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicOperationInfo {
    /// Operation name or symbol
    pub name: String,
    
    /// Type of operation
    pub operation_type: OperationType,
    
    /// Human-readable description
    pub description: String,
    
    /// Usage examples
    pub examples: Vec<String>,
    
    /// Documentation URL (optional)
    pub documentation_url: Option<String>,
    
    /// FHIRPath specification section (optional)
    pub spec_section: Option<String>,
}

/// Operation type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    /// Function call (e.g., count(), first(), where())
    Function,
    
    /// Binary operator with precedence and associativity
    BinaryOperator { 
        precedence: u8, 
        associativity: Associativity 
    },
    
    /// Unary operator (e.g., not, -)
    UnaryOperator,
}

/// Operator associativity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Associativity {
    /// Left-to-right associativity (a + b + c = (a + b) + c)
    Left,
    /// Right-to-left associativity (a = b = c means a = (b = c))
    Right,
    /// Non-associative (a = b = c is an error)
    None,
}

/// Type constraints and signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConstraints {
    /// Input type constraints
    pub input_types: Vec<TypeConstraint>,
    
    /// Parameter type constraints
    pub parameters: Vec<ParameterConstraint>,
    
    /// Return type constraint
    pub return_type: TypeConstraint,
    
    /// Whether the operation accepts variable arguments
    pub variadic: bool,
    
    /// Context requirements (what the operation needs from evaluation context)
    pub context_requirements: ContextRequirements,
}

impl Default for TypeConstraints {
    fn default() -> Self {
        Self {
            input_types: vec![TypeConstraint::Any],
            parameters: vec![],
            return_type: TypeConstraint::Any,
            variadic: false,
            context_requirements: ContextRequirements::default(),
        }
    }
}

/// Type constraint for a single type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeConstraint {
    /// Any type allowed
    Any,
    
    /// Specific FHIRPath type
    Specific(FhirPathType),
    
    /// One of several types
    OneOf(Vec<FhirPathType>),
    
    /// Collection of specific type
    Collection(Box<TypeConstraint>),
    
    /// Optional type (may be empty)
    Optional(Box<TypeConstraint>),
    
    /// Type must be convertible to target type
    ConvertibleTo(FhirPathType),
    
    /// Custom type predicate
    Custom {
        description: String,
        validator_name: String,
    },
}

/// FHIRPath type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FhirPathType {
    /// Boolean type
    Boolean,
    /// Integer type
    Integer,
    /// Decimal type
    Decimal,
    /// String type
    String,
    /// Date type
    Date,
    /// DateTime type
    DateTime,
    /// Time type
    Time,
    /// Quantity type (with unit)
    Quantity,
    /// FHIR Resource type
    Resource(String),
    /// FHIR DataType
    DataType(String),
    /// Collection type
    Collection,
    /// Empty type
    Empty,
}

impl From<&FhirPathValue> for FhirPathType {
    fn from(value: &FhirPathValue) -> Self {
        match value {
            FhirPathValue::Boolean(_) => Self::Boolean,
            FhirPathValue::Integer(_) => Self::Integer,
            FhirPathValue::Decimal(_) => Self::Decimal,
            FhirPathValue::String(_) => Self::String,
            FhirPathValue::Date(_) => Self::Date,
            FhirPathValue::DateTime(_) => Self::DateTime,
            FhirPathValue::Time(_) => Self::Time,
            FhirPathValue::Quantity(_) => Self::Quantity,
            FhirPathValue::Collection(_) => Self::Collection,
            FhirPathValue::Empty => Self::Empty,
            FhirPathValue::Resource(_) => Self::Resource("Resource".to_string()),
            FhirPathValue::JsonValue(_) => Self::DataType("JsonValue".to_string()),
            FhirPathValue::TypeInfoObject { .. } => Self::DataType("TypeInfo".to_string()),
        }
    }
}

/// Parameter constraint with name and type information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterConstraint {
    /// Parameter name
    pub name: String,
    
    /// Parameter type constraint
    pub type_constraint: TypeConstraint,
    
    /// Whether parameter is optional
    pub optional: bool,
    
    /// Default value (if any)
    pub default_value: Option<String>,
    
    /// Parameter description
    pub description: String,
}

/// Context requirements for operation evaluation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextRequirements {
    /// Requires access to original resource
    pub needs_resource: bool,
    
    /// Requires access to root resource
    pub needs_root_resource: bool,
    
    /// Requires access to environment variables
    pub needs_variables: bool,
    
    /// Requires access to model provider
    pub needs_model_provider: bool,
    
    /// Requires specific evaluation mode
    pub evaluation_mode: Option<String>,
}

/// Performance characteristics and optimization hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetadata {
    /// Computational complexity classification
    pub complexity: PerformanceComplexity,
    
    /// Whether operation supports synchronous evaluation
    pub supports_sync: bool,
    
    /// Average execution time in nanoseconds
    pub avg_time_ns: u64,
    
    /// Memory usage hint in bytes
    pub memory_usage: u64,
    
    /// Whether operation can be cached
    pub cacheable: bool,
    
    /// Whether operation is side-effect free
    pub pure: bool,
    
    /// Optimization hints
    pub optimization_hints: Vec<OptimizationHint>,
}

impl From<LegacyPerformanceMetadata> for PerformanceMetadata {
    fn from(legacy: LegacyPerformanceMetadata) -> Self {
        let memory_bytes = match legacy.memory_usage {
            crate::enhanced_metadata::MemoryUsage::Minimal => 64,
            crate::enhanced_metadata::MemoryUsage::Linear => 256,
            crate::enhanced_metadata::MemoryUsage::Exponential => 4096,
            crate::enhanced_metadata::MemoryUsage::Streaming => 128,
            crate::enhanced_metadata::MemoryUsage::Custom(_) => 1024,
        };

        let time_ns = match legacy.execution_time {
            crate::enhanced_metadata::ExecutionTime::UltraFast => 10,
            crate::enhanced_metadata::ExecutionTime::Fast => 1000,
            crate::enhanced_metadata::ExecutionTime::Moderate => 10000,
            crate::enhanced_metadata::ExecutionTime::Slow => 100000,
            crate::enhanced_metadata::ExecutionTime::VerySlow => 1000000,
        };

        Self {
            complexity: legacy.complexity,
            supports_sync: true, // Legacy assumed sync support
            avg_time_ns: time_ns,
            memory_usage: memory_bytes,
            cacheable: legacy.cacheable,
            pure: legacy.is_pure,
            optimization_hints: vec![],
        }
    }
}

/// Optimization hints for the evaluator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationHint {
    /// Operation can be inlined for small arguments
    Inlineable,
    
    /// Operation benefits from argument pre-evaluation
    PreEvaluateArgs,
    
    /// Operation can be vectorized
    Vectorizable,
    
    /// Operation has specialized fast paths
    HasFastPaths,
    
    /// Operation should be deferred until needed
    Lazy,
    
    /// Operation benefits from parallel execution
    Parallelizable,
}

/// LSP (Language Server Protocol) support metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LspMetadata {
    /// Completion item information
    pub completion: CompletionInfo,
    
    /// Hover information
    pub hover: HoverInfo,
    
    /// Signature help information
    pub signature_help: SignatureHelpInfo,
    
    /// Diagnostic information
    pub diagnostics: DiagnosticInfo,
}

/// Completion item information for LSP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionInfo {
    /// Completion item label
    pub label: String,
    
    /// Additional detail text
    pub detail: String,
    
    /// Documentation string
    pub documentation: String,
    
    /// Completion kind (function, operator, etc.)
    pub kind: CompletionKind,
    
    /// Text to insert when completing
    pub insert_text: String,
    
    /// Whether to show in completion list
    pub visible: bool,
}

/// LSP completion item kind
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum CompletionKind {
    #[default]
    Function,
    Operator,
    Keyword,
    Variable,
    Constant,
}

/// Hover information for LSP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HoverInfo {
    /// Content to show on hover
    pub content: String,
    
    /// Content format (markdown, plaintext)
    pub format: HoverFormat,
}

/// Hover content format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum HoverFormat {
    #[default]
    Markdown,
    PlainText,
}

/// Signature help information for LSP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignatureHelpInfo {
    /// Signature label
    pub label: String,
    
    /// Documentation for the signature
    pub documentation: String,
    
    /// Parameter information
    pub parameters: Vec<ParameterInfo>,
}

/// Parameter information for signature help
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterInfo {
    /// Parameter label
    pub label: String,
    
    /// Parameter documentation
    pub documentation: String,
    
    /// Start position in signature label
    pub start: usize,
    
    /// End position in signature label
    pub end: usize,
}

/// Diagnostic information for LSP
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    /// Common error patterns
    pub error_patterns: Vec<ErrorPattern>,
    
    /// Validation rules
    pub validation_rules: Vec<ValidationRule>,
}

/// Error pattern for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    /// Pattern description
    pub description: String,
    
    /// Error message template
    pub message: String,
    
    /// Severity level
    pub severity: DiagnosticSeverity,
}

/// Validation rule for diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule description
    pub description: String,
    
    /// Validation function name
    pub validator: String,
    
    /// Error message if validation fails
    pub error_message: String,
}

/// Diagnostic severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Operation-specific metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationSpecificMetadata {
    /// Function-specific metadata
    Function(FunctionMetadata),
    
    /// Operator-specific metadata
    Operator(OperatorMetadata),
}

/// Function-specific metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function category (collection, string, math, etc.)
    pub category: String,
    
    /// Function signature for overload resolution
    pub signature: Option<FunctionSignature>,
    
    /// Whether function supports lambda expressions
    pub supports_lambda: bool,
    
    /// Lambda parameter information
    pub lambda_parameters: Vec<LambdaParameterInfo>,
    
    /// Aggregate function information
    pub aggregate_info: Option<AggregateInfo>,
}

/// Lambda parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaParameterInfo {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub parameter_type: LambdaParameterType,
    
    /// Description
    pub description: String,
}

/// Lambda parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaParameterType {
    /// Current item in iteration
    CurrentItem,
    
    /// Index in iteration
    Index,
    
    /// Accumulator value
    Accumulator,
    
    /// Custom parameter
    Custom(String),
}

/// Aggregate function information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateInfo {
    /// Initial value for aggregation
    pub initial_value: String,
    
    /// Whether aggregation is commutative
    pub commutative: bool,
    
    /// Whether aggregation is associative
    pub associative: bool,
}

/// Operator-specific metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OperatorMetadata {
    /// Operator symbol
    pub symbol: String,
    
    /// Alternative symbols (aliases)
    pub aliases: Vec<String>,
    
    /// Operator category (arithmetic, logical, comparison)
    pub category: OperatorCategory,
    
    /// Short-circuit evaluation support
    pub short_circuit: bool,
    
    /// Commutative property
    pub commutative: bool,
    
    /// Associative property
    pub associative: bool,
}

/// Operator categories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum OperatorCategory {
    #[default]
    Arithmetic,
    Logical,
    Comparison,
    Collection,
    String,
    TypeCheck,
}

/// Metadata builder for creating operation metadata
pub struct MetadataBuilder {
    metadata: OperationMetadata,
}

impl MetadataBuilder {
    /// Create a new metadata builder
    pub fn new(name: &str, operation_type: OperationType) -> Self {
        let specific = match &operation_type {
            OperationType::Function => OperationSpecificMetadata::Function(FunctionMetadata::default()),
            OperationType::BinaryOperator { .. } | OperationType::UnaryOperator => {
                OperationSpecificMetadata::Operator(OperatorMetadata::default())
            }
        };

        Self {
            metadata: OperationMetadata {
                basic: BasicOperationInfo {
                    name: name.to_string(),
                    operation_type,
                    description: String::new(),
                    examples: vec![],
                    documentation_url: None,
                    spec_section: None,
                },
                types: TypeConstraints::default(),
                performance: PerformanceMetadata {
                    complexity: PerformanceComplexity::Linear,
                    supports_sync: false,
                    avg_time_ns: 1000,
                    memory_usage: 256,
                    cacheable: true,
                    pure: true,
                    optimization_hints: vec![],
                },
                lsp: LspMetadata::default(),
                specific,
            },
        }
    }

    /// Set operation description
    pub fn description(mut self, description: &str) -> Self {
        self.metadata.basic.description = description.to_string();
        self
    }

    /// Add usage example
    pub fn example(mut self, example: &str) -> Self {
        self.metadata.basic.examples.push(example.to_string());
        self
    }

    /// Set return type constraint
    pub fn returns(mut self, return_type: TypeConstraint) -> Self {
        self.metadata.types.return_type = return_type;
        self
    }

    /// Add parameter constraint
    pub fn parameter(mut self, name: &str, type_constraint: TypeConstraint) -> Self {
        self.metadata.types.parameters.push(ParameterConstraint {
            name: name.to_string(),
            type_constraint,
            optional: false,
            default_value: None,
            description: String::new(),
        });
        self
    }

    /// Set performance characteristics
    pub fn performance(mut self, complexity: PerformanceComplexity, supports_sync: bool) -> Self {
        self.metadata.performance.complexity = complexity;
        self.metadata.performance.supports_sync = supports_sync;
        self
    }

    /// Mark as variadic
    pub fn variadic(mut self) -> Self {
        self.metadata.types.variadic = true;
        self
    }

    /// Build the metadata
    pub fn build(self) -> OperationMetadata {
        self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_builder() {
        let metadata = MetadataBuilder::new("test", OperationType::Function)
            .description("Test function")
            .example("test()")
            .returns(TypeConstraint::Specific(FhirPathType::String))
            .parameter("arg1", TypeConstraint::Specific(FhirPathType::Integer))
            .performance(PerformanceComplexity::Constant, true)
            .build();

        assert_eq!(metadata.basic.name, "test");
        assert_eq!(metadata.basic.description, "Test function");
        assert_eq!(metadata.basic.examples.len(), 1);
        assert_eq!(metadata.types.parameters.len(), 1);
        assert_eq!(metadata.performance.complexity, PerformanceComplexity::Constant);
        assert!(metadata.performance.supports_sync);
    }

    #[test]
    fn test_type_constraint_from_value() {
        let value = FhirPathValue::Integer(42);
        let fhir_type = FhirPathType::from(&value);
        assert_eq!(fhir_type, FhirPathType::Integer);

        let value = FhirPathValue::String("test".into());
        let fhir_type = FhirPathType::from(&value);
        assert_eq!(fhir_type, FhirPathType::String);
    }

    #[test]
    fn test_operation_type_equality() {
        let op1 = OperationType::Function;
        let op2 = OperationType::Function;
        assert_eq!(op1, op2);

        let op3 = OperationType::BinaryOperator { 
            precedence: 10, 
            associativity: Associativity::Left 
        };
        let op4 = OperationType::BinaryOperator { 
            precedence: 10, 
            associativity: Associativity::Left 
        };
        assert_eq!(op3, op4);
    }

    #[test]
    fn test_serialization() {
        let metadata = MetadataBuilder::new("test", OperationType::Function)
            .description("Test function")
            .build();

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: OperationMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.basic.name, "test");
        assert_eq!(deserialized.basic.description, "Test function");
    }
}