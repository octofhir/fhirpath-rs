//! Function registry for FHIRPath function implementations
//!
//! This module implements the function registry with metadata, signatures, and parameter information.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Metadata for a function describing its behavior and signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// The function name (e.g., "count", "where", "select")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Function signature information
    pub signature: FunctionSignature,
    /// Whether this function propagates empty values
    pub empty_propagation: EmptyPropagation,
    /// Whether this function is deterministic
    pub deterministic: bool,
    /// Function category for grouping
    pub category: FunctionCategory,
    /// Whether the function requires terminology provider
    pub requires_terminology: bool,
    /// Whether the function requires model provider
    pub requires_model: bool,
}

/// Function signature with parameter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Input collection type (what the function operates on)
    pub input_type: String,
    /// Function parameters
    pub parameters: Vec<FunctionParameter>,
    /// Return type
    pub return_type: String,
    /// Whether the signature is polymorphic
    pub polymorphic: bool,
    /// Minimum number of parameters required
    pub min_params: usize,
    /// Maximum number of parameters allowed (None = unlimited)
    pub max_params: Option<usize>,
}

/// Function parameter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (or types for polymorphic parameters)
    pub parameter_type: Vec<String>,
    /// Whether the parameter is optional
    pub optional: bool,
    /// Whether the parameter is an expression (evaluated lazily)
    pub is_expression: bool,
    /// Parameter description
    pub description: String,
    /// Default value if parameter is optional
    pub default_value: Option<String>,
}

/// Empty value propagation behavior for functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmptyPropagation {
    /// Propagate empty if input collection is empty
    Propagate,
    /// Don't propagate empty (function can work on empty collections)
    NoPropagation,
    /// Custom propagation logic (handled by the function)
    Custom,
}

/// Function categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum FunctionCategory {
    /// Existence functions (empty, exists, all, etc.)
    Existence,
    /// Filtering and projection functions (where, select, repeat, etc.)
    FilteringProjection,
    /// Subsetting functions (first, last, tail, take, skip, etc.)
    Subsetting,
    /// Combining functions (union, combine)
    Combining,
    /// Conversion functions (toString, toInteger, etc.)
    Conversion,
    /// Logic functions (not, comparable)
    Logic,
    /// String manipulation functions (indexOf, substring, etc.)
    StringManipulation,
    /// Math functions (abs, ceiling, floor, etc.)
    Math,
    /// Tree navigation functions (children, descendants)
    TreeNavigation,
    /// Utility functions (trace, now, today, etc.)
    Utility,
    /// Terminology functions (memberOf, subsumes, etc.)
    Terminology,
    /// Type functions (is, as, ofType)
    Types,
    /// Aggregate functions (aggregate)
    Aggregate,
}

/// Trait for evaluating functions
#[async_trait]
pub trait FunctionEvaluator: Send + Sync {
    /// Evaluate the function
    /// - input: The input collection that the function operates on
    /// - context: Evaluation context with variables and providers
    /// - args: Function argument expressions (not yet evaluated)
    /// - evaluator: Async evaluator for argument expressions
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult>;

    /// Get metadata for this function
    fn metadata(&self) -> &FunctionMetadata;

    /// Check if the function can handle the given input type and argument count
    fn can_handle(&self, input_type: &str, arg_count: usize) -> bool {
        let metadata = self.metadata();
        let signature = &metadata.signature;

        // Check parameter count
        let param_count_ok = arg_count >= signature.min_params
            && signature.max_params.map_or(true, |max| arg_count <= max);

        if !param_count_ok {
            return false;
        }

        // Check input type compatibility
        signature.polymorphic || signature.input_type == input_type || signature.input_type == "Any"
    }

    /// Validate argument types against the function signature
    fn validate_arguments(&self, args: &[String]) -> Result<()> {
        let metadata = self.metadata();
        let signature = &metadata.signature;

        // Check parameter count
        if args.len() < signature.min_params {
            return Err(crate::core::FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!(
                    "Function '{}' requires at least {} arguments, got {}",
                    metadata.name,
                    signature.min_params,
                    args.len()
                ),
            ));
        }

        if let Some(max_params) = signature.max_params {
            if args.len() > max_params {
                return Err(crate::core::FhirPathError::evaluation_error(
                    crate::core::error_code::FP0053,
                    format!(
                        "Function '{}' accepts at most {} arguments, got {}",
                        metadata.name,
                        max_params,
                        args.len()
                    ),
                ));
            }
        }

        // TODO: Add type checking for arguments when type system is more mature

        Ok(())
    }
}

