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

//! Metadata builder utilities for creating enhanced function metadata

use crate::enhanced_metadata::*;
use crate::function::{CompletionVisibility, FunctionCategory, FunctionMetadata, LspMetadata};
use crate::signature::FunctionSignature;
use crate::unified_function::ExecutionMode;
use octofhir_fhirpath_model::types::TypeInfo;

/// Builder for creating enhanced function metadata with a fluent API
pub struct MetadataBuilder {
    metadata: EnhancedFunctionMetadata,
}

impl MetadataBuilder {
    /// Create a new metadata builder for a function
    pub fn new(name: &str, category: FunctionCategory) -> Self {
        let signature = FunctionSignature::new(name, vec![], TypeInfo::Any);
        
        Self {
            metadata: EnhancedFunctionMetadata {
                basic: FunctionMetadata {
                    name: name.to_string(),
                    display_name: name.to_string(),
                    category: category.clone(),
                    description: String::new(),
                    examples: Vec::new(),
                    input_types: Vec::new(),
                    supports_collections: false,
                    requires_collection: false,
                    output_type: "Any".to_string(),
                    output_is_collection: false,
                    is_pure: false,
                    lsp_info: LspMetadata {
                        snippet: format!("{}()", name),
                        sort_priority: category.sort_priority(),
                        completion_visibility: CompletionVisibility::Contextual,
                        keywords: Vec::new(),
                    },
                },
                signature,
                execution_mode: ExecutionMode::Sync,
                type_constraints: TypeConstraints::default(),
                performance: PerformanceMetadata::default(),
                lsp: LspMetadata {
                    snippet: format!("{}()", name),
                    sort_priority: category.sort_priority(),
                    completion_visibility: CompletionVisibility::Contextual,
                    keywords: Vec::new(),
                },
                analyzer: AnalyzerMetadata::default(),
                lambda: LambdaMetadata::default(),
            },
        }
    }
    
    /// Set the display name
    pub fn display_name(mut self, name: &str) -> Self {
        self.metadata.basic.display_name = name.to_string();
        self
    }
    
    /// Set the description
    pub fn description(mut self, description: &str) -> Self {
        self.metadata.basic.description = description.to_string();
        self
    }
    
    /// Add an example
    pub fn example(mut self, example: &str) -> Self {
        self.metadata.basic.examples.push(example.to_string());
        self
    }
    
    /// Set the function signature
    pub fn signature(mut self, signature: FunctionSignature) -> Self {
        self.metadata.signature = signature;
        self
    }
    
    /// Set execution mode
    pub fn execution_mode(mut self, mode: ExecutionMode) -> Self {
        self.metadata.execution_mode = mode;
        self
    }
    
    /// Set input type constraints
    pub fn input_types(mut self, types: Vec<TypePattern>) -> Self {
        self.metadata.type_constraints.input_types = types;
        self
    }
    
    /// Set whether function supports collections
    pub fn supports_collections(mut self, supports: bool) -> Self {
        self.metadata.basic.supports_collections = supports;
        self.metadata.type_constraints.supports_collections = supports;
        self
    }
    
    /// Set whether function requires collection input
    pub fn requires_collection(mut self, requires: bool) -> Self {
        self.metadata.basic.requires_collection = requires;
        self.metadata.type_constraints.requires_collection = requires;
        self
    }
    
    /// Set output type
    pub fn output_type(mut self, type_pattern: TypePattern) -> Self {
        self.metadata.type_constraints.output_type = type_pattern.clone();
        self.metadata.basic.output_type = type_pattern.description();
        self
    }
    
    /// Set whether output is always a collection
    pub fn output_is_collection(mut self, is_collection: bool) -> Self {
        self.metadata.basic.output_is_collection = is_collection;
        self.metadata.type_constraints.output_is_collection = is_collection;
        self
    }
    
