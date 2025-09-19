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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionMetadata {
    /// The function name (e.g., "count", "where", "select")
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Function signature information
    pub signature: FunctionSignature,
    /// Argument evaluation strategy
    #[serde(default)]
    pub argument_evaluation: ArgumentEvaluationStrategy,
    /// Null propagation strategy
    #[serde(default)]
    pub null_propagation: NullPropagationStrategy,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum EmptyPropagation {
    #[default]
    /// Propagate empty if input collection is empty
    Propagate,
    /// Don't propagate empty (function can work on empty collections)
    NoPropagation,
    /// Custom propagation logic (handled by the function)
    Custom,
}

/// Argument evaluation strategy for function parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArgumentEvaluationStrategy {
    /// Evaluate in current context (default)
    Current,
    /// Evaluate in root context (combine, union, etc.)
    Root,
    /// Evaluate in iteration context with $this/$index
    Iteration,
    /// Lazy evaluation (where, select, etc.)
    Lazy,
}

impl Default for ArgumentEvaluationStrategy {
    fn default() -> Self {
        Self::Current
    }
}

/// Null propagation strategy for function evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NullPropagationStrategy {
    /// No null propagation
    None,
    /// Propagate null if focus is empty/null
    Focus,
    /// Propagate null if any argument is empty/null
    Arguments,
    /// Custom null handling
    Custom,
}

impl Default for NullPropagationStrategy {
    fn default() -> Self {
        Self::Focus
    }
}

/// Function categories for organization
#[derive(Debug, Clone, Serialize, Deserialize, Eq, Hash, PartialEq, Default)]
pub enum FunctionCategory {
    #[default]
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
    /// CDA-specific functions
    CDA,
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

/// Pure function evaluator trait for business logic functions
/// Functions receive pre-evaluated arguments
/// and only implement business logic without context management
#[async_trait]
pub trait PureFunctionEvaluator: Send + Sync {
    /// Evaluate the function with pre-evaluated arguments
    /// - input: The input collection that the function operates on
    /// - args: Pre-evaluated function arguments (each Vec<FhirPathValue> is one argument)
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
    ) -> Result<EvaluationResult>;

    /// Get metadata for this function
    fn metadata(&self) -> &FunctionMetadata;
}

/// Provider-dependent pure function evaluator trait for functions that need providers
/// but are otherwise simple business logic (terminology functions, type functions, etc.)
#[async_trait]
pub trait ProviderPureFunctionEvaluator: Send + Sync {
    /// Evaluate the function with pre-evaluated arguments and provider access
    /// - input: The input collection that the function operates on
    /// - args: Pre-evaluated function arguments (each Vec<FhirPathValue> is one argument)
    /// - context: Evaluation context providing access to terminology/model/trace providers
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        args: Vec<Vec<FhirPathValue>>,
        context: &crate::evaluator::EvaluationContext,
    ) -> Result<EvaluationResult>;

    /// Get metadata for this function
    fn metadata(&self) -> &FunctionMetadata;
}

/// Lazy function evaluator trait for complex functions that need expression control
/// This is for functions that need to control their own argument evaluation
/// (like where, select, aggregate, etc.)
#[async_trait]
pub trait LazyFunctionEvaluator: Send + Sync {
    /// Evaluate the function with control over argument evaluation
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
}

/// Enum to wrap different function evaluator types
#[derive(Clone)]
pub enum FunctionEvaluatorWrapper {
    /// Standard function evaluator (current interface)
    Standard(Arc<dyn FunctionEvaluator>),
    /// Pure function evaluator (business logic only)
    Pure(Arc<dyn PureFunctionEvaluator>),
    /// Provider-dependent pure function evaluator (needs providers)
    ProviderPure(Arc<dyn ProviderPureFunctionEvaluator>),
    /// Lazy function evaluator (complex expression handling)
    Lazy(Arc<dyn LazyFunctionEvaluator>),
}

impl FunctionEvaluatorWrapper {
    /// Get metadata from any function evaluator type
    pub fn metadata(&self) -> &FunctionMetadata {
        match self {
            FunctionEvaluatorWrapper::Standard(evaluator) => evaluator.metadata(),
            FunctionEvaluatorWrapper::Pure(evaluator) => evaluator.metadata(),
            FunctionEvaluatorWrapper::ProviderPure(evaluator) => evaluator.metadata(),
            FunctionEvaluatorWrapper::Lazy(evaluator) => evaluator.metadata(),
        }
    }