/// Registry for function evaluators
pub struct FunctionRegistry {
    /// Function evaluators by name
    functions: HashMap<String, Arc<dyn FunctionEvaluator>>,
    /// Metadata cache for introspection
    metadata_cache: HashMap<String, FunctionMetadata>,
    /// Functions grouped by category
    categories: HashMap<FunctionCategory, Vec<String>>,
}

impl FunctionRegistry {
    /// Create a new empty function registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            metadata_cache: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a function evaluator
    pub fn register_function(&mut self, evaluator: Arc<dyn FunctionEvaluator>) {
        let metadata = evaluator.metadata().clone();
        let name = metadata.name.clone();
        let category = metadata.category.clone();

        // Add to main registry
        self.functions.insert(name.clone(), evaluator);

        // Cache metadata
        self.metadata_cache.insert(name.clone(), metadata);

        // Add to category index
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Get function evaluator by name
    pub fn get_function(&self, name: &str) -> Option<&Arc<dyn FunctionEvaluator>> {
        self.functions.get(name)
    }

    /// Get function metadata by name
    pub fn get_metadata(&self, name: &str) -> Option<&FunctionMetadata> {
        self.metadata_cache.get(name)
    }

    /// Get all registered function names
    pub fn list_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// Get functions by category
    pub fn get_functions_by_category(&self, category: &FunctionCategory) -> Vec<&String> {
        self.categories
            .get(category)
            .map(|names| names.iter().collect())
            .unwrap_or_default()
    }

    /// Get all function metadata for introspection
    pub fn all_metadata(&self) -> &HashMap<String, FunctionMetadata> {
        &self.metadata_cache
    }

    /// Find functions that can handle the given input type and argument count
    pub fn find_compatible_functions(&self, input_type: &str, arg_count: usize) -> Vec<&String> {
        self.functions
            .iter()
            .filter(|(_, evaluator)| evaluator.can_handle(input_type, arg_count))
            .map(|(name, _)| name)
            .collect()
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get function categories
    pub fn get_categories(&self) -> Vec<&FunctionCategory> {
        self.categories.keys().collect()
    }

    /// Search functions by name pattern
    pub fn search_functions(&self, pattern: &str) -> Vec<&String> {
        self.functions
            .keys()
            .filter(|name| name.contains(pattern))
            .collect()
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating function registries with default functions
pub struct FunctionRegistryBuilder {
    registry: FunctionRegistry,
}

impl FunctionRegistryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry: FunctionRegistry::new(),
        }
    }

    /// Add default existence functions (empty, exists, all, count, etc.)
    pub fn with_existence_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register existence functions
        self.registry
            .register_function(EmptyFunctionEvaluator::create());
        self.registry
            .register_function(ExistsFunctionEvaluator::create());
        self.registry
            .register_function(HasValueFunctionEvaluator::create());
        self.registry
            .register_function(CountFunctionEvaluator::create());
        self.registry
            .register_function(AllFunctionEvaluator::create());
        self.registry
            .register_function(AllTrueFunctionEvaluator::create());
        self.registry
            .register_function(AnyTrueFunctionEvaluator::create());

        self
    }

    /// Add default filtering and projection functions (where, select, repeat, etc.)
    pub fn with_filtering_projection_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register filtering and projection functions
        self.registry
            .register_function(ExcludeFunctionEvaluator::create());
        self.registry
            .register_function(WhereFunctionEvaluator::create());
        self.registry
            .register_function(SelectFunctionEvaluator::create());
        self.registry
            .register_function(OfTypeFunctionEvaluator::create());
        self.registry
            .register_function(RepeatFunctionEvaluator::create());
        self.registry
            .register_function(ResolveFunctionEvaluator::create());