    /// Set whether function is pure
    pub fn pure(mut self, is_pure: bool) -> Self {
        self.metadata.basic.is_pure = is_pure;
        self.metadata.performance.is_pure = is_pure;
        self.metadata.performance.cacheable = is_pure;
        self
    }
    
    /// Set lambda expression support
    pub fn supports_lambda(mut self, supports: bool) -> Self {
        self.metadata.lambda.supports_lambda_evaluation = supports;
        self
    }
    
    /// Set lambda argument indices
    pub fn lambda_argument_indices(mut self, indices: Vec<usize>) -> Self {
        self.metadata.lambda.lambda_argument_indices = indices;
        self
    }
    
    /// Set lambda description
    pub fn lambda_description(mut self, description: &str) -> Self {
        self.metadata.lambda.lambda_description = Some(description.to_string());
        self
    }
    
    /// Set whether lambda evaluation is required
    pub fn requires_lambda(mut self, requires: bool) -> Self {
        self.metadata.lambda.requires_lambda_evaluation = requires;
        self
    }
    
    /// Set performance complexity
    pub fn complexity(mut self, complexity: PerformanceComplexity) -> Self {
        self.metadata.performance.complexity = complexity;
        self
    }
    
    /// Set memory usage characteristics
    pub fn memory_usage(mut self, usage: MemoryUsage) -> Self {
        self.metadata.performance.memory_usage = usage;
        self
    }
    
    /// Set execution time category
    pub fn execution_time(mut self, time: ExecutionTime) -> Self {
        self.metadata.performance.execution_time = time;
        self
    }
    
    /// Add external dependency
    pub fn external_dependency(mut self, dependency: ExternalDependency) -> Self {
        self.metadata.analyzer.external_dependencies.push(dependency);
        self
    }
    
    /// Add usage pattern
    pub fn usage_pattern(mut self, description: &str, example: &str, context: &str) -> Self {
        self.metadata.analyzer.usage_patterns.push(UsagePattern {
            description: description.to_string(),
            example: example.to_string(),
            context: context.to_string(),
            frequency: UsageFrequency::Common,
        });
        self
    }
    
    /// Add usage pattern with frequency
    pub fn usage_pattern_with_frequency(
        mut self, 
        description: &str, 
        example: &str, 
        context: &str,
        frequency: UsageFrequency
    ) -> Self {
        self.metadata.analyzer.usage_patterns.push(UsagePattern {
            description: description.to_string(),
            example: example.to_string(),
            context: context.to_string(),
            frequency,
        });
        self
    }
    
    /// Add related function
    pub fn related_function(mut self, function_name: &str) -> Self {
        self.metadata.analyzer.related_functions.push(function_name.to_string());
        self
    }
    
    /// Set LSP snippet
    pub fn lsp_snippet(mut self, snippet: &str) -> Self {
        self.metadata.basic.lsp_info.snippet = snippet.to_string();
        self.metadata.lsp.snippet = snippet.to_string();
        self
    }
    
    /// Set completion visibility
    pub fn completion_visibility(mut self, visibility: CompletionVisibility) -> Self {
        self.metadata.basic.lsp_info.completion_visibility = visibility.clone();
        self.metadata.lsp.completion_visibility = visibility;
        self
    }
    
    /// Set sort priority for LSP completion
    pub fn sort_priority(mut self, priority: u32) -> Self {
        self.metadata.basic.lsp_info.sort_priority = priority;
        self.metadata.lsp.sort_priority = priority;
        self
    }
    
    /// Add keywords for search/filtering
    pub fn keywords(mut self, keywords: Vec<&str>) -> Self {
        let keyword_strings: Vec<String> = keywords.iter().map(|s| s.to_string()).collect();
        self.metadata.basic.lsp_info.keywords = keyword_strings.clone();
        self.metadata.lsp.keywords = keyword_strings;
        self
    }
    
    /// Set maturity level
    pub fn maturity_level(mut self, level: MaturityLevel) -> Self {
        self.metadata.analyzer.maturity_level = level;
        self
    }
    
