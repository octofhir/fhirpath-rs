//! Function registry and built-in functions

use crate::signature::{FunctionSignature, ParameterInfo};
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

    /// Get function signatures by name
    pub fn get_signatures(&self, name: &str) -> Option<&[FunctionSignature]> {
        self.signatures.get(name).map(|v| v.as_slice())
    }

    /// Find function by name and argument types for overload resolution
    pub fn resolve_function(&self, name: &str, arg_types: &[TypeInfo]) -> Option<Arc<dyn FhirPathFunction>> {
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

/// type() function - returns type information with namespace and name
pub struct TypeFunction;

impl FhirPathFunction for TypeFunction {
    fn name(&self) -> &str { "type" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "type",
                vec![],
                TypeInfo::Any, // Returns an object with namespace and name properties
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    // For single-item collections, return type of the item
                    self.get_type_info(items.get(0).unwrap())
                } else if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    // For multi-item collections, this is unclear in FHIRPath spec
                    // Return empty for now
                    Ok(FhirPathValue::Empty)
                }
            }
            single_value => self.get_type_info(single_value),
        }
    }

    fn documentation(&self) -> &str {
        "type() - Returns the type information of the value as an object with namespace and name properties"
    }
}

impl TypeFunction {
    fn get_type_info(&self, value: &FhirPathValue) -> FunctionResult<FhirPathValue> {
        let (namespace, name) = match value {
            FhirPathValue::Boolean(_) => ("System", "Boolean"),
            FhirPathValue::Integer(_) => ("System", "Integer"), 
            FhirPathValue::Decimal(_) => ("System", "Decimal"),
            FhirPathValue::String(_) => ("System", "String"),
            FhirPathValue::Date(_) => ("System", "Date"),
            FhirPathValue::DateTime(_) => ("System", "DateTime"),
            FhirPathValue::Time(_) => ("System", "Time"),
            FhirPathValue::Quantity(_) => ("System", "Quantity"),
            FhirPathValue::Collection(_) => ("System", "Collection"),
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.resource_type() {
                    ("FHIR", resource_type)
                } else {
                    ("FHIR", "Resource")
                }
            }
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
        };

        // Create a JSON object with namespace and name properties
        let type_info = serde_json::json!({
            "namespace": namespace,
            "name": name
        });

        Ok(FhirPathValue::collection(vec![FhirPathValue::Resource(
            FhirResource::from_json(type_info)
        )]))
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
    registry.register(SelectFunction);

    // String functions
    registry.register(SubstringFunction);
    registry.register(StartsWithFunction);
    registry.register(EndsWithFunction);
    registry.register(ContainsFunction);

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

    // Type functions
    registry.register(ToStringFunction);
    registry.register(ToIntegerFunction);
    registry.register(ToDecimalFunction);
    registry.register(TypeFunction);
    
    // Type checking functions
    registry.register(ConvertsToIntegerFunction);
    registry.register(ConvertsToDecimalFunction);
    registry.register(ConvertsToStringFunction);
    registry.register(ConvertsToBooleanFunction);

    // Advanced functions with multiple parameter types
    registry.register(IifFunction);
    registry.register(WhereFunction);

    // Date/Time functions
    registry.register(NowFunction);
    registry.register(TodayFunction);

    // Boolean logic functions
    registry.register(NotFunction);
}

// Built-in function implementations

/// count() function - returns the number of items in a collection
struct CountFunction;

impl FhirPathFunction for CountFunction {
    fn name(&self) -> &str { "count" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "count",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(count as i64)]))
    }
}

/// empty() function - returns true if the collection is empty
struct EmptyFunction;

impl FhirPathFunction for EmptyFunction {
    fn name(&self) -> &str { "empty" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "empty",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(context.input.is_empty())]))
    }
}

/// exists() function - returns true if the collection is not empty
pub struct ExistsFunction;

impl FhirPathFunction for ExistsFunction {
    fn name(&self) -> &str { "exists" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exists",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if args.is_empty() {
            // No criteria - just check if input is non-empty
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!context.input.is_empty())]))
        } else {
            // TODO: Implement exists with criteria - need lambda evaluation
            // For now, return false as placeholder
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }
}

impl LambdaFunction for ExistsFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        lambda_context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            // No criteria - just check if input is non-empty
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!lambda_context.context.input.is_empty())]));
        }

        let criteria_expr = &args[0];
        
        match &lambda_context.context.input {
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    // Evaluate criteria with each item as $this context
                    let result = (lambda_context.evaluator)(criteria_expr, item)
                        .map_err(|e| FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Error evaluating criteria: {}", e),
                        })?;
                    
                    // Check if result is truthy
                    if is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
                    }
                }
                // No items passed the criteria
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
            }
            FhirPathValue::Empty => {
                // Empty collection - exists() returns false for empty collections
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
            }
            single_item => {
                // Single item - evaluate criteria against it
                let result = (lambda_context.evaluator)(criteria_expr, single_item)
                    .map_err(|e| FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Error evaluating criteria: {}", e),
                    })?;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_truthy(&result))]))
            }
        }
    }
}

/// first() function - returns the first item in a collection
struct FirstFunction;

impl FhirPathFunction for FirstFunction {
    fn name(&self) -> &str { "first" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "first",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if let Some(first) = items.first() {
                    Ok(first.clone())
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            single => Ok(single.clone()),
        }
    }
}

/// last() function - returns the last item in a collection
struct LastFunction;

impl FhirPathFunction for LastFunction {
    fn name(&self) -> &str { "last" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "last",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if let Some(last) = items.last() {
                    Ok(last.clone())
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            single => Ok(single.clone()),
        }
    }
}

/// length() function - returns the length of a string or collection
struct LengthFunction;

impl FhirPathFunction for LengthFunction {
    fn name(&self) -> &str { "length" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "length",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(s.len() as i64)])),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection - length() returns empty per FHIRPath spec
                    Ok(FhirPathValue::Empty) 
                } else {
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(items.len() as i64)]))
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty), // Empty returns empty, not 0
            FhirPathValue::Resource(resource) => {
                // For resources, check if it's effectively empty (like {})
                let json = resource.as_json();
                if json.is_object() && json.as_object().unwrap().is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    // Non-empty resources don't have a length in FHIRPath
                    Ok(FhirPathValue::Empty)
                }
            },
            _ => {
                // Other single values don't have a length in FHIRPath spec
                Ok(FhirPathValue::Empty)
            }
        }
    }
}

/// distinct() function - returns unique items in a collection
struct DistinctFunction;

