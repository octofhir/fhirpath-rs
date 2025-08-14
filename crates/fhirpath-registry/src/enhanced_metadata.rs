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

//! Enhanced metadata structures for function registry

use crate::function::{FunctionMetadata, LspMetadata};
use crate::signature::FunctionSignature;
use crate::unified_function::ExecutionMode;
use octofhir_fhirpath_model::types::TypeInfo;
use serde::{Deserialize, Serialize};

/// Enhanced function metadata with rich information for LSP and analyzer support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedFunctionMetadata {
    /// Basic function information (maintains compatibility)
    pub basic: FunctionMetadata,
    
    /// Function signature for validation
    pub signature: FunctionSignature,
    
    /// Execution characteristics
    pub execution_mode: ExecutionMode,
    
    /// Type constraints and applicability
    pub type_constraints: TypeConstraints,
    
    /// Performance characteristics
    pub performance: PerformanceMetadata,
    
    /// LSP-specific information
    pub lsp: LspMetadata,
    
    /// Analyzer-specific information
    pub analyzer: AnalyzerMetadata,
    
    /// Lambda expression support information
    pub lambda: LambdaMetadata,
}

/// Type constraints for intelligent function applicability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeConstraints {
    /// Input type patterns this function accepts
    pub input_types: Vec<TypePattern>,
    
    /// Whether function works on collections
    pub supports_collections: bool,
    
    /// Whether function requires collection input
    pub requires_collection: bool,
    
    /// Output type information
    pub output_type: TypePattern,
    
    /// Whether output is always a collection
    pub output_is_collection: bool,
}

/// Flexible type pattern matching for sophisticated type constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TypePattern {
    /// Exact type match
    Exact(TypeInfo),
    
    /// Any of these types
    OneOf(Vec<TypeInfo>),
    
    /// Any type (no constraints)
    Any,
    
    /// Collection of specific type
    CollectionOf(Box<TypePattern>),
    
    /// Numeric types (Integer or Decimal)
    Numeric,
    
    /// String-like types (String, Code, Id, Uri, etc.)
    StringLike,
    
    /// Date/time types
    DateTime,
    
    /// Boolean type
    Boolean,
    
    /// FHIR Resource types
    Resource,
    
    /// Quantity type
    Quantity,
}

impl TypePattern {
    /// Check if a type matches this pattern
    pub fn matches(&self, type_info: &TypeInfo) -> bool {
        match self {
            TypePattern::Any => true,
            TypePattern::Exact(expected) => expected == type_info,
            TypePattern::OneOf(types) => types.contains(type_info),
            TypePattern::CollectionOf(inner_pattern) => {
                if let TypeInfo::Collection(inner_type) = type_info {
                    inner_pattern.matches(inner_type)
                } else {
                    false
                }
            }
            TypePattern::Numeric => {
                matches!(type_info, TypeInfo::Integer | TypeInfo::Decimal)
            }
            TypePattern::StringLike => {
                matches!(type_info, TypeInfo::String)
            }
            TypePattern::DateTime => {
                matches!(type_info, 
                    TypeInfo::DateTime | 
                    TypeInfo::Date | 
                    TypeInfo::Time
                )
            }
            TypePattern::Boolean => matches!(type_info, TypeInfo::Boolean),
            TypePattern::Resource => matches!(type_info, TypeInfo::Resource(_)),
            TypePattern::Quantity => matches!(type_info, TypeInfo::Quantity),
        }
    }
    
    /// Get a human-readable description of this pattern
    pub fn description(&self) -> String {
        match self {
            TypePattern::Any => "Any".to_string(),
            TypePattern::Exact(type_info) => format!("{}", type_info),
            TypePattern::OneOf(types) => {
                let type_names: Vec<String> = types.iter().map(|t| format!("{}", t)).collect();
                format!("One of: {}", type_names.join(", "))
            }
            TypePattern::CollectionOf(inner) => format!("Collection<{}>", inner.description()),
            TypePattern::Numeric => "Numeric (Integer or Decimal)".to_string(),
            TypePattern::StringLike => "String-like (String, Code, Id, Uri, etc.)".to_string(),
            TypePattern::DateTime => "Date/Time (DateTime, Date, Time, Instant)".to_string(),
            TypePattern::Boolean => "Boolean".to_string(),
            TypePattern::Resource => "FHIR Resource".to_string(),
            TypePattern::Quantity => "Quantity".to_string(),
        }
    }
}