    /// Check if the function can handle the given input type and argument count
    pub fn can_handle(&self, input_type: &str, arg_count: usize) -> bool {
        match self {
            FunctionEvaluatorWrapper::Standard(evaluator) => {
                evaluator.can_handle(input_type, arg_count)
            }
            FunctionEvaluatorWrapper::Pure(evaluator) => {
                // Use same logic as standard functions
                let metadata = evaluator.metadata();
                let signature = &metadata.signature;

                // Check parameter count
                let param_count_ok = arg_count >= signature.min_params
                    && signature.max_params.map_or(true, |max| arg_count <= max);

                if !param_count_ok {
                    return false;
                }

                // Check input type compatibility
                signature.polymorphic
                    || signature.input_type == input_type
                    || signature.input_type == "Any"
            }
            FunctionEvaluatorWrapper::ProviderPure(evaluator) => {
                // Use same logic as pure functions
                let metadata = evaluator.metadata();
                let signature = &metadata.signature;

                // Check parameter count
                let param_count_ok = arg_count >= signature.min_params
                    && signature.max_params.map_or(true, |max| arg_count <= max);

                if !param_count_ok {
                    return false;
                }

                // Check input type compatibility
                signature.polymorphic
                    || signature.input_type == input_type
                    || signature.input_type == "Any"
            }
            FunctionEvaluatorWrapper::Lazy(evaluator) => {
                // Use same logic as standard functions
                let metadata = evaluator.metadata();
                let signature = &metadata.signature;

                // Check parameter count
                let param_count_ok = arg_count >= signature.min_params
                    && signature.max_params.map_or(true, |max| arg_count <= max);

                if !param_count_ok {
                    return false;
                }

                // Check input type compatibility
                signature.polymorphic
                    || signature.input_type == input_type
                    || signature.input_type == "Any"
            }
        }
    }
}