impl FhirPathFunction for DistinctFunction {
    fn name(&self) -> &str { "distinct" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "distinct",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut unique_items = Vec::new();
                for item in items.iter() {
                    if !unique_items.contains(item) {
                        unique_items.push(item.clone());
                    }
                }
                Ok(FhirPathValue::collection(unique_items))
            }
            single => Ok(single.clone()),
        }
    }
}

// String functions

/// substring() function
struct SubstringFunction;

impl FhirPathFunction for SubstringFunction {
    fn name(&self) -> &str { "substring" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "substring",
                vec![
                    ParameterInfo::required("start", TypeInfo::Integer),
                    ParameterInfo::optional("length", TypeInfo::Integer),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if let FhirPathValue::String(s) = &context.input {
            if let Some(FhirPathValue::Integer(start)) = args.get(0) {
                let start_idx = (*start as usize).min(s.len());

                let result = if let Some(FhirPathValue::Integer(length)) = args.get(1) {
                    let end_idx = (start_idx + *length as usize).min(s.len());
                    s.chars().skip(start_idx).take(end_idx - start_idx).collect()
                } else {
                    s.chars().skip(start_idx).collect()
                };

                Ok(FhirPathValue::collection(vec![FhirPathValue::String(result)]))
            } else {
                Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: args.get(0).map(|v| v.type_name()).unwrap_or("None").to_string(),
                })
            }
        } else {
            Ok(FhirPathValue::collection(vec![]))
        }
    }
}

/// startsWith() function
struct StartsWithFunction;

impl FhirPathFunction for StartsWithFunction {
    fn name(&self) -> &str { "startsWith" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "startsWith",
                vec![ParameterInfo::required("prefix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if let (FhirPathValue::String(s), Some(FhirPathValue::String(prefix))) = (&context.input, args.get(0)) {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(s.starts_with(prefix))]))
        } else {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }
}

/// endsWith() function
struct EndsWithFunction;

impl FhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str { "endsWith" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "endsWith",
                vec![ParameterInfo::required("suffix", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if let (FhirPathValue::String(s), Some(FhirPathValue::String(suffix))) = (&context.input, args.get(0)) {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(s.ends_with(suffix))]))
        } else {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }
}

/// contains() function for strings
struct ContainsFunction;

impl FhirPathFunction for ContainsFunction {
    fn name(&self) -> &str { "contains" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "contains",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let substring = match args.get(0) {
            Some(FhirPathValue::String(s)) => s,
            Some(FhirPathValue::Collection(items)) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                    }
                } else {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                }
            },
            _ => return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
        };

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                    }
                } else if items.len() == 0 {
                    // Empty collection - return empty collection per FHIRPath spec
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(input_string.contains(substring))]))
    }
}

/// matches() function - regex pattern matching
struct MatchesFunction;

impl FhirPathFunction for MatchesFunction {
    fn name(&self) -> &str { "matches" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "matches",
                vec![ParameterInfo::required("pattern", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
        };

        if let Some(FhirPathValue::String(pattern)) = args.get(0) {
            match regex::Regex::new(pattern) {
                Ok(re) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(re.is_match(input_string))])),
                Err(_) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)])),
            }
        } else {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }
}

/// replace() function - simple string replacement
struct ReplaceFunction;

impl FhirPathFunction for ReplaceFunction {
    fn name(&self) -> &str { "replace" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replace",
                vec![
                    ParameterInfo::required("substring", TypeInfo::String),
                    ParameterInfo::required("replacement", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty),
        };

        if let (Some(FhirPathValue::String(substring)), Some(FhirPathValue::String(replacement))) = 
            (args.get(0), args.get(1)) {
            let result = input_string.replace(substring, replacement);
            Ok(FhirPathValue::collection(vec![FhirPathValue::String(result)]))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// replaceMatches() function - regex-based string replacement
struct ReplaceMatchesFunction;

impl FhirPathFunction for ReplaceMatchesFunction {
    fn name(&self) -> &str { "replaceMatches" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "replaceMatches",
                vec![
                    ParameterInfo::required("pattern", TypeInfo::String),
                    ParameterInfo::required("replacement", TypeInfo::String),
                ],
                TypeInfo::String,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty),
        };

        if let (Some(FhirPathValue::String(pattern)), Some(FhirPathValue::String(replacement))) = 
            (args.get(0), args.get(1)) {
            match regex::Regex::new(pattern) {
                Ok(re) => {
                    let result = re.replace_all(input_string, replacement).to_string();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::String(result)]))
                }
                Err(_) => Ok(FhirPathValue::collection(vec![FhirPathValue::String(input_string.clone())])),
            }
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// split() function - split string by delimiter
struct SplitFunction;

impl FhirPathFunction for SplitFunction {
    fn name(&self) -> &str { "split" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "split",
                vec![ParameterInfo::required("delimiter", TypeInfo::String)],
                TypeInfo::Collection(Box::new(TypeInfo::String)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty),
        };

        if let Some(FhirPathValue::String(delimiter)) = args.get(0) {
            let parts: Vec<FhirPathValue> = input_string
                .split(delimiter)
                .map(|s| FhirPathValue::String(s.to_string()))
                .collect();
            Ok(FhirPathValue::collection(parts))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// trim() function - remove leading and trailing whitespace
struct TrimFunction;

impl FhirPathFunction for TrimFunction {
    fn name(&self) -> &str { "trim" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "trim",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::String(s) => {
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(s.trim().to_string())]))
            },
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        Ok(FhirPathValue::collection(vec![FhirPathValue::String(s.trim().to_string())]))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::Empty)
                }
            },
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// toChars() function - convert string to character collection
struct ToCharsFunction;

impl FhirPathFunction for ToCharsFunction {
    fn name(&self) -> &str { "toChars" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toChars",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::String)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty),
        };

        let chars: Vec<FhirPathValue> = input_string
            .chars()
            .map(|c| FhirPathValue::String(c.to_string()))
            .collect();
        Ok(FhirPathValue::collection(chars))
    }
}

/// indexOf() function - find index of substring
struct IndexOfFunction;

impl FhirPathFunction for IndexOfFunction {
    fn name(&self) -> &str { "indexOf" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "indexOf",
                vec![ParameterInfo::required("substring", TypeInfo::String)],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let input_string = match &context.input {
            FhirPathValue::String(s) => s,
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    if let Some(FhirPathValue::String(s)) = items.get(0) {
                        s
                    } else {
                        return Ok(FhirPathValue::Empty);
                    }
                } else if items.is_empty() {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            },
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            _ => return Ok(FhirPathValue::Empty),
        };