impl std::fmt::Display for TypePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Performance characteristics for optimization and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetadata {
    /// Is this a pure function (deterministic, no side effects)
    pub is_pure: bool,
    
    /// Estimated computational complexity
    pub complexity: PerformanceComplexity,
    
    /// Whether result should be cached
    pub cacheable: bool,
    
    /// Memory usage characteristics
    pub memory_usage: MemoryUsage,
    
    /// Typical execution time category
    pub execution_time: ExecutionTime,
}

/// Computational complexity categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PerformanceComplexity {
    /// Constant time O(1)
    Constant,
    
    /// Logarithmic time O(log n)
    Logarithmic,
    
    /// Linear time O(n)
    Linear,
    
    /// Linearithmic time O(n log n)
    Linearithmic,
    
    /// Quadratic time O(n²)
    Quadratic,
    
    /// Custom complexity with description
    Custom(String),
}

impl std::fmt::Display for PerformanceComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerformanceComplexity::Constant => write!(f, "O(1)"),
            PerformanceComplexity::Logarithmic => write!(f, "O(log n)"),
            PerformanceComplexity::Linear => write!(f, "O(n)"),
            PerformanceComplexity::Linearithmic => write!(f, "O(n log n)"),
            PerformanceComplexity::Quadratic => write!(f, "O(n²)"),
            PerformanceComplexity::Custom(desc) => write!(f, "{}", desc),
        }
    }
}

/// Memory usage patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryUsage {
    /// Minimal memory allocation
    Minimal,
    
    /// Memory proportional to input size
    Linear,
    
    /// Potentially large memory allocations
    Exponential,
    
    /// Processes data in chunks (streaming)
    Streaming,
    
    /// Custom memory pattern with description
    Custom(String),
}

impl std::fmt::Display for MemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryUsage::Minimal => write!(f, "Minimal"),
            MemoryUsage::Linear => write!(f, "Linear"),
            MemoryUsage::Exponential => write!(f, "Exponential"),
            MemoryUsage::Streaming => write!(f, "Streaming"),
            MemoryUsage::Custom(desc) => write!(f, "{}", desc),
        }
    }
}

/// Typical execution time categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionTime {
    /// Ultra-fast (<1µs)
    UltraFast,
    
    /// Fast (<10µs)
    Fast,
    
    /// Moderate (<100µs)
    Moderate,
    
    /// Slow (<1ms)
    Slow,
    
    /// Very slow (>1ms)
    VerySlow,
}

impl std::fmt::Display for ExecutionTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionTime::UltraFast => write!(f, "<1µs"),
            ExecutionTime::Fast => write!(f, "<10µs"),
            ExecutionTime::Moderate => write!(f, "<100µs"),
            ExecutionTime::Slow => write!(f, "<1ms"),
            ExecutionTime::VerySlow => write!(f, ">1ms"),
        }
    }
}

/// Analyzer-specific metadata for static analysis and tooling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerMetadata {
    /// Whether this function has side effects
    pub has_side_effects: bool,
    
    /// Dependencies on external systems
    pub external_dependencies: Vec<ExternalDependency>,
    
    /// Common usage patterns
    pub usage_patterns: Vec<UsagePattern>,
    
    /// Related functions (similar or alternative functions)
    pub related_functions: Vec<String>,
    
    /// Function maturity level
    pub maturity_level: MaturityLevel,
    
    /// Deprecation information if applicable
    pub deprecation: Option<DeprecationInfo>,
}

/// Lambda expression support metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaMetadata {
    /// Whether this function supports lambda expressions
    pub supports_lambda_evaluation: bool,
    
    /// Which argument indices should remain as expressions (not pre-evaluated)
    pub lambda_argument_indices: Vec<usize>,
    
    /// Description of lambda evaluation behavior
    pub lambda_description: Option<String>,
    
    /// Whether lambda evaluation is required (cannot fallback to traditional)
    pub requires_lambda_evaluation: bool,
}

