//! Function registry and built-in functions

use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;

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

/// Register all built-in FHIRPath functions
pub fn register_builtin_functions(registry: &mut FunctionRegistry) {
    // Collection functions
    registry.register(CountFunction);
    registry.register(EmptyFunction);
    registry.register(ExistsFunction);
    registry.register(FirstFunction);
    registry.register(LastFunction);
    registry.register(LengthFunction);
    registry.register(DistinctFunction);

    // String functions
    registry.register(SubstringFunction);
    registry.register(StartsWithFunction);
    registry.register(EndsWithFunction);
    registry.register(ContainsFunction);

    // Math functions
    registry.register(AbsFunction);

    // Type functions
    registry.register(ToStringFunction);
    registry.register(ToIntegerFunction);
    registry.register(ToDecimalFunction);

    // Advanced functions with multiple parameter types
    registry.register(IifFunction);
    registry.register(WhereFunction);
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

        Ok(FhirPathValue::Integer(count as i64))
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
        Ok(FhirPathValue::Boolean(context.input.is_empty()))
    }
}

/// exists() function - returns true if the collection is not empty
struct ExistsFunction;

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
            Ok(FhirPathValue::Boolean(!context.input.is_empty()))
        } else {
            // TODO: Implement exists with criteria
            Ok(FhirPathValue::Boolean(false))
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
                    Ok(FhirPathValue::Empty)
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
                    Ok(FhirPathValue::Empty)
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
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
            FhirPathValue::Collection(items) => Ok(FhirPathValue::Integer(items.len() as i64)),
            FhirPathValue::Empty => Ok(FhirPathValue::Integer(0)),
            _ => Ok(FhirPathValue::Integer(1)),
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

                Ok(FhirPathValue::String(result))
            } else {
                Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "Integer".to_string(),
                    actual: args.get(0).map(|v| v.type_name()).unwrap_or("None").to_string(),
                })
            }
        } else {
            Ok(FhirPathValue::Empty)
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
            Ok(FhirPathValue::Boolean(s.starts_with(prefix)))
        } else {
            Ok(FhirPathValue::Boolean(false))
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
            Ok(FhirPathValue::Boolean(s.ends_with(suffix)))
        } else {
            Ok(FhirPathValue::Boolean(false))
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

        if let (FhirPathValue::String(s), Some(FhirPathValue::String(substring))) = (&context.input, args.get(0)) {
            Ok(FhirPathValue::Boolean(s.contains(substring)))
        } else {
            Ok(FhirPathValue::Boolean(false))
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

        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i.abs())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d.abs())),
            FhirPathValue::Quantity(q) => {
                let mut abs_q = q.clone();
                abs_q.value = abs_q.value.abs();
                Ok(FhirPathValue::Quantity(abs_q))
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

        match context.input.to_string_value() {
            Some(s) => Ok(FhirPathValue::String(s)),
            None => Ok(FhirPathValue::Empty),
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
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            FhirPathValue::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => Ok(FhirPathValue::Integer(i)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }
            FhirPathValue::Boolean(b) => Ok(FhirPathValue::Integer(if *b { 1 } else { 0 })),
            _ => Ok(FhirPathValue::Empty),
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
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(*d)),
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Decimal(rust_decimal::Decimal::from(*i))),
            FhirPathValue::String(s) => {
                match s.parse::<rust_decimal::Decimal>() {
                    Ok(d) => Ok(FhirPathValue::Decimal(d)),
                    Err(_) => Ok(FhirPathValue::Empty),
                }
            }
            _ => Ok(FhirPathValue::Empty),
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
                    Ok(FhirPathValue::Empty)
                }
            }
        }
    }

    fn documentation(&self) -> &str {
        "where(condition) - Returns a collection containing only items for which condition evaluates to true"
    }
}