        if let Some(FhirPathValue::String(substring)) = args.get(0) {
            match input_string.find(substring) {
                Some(index) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(index as i64)])),
                None => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(-1)])),
            }
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

// Math functions

/// abs() function
struct AbsFunction;

impl FhirPathFunction for AbsFunction {
    fn name(&self) -> &str { "abs" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "abs",
                vec![],
                TypeInfo::Any, // Can return Integer, Decimal, or Quantity
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i.abs())])),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d.abs())])),
            FhirPathValue::Quantity(q) => {
                let mut abs_q = q.clone();
                abs_q.value = abs_q.value.abs();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Quantity(abs_q)]))
            }
            FhirPathValue::Collection(items) => {
                // Handle collections by applying abs to each element
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(i.abs())),
                        FhirPathValue::Decimal(d) => results.push(FhirPathValue::Decimal(d.abs())),
                        FhirPathValue::Quantity(q) => {
                            let mut abs_q = q.clone();
                            abs_q.value = abs_q.value.abs();
                            results.push(FhirPathValue::Quantity(abs_q));
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// ceiling() function
struct CeilingFunction;

impl FhirPathFunction for CeilingFunction {
    fn name(&self) -> &str { "ceiling" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "ceiling",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
            FhirPathValue::Decimal(d) => {
                let ceiling = d.ceil();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(ceiling.to_i64().unwrap_or(0))]))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(*i)),
                        FhirPathValue::Decimal(d) => {
                            let ceiling = d.ceil();
                            results.push(FhirPathValue::Integer(ceiling.to_i64().unwrap_or(0)));
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// floor() function
struct FloorFunction;

impl FhirPathFunction for FloorFunction {
    fn name(&self) -> &str { "floor" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "floor",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
            FhirPathValue::Decimal(d) => {
                let floor = d.floor();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(floor.to_i64().unwrap_or(0))]))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(*i)),
                        FhirPathValue::Decimal(d) => {
                            let floor = d.floor();
                            results.push(FhirPathValue::Integer(floor.to_i64().unwrap_or(0)));
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// round() function
struct RoundFunction;

impl FhirPathFunction for RoundFunction {
    fn name(&self) -> &str { "round" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "round",
                vec![
                    ParameterInfo::optional("precision", TypeInfo::Integer),
                ],
                TypeInfo::Any, // Can return Integer or Decimal
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Get precision parameter (default to 0)
        let precision = if args.is_empty() {
            0
        } else {
            match &args[0] {
                FhirPathValue::Integer(p) => *p,
                _ => 0,
            }
        };

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
            FhirPathValue::Decimal(d) => {
                if precision == 0 {
                    let rounded = d.round();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(rounded.to_i64().unwrap_or(0))]))
                } else {
                    let rounded = d.round_dp(precision as u32);
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(rounded)]))
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(*i)),
                        FhirPathValue::Decimal(d) => {
                            if precision == 0 {
                                let rounded = d.round();
                                results.push(FhirPathValue::Integer(rounded.to_i64().unwrap_or(0)));
                            } else {
                                let rounded = d.round_dp(precision as u32);
                                results.push(FhirPathValue::Decimal(rounded));
                            }
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// sqrt() function
struct SqrtFunction;

impl FhirPathFunction for SqrtFunction {
    fn name(&self) -> &str { "sqrt" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "sqrt",
                vec![],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i < 0 {
                    Ok(FhirPathValue::Empty) // Negative square root returns empty
                } else {
                    let f = *i as f64;
                    let sqrt = f.sqrt();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(sqrt).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() {
                    Ok(FhirPathValue::Empty) // Negative square root returns empty
                } else {
                    let f = d.to_f64().unwrap_or(0.0);
                    let sqrt = f.sqrt();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(sqrt).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => {
                            if i >= &0 {
                                let f = *i as f64;
                                let sqrt = f.sqrt();
                                if let Some(dec) = rust_decimal::Decimal::from_f64(sqrt) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            if !d.is_sign_negative() {
                                let f = d.to_f64().unwrap_or(0.0);
                                let sqrt = f.sqrt();
                                if let Some(dec) = rust_decimal::Decimal::from_f64(sqrt) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// truncate() function
struct TruncateFunction;

impl FhirPathFunction for TruncateFunction {
    fn name(&self) -> &str { "truncate" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "truncate",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
            FhirPathValue::Decimal(d) => {
                let truncated = d.trunc();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(truncated.to_i64().unwrap_or(0))]))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => results.push(FhirPathValue::Integer(*i)),
                        FhirPathValue::Decimal(d) => {
                            let truncated = d.trunc();
                            results.push(FhirPathValue::Integer(truncated.to_i64().unwrap_or(0)));
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// exp() function
struct ExpFunction;

impl FhirPathFunction for ExpFunction {
    fn name(&self) -> &str { "exp" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exp",
                vec![],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => {
                let f = *i as f64;
                let exp = f.exp();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    rust_decimal::Decimal::from_f64(exp).unwrap_or_default()
                )]))
            }
            FhirPathValue::Decimal(d) => {
                let f = d.to_f64().unwrap_or(0.0);
                let exp = f.exp();
                Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                    rust_decimal::Decimal::from_f64(exp).unwrap_or_default()
                )]))
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => {
                            let f = *i as f64;
                            let exp = f.exp();
                            if let Some(dec) = rust_decimal::Decimal::from_f64(exp) {
                                results.push(FhirPathValue::Decimal(dec));
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            let f = d.to_f64().unwrap_or(0.0);
                            let exp = f.exp();
                            if let Some(dec) = rust_decimal::Decimal::from_f64(exp) {
                                results.push(FhirPathValue::Decimal(dec));
                            }
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// ln() function
struct LnFunction;

impl FhirPathFunction for LnFunction {
    fn name(&self) -> &str { "ln" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "ln",
                vec![],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i <= 0 {
                    Ok(FhirPathValue::Empty) // ln of non-positive returns empty
                } else {
                    let f = *i as f64;
                    let ln = f.ln();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(ln).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() || d.is_zero() {
                    Ok(FhirPathValue::Empty) // ln of non-positive returns empty
                } else {
                    let f = d.to_f64().unwrap_or(0.0);
                    let ln = f.ln();
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(ln).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => {
                            if i > &0 {
                                let f = *i as f64;
                                let ln = f.ln();
                                if let Some(dec) = rust_decimal::Decimal::from_f64(ln) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            if !d.is_sign_negative() && !d.is_zero() {
                                let f = d.to_f64().unwrap_or(0.0);
                                let ln = f.ln();
                                if let Some(dec) = rust_decimal::Decimal::from_f64(ln) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

/// log() function
struct LogFunction;

impl FhirPathFunction for LogFunction {
    fn name(&self) -> &str { "log" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "log",
                vec![
                    ParameterInfo::required("base", TypeInfo::Any), // Integer or Decimal
                ],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // Handle empty input
        if context.input.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // Get base
        let base_f64 = match &args[0] {
            FhirPathValue::Integer(b) => {
                if *b <= 0 {
                    return Ok(FhirPathValue::Empty);
                }
                *b as f64
            }
            FhirPathValue::Decimal(b) => {
                if b.is_sign_negative() || b.is_zero() {
                    return Ok(FhirPathValue::Empty);
                }
                b.to_f64().unwrap_or(0.0)
            }
            _ => return Ok(FhirPathValue::Empty),
        };

        match &context.input {
            FhirPathValue::Integer(i) => {
                if *i <= 0 {
                    Ok(FhirPathValue::Empty)
                } else {
                    let f = *i as f64;
                    let log = f.log(base_f64);
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(log).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Decimal(d) => {
                if d.is_sign_negative() || d.is_zero() {
                    Ok(FhirPathValue::Empty)
                } else {
                    let f = d.to_f64().unwrap_or(0.0);
                    let log = f.log(base_f64);
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(
                        rust_decimal::Decimal::from_f64(log).unwrap_or_default()
                    )]))
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match item {
                        FhirPathValue::Integer(i) => {
                            if i > &0 {
                                let f = *i as f64;
                                let log = f.log(base_f64);
                                if let Some(dec) = rust_decimal::Decimal::from_f64(log) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        FhirPathValue::Decimal(d) => {
                            if !d.is_sign_negative() && !d.is_zero() {
                                let f = d.to_f64().unwrap_or(0.0);
                                let log = f.log(base_f64);
                                if let Some(dec) = rust_decimal::Decimal::from_f64(log) {
                                    results.push(FhirPathValue::Decimal(dec));
                                }
                            }
                        }
                        _ => {} // Skip non-numeric values
                    }
                }
                if results.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    Ok(FhirPathValue::collection(results))
                }
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

// Type conversion functions

/// toString() function
struct ToStringFunction;

impl FhirPathFunction for ToStringFunction {
    fn name(&self) -> &str { "toString" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toString",
                vec![],
                TypeInfo::String,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    // Single item in collection - convert it to string
                    if let Some(item) = items.get(0) {
                        let string_val = match item {
                            FhirPathValue::String(s) => s.clone(),
                            FhirPathValue::Integer(i) => i.to_string(),
                            FhirPathValue::Decimal(d) => d.to_string(),
                            FhirPathValue::Boolean(b) => b.to_string(),
                            FhirPathValue::Date(d) => d.format("%Y-%m-%d").to_string(),
                            FhirPathValue::DateTime(dt) => dt.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string(),
                            FhirPathValue::Time(t) => t.format("%H:%M:%S").to_string(),
                            _ => return Ok(FhirPathValue::Empty),
                        };
                        Ok(FhirPathValue::collection(vec![FhirPathValue::String(string_val)]))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    // Multiple items - can't convert to string
                    Ok(FhirPathValue::Empty)
                }
            },
            _ => {
                // Single value - convert to string
                let string_val = match &context.input {
                    FhirPathValue::String(s) => s.clone(),
                    FhirPathValue::Integer(i) => i.to_string(),
                    FhirPathValue::Decimal(d) => d.to_string(),
                    FhirPathValue::Boolean(b) => b.to_string(),
                    FhirPathValue::Date(d) => d.format("%Y-%m-%d").to_string(),
                    FhirPathValue::DateTime(dt) => dt.format("%Y-%m-%dT%H:%M:%S%.3f%z").to_string(),
                    FhirPathValue::Time(t) => t.format("%H:%M:%S").to_string(),
                    _ => return Ok(FhirPathValue::Empty),
                };
                Ok(FhirPathValue::collection(vec![FhirPathValue::String(string_val)]))
            }
        }
    }
}

/// toInteger() function
struct ToIntegerFunction;

impl FhirPathFunction for ToIntegerFunction {
    fn name(&self) -> &str { "toInteger" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toInteger",
                vec![],
                TypeInfo::Integer,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    // Single item in collection - convert it to integer
                    if let Some(item) = items.get(0) {
                        match item {
                            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
                            FhirPathValue::String(s) => {
                                // According to FHIRPath spec, toInteger() should only convert
                                // strings that represent pure integers, not decimals
                                match s.parse::<i64>() {
                                    Ok(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)])),
                                    Err(_) => Ok(FhirPathValue::Empty),
                                }
                            }
                            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(if *b { 1 } else { 0 })])),
                            FhirPathValue::Decimal(d) => {
                                if d.fract().is_zero() {
                                    if let Some(i) = d.to_i64() {
                                        Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)]))
                                    } else {
                                        Ok(FhirPathValue::Empty)
                                    }
                                } else {
                                    Ok(FhirPathValue::Empty)
                                }
                            }
                            _ => Ok(FhirPathValue::Empty),
                        }
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    // Multiple items - can't convert to integer
                    Ok(FhirPathValue::Empty)
                }
            },
            _ => {
                // Single value - convert to integer
                match &context.input {
                    FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(*i)])),
                    FhirPathValue::String(s) => {
                        // According to FHIRPath spec, toInteger() should only convert
                        // strings that represent pure integers, not decimals
                        match s.parse::<i64>() {
                            Ok(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)])),
                            Err(_) => Ok(FhirPathValue::Empty),
                        }
                    }
                    FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(if *b { 1 } else { 0 })])),
                    FhirPathValue::Decimal(d) => {
                        if d.fract().is_zero() {
                            if let Some(i) = d.to_i64() {
                                Ok(FhirPathValue::collection(vec![FhirPathValue::Integer(i)]))
                            } else {
                                Ok(FhirPathValue::Empty)
                            }
                        } else {
                            Ok(FhirPathValue::Empty)
                        }
                    }
                    _ => Ok(FhirPathValue::Empty),
                }
            }
        }
    }
}