    /// Mark as deprecated with information
    pub fn deprecated(mut self, since: &str, reason: &str, alternatives: Vec<&str>) -> Self {
        self.metadata.analyzer.deprecation = Some(DeprecationInfo {
            since_version: since.to_string(),
            reason: reason.to_string(),
            alternatives: alternatives.iter().map(|s| s.to_string()).collect(),
            removal_version: None,
        });
        self.metadata.analyzer.maturity_level = MaturityLevel::Deprecated;
        self
    }
    
    /// Mark function as having side effects
    pub fn has_side_effects(mut self, has_effects: bool) -> Self {
        self.metadata.analyzer.has_side_effects = has_effects;
        // Side effects usually mean not pure
        if has_effects {
            self.metadata.performance.is_pure = false;
            self.metadata.performance.cacheable = false;
        }
        self
    }
    
    /// Build the metadata
    pub fn build(self) -> EnhancedFunctionMetadata {
        // Ensure lambda support is consistent
        let mut metadata = self.metadata;
        if metadata.lambda.supports_lambda_evaluation && !metadata.lambda.lambda_argument_indices.is_empty() {
            // Auto-enable async mode for lambda functions
            if matches!(metadata.execution_mode, ExecutionMode::Sync) {
                metadata.execution_mode = ExecutionMode::SyncFirst;
            }
        }
        metadata
    }
}

/// Convenience functions for common metadata patterns
impl MetadataBuilder {
    /// Create metadata for a collection function
    pub fn collection_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::Collections)
            .supports_collections(true)
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Fast)
            .memory_usage(MemoryUsage::Minimal)
            .input_types(vec![TypePattern::Any])
            .keywords(vec!["collection", "array", "list"])
    }
    
    /// Create metadata for a lambda function
    pub fn lambda_function(name: &str, category: FunctionCategory) -> Self {
        Self::new(name, category)
            .supports_lambda(true)
            .execution_mode(ExecutionMode::Async)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Moderate)
            .memory_usage(MemoryUsage::Linear)
    }
    
    /// Create metadata for a string function
    pub fn string_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::StringOperations)
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Fast)
            .memory_usage(MemoryUsage::Linear)
            .keywords(vec!["string", "text", "manipulation"])
    }
    
    /// Create metadata for a math function
    pub fn math_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::MathNumbers)
            .input_types(vec![TypePattern::Numeric])
            .output_type(TypePattern::Numeric)
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .complexity(PerformanceComplexity::Constant)
            .execution_time(ExecutionTime::UltraFast)
            .memory_usage(MemoryUsage::Minimal)
            .keywords(vec!["math", "number", "calculation"])
    }
    
    /// Create metadata for an async FHIR function
    pub fn async_fhir_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::FhirSpecific)
            .execution_mode(ExecutionMode::Async)
            .external_dependency(ExternalDependency::ModelProvider)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Moderate)
            .memory_usage(MemoryUsage::Linear)
            .keywords(vec!["fhir", "resource", "async"])
    }
    
    /// Create metadata for a type conversion function
    pub fn type_conversion_function(name: &str, input_type: TypePattern, output_type: TypePattern) -> Self {
        Self::new(name, FunctionCategory::TypeConversion)
            .input_types(vec![input_type])
            .output_type(output_type)
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .complexity(PerformanceComplexity::Constant)
            .execution_time(ExecutionTime::Fast)
            .memory_usage(MemoryUsage::Minimal)
            .keywords(vec!["convert", "cast", "type", "transform"])
    }
    
    /// Create metadata for a boolean function
    pub fn boolean_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::BooleanLogic)
            .output_type(TypePattern::Boolean)
            .execution_mode(ExecutionMode::Sync)
            .pure(true)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Fast)
            .memory_usage(MemoryUsage::Minimal)
            .keywords(vec!["boolean", "logic", "condition"])
    }
    
    /// Create metadata for a filtering function
    pub fn filtering_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::Collections)
            .supports_collections(true)
            .requires_collection(true)
            .output_is_collection(true)
            .execution_mode(ExecutionMode::Sync)
            .complexity(PerformanceComplexity::Linear)
            .execution_time(ExecutionTime::Moderate)
            .memory_usage(MemoryUsage::Linear)
            .keywords(vec!["filter", "where", "select", "collection"])
    }
    
    /// Create metadata for a utility function
    pub fn utility_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::Utilities)
            .execution_mode(ExecutionMode::Sync)
            .complexity(PerformanceComplexity::Constant)
            .execution_time(ExecutionTime::Fast)
            .keywords(vec!["utility", "helper", "tool"])
    }
    
    /// Create metadata for a date/time function
    pub fn datetime_function(name: &str) -> Self {
        Self::new(name, FunctionCategory::DateTime)
            .output_type(TypePattern::DateTime)
            .execution_mode(ExecutionMode::Sync)
            .external_dependency(ExternalDependency::SystemTime)
            .complexity(PerformanceComplexity::Constant)
            .execution_time(ExecutionTime::Fast)
            .keywords(vec!["date", "time", "datetime", "temporal"])
    }
}