/// External system dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExternalDependency {
    /// Requires ModelProvider for type information
    ModelProvider,
    
    /// Requires network access
    NetworkAccess,
    
    /// Requires file system access
    FileSystem,
    
    /// Depends on system time
    SystemTime,
    
    /// Uses random number generation
    Random,
    
    /// Requires configuration
    Configuration,
    
    /// Custom dependency with description
    Custom(String),
}

impl std::fmt::Display for ExternalDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalDependency::ModelProvider => write!(f, "Model Provider"),
            ExternalDependency::NetworkAccess => write!(f, "Network Access"),
            ExternalDependency::FileSystem => write!(f, "File System"),
            ExternalDependency::SystemTime => write!(f, "System Time"),
            ExternalDependency::Random => write!(f, "Random"),
            ExternalDependency::Configuration => write!(f, "Configuration"),
            ExternalDependency::Custom(desc) => write!(f, "{}", desc),
        }
    }
}

/// Common usage patterns for documentation and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    /// Pattern description
    pub description: String,
    
    /// Example expression demonstrating the pattern
    pub example: String,
    
    /// Context where this pattern is commonly used
    pub context: String,
    
    /// Expected frequency of this usage pattern
    pub frequency: UsageFrequency,
}

/// Usage frequency categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UsageFrequency {
    /// Very common usage
    VeryCommon,
    
    /// Common usage
    Common,
    
    /// Moderate usage
    Moderate,
    
    /// Rare usage
    Rare,
    
    /// Advanced/expert usage
    Advanced,
}

impl std::fmt::Display for UsageFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UsageFrequency::VeryCommon => write!(f, "Very Common"),
            UsageFrequency::Common => write!(f, "Common"),
            UsageFrequency::Moderate => write!(f, "Moderate"),
            UsageFrequency::Rare => write!(f, "Rare"),
            UsageFrequency::Advanced => write!(f, "Advanced"),
        }
    }
}

/// Function maturity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MaturityLevel {
    /// Stable, well-tested function
    Stable,
    
    /// Beta function, may have minor changes
    Beta,
    
    /// Alpha function, experimental
    Alpha,
    
    /// Deprecated, use alternatives
    Deprecated,
}

impl std::fmt::Display for MaturityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaturityLevel::Stable => write!(f, "Stable"),
            MaturityLevel::Beta => write!(f, "Beta"),
            MaturityLevel::Alpha => write!(f, "Alpha"),
            MaturityLevel::Deprecated => write!(f, "Deprecated"),
        }
    }
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    /// Version when function was deprecated
    pub since_version: String,
    
    /// Reason for deprecation
    pub reason: String,
    
    /// Alternative functions to use instead
    pub alternatives: Vec<String>,
    
    /// When the function will be removed (if known)
    pub removal_version: Option<String>,
}

impl Default for PerformanceMetadata {
    fn default() -> Self {
        Self {
            is_pure: false,
            complexity: PerformanceComplexity::Linear,
            cacheable: false,
            memory_usage: MemoryUsage::Minimal,
            execution_time: ExecutionTime::Fast,
        }
    }
}

impl Default for AnalyzerMetadata {
    fn default() -> Self {
        Self {
            has_side_effects: false,
            external_dependencies: Vec::new(),
            usage_patterns: Vec::new(),
            related_functions: Vec::new(),
            maturity_level: MaturityLevel::Stable,
            deprecation: None,
        }
    }
}

impl Default for TypeConstraints {
    fn default() -> Self {
        Self {
            input_types: vec![TypePattern::Any],
            supports_collections: true,
            requires_collection: false,
            output_type: TypePattern::Any,
            output_is_collection: false,
        }
    }
}

impl Default for LambdaMetadata {
    fn default() -> Self {
        Self {
            supports_lambda_evaluation: false,
            lambda_argument_indices: Vec::new(),
            lambda_description: None,
            requires_lambda_evaluation: false,
        }
    }
}