/// toDecimal() function
struct ToDecimalFunction;

impl FhirPathFunction for ToDecimalFunction {
    fn name(&self) -> &str { "toDecimal" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "toDecimal",
                vec![],
                TypeInfo::Decimal,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    // Single item in collection - convert it to decimal
                    if let Some(item) = items.get(0) {
                        match item {
                            FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(*d)])),
                            FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))])),
                            FhirPathValue::String(s) => {
                                match s.parse::<rust_decimal::Decimal>() {
                                    Ok(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d)])),
                                    Err(_) => Ok(FhirPathValue::Empty),
                                }
                            }
                            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(rust_decimal::Decimal::from(if *b { 1 } else { 0 }))])),
                            _ => Ok(FhirPathValue::Empty),
                        }
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                } else if items.is_empty() {
                    Ok(FhirPathValue::Empty)
                } else {
                    // Multiple items - can't convert to decimal
                    Ok(FhirPathValue::Empty)
                }
            },
            _ => {
                // Single value - convert to decimal
                match &context.input {
                    FhirPathValue::Decimal(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(*d)])),
                    FhirPathValue::Integer(i) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))])),
                    FhirPathValue::String(s) => {
                        match s.parse::<rust_decimal::Decimal>() {
                            Ok(d) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(d)])),
                            Err(_) => Ok(FhirPathValue::Empty),
                        }
                    }
                    FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Decimal(rust_decimal::Decimal::from(if *b { 1 } else { 0 }))])),
                    _ => Ok(FhirPathValue::Empty),
                }
            }
        }
    }
}

