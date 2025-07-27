//! Function registry and built-in functions

use crate::signature::{FunctionSignature, ParameterInfo};
use crate::functions::*;

// Re-export commonly used function types for external crates
// Note: Lambda evaluation is not yet fully implemented
// pub use crate::functions::boolean::{AllFunction, AnyFunction};
// pub use crate::functions::collection::ExistsFunction;
use fhirpath_model::{FhirPathValue, TypeInfo, FhirResource};
use rustc_hash::FxHashMap;
use rust_decimal::prelude::{ToPrimitive, FromPrimitive};
use std::sync::Arc;
use thiserror::Error;

// For expression evaluation in lambda functions
use fhirpath_ast::ExpressionNode;

/// Result type for function operations
pub type FunctionResult<T> = Result<T, FunctionError>;

/// Function evaluation errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum FunctionError {
    /// Invalid number of arguments
    #[error("Function '{name}' expects {min}-{} arguments, got {actual}", max.map_or("∞".to_string(), |n| n.to_string()))]
    InvalidArity {
        /// Function name
        name: String,
        /// Minimum arguments
        min: usize,
        /// Maximum arguments (None for unlimited)
        max: Option<usize>,
        /// Actual arguments provided
        actual: usize,
    },

    /// Invalid argument type
    #[error("Function '{name}' argument {index} expects {expected}, got {actual}")]
    InvalidArgumentType {
        /// Function name
        name: String,
        /// Argument index
        index: usize,
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },

    /// Runtime evaluation error
    #[error("Function '{name}' evaluation error: {message}")]
    EvaluationError {
        /// Function name
        name: String,
        /// Error message
        message: String,
    },
}

/// Lambda evaluator type - takes an expression and context and returns a result
pub type LambdaEvaluator<'a> = dyn Fn(&ExpressionNode, &FhirPathValue) -> Result<FhirPathValue, FunctionError> + 'a;

/// Context for function evaluation
#[derive(Debug, Clone)]
pub struct EvaluationContext {
    /// Current input value
    pub input: FhirPathValue,
    /// Root input value
    pub root: FhirPathValue,
    /// Variables in scope
    pub variables: FxHashMap<String, FhirPathValue>,
}

/// Extended context for lambda-supporting functions
pub struct LambdaEvaluationContext<'a> {
    /// Basic evaluation context
    pub context: &'a EvaluationContext,
    /// Lambda expression evaluator
    pub evaluator: &'a LambdaEvaluator<'a>,
}

impl EvaluationContext {
    /// Create a new evaluation context
    pub fn new(input: FhirPathValue) -> Self {
        Self {
            root: input.clone(),
            input,
            variables: FxHashMap::default(),
        }
    }
}