/// Registry for function evaluators
pub struct FunctionRegistry {
    /// Function evaluators by name (all interface types)
    functions: HashMap<String, FunctionEvaluatorWrapper>,
    /// Standard function evaluators by name (for backward compatibility)
    standard_functions: HashMap<String, Arc<dyn FunctionEvaluator>>,
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
            standard_functions: HashMap::new(),
            metadata_cache: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a pure function evaluator (new simplified interface)
    pub fn register_pure_function(&mut self, evaluator: Arc<dyn PureFunctionEvaluator>) {
        let metadata = evaluator.metadata().clone();
        let name = metadata.name.clone();
        let category = metadata.category.clone();

        let wrapper = FunctionEvaluatorWrapper::Pure(evaluator);
        self.functions.insert(name.clone(), wrapper);
        self.metadata_cache.insert(name.clone(), metadata);

        // Update categories
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Register a provider-dependent pure function evaluator (needs providers)
    pub fn register_provider_pure_function(
        &mut self,
        evaluator: Arc<dyn ProviderPureFunctionEvaluator>,
    ) {
        let metadata = evaluator.metadata().clone();
        let name = metadata.name.clone();
        let category = metadata.category.clone();

        let wrapper = FunctionEvaluatorWrapper::ProviderPure(evaluator);
        self.functions.insert(name.clone(), wrapper);
        self.metadata_cache.insert(name.clone(), metadata);

        // Update categories
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Register a lazy function evaluator (new lazy interface)
    pub fn register_lazy_function(&mut self, evaluator: Arc<dyn LazyFunctionEvaluator>) {
        let metadata = evaluator.metadata().clone();
        let name = metadata.name.clone();
        let category = metadata.category.clone();

        let wrapper = FunctionEvaluatorWrapper::Lazy(evaluator);
        self.functions.insert(name.clone(), wrapper);
        self.metadata_cache.insert(name.clone(), metadata);

        // Update categories
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(name);
    }

    /// Get function evaluator by name (standard interface for backward compatibility)
    pub fn get_function(&self, name: &str) -> Option<&Arc<dyn FunctionEvaluator>> {
        self.standard_functions.get(name)
    }

    /// Get function evaluator wrapper by name (new interface)
    pub fn get_function_wrapper(&self, name: &str) -> Option<&FunctionEvaluatorWrapper> {
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
            .register_pure_function(EmptyFunctionEvaluator::create());
        self.registry
            .register_lazy_function(ExistsFunctionEvaluator::create());
        self.registry
            .register_pure_function(HasValueFunctionEvaluator::create());
        self.registry
            .register_pure_function(CountFunctionEvaluator::create());
        self.registry
            .register_lazy_function(AllFunctionEvaluator::create());
        self.registry
            .register_pure_function(AllTrueFunctionEvaluator::create());
        self.registry
            .register_pure_function(AnyTrueFunctionEvaluator::create());

        self
    }

    /// Add default filtering and projection functions (where, select, repeat, etc.)
    pub fn with_filtering_projection_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register filtering and projection functions
        self.registry
            .register_pure_function(ExcludeFunctionEvaluator::create());
        self.registry
            .register_lazy_function(WhereFunctionEvaluator::create());
        self.registry
            .register_lazy_function(SelectFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(OfTypeFunctionEvaluator::create());
        self.registry
            .register_lazy_function(RepeatFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(ResolveFunctionEvaluator::create());
        self.registry
            .register_pure_function(ExtensionFunctionEvaluator::create());

        self
    }

    /// Add default subsetting functions (first, last, tail, take, skip, etc.)
    pub fn with_subsetting_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register subsetting functions
        self.registry
            .register_pure_function(FirstFunctionEvaluator::create());
        self.registry
            .register_pure_function(LastFunctionEvaluator::create());
        self.registry
            .register_pure_function(SingleFunctionEvaluator::create());
        self.registry
            .register_pure_function(TailFunctionEvaluator::create());
        self.registry
            .register_pure_function(SkipFunctionEvaluator::create());
        self.registry
            .register_lazy_function(TakeFunctionEvaluator::create());
        self.registry
            .register_pure_function(DistinctFunctionEvaluator::create());
        self.registry
            .register_lazy_function(SortFunctionEvaluator::create());
        self.registry
            .register_pure_function(IntersectFunctionEvaluator::create());
        self.registry
            .register_pure_function(SubsetOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(SupersetOfFunctionEvaluator::create());

        self
    }

    /// Add default combining functions (union, combine)
    pub fn with_combining_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register combining functions
        self.registry
            .register_lazy_function(CoalesceFunctionEvaluator::create());
        self.registry.register_pure_function(
            crate::evaluator::functions::combine_function::CombineFunctionEvaluator::create(),
        );
        self.registry
            .register_pure_function(UnionFunctionEvaluator::create());

        self
    }

    /// Add default conversion functions (toString, toInteger, etc.)
    pub fn with_conversion_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register conversion functions
        self.registry
            .register_pure_function(ToStringFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToIntegerFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToDecimalFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToBooleanFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToDateFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToDateTimeFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToTimeFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToQuantityFunctionEvaluator::create());

        // Register conversion test functions
        self.registry
            .register_pure_function(ConvertsToStringFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToIntegerFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToDecimalFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToBooleanFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToDateFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToDateTimeFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToTimeFunctionEvaluator::create());
        self.registry
            .register_pure_function(ConvertsToQuantityFunctionEvaluator::create());

        self
    }

    /// Add default string manipulation functions
    pub fn with_string_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register string manipulation functions
        self.registry
            .register_pure_function(EncodeFunctionEvaluator::create());
        self.registry
            .register_pure_function(DecodeFunctionEvaluator::create());
        self.registry
            .register_pure_function(EscapeFunctionEvaluator::create());
        self.registry
            .register_pure_function(UnescapeFunctionEvaluator::create());
        self.registry
            .register_pure_function(TrimFunctionEvaluator::create());
        self.registry
            .register_pure_function(SplitFunctionEvaluator::create());
        self.registry
            .register_pure_function(JoinFunctionEvaluator::create());
        self.registry
            .register_pure_function(ReplaceFunctionEvaluator::create());
        self.registry
            .register_pure_function(ReplaceMatchesFunctionEvaluator::create());
        self.registry
            .register_pure_function(ToCharsFunctionEvaluator::create());

        // Advanced string functions
        self.registry
            .register_pure_function(LengthFunctionEvaluator::create());
        self.registry
            .register_pure_function(SubstringFunctionEvaluator::create());
        self.registry
            .register_pure_function(ContainsFunctionEvaluator::create());
        self.registry
            .register_pure_function(StartsWithFunctionEvaluator::create());
        self.registry
            .register_pure_function(EndsWithFunctionEvaluator::create());
        self.registry
            .register_pure_function(UpperFunctionEvaluator::create());
        self.registry
            .register_pure_function(LowerFunctionEvaluator::create());

        self
    }