/// iif() function - conditional expression
struct IifFunction;

impl FhirPathFunction for IifFunction {
    fn name(&self) -> &str { "iif" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "iif",
                vec![
                    ParameterInfo::required("condition", TypeInfo::Boolean),
                    ParameterInfo::required("true_result", TypeInfo::Any),
                    ParameterInfo::optional("false_result", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match args.get(0) {
            Some(FhirPathValue::Boolean(true)) => {
                Ok(args.get(1).cloned().unwrap_or(FhirPathValue::Empty))
            }
            Some(FhirPathValue::Boolean(false)) => {
                Ok(args.get(2).cloned().unwrap_or(FhirPathValue::Empty))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }

    fn documentation(&self) -> &str {
        "iif(condition, true_result, false_result) - Returns true_result if condition is true, false_result otherwise"
    }
}

/// where() function - filter collection based on condition
struct WhereFunction;

impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str { "where" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "where",
                vec![ParameterInfo::required("condition", TypeInfo::Boolean)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let condition_expr = args.get(0).ok_or_else(|| FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "Missing condition argument".to_string(),
        })?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    // In a real implementation, we would evaluate the condition expression
                    // against each item. For now, we just check if the condition is true.
                    if matches!(condition_expr, FhirPathValue::Boolean(true)) {
                        results.push(item.clone());
                    }
                }
                Ok(FhirPathValue::collection(results))
            }
            other => {
                // For non-collection input, treat as single-item collection
                if matches!(condition_expr, FhirPathValue::Boolean(true)) {
                    Ok(other.clone())
                } else {
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
        }
    }

    fn documentation(&self) -> &str {
        "where(condition) - Returns a collection containing only items for which condition evaluates to true"
    }
}

/// take() function - returns the first n items from a collection
struct TakeFunction;

impl FhirPathFunction for TakeFunction {
    fn name(&self) -> &str { "take" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "take",
                vec![ParameterInfo::required("num", TypeInfo::Integer)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if let Some(FhirPathValue::Integer(n)) = args.get(0) {
            let n = *n as usize;
            match &context.input {
                FhirPathValue::Collection(items) => {
                    let taken: Vec<_> = items.iter().take(n).cloned().collect();
                    Ok(FhirPathValue::collection(taken))
                }
                single if n > 0 => Ok(single.clone()),
                _ => Ok(FhirPathValue::collection(vec![])),
            }
        } else {
            Ok(FhirPathValue::collection(vec![]))
        }
    }

    fn documentation(&self) -> &str {
        "take(num) - Returns a collection containing the first num items"
    }
}

/// skip() function - returns all items except the first n from a collection
struct SkipFunction;

impl FhirPathFunction for SkipFunction {
    fn name(&self) -> &str { "skip" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "skip",
                vec![ParameterInfo::required("num", TypeInfo::Integer)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if let Some(FhirPathValue::Integer(n)) = args.get(0) {
            let n = *n as usize;
            match &context.input {
                FhirPathValue::Collection(items) => {
                    let skipped: Vec<_> = items.iter().skip(n).cloned().collect();
                    Ok(FhirPathValue::collection(skipped))
                }
                single if n == 0 => Ok(single.clone()),
                _ => Ok(FhirPathValue::collection(vec![])),
            }
        } else {
            Ok(FhirPathValue::collection(vec![]))
        }
    }

    fn documentation(&self) -> &str {
        "skip(num) - Returns a collection containing all items except the first num items"
    }
}

/// tail() function - returns all items except the first from a collection
struct TailFunction;

impl FhirPathFunction for TailFunction {
    fn name(&self) -> &str { "tail" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "tail",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                let tail: Vec<_> = items.iter().skip(1).cloned().collect();
                Ok(FhirPathValue::collection(tail))
            }
            _ => Ok(FhirPathValue::collection(vec![])),
        }
    }

    fn documentation(&self) -> &str {
        "tail() - Returns a collection containing all items except the first"
    }
}

/// select() function - projects each item to a new value
struct SelectFunction;

impl FhirPathFunction for SelectFunction {
    fn name(&self) -> &str { "select" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "select",
                vec![ParameterInfo::required("projection", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let projection = args.get(0).ok_or_else(|| FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "Missing projection argument".to_string(),
        })?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for _item in items.iter() {
                    // In a real implementation, we would evaluate the projection expression
                    // against each item. For now, we just return the projection value.
                    results.push(projection.clone());
                }
                Ok(FhirPathValue::collection(results))
            }
            _single => {
                // For non-collection input, treat as single-item collection
                Ok(FhirPathValue::collection(vec![projection.clone()]))
            }
        }
    }

    fn documentation(&self) -> &str {
        "select(projection) - Projects each item in the collection to a new value"
    }
}

/// now() function - returns the current date and time
struct NowFunction;

impl FhirPathFunction for NowFunction {
    fn name(&self) -> &str { "now" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "now",
                vec![],
                TypeInfo::DateTime,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let now = chrono::Utc::now();
        Ok(FhirPathValue::collection(vec![FhirPathValue::DateTime(now)]))
    }

    fn documentation(&self) -> &str {
        "now() - Returns the current date and time"
    }
}

/// today() function - returns the current date
struct TodayFunction;

impl FhirPathFunction for TodayFunction {
    fn name(&self) -> &str { "today" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "today",
                vec![],
                TypeInfo::Date,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let today = chrono::Utc::now().date_naive();
        Ok(FhirPathValue::collection(vec![FhirPathValue::Date(today)]))
    }