/// Trait for implementing FHIRPath functions
pub trait FhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with given arguments
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Validate arguments before evaluation (both arity and types)
    fn validate_args(&self, args: &[FhirPathValue]) -> FunctionResult<()> {
        let sig = self.signature();
        let arg_count = args.len();

        // Check arity
        if arg_count < sig.min_arity {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: sig.min_arity,
                max: sig.max_arity,
                actual: arg_count,
            });
        }

        if let Some(max) = sig.max_arity {
            if arg_count > max {
                return Err(FunctionError::InvalidArity {
                    name: self.name().to_string(),
                    min: sig.min_arity,
                    max: sig.max_arity,
                    actual: arg_count,
                });
            }
        }

        // Check argument types
        for (i, arg) in args.iter().enumerate() {
            if let Some(param) = sig.parameters.get(i) {
                let arg_type = arg.to_type_info();
                if !param.param_type.is_compatible_with(&arg_type) {
                    return Err(FunctionError::InvalidArgumentType {
                        name: self.name().to_string(),
                        index: i,
                        expected: param.param_type.to_string(),
                        actual: arg_type.to_string(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Trait for functions that need to evaluate lambda expressions
pub trait LambdaFunction: FhirPathFunction {
    /// Evaluate function with lambda expressions
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
}

/// Registry for FHIRPath functions
#[derive(Clone)]
pub struct FunctionRegistry {
    functions: FxHashMap<String, Arc<dyn FhirPathFunction>>,
    signatures: FxHashMap<String, Vec<FunctionSignature>>,
}

impl FunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            functions: FxHashMap::default(),
            signatures: FxHashMap::default(),
        }
    }

    /// Register a function
    pub fn register<F: FhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        let signature = function.signature().clone();

        self.functions.insert(name.clone(), Arc::new(function));
        self.signatures.entry(name).or_insert_with(Vec::new).push(signature);
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn FhirPathFunction>> {
        self.functions.get(name).cloned()
    }

    /// Check if a function exists
    pub fn contains(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all registered function names
    pub fn function_names(&self) -> Vec<&str> {
        self.functions.keys().map(|s| s.as_str()).collect()
    }

    /// Get all signatures for a function name
    pub fn get_signatures(&self, name: &str) -> Option<&[FunctionSignature]> {
        self.signatures.get(name).map(|v| v.as_slice())
    }

    /// Get function by name and argument types
    pub fn get_function_for_types(&self, name: &str, arg_types: &[TypeInfo]) -> Option<Arc<dyn FhirPathFunction>> {
        if let Some(function) = self.get(name) {
            let sig = function.signature();
            if sig.matches(arg_types) {
                return Some(function);
            }
        }
        None
    }

    /// Get best matching signature for given argument types
    pub fn get_best_signature(&self, name: &str, arg_types: &[TypeInfo]) -> Option<&FunctionSignature> {
        self.get_signatures(name)?
            .iter()
            .find(|sig| sig.matches(arg_types))
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all built-in FHIRPath functions
pub fn register_builtin_functions(registry: &mut FunctionRegistry) {
    // Collection functions
    registry.register(CountFunction);
    registry.register(EmptyFunction);
    registry.register(ExistsFunction);
    registry.register(AllFunction);
    registry.register(AnyFunction);
    registry.register(AllTrueFunction);
    registry.register(DescendantsFunction);
    registry.register(AggregateFunction);
    registry.register(FirstFunction);
    registry.register(LastFunction);
    registry.register(LengthFunction);
    registry.register(DistinctFunction);
    registry.register(IsDistinctFunction);
    registry.register(SingleFunction);
    registry.register(IntersectFunction);
    registry.register(ExcludeFunction);
    registry.register(CombineFunction);
    registry.register(SortFunction);
    registry.register(TakeFunction);
    registry.register(SkipFunction);
    registry.register(TailFunction);
    registry.register(SubsetOfFunction);
    registry.register(SupersetOfFunction);

    // Boolean functions
    registry.register(NotFunction);

    // String functions
    registry.register(SubstringFunction);
    registry.register(StartsWithFunction);
    registry.register(EndsWithFunction);
    registry.register(ContainsFunction);
    registry.register(MatchesFunction);
    registry.register(MatchesFullFunction);
    registry.register(ReplaceFunction);
    registry.register(ReplaceMatchesFunction);
    registry.register(SplitFunction);
    registry.register(JoinFunction);
    registry.register(TrimFunction);
    registry.register(ToCharsFunction);
    registry.register(IndexOfFunction);
    registry.register(UpperFunction);
    registry.register(LowerFunction);
    registry.register(EncodeFunction);
    registry.register(DecodeFunction);
    registry.register(EscapeFunction);
    registry.register(UnescapeFunction);

    // Math functions
    registry.register(AbsFunction);
    registry.register(CeilingFunction);
    registry.register(FloorFunction);
    registry.register(RoundFunction);
    registry.register(SqrtFunction);
    registry.register(TruncateFunction);
    registry.register(ExpFunction);
    registry.register(LnFunction);
    registry.register(LogFunction);
    registry.register(PowerFunction);
    registry.register(PrecisionFunction);

    // Aggregate functions
    registry.register(SumFunction);
    registry.register(AvgFunction);
    registry.register(MinFunction);
    registry.register(MaxFunction);

    // Type conversion functions
    registry.register(ToStringFunction);
    registry.register(ToIntegerFunction);
    registry.register(ToDecimalFunction);
    registry.register(TypeFunction);
    registry.register(ConvertsToIntegerFunction);
    registry.register(ConvertsToDecimalFunction);
    registry.register(ConvertsToStringFunction);
    registry.register(ConvertsToBooleanFunction);

    // Filtering functions
    registry.register(WhereFunction);
    registry.register(SelectFunction);

    // DateTime functions
    registry.register(NowFunction);
    registry.register(TodayFunction);

    // Utility functions
    registry.register(IifFunction);
    registry.register(TraceFunction);
    registry.register(ConformsToFunction);
    registry.register(DefineVariableFunction);
    registry.register(RepeatFunction);

    // FHIR type functions
    registry.register(IsFunction);
    registry.register(ComparableFunction);
}