    /// Add default math functions
    pub fn with_math_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register math functions
        self.registry
            .register_pure_function(AbsFunctionEvaluator::create());
        self.registry
            .register_pure_function(CeilingFunctionEvaluator::create());
        self.registry
            .register_pure_function(FloorFunctionEvaluator::create());
        self.registry
            .register_pure_function(ExpFunctionEvaluator::create());
        self.registry
            .register_pure_function(LnFunctionEvaluator::create());
        self.registry
            .register_pure_function(LogFunctionEvaluator::create());
        self.registry
            .register_pure_function(SqrtFunctionEvaluator::create());
        self.registry
            .register_pure_function(PowerFunctionEvaluator::create());
        self.registry
            .register_pure_function(RoundFunctionEvaluator::create());
        self.registry
            .register_pure_function(TruncateFunctionEvaluator::create());

        self
    }

    /// Add default tree navigation functions
    pub fn with_tree_navigation_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register tree navigation functions
        self.registry
            .register_pure_function(ChildrenFunctionEvaluator::create());
        self.registry
            .register_pure_function(DescendantsFunctionEvaluator::create());
        self.registry
            .register_lazy_function(RepeatAllFunctionEvaluator::create());

        self
    }

    /// Add default utility functions
    pub fn with_utility_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register utility functions
        self.registry
            .register_lazy_function(DefineVariableFunctionEvaluator::create());
        self.registry
            .register_pure_function(NowFunctionEvaluator::create());
        self.registry
            .register_pure_function(TodayFunctionEvaluator::create());
        self.registry
            .register_lazy_function(TraceFunctionEvaluator::create());

        // Register temporal extraction functions
        self.registry
            .register_pure_function(DayOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(MonthOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(YearOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(HourOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(MinuteOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(SecondOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(TimezoneOffsetOfFunctionEvaluator::create());

        // Register logic functions
        self.registry
            .register_pure_function(IsDistinctFunctionEvaluator::create());
        self.registry
            .register_pure_function(NotFunctionEvaluator::create());
        self.registry
            .register_pure_function(ComparableFunctionEvaluator::create());
        self.registry
            .register_lazy_function(IsFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(AsFunctionEvaluator::create());
        self.registry
            .register_pure_function(TypeFunctionEvaluator::create());

        // Enhanced functions (FHIRPath 3.0.0-ballot)
        self.registry
            .register_pure_function(IndexOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(LastIndexOfFunctionEvaluator::create());
        self.registry
            .register_pure_function(MatchesFunctionEvaluator::create());
        self.registry
            .register_pure_function(MatchesFullFunctionEvaluator::create());
        self.registry
            .register_pure_function(PrecisionFunctionEvaluator::create());
        self.registry
            .register_pure_function(LowBoundaryFunctionEvaluator::create());
        self.registry
            .register_pure_function(HighBoundaryFunctionEvaluator::create());

        // Advanced utility functions (Phase 7)
        self.registry
            .register_lazy_function(IifFunctionEvaluator::create());

        self
    }

    /// Add terminology functions (requires terminology provider)
    pub fn with_terminology_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register terminology functions (FHIRPath 3.0.0-ballot)
        self.registry
            .register_provider_pure_function(SimpleExpandFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(ExpandFunctionEvaluator::create());
        self.registry
            .register_lazy_function(LookupFunctionEvaluator::create());
        self.registry
            .register_lazy_function(ValidateVSFunctionEvaluator::create());
        self.registry
            .register_lazy_function(ValidateCSFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(SubsumesFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(SubsumedByFunctionEvaluator::create());
        self.registry
            .register_lazy_function(TranslateFunctionEvaluator::create());
        self.registry
            .register_lazy_function(MemberOfFunctionEvaluator::create());

        self
    }

    /// Add default type functions
    pub fn with_type_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register type operations
        self.registry
            .register_lazy_function(IsFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(AsFunctionEvaluator::create());
        self.registry
            .register_pure_function(TypeFunctionEvaluator::create());
        self.registry
            .register_provider_pure_function(ConformsToFunctionEvaluator::create());

        self
    }

    /// Add aggregate functions
    pub fn with_aggregate_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register aggregate functions
        self.registry
            .register_lazy_function(AggregateFunctionEvaluator::create());

        self
    }

    /// Add CDA-specific functions
    pub fn with_cda_functions(mut self) -> Self {
        use crate::evaluator::functions::*;

        // Register CDA functions
        self.registry
            .register_pure_function(HasTemplateIdOfFunctionEvaluator::create());

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
pub fn create_function_registry() -> FunctionRegistry {
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
        .with_cda_functions()
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