    fn documentation(&self) -> &str {
        "today() - Returns the current date"
    }
}

/// not() function - logical negation
struct NotFunction;

impl FhirPathFunction for NotFunction {
    fn name(&self) -> &str { "not" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "not",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)])),
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection is false, not becomes true
                    Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]))
                } else if items.len() == 1 {
                    // Single item collection - apply not to the item
                    match items.get(0) {
                        Some(FhirPathValue::Boolean(b)) => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!b)])),
                        _ => Ok(FhirPathValue::collection(vec![]))
                    }
                } else {
                    // Multiple items - not applicable, return empty
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)])),
            _ => Ok(FhirPathValue::collection(vec![])),
        }
    }

    fn documentation(&self) -> &str {
        "not() - Returns the logical negation of the input"
    }
}

// Type checking functions

/// convertsToInteger() function - checks if input can be converted to Integer
struct ConvertsToIntegerFunction;

impl FhirPathFunction for ConvertsToIntegerFunction {
    fn name(&self) -> &str { "convertsToInteger" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToInteger",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let can_convert = match &context.input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item to convert
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Integer(_)) => true,
                        Some(FhirPathValue::String(s)) => s.parse::<i64>().is_ok(),
                        Some(FhirPathValue::Decimal(d)) => d.fract().is_zero(),
                        Some(FhirPathValue::Boolean(_)) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => s.parse::<i64>().is_ok(),
            FhirPathValue::Decimal(d) => d.fract().is_zero(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
    }

    fn documentation(&self) -> &str {
        "convertsToInteger() - Returns true if the input can be converted to an Integer"
    }
}

/// convertsToDecimal() function - checks if input can be converted to Decimal
struct ConvertsToDecimalFunction;

impl FhirPathFunction for ConvertsToDecimalFunction {
    fn name(&self) -> &str { "convertsToDecimal" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToDecimal",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let can_convert = match &context.input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item to convert
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Decimal(_)) => true,
                        Some(FhirPathValue::Integer(_)) => true,
                        Some(FhirPathValue::String(s)) => s.parse::<rust_decimal::Decimal>().is_ok(),
                        Some(FhirPathValue::Boolean(_)) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::String(s) => s.parse::<rust_decimal::Decimal>().is_ok(),
            FhirPathValue::Boolean(_) => true,
            _ => false,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
    }

    fn documentation(&self) -> &str {
        "convertsToDecimal() - Returns true if the input can be converted to a Decimal"
    }
}

/// convertsToString() function - checks if input can be converted to String
struct ConvertsToStringFunction;

impl FhirPathFunction for ConvertsToStringFunction {
    fn name(&self) -> &str { "convertsToString" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToString",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let can_convert = match &context.input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item to convert
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::String(_)) => true,
                        Some(FhirPathValue::Integer(_)) => true,
                        Some(FhirPathValue::Decimal(_)) => true,
                        Some(FhirPathValue::Boolean(_)) => true,
                        Some(FhirPathValue::Date(_)) => true,
                        Some(FhirPathValue::DateTime(_)) => true,
                        Some(FhirPathValue::Time(_)) => true,
                        Some(FhirPathValue::Quantity(_)) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FhirPathValue::String(_) => true,
            FhirPathValue::Integer(_) => true,
            FhirPathValue::Decimal(_) => true,
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::Date(_) => true,
            FhirPathValue::DateTime(_) => true,
            FhirPathValue::Time(_) => true,
            FhirPathValue::Quantity(_) => true,
            _ => false,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
    }

    fn documentation(&self) -> &str {
        "convertsToString() - Returns true if the input can be converted to a String"
    }
}

/// convertsToBoolean() function - checks if input can be converted to Boolean
struct ConvertsToBooleanFunction;

impl FhirPathFunction for ConvertsToBooleanFunction {
    fn name(&self) -> &str { "convertsToBoolean" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "convertsToBoolean",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let can_convert = match &context.input {
            FhirPathValue::Collection(items) => {
                // Must have exactly one item to convert
                if items.len() == 1 {
                    match items.get(0) {
                        Some(FhirPathValue::Boolean(_)) => true,
                        Some(FhirPathValue::String(s)) => {
                            let s_lower = s.to_lowercase();
                            matches!(s_lower.as_str(), "true" | "false" | "t" | "f" | "yes" | "no" | "y" | "n" | "1" | "0")
                        }
                        Some(FhirPathValue::Integer(i)) => i == &0 || i == &1,
                        Some(FhirPathValue::Decimal(d)) => {
                            let zero = rust_decimal::Decimal::from(0);
                            let one = rust_decimal::Decimal::from(1);
                            d == &zero || d == &one
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FhirPathValue::Boolean(_) => true,
            FhirPathValue::String(s) => {
                let s_lower = s.to_lowercase();
                matches!(s_lower.as_str(), "true" | "false" | "t" | "f" | "yes" | "no" | "y" | "n" | "1" | "0")
            }
            FhirPathValue::Integer(i) => *i == 0 || *i == 1,
            FhirPathValue::Decimal(d) => {
                let zero = rust_decimal::Decimal::from(0);
                let one = rust_decimal::Decimal::from(1);
                *d == zero || *d == one
            }
            _ => false,
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(can_convert)]))
    }

    fn documentation(&self) -> &str {
        "convertsToBoolean() - Returns true if the input can be converted to a Boolean"
    }
}

/// all() function - returns true if criteria is true for all items
pub struct AllFunction;

impl FhirPathFunction for AllFunction {
    fn name(&self) -> &str { "all" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "all",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if args.is_empty() {
            // No criteria - check if all items exist (non-empty means all exist)
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!context.input.is_empty())]))
        } else {
            // TODO: Implement all with criteria - need lambda evaluation
            // For now, return false as placeholder
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }

    fn documentation(&self) -> &str {
        "all(criteria) - Returns true if criteria is true for all items in the collection"
    }
}

impl LambdaFunction for AllFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        lambda_context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            // No criteria - check if all items exist (non-empty means all exist)
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!lambda_context.context.input.is_empty())]));
        }

        let criteria_expr = &args[0];
        
        match &lambda_context.context.input {
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    // Evaluate criteria with each item as $this context
                    let result = (lambda_context.evaluator)(criteria_expr, item)
                        .map_err(|e| FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Error evaluating criteria: {}", e),
                        })?;
                    
                    // Check if result is truthy
                    if !is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]));
                    }
                }
                // All items passed the criteria
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]))
            }
            FhirPathValue::Empty => {
                // Empty collection - all() returns true for empty collections
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]))
            }
            single_item => {
                // Single item - evaluate criteria against it
                let result = (lambda_context.evaluator)(criteria_expr, single_item)
                    .map_err(|e| FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Error evaluating criteria: {}", e),
                    })?;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_truthy(&result))]))
            }
        }
    }
}