        self
    }

    /// Add default subsetting functions (first, last, tail, take, skip, etc.)
    pub fn with_subsetting_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register subsetting functions
        self.registry
            .register_function(FirstFunctionEvaluator::create());
        self.registry
            .register_function(LastFunctionEvaluator::create());
        self.registry
            .register_function(SingleFunctionEvaluator::create());
        self.registry
            .register_function(TailFunctionEvaluator::create());
        self.registry
            .register_function(SkipFunctionEvaluator::create());
        self.registry
            .register_function(TakeFunctionEvaluator::create());
        self.registry
            .register_function(DistinctFunctionEvaluator::create());
        self.registry
            .register_function(SortFunctionEvaluator::create());
        self.registry
            .register_function(IntersectFunctionEvaluator::create());
        self.registry
            .register_function(SubsetOfFunctionEvaluator::create());
        self.registry
            .register_function(SupersetOfFunctionEvaluator::create());

        self
    }

    /// Add default combining functions (union, combine)
    pub fn with_combining_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register combining functions
        self.registry
            .register_function(CoalesceFunctionEvaluator::create());
        self.registry
            .register_function(CombineFunctionEvaluator::create());
        self.registry
            .register_function(UnionFunctionEvaluator::create());

        self
    }

    /// Add default conversion functions (toString, toInteger, etc.)
    pub fn with_conversion_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register conversion functions
        self.registry
            .register_function(ToStringFunctionEvaluator::create());
        self.registry
            .register_function(ToIntegerFunctionEvaluator::create());
        self.registry
            .register_function(ToDecimalFunctionEvaluator::create());
        self.registry
            .register_function(ToBooleanFunctionEvaluator::create());
        self.registry
            .register_function(ToDateFunctionEvaluator::create());
        self.registry
            .register_function(ToDateTimeFunctionEvaluator::create());
        self.registry
            .register_function(ToTimeFunctionEvaluator::create());
        self.registry
            .register_function(ToQuantityFunctionEvaluator::create());

        // Register conversion test functions
        self.registry
            .register_function(ConvertsToStringFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToIntegerFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToDecimalFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToBooleanFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToDateFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToDateTimeFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToTimeFunctionEvaluator::create());
        self.registry
            .register_function(ConvertsToQuantityFunctionEvaluator::create());

        self
    }

    /// Add default string manipulation functions
    pub fn with_string_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register string manipulation functions
        self.registry
            .register_function(EncodeFunctionEvaluator::create());
        self.registry
            .register_function(DecodeFunctionEvaluator::create());
        self.registry
            .register_function(EscapeFunctionEvaluator::create());
        self.registry
            .register_function(UnescapeFunctionEvaluator::create());
        self.registry
            .register_function(TrimFunctionEvaluator::create());
        self.registry
            .register_function(SplitFunctionEvaluator::create());
        self.registry
            .register_function(JoinFunctionEvaluator::create());
        self.registry
            .register_function(ReplaceFunctionEvaluator::create());
        self.registry
            .register_function(ReplaceMatchesFunctionEvaluator::create());
        self.registry
            .register_function(ToCharsFunctionEvaluator::create());

        // Advanced string functions
        self.registry
            .register_function(LengthFunctionEvaluator::create());
        self.registry
            .register_function(SubstringFunctionEvaluator::create());
        self.registry
            .register_function(ContainsFunctionEvaluator::create());
        self.registry
            .register_function(StartsWithFunctionEvaluator::create());
        self.registry
            .register_function(EndsWithFunctionEvaluator::create());
        self.registry
            .register_function(UpperFunctionEvaluator::create());
        self.registry
            .register_function(LowerFunctionEvaluator::create());

        self
    }

    /// Add default math functions
    pub fn with_math_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register math functions
        self.registry
            .register_function(AbsFunctionEvaluator::create());
        self.registry
            .register_function(CeilingFunctionEvaluator::create());
        self.registry
            .register_function(FloorFunctionEvaluator::create());
        self.registry
            .register_function(ExpFunctionEvaluator::create());
        self.registry
            .register_function(LnFunctionEvaluator::create());
        self.registry
            .register_function(LogFunctionEvaluator::create());
        self.registry
            .register_function(SqrtFunctionEvaluator::create());
        self.registry
            .register_function(PowerFunctionEvaluator::create());
        self.registry
            .register_function(RoundFunctionEvaluator::create());
        self.registry
            .register_function(TruncateFunctionEvaluator::create());

        self
    }

    /// Add default tree navigation functions
    pub fn with_tree_navigation_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register tree navigation functions
        self.registry
            .register_function(ChildrenFunctionEvaluator::create());
        self.registry
            .register_function(DescendantsFunctionEvaluator::create());
        self.registry
            .register_function(RepeatAllFunctionEvaluator::create());

        self
    }

    /// Add default utility functions
    pub fn with_utility_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register utility functions
        self.registry
            .register_function(DefineVariableFunctionEvaluator::create());
        self.registry
            .register_function(NowFunctionEvaluator::create());
        self.registry
            .register_function(TodayFunctionEvaluator::create());
        self.registry
            .register_function(TraceFunctionEvaluator::create());

        // Register temporal extraction functions
        self.registry
            .register_function(DayOfFunctionEvaluator::create());
        self.registry
            .register_function(MonthOfFunctionEvaluator::create());
        self.registry
            .register_function(YearOfFunctionEvaluator::create());
        self.registry
            .register_function(HourOfFunctionEvaluator::create());
        self.registry
            .register_function(MinuteOfFunctionEvaluator::create());
        self.registry
            .register_function(SecondOfFunctionEvaluator::create());
        self.registry
            .register_function(TimezoneOffsetOfFunctionEvaluator::create());

        // Register logic functions
        self.registry
            .register_function(IsDistinctFunctionEvaluator::create());
        self.registry
            .register_function(NotFunctionEvaluator::create());
        self.registry
            .register_function(ComparableFunctionEvaluator::create());
        self.registry
            .register_function(IsFunctionEvaluator::create());
        self.registry
            .register_function(TypeFunctionEvaluator::create());

        // Enhanced functions (FHIRPath 3.0.0-ballot)
        self.registry
            .register_function(IndexOfFunctionEvaluator::create());
        self.registry
            .register_function(LastIndexOfFunctionEvaluator::create());
        self.registry
            .register_function(MatchesFunctionEvaluator::create());
        self.registry
            .register_function(MatchesFullFunctionEvaluator::create());
        self.registry
            .register_function(PrecisionFunctionEvaluator::create());
        self.registry
            .register_function(LowBoundaryFunctionEvaluator::create());
        self.registry
            .register_function(HighBoundaryFunctionEvaluator::create());

        // Advanced utility functions (Phase 7)
        self.registry
            .register_function(IifFunctionEvaluator::create());

        self
    }

    /// Add terminology functions (requires terminology provider)
    pub fn with_terminology_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register terminology functions (FHIRPath 3.0.0-ballot)
        self.registry
            .register_function(SimpleExpandFunctionEvaluator::create());
        self.registry
            .register_function(ExpandFunctionEvaluator::create());
        self.registry
            .register_function(LookupFunctionEvaluator::create());
        self.registry
            .register_function(ValidateVSFunctionEvaluator::create());
        self.registry
            .register_function(ValidateCSFunctionEvaluator::create());
        self.registry
            .register_function(SubsumesFunctionEvaluator::create());
        self.registry
            .register_function(SubsumedByFunctionEvaluator::create());
        self.registry
            .register_function(TranslateFunctionEvaluator::create());
        self.registry
            .register_function(MemberOfFunctionEvaluator::create());

        self
    }

    /// Add default type functions
    pub fn with_type_functions(mut self) -> Self {
        // TODO: Implement default type functions in Phase 3
        self
    }

    /// Add aggregate functions
    pub fn with_aggregate_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register aggregate functions
        self.registry
            .register_function(AggregateFunctionEvaluator::create());

        self
    }

    /// Add advanced functions from Phase 7 (convenience method)
    pub fn with_advanced_functions(mut self) -> Self {
        // TODO: Implement aggregate functions in Phase 7
        self
    }

    /// Register a custom function
    pub fn register_function(mut self, evaluator: Arc<dyn FunctionEvaluator>) -> Self {
        self.registry.register_function(evaluator);
        self
    }

    /// Build the function registry
    pub fn build(self) -> FunctionRegistry {
        self.registry
    }
}