/// Helper functions for creating common type patterns
pub mod type_patterns {
    use super::*;
    
    /// String collection type pattern
    pub fn string_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::StringLike))
    }
    
    /// Integer collection type pattern  
    pub fn integer_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::Exact(TypeInfo::Integer)))
    }
    
    /// Numeric collection type pattern
    pub fn numeric_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::Numeric))
    }
    
    /// Boolean collection type pattern
    pub fn boolean_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::Boolean))
    }
    
    /// Resource collection type pattern
    pub fn resource_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::Resource))
    }
    
    /// Any collection type pattern
    pub fn any_collection() -> TypePattern {
        TypePattern::CollectionOf(Box::new(TypePattern::Any))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metadata_builder_basic() {
        let metadata = MetadataBuilder::new("test", FunctionCategory::Collections)
            .display_name("Test Function")
            .description("A test function")
            .example("Patient.test()")
            .pure(true)
            .build();
        
        assert_eq!(metadata.basic.name, "test");
        assert_eq!(metadata.basic.display_name, "Test Function");
        assert_eq!(metadata.basic.description, "A test function");
        assert_eq!(metadata.basic.examples, vec!["Patient.test()"]);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
    }
    
    #[test]
    fn test_collection_function_preset() {
        let metadata = MetadataBuilder::collection_function("count")
            .description("Returns the number of items")
            .example("Patient.name.count()")
            .build();
        
        assert_eq!(metadata.basic.name, "count");
        assert_eq!(metadata.basic.category, FunctionCategory::Collections);
        assert!(metadata.basic.supports_collections);
        assert!(metadata.performance.is_pure);
        assert_eq!(metadata.execution_mode, ExecutionMode::Sync);
        assert_eq!(metadata.performance.complexity, PerformanceComplexity::Linear);
    }
    
    #[test]
    fn test_type_patterns() {
        let metadata = MetadataBuilder::string_function("upper")
            .input_types(vec![TypePattern::StringLike])
            .output_type(TypePattern::StringLike)
            .build();
        
        assert_eq!(metadata.type_constraints.input_types, vec![TypePattern::StringLike]);
        assert_eq!(metadata.type_constraints.output_type, TypePattern::StringLike);
    }
    
    #[test]
    fn test_lsp_metadata() {
        let metadata = MetadataBuilder::new("test", FunctionCategory::Collections)
            .lsp_snippet("test($1)")
            .keywords(vec!["test", "example"])
            .completion_visibility(CompletionVisibility::Always)
            .build();
        
        assert_eq!(metadata.lsp.snippet, "test($1)");
        assert_eq!(metadata.lsp.keywords, vec!["test", "example"]);
        assert_eq!(metadata.lsp.completion_visibility, CompletionVisibility::Always);
    }
}