/// any() function - returns true if criteria is true for any item
pub struct AnyFunction;

impl FhirPathFunction for AnyFunction {
    fn name(&self) -> &str { "any" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "any",
                vec![ParameterInfo::optional("criteria", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        if args.is_empty() {
            // No criteria - check if any items exist (non-empty means some exist)
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!context.input.is_empty())]))
        } else {
            // TODO: Implement any with criteria - need lambda evaluation
            // For now, return false as placeholder
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
        }
    }

    fn documentation(&self) -> &str {
        "any(criteria) - Returns true if criteria is true for any item in the collection"
    }
}

impl LambdaFunction for AnyFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        lambda_context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            // No criteria - check if any items exist (non-empty means some exist)
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(!lambda_context.context.input.is_empty())]));
        }

        let criteria_expr = &args[0];
        
        match &lambda_context.context.input {
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    // Evaluate criteria with each item as $this context
                    let result = (lambda_context.evaluator)(criteria_expr, item)
                        .map_err(|e| FunctionError::EvaluationError {
                            name: self.name().to_string(),
                            message: format!("Error evaluating criteria: {}", e),
                        })?;
                    
                    // Check if result is truthy
                    if is_truthy(&result) {
                        return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(true)]));
                    }
                }
                // No items passed the criteria
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
            }
            FhirPathValue::Empty => {
                // Empty collection - any() returns false for empty collections
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(false)]))
            }
            single_item => {
                // Single item - evaluate criteria against it
                let result = (lambda_context.evaluator)(criteria_expr, single_item)
                    .map_err(|e| FunctionError::EvaluationError {
                        name: self.name().to_string(),
                        message: format!("Error evaluating criteria: {}", e),
                    })?;
                Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_truthy(&result))]))
            }
        }
    }
}

/// Fast discriminant type for grouping FhirPathValue instances
/// This enables O(1) average-case lookups while maintaining correctness
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ValueDiscriminant {
    Boolean(bool),
    Integer(i64),
    Decimal(rust_decimal::Decimal),
    StringEmpty,
    StringShort(String),
    StringLong(usize, String, String), // length, prefix, suffix
    Date(chrono::NaiveDate),
    DateTime(chrono::DateTime<chrono::Utc>),
    Time(chrono::NaiveTime),
    Quantity(rust_decimal::Decimal, Option<String>),
    Collection,
    Resource,
    Empty,
}

/// isDistinct() function - returns true if the collection contains no duplicates
pub struct IsDistinctFunction;

impl IsDistinctFunction {
    /// Optimized duplicate detection using hash-like approach
    /// Time complexity: O(n) average case, O(n²) worst case
    /// Space complexity: O(n)
    #[inline]
    fn has_no_duplicates<'a, I>(items: I) -> bool 
    where
        I: Iterator<Item = &'a FhirPathValue>,
    {
        use std::collections::HashMap;
        
        // Use a HashMap with custom discriminant keys for fast duplicate detection
        let mut seen: HashMap<ValueDiscriminant, Vec<&'a FhirPathValue>> = HashMap::new();
        
        for item in items {
            let discriminant = Self::create_discriminant(item);
            
            // Check if we've seen this discriminant before
            if let Some(existing_items) = seen.get_mut(&discriminant) {
                // We have a potential match - need to check for actual equality
                // within items that have the same discriminant
                for existing_item in existing_items.iter() {
                    if item == *existing_item {
                        return false; // Duplicate found
                    }
                }
                existing_items.push(item);
            } else {
                // First time seeing this discriminant
                seen.insert(discriminant, vec![item]);
            }
        }
        
        true // No duplicates found
    }
    
    /// Create a fast discriminant for FhirPathValue that enables efficient grouping
    /// This approach groups similar values together while avoiding expensive string operations
    /// for simple cases
    #[inline]
    fn create_discriminant(value: &FhirPathValue) -> ValueDiscriminant {
        match value {
            FhirPathValue::Boolean(b) => ValueDiscriminant::Boolean(*b),
            FhirPathValue::Integer(i) => ValueDiscriminant::Integer(*i),
            FhirPathValue::Decimal(d) => ValueDiscriminant::Decimal(*d),
            FhirPathValue::String(s) => {
                // For strings, use length + first few chars as discriminant for performance
                if s.is_empty() {
                    ValueDiscriminant::StringEmpty
                } else if s.len() <= 8 {
                    // For short strings, use the full string
                    ValueDiscriminant::StringShort(s.clone())
                } else {
                    // For long strings, use length + prefix + suffix for fast discrimination
                    let prefix = &s[..4];
                    let suffix = &s[s.len()-4..];
                    ValueDiscriminant::StringLong(s.len(), prefix.to_string(), suffix.to_string())
                }
            },
            FhirPathValue::Date(d) => ValueDiscriminant::Date(*d),
            FhirPathValue::DateTime(dt) => ValueDiscriminant::DateTime(*dt),
            FhirPathValue::Time(t) => ValueDiscriminant::Time(*t),
            FhirPathValue::Quantity(q) => ValueDiscriminant::Quantity(q.value, q.unit.clone()),
            FhirPathValue::Collection(_) => ValueDiscriminant::Collection,
            FhirPathValue::Resource(_) => ValueDiscriminant::Resource,
            FhirPathValue::Empty => ValueDiscriminant::Empty,
        }
    }
}

impl FhirPathFunction for IsDistinctFunction {
    fn name(&self) -> &str { "isDistinct" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "isDistinct",
                vec![],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let is_distinct = match &context.input {
            FhirPathValue::Collection(items) => {
                Self::has_no_duplicates(items.iter())
            }
            FhirPathValue::Empty => true, // Empty collection is distinct
            _ => true, // Single values are always distinct
        };

        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(is_distinct)]))
    }

    fn documentation(&self) -> &str {
        "isDistinct() - Returns true if the collection contains no duplicate values"
    }
}

/// single() function - returns the single item in a collection, error if not exactly one
struct SingleFunction;

impl FhirPathFunction for SingleFunction {
    fn name(&self) -> &str { "single" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "single",
                vec![],
                TypeInfo::Any,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() == 1 {
                    Ok(items.get(0).unwrap().clone())
                } else {
                    // Per FHIRPath spec, single() returns empty if not exactly one item
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            single => Ok(single.clone()), // Single non-collection value
        }
    }

    fn documentation(&self) -> &str {
        "single() - Returns the single item in the collection, empty if not exactly one item"
    }
}