impl Default for FunctionRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a comprehensive function registry with all FHIRPath functions
pub fn create_comprehensive_function_registry() -> FunctionRegistry {
    FunctionRegistryBuilder::new()
        .with_existence_functions()
        .with_filtering_projection_functions()
        .with_subsetting_functions()
        .with_combining_functions()
        .with_conversion_functions()
        .with_string_functions()
        .with_math_functions()
        .with_tree_navigation_functions()
        .with_utility_functions()
        .with_terminology_functions()
        .with_type_functions()
        .with_aggregate_functions()
        .build()
}

/// Create a basic function registry for Phase 1 (minimal functions for testing)
pub fn create_basic_function_registry() -> FunctionRegistry {
    FunctionRegistryBuilder::new()
        .with_existence_functions()
        .with_subsetting_functions()
        .build()
}

/// Create a standard function registry with core functions (includes where, select, first, etc.)
pub fn create_standard_function_registry() -> FunctionRegistry {
    FunctionRegistryBuilder::new()
        .with_filtering_projection_functions()
        .with_subsetting_functions()
        .with_existence_functions()
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_registry_creation() {
        let registry = FunctionRegistry::new();
        assert!(registry.functions.is_empty());
        assert!(registry.metadata_cache.is_empty());
        assert!(registry.categories.is_empty());
    }

    #[test]
    fn test_function_registry_builder() {
        let registry = FunctionRegistryBuilder::new()
            .with_existence_functions()
            .with_string_functions()
            .build();

        // Test that registry was created
        // TODO: Add specific tests when functions are implemented
    }

    #[test]
    fn test_function_signature_validation() {
        // TODO: Add tests for function signature validation
    }
}