/// intersect() function - returns the intersection of two collections
struct IntersectFunction;

impl FhirPathFunction for IntersectFunction {
    fn name(&self) -> &str { "intersect" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "intersect",
                vec![ParameterInfo::required("other", TypeInfo::Collection(Box::new(TypeInfo::Any)))],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let other = args.get(0).ok_or_else(|| FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "Missing other collection argument".to_string(),
        })?;

        let left_items = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => return Ok(FhirPathValue::collection(vec![single.clone()])), // Treat single as collection
        };

        let right_items = match other {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => {
                // Check if the single item is in left collection
                if left_items.contains(single) {
                    return Ok(FhirPathValue::collection(vec![single.clone()]));
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
        };

        let mut result = Vec::new();
        for item in left_items.iter() {
            if right_items.contains(item) && !result.contains(item) {
                result.push(item.clone());
            }
        }

        Ok(FhirPathValue::collection(result))
    }

    fn documentation(&self) -> &str {
        "intersect(other) - Returns items that are in both collections"
    }
}

/// exclude() function - returns items from the first collection that are not in the second
struct ExcludeFunction;

impl FhirPathFunction for ExcludeFunction {
    fn name(&self) -> &str { "exclude" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "exclude",
                vec![ParameterInfo::required("other", TypeInfo::Collection(Box::new(TypeInfo::Any)))],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let other = args.get(0).ok_or_else(|| FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "Missing other collection argument".to_string(),
        })?;

        let left_items = match &context.input {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single => {
                // Check if the single item should be excluded
                let should_exclude = match other {
                    FhirPathValue::Collection(other_items) => other_items.contains(single),
                    FhirPathValue::Empty => false,
                    other_single => other_single == single,
                };
                if should_exclude {
                    return Ok(FhirPathValue::Empty);
                } else {
                    return Ok(FhirPathValue::collection(vec![single.clone()]));
                }
            }
        };

        let right_items = match other {
            FhirPathValue::Collection(items) => items,
            FhirPathValue::Empty => return Ok(context.input.clone()), // Nothing to exclude
            single => {
                // Exclude single item from collection
                let result: Vec<_> = left_items.iter().filter(|&item| item != single).cloned().collect();
                return Ok(FhirPathValue::collection(result));
            }
        };

        let result: Vec<_> = left_items.iter()
            .filter(|&item| !right_items.contains(item))
            .cloned()
            .collect();

        Ok(FhirPathValue::collection(result))
    }

    fn documentation(&self) -> &str {
        "exclude(other) - Returns items from the first collection that are not in the second"
    }
}

/// combine() function - returns the union of two collections without duplicates
struct CombineFunction;

impl FhirPathFunction for CombineFunction {
    fn name(&self) -> &str { "combine" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "combine",
                vec![ParameterInfo::required("other", TypeInfo::Collection(Box::new(TypeInfo::Any)))],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let other = args.get(0).ok_or_else(|| FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "Missing other collection argument".to_string(),
        })?;

        let mut result = Vec::new();

        // Add items from first collection
        match &context.input {
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    if !result.contains(item) {
                        result.push(item.clone());
                    }
                }
            }
            FhirPathValue::Empty => {}
            single => {
                result.push(single.clone());
            }
        }

        // Add items from second collection (avoiding duplicates)
        match other {
            FhirPathValue::Collection(items) => {
                for item in items.iter() {
                    if !result.contains(item) {
                        result.push(item.clone());
                    }
                }
            }
            FhirPathValue::Empty => {}
            single => {
                if !result.contains(single) {
                    result.push(single.clone());
                }
            }
        }

        Ok(FhirPathValue::collection(result))
    }

    fn documentation(&self) -> &str {
        "combine(other) - Returns the union of two collections without duplicates"
    }
}

/// sort() function - sorts a collection
struct SortFunction;

impl FhirPathFunction for SortFunction {
    fn name(&self) -> &str { "sort" }

    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "sort",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        match &context.input {
            FhirPathValue::Collection(items) => {
                let mut sorted_items = items.clone().into_vec();
                sorted_items.sort_by(|a, b| compare_fhir_values(a, b));
                Ok(FhirPathValue::collection(sorted_items))
            }
            other => Ok(other.clone()), // Single values don't need sorting
        }
    }

    fn documentation(&self) -> &str {
        "sort() - Sorts the collection in ascending order"
    }
}

/// Compare two FhirPathValues for sorting
fn compare_fhir_values(a: &FhirPathValue, b: &FhirPathValue) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    
    match (a, b) {
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.cmp(b),
        (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a.cmp(b),
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a.cmp(b),
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a.cmp(b),
        // Mixed types - use type ordering: Boolean < Integer < Decimal < String < Date < DateTime < Time
        (FhirPathValue::Boolean(_), _) => Ordering::Less,
        (_, FhirPathValue::Boolean(_)) => Ordering::Greater,
        (FhirPathValue::Integer(_), FhirPathValue::Decimal(_)) => Ordering::Less,
        (FhirPathValue::Decimal(_), FhirPathValue::Integer(_)) => Ordering::Greater,
        (FhirPathValue::Integer(_), _) => Ordering::Less,
        (_, FhirPathValue::Integer(_)) => Ordering::Greater,
        (FhirPathValue::Decimal(_), _) => Ordering::Less,
        (_, FhirPathValue::Decimal(_)) => Ordering::Greater,
        (FhirPathValue::String(_), FhirPathValue::Date(_)) => Ordering::Less,
        (FhirPathValue::Date(_), FhirPathValue::String(_)) => Ordering::Greater,
        (FhirPathValue::String(_), _) => Ordering::Less,
        (_, FhirPathValue::String(_)) => Ordering::Greater,
        (FhirPathValue::Date(_), _) => Ordering::Less,
        (_, FhirPathValue::Date(_)) => Ordering::Greater,
        (FhirPathValue::DateTime(_), _) => Ordering::Less,
        (_, FhirPathValue::DateTime(_)) => Ordering::Greater,
        // All other cases (including collections and empty values)
        _ => Ordering::Equal,
    }
}

/// Helper function to determine if a FhirPathValue is truthy
fn is_truthy(value: &FhirPathValue) -> bool {
    match value {
        FhirPathValue::Boolean(b) => *b,
        FhirPathValue::Collection(items) => {
            // Collection is truthy if it's non-empty and contains truthy values
            !items.is_empty() && items.iter().any(is_truthy)
        }
        FhirPathValue::Empty => false,
        _ => true, // Non-empty values are generally truthy
    }
}
