//! Function registry and built-in functions

// Import from modular structure
use crate::registry::cache::{
    CacheConfig, FunctionCacheKey, FunctionResolutionCache, FunctionResultCache,
};
use crate::registry::compiled_signatures::{CompilationStats, CompiledSignatureRegistry};
use crate::registry::functions::boolean::*;
use crate::registry::functions::cda::*;
use crate::registry::functions::collection::*;
use crate::registry::functions::datetime::*;
use crate::registry::functions::fhir_types::*;
use crate::registry::functions::filtering::*;
use crate::registry::functions::math::*;
use crate::registry::functions::string::*;
use crate::registry::functions::type_conversion::*;
use crate::registry::functions::utility::*;
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

// Re-export commonly used function types for external crates
// Note: Lambda evaluation is not yet fully implemented
// pub use crate::registry::functions::boolean::{AllFunction, AnyFunction};
// pub use crate::registry::functions::collection::ExistsFunction;
use crate::model::{FhirPathValue, TypeInfo};
use rustc_hash::FxHashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;
use thiserror::Error;

// For expression evaluation in lambda functions
use crate::ast::ExpressionNode;

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

/// Lambda evaluator type - takes an expression and context and returns a result (async)
pub type LambdaEvaluator<'a> = dyn Fn(
        &ExpressionNode,
        &FhirPathValue,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FhirPathValue, FunctionError>> + 'a>,
    > + 'a;

/// Enhanced lambda evaluator type that supports additional variables injection (async)
pub type EnhancedLambdaEvaluator<'a> = dyn Fn(
        &ExpressionNode,
        &FhirPathValue,
        &VarMap,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<FhirPathValue, FunctionError>> + 'a>,
    > + 'a;

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
    /// Enhanced lambda expression evaluator with variable injection support
    pub enhanced_evaluator: Option<&'a EnhancedLambdaEvaluator<'a>>,
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

/// Const generic trait for arity-specific functions with compile-time argument count checking
pub trait AritySpecificFunction<const ARITY: usize>: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with exactly ARITY arguments (compile-time checked)
    fn evaluate(
        &self,
        args: [FhirPathValue; ARITY],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Check if this function is pure (deterministic with no side effects)
    fn is_pure(&self) -> bool {
        false // Default to non-pure for safety
    }
}

/// Wrapper that converts arity-specific functions to the general FhirPathFunction trait
pub struct ArityWrapper<T, const ARITY: usize> {
    inner: T,
}

impl<T: AritySpecificFunction<ARITY>, const ARITY: usize> ArityWrapper<T, ARITY> {
    /// Create a new arity wrapper
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: AritySpecificFunction<ARITY>, const ARITY: usize> FhirPathFunction
    for ArityWrapper<T, ARITY>
{
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn human_friendly_name(&self) -> &str {
        self.inner.human_friendly_name()
    }

    fn signature(&self) -> &FunctionSignature {
        self.inner.signature()
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Compile-time arity checking - convert slice to array
        if args.len() != ARITY {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: ARITY,
                max: Some(ARITY),
                actual: args.len(),
            });
        }

        // Convert slice to array - this is safe because we checked the length
        let array_args: [FhirPathValue; ARITY] =
            args.to_vec()
                .try_into()
                .map_err(|_| FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Failed to convert arguments to fixed-size array".to_string(),
                })?;

        self.inner.evaluate(array_args, context)
    }

    fn documentation(&self) -> &str {
        self.inner.documentation()
    }

    fn is_pure(&self) -> bool {
        self.inner.is_pure()
    }
}

/// Specialized traits for common arities with more ergonomic interfaces
///
/// Nullary function (no arguments)
pub trait NullaryFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    /// Get the human-readable function name
    fn human_friendly_name(&self) -> &str;
    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;
    /// Evaluate the function with no arguments
    fn evaluate(&self, context: &EvaluationContext) -> FunctionResult<FhirPathValue>;
    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }
    /// Check if function is pure (no side effects)
    fn is_pure(&self) -> bool {
        false
    }
}

/// Unary function (one argument)
pub trait UnaryFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    /// Get the human-readable function name
    fn human_friendly_name(&self) -> &str;
    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;
    /// Evaluate the function with one argument
    fn evaluate(
        &self,
        arg: FhirPathValue,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }
    /// Check if function is pure (no side effects)
    fn is_pure(&self) -> bool {
        false
    }
}

/// Binary function (two arguments)
pub trait BinaryFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    /// Get the human-readable function name
    fn human_friendly_name(&self) -> &str;
    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;
    /// Evaluate the function with two arguments
    fn evaluate(
        &self,
        arg1: FhirPathValue,
        arg2: FhirPathValue,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }
    /// Check if function is pure (no side effects)
    fn is_pure(&self) -> bool {
        false
    }
}

/// Ternary function (three arguments)
pub trait TernaryFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    /// Get the human-readable function name
    fn human_friendly_name(&self) -> &str;
    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;
    /// Evaluate the function with three arguments
    fn evaluate(
        &self,
        arg1: FhirPathValue,
        arg2: FhirPathValue,
        arg3: FhirPathValue,
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }
    /// Check if function is pure (no side effects)
    fn is_pure(&self) -> bool {
        false
    }
}

// Implement AritySpecificFunction for the specialized traits
impl<T: NullaryFunction> AritySpecificFunction<0> for T {
    fn name(&self) -> &str {
        self.name()
    }
    fn human_friendly_name(&self) -> &str {
        self.human_friendly_name()
    }
    fn signature(&self) -> &FunctionSignature {
        self.signature()
    }
    fn evaluate(
        &self,
        _args: [FhirPathValue; 0],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.evaluate(context)
    }
    fn documentation(&self) -> &str {
        self.documentation()
    }
    fn is_pure(&self) -> bool {
        self.is_pure()
    }
}

impl<T: UnaryFunction> AritySpecificFunction<1> for T {
    fn name(&self) -> &str {
        self.name()
    }
    fn human_friendly_name(&self) -> &str {
        self.human_friendly_name()
    }
    fn signature(&self) -> &FunctionSignature {
        self.signature()
    }
    fn evaluate(
        &self,
        args: [FhirPathValue; 1],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        let [arg] = args;
        self.evaluate(arg, context)
    }
    fn documentation(&self) -> &str {
        self.documentation()
    }
    fn is_pure(&self) -> bool {
        self.is_pure()
    }
}

impl<T: BinaryFunction> AritySpecificFunction<2> for T {
    fn name(&self) -> &str {
        self.name()
    }
    fn human_friendly_name(&self) -> &str {
        self.human_friendly_name()
    }
    fn signature(&self) -> &FunctionSignature {
        self.signature()
    }
    fn evaluate(
        &self,
        args: [FhirPathValue; 2],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        let [arg1, arg2] = args;
        self.evaluate(arg1, arg2, context)
    }
    fn documentation(&self) -> &str {
        self.documentation()
    }
    fn is_pure(&self) -> bool {
        self.is_pure()
    }
}

impl<T: TernaryFunction> AritySpecificFunction<3> for T {
    fn name(&self) -> &str {
        self.name()
    }
    fn human_friendly_name(&self) -> &str {
        self.human_friendly_name()
    }
    fn signature(&self) -> &FunctionSignature {
        self.signature()
    }
    fn evaluate(
        &self,
        args: [FhirPathValue; 3],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        let [arg1, arg2, arg3] = args;
        self.evaluate(arg1, arg2, arg3, context)
    }
    fn documentation(&self) -> &str {
        self.documentation()
    }
    fn is_pure(&self) -> bool {
        self.is_pure()
    }
}

/// Synchronous trait for implementing FHIRPath functions (backward compatibility)
/// This trait will be deprecated in favor of AsyncFhirPathFunction
pub trait FhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with given arguments
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Check if this function is pure (deterministic with no side effects)
    /// Pure functions can be safely cached based on their arguments
    fn is_pure(&self) -> bool {
        false // Default to non-pure for safety
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

/// Async trait for implementing FHIRPath functions
/// This is the preferred trait for new function implementations
#[async_trait]
pub trait AsyncFhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with given arguments (async)
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Check if this function is pure (deterministic with no side effects)
    /// Pure functions can be safely cached based on their arguments
    fn is_pure(&self) -> bool {
        false // Default to non-pure for safety
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

/// Backward compatibility wrapper trait for existing synchronous function implementations
/// This allows existing sync functions to be used as async functions
pub trait SyncFhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;

    /// Get the human-friendly name for the function (for LSP and documentation)
    fn human_friendly_name(&self) -> &str;

    /// Get the function signature
    fn signature(&self) -> &FunctionSignature;

    /// Evaluate the function with given arguments (synchronous)
    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;

    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }

    /// Check if this function is pure (deterministic with no side effects)
    /// Pure functions can be safely cached based on their arguments
    fn is_pure(&self) -> bool {
        false // Default to non-pure for safety
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
#[async_trait(?Send)]
pub trait LambdaFunction: FhirPathFunction {
    /// Evaluate function with lambda expressions
    async fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue>;
}

/// Hybrid function implementation supporting both trait-based and closure-based functions
#[derive(Clone)]
pub enum FunctionImpl {
    /// Traditional trait-based function
    Trait(Arc<dyn FhirPathFunction>),
    /// Synchronous function implementation
    Sync(Arc<dyn SyncFhirPathFunction>),
    /// Asynchronous function implementation
    Async(Arc<dyn AsyncFhirPathFunction>),
    /// Lightweight closure-based function
    Closure {
        /// Function name
        name: String,
        /// Human-friendly name
        friendly_name: String,
        /// Function signature
        signature: FunctionSignature,
        /// Documentation
        documentation: String,
        /// The actual function implementation
        func: Arc<
            dyn Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue>
                + Send
                + Sync,
        >,
    },
}

impl std::fmt::Debug for FunctionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionImpl::Trait(func) => {
                f.debug_struct("Trait").field("name", &func.name()).finish()
            }
            FunctionImpl::Sync(func) => f.debug_struct("Sync").field("name", &func.name()).finish(),
            FunctionImpl::Async(func) => {
                f.debug_struct("Async").field("name", &func.name()).finish()
            }
            FunctionImpl::Closure {
                name,
                friendly_name,
                ..
            } => f
                .debug_struct("Closure")
                .field("name", name)
                .field("friendly_name", friendly_name)
                .finish(),
        }
    }
}

impl FunctionImpl {
    /// Get the function name
    pub fn name(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.name(),
            FunctionImpl::Sync(f) => f.name(),
            FunctionImpl::Async(f) => f.name(),
            FunctionImpl::Closure { name, .. } => name,
        }
    }

    /// Get the human-friendly name
    pub fn human_friendly_name(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.human_friendly_name(),
            FunctionImpl::Sync(f) => f.human_friendly_name(),
            FunctionImpl::Async(f) => f.human_friendly_name(),
            FunctionImpl::Closure { friendly_name, .. } => friendly_name,
        }
    }

    /// Get the function signature
    pub fn signature(&self) -> &FunctionSignature {
        match self {
            FunctionImpl::Trait(f) => f.signature(),
            FunctionImpl::Sync(f) => f.signature(),
            FunctionImpl::Async(f) => f.signature(),
            FunctionImpl::Closure { signature, .. } => signature,
        }
    }

    /// Get function documentation
    pub fn documentation(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.documentation(),
            FunctionImpl::Sync(f) => f.documentation(),
            FunctionImpl::Async(f) => f.documentation(),
            FunctionImpl::Closure { documentation, .. } => documentation,
        }
    }

    /// Check if this function is pure
    pub fn is_pure(&self) -> bool {
        match self {
            FunctionImpl::Trait(f) => f.is_pure(),
            FunctionImpl::Sync(f) => f.is_pure(),
            FunctionImpl::Async(f) => f.is_pure(),
            FunctionImpl::Closure { .. } => false, // Default to non-pure for closure functions
        }
    }

    /// Evaluate the function
    pub fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        match self {
            FunctionImpl::Trait(f) => f.evaluate(args, context),
            FunctionImpl::Sync(f) => f.evaluate_sync(args, context),
            FunctionImpl::Async(_f) => {
                // For now, async functions are not supported in sync context
                // This will be handled by the async evaluate method
                Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Async function called in sync context".to_string(),
                })
            }
            FunctionImpl::Closure { func, .. } => func(args, context),
        }
    }

    /// Evaluate the function asynchronously
    pub async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        match self {
            FunctionImpl::Sync(func) => func.evaluate_sync(args, context),
            FunctionImpl::Async(func) => func.evaluate(args, context).await,
            FunctionImpl::Trait(func) => func.evaluate(args, context),
            FunctionImpl::Closure { func, .. } => func(args, context),
        }
    }

    /// Validate arguments
    pub fn validate_args(&self, args: &[FhirPathValue]) -> FunctionResult<()> {
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

        // Type validation would go here if needed
        Ok(())
    }
}

/// Enhanced registry for FHIRPath functions supporting hybrid registration
#[derive(Clone)]
pub struct FunctionRegistry {
    functions: FxHashMap<String, FunctionImpl>,
    signatures: FxHashMap<String, Vec<FunctionSignature>>,
    /// Cache for resolved functions by name and argument types
    resolution_cache: Arc<FunctionResolutionCache>,
    /// Cache for pure function results
    result_cache: Arc<FunctionResultCache>,
    /// Cache configuration
    cache_config: Arc<CacheConfig>,
    /// Pre-compiled signatures for ultra-fast dispatch
    compiled_signatures: Arc<std::sync::Mutex<CompiledSignatureRegistry>>,
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let compilation_stats = if let Ok(compiled) = self.compiled_signatures.lock() {
            compiled.compilation_stats()
        } else {
            CompilationStats {
                total_functions: 0,
                total_signatures: 0,
                specialized_count: 0,
                dispatch_entries: 0,
                avg_signatures_per_function: 0.0,
            }
        };

        f.debug_struct("FunctionRegistry")
            .field("function_count", &self.functions.len())
            .field("signature_count", &self.signatures.len())
            .field("resolution_cache", &self.resolution_cache)
            .field("result_cache", &self.result_cache)
            .field("cache_config", &self.cache_config)
            .field("compiled_signatures_stats", &compilation_stats.to_string())
            .finish()
    }
}

impl FunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new registry with custom cache configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let resolution_cache = Arc::new(FunctionResolutionCache::new(config.clone()));
        let result_cache = Arc::new(FunctionResultCache::new(config.clone()));

        Self {
            functions: FxHashMap::default(),
            signatures: FxHashMap::default(),
            resolution_cache,
            result_cache,
            cache_config: Arc::new(config),
            compiled_signatures: Arc::new(std::sync::Mutex::new(CompiledSignatureRegistry::new())),
        }
    }

    /// Register a trait-based function
    pub fn register<F: FhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        let signature = function.signature().clone();
        let func_impl = FunctionImpl::Trait(Arc::new(function));

        self.functions.insert(name.clone(), func_impl);
        self.signatures
            .entry(name.clone())
            .or_default()
            .push(signature.clone());

        // Compile the signature for fast dispatch
        if let Ok(mut compiled) = self.compiled_signatures.lock() {
            compiled.register_signature(name, signature);
        }
    }

    /// Register a synchronous function
    pub fn register_sync<F: SyncFhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        let signature = function.signature().clone();
        let func_impl = FunctionImpl::Sync(Arc::new(function));

        self.functions.insert(name.clone(), func_impl);
        self.signatures
            .entry(name.clone())
            .or_default()
            .push(signature.clone());

        // Compile the signature for fast dispatch
        if let Ok(mut compiled) = self.compiled_signatures.lock() {
            compiled.register_signature(name, signature);
        }
    }

    /// Register an asynchronous function
    pub fn register_async<F: AsyncFhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        let signature = function.signature().clone();
        let func_impl = FunctionImpl::Async(Arc::new(function));

        self.functions.insert(name.clone(), func_impl);
        self.signatures
            .entry(name.clone())
            .or_default()
            .push(signature.clone());

        // Compile the signature for fast dispatch
        if let Ok(mut compiled) = self.compiled_signatures.lock() {
            compiled.register_signature(name, signature);
        }
    }

    /// Register a closure-based function (new hybrid approach)
    pub fn register_closure<F>(
        &mut self,
        name: impl Into<String>,
        friendly_name: impl Into<String>,
        signature: FunctionSignature,
        documentation: impl Into<String>,
        func: F,
    ) where
        F: Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue>
            + Send
            + Sync
            + 'static,
    {
        let name = name.into();
        let func_impl = FunctionImpl::Closure {
            name: name.clone(),
            friendly_name: friendly_name.into(),
            signature: signature.clone(),
            documentation: documentation.into(),
            func: Arc::new(func),
        };

        self.functions.insert(name.clone(), func_impl);
        self.signatures
            .entry(name.clone())
            .or_default()
            .push(signature.clone());

        // Compile the signature for fast dispatch
        if let Ok(mut compiled) = self.compiled_signatures.lock() {
            compiled.register_signature(name, signature);
        }
    }

    /// Register a simple closure-based function with minimal boilerplate
    pub fn register_simple<F>(
        &mut self,
        name: impl Into<String>,
        min_arity: usize,
        max_arity: Option<usize>,
        func: F,
    ) where
        F: Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue>
            + Send
            + Sync
            + 'static,
    {
        let name_str = name.into();

        // Create parameter info for generic types based on arity
        let mut parameters = Vec::new();
        if let Some(max) = max_arity {
            for i in 0..max {
                let param_name = format!("arg{i}");
                let is_optional = i >= min_arity;
                if is_optional {
                    parameters.push(ParameterInfo::optional(param_name, TypeInfo::Any));
                } else {
                    parameters.push(ParameterInfo::required(param_name, TypeInfo::Any));
                }
            }
        } else {
            // For variadic functions, create min_arity required parameters
            for i in 0..min_arity {
                let param_name = format!("arg{i}");
                parameters.push(ParameterInfo::required(param_name, TypeInfo::Any));
            }
        }

        let signature = FunctionSignature {
            name: name_str.clone(),
            min_arity,
            max_arity,
            parameters,
            return_type: TypeInfo::Any,
        };

        self.register_closure(
            name_str.clone(),
            name_str.clone(),
            signature,
            format!("Auto-generated function: {name_str}"),
            func,
        );
    }

    /// Register an arity-specific function using const generics
    pub fn register_arity<F: AritySpecificFunction<ARITY> + 'static, const ARITY: usize>(
        &mut self,
        function: F,
    ) {
        let wrapper = ArityWrapper::new(function);
        self.register(wrapper);
    }

    /// Register a nullary function (0 arguments)
    pub fn register_nullary<F: NullaryFunction + 'static>(&mut self, function: F) {
        self.register_arity::<_, 0>(function);
    }

    /// Register a unary function (1 argument)
    pub fn register_unary<F: UnaryFunction + 'static>(&mut self, function: F) {
        self.register_arity::<_, 1>(function);
    }

    /// Register a binary function (2 arguments)
    pub fn register_binary<F: BinaryFunction + 'static>(&mut self, function: F) {
        self.register_arity::<_, 2>(function);
    }

    /// Register a ternary function (3 arguments)
    pub fn register_ternary<F: TernaryFunction + 'static>(&mut self, function: F) {
        self.register_arity::<_, 3>(function);
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<&FunctionImpl> {
        self.functions.get(name)
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

    /// Clear the resolved function cache
    pub fn clear_cache(&self) {
        self.resolution_cache.clear();
        self.result_cache.clear();
    }

    /// Get function by name and argument types with caching
    pub fn get_function_for_types(
        &self,
        name: &str,
        arg_types: &[TypeInfo],
    ) -> Option<Arc<FunctionImpl>> {
        // Create cache key
        let cache_key = FunctionCacheKey::new(name, arg_types.to_vec());

        // Check cache first
        if let Some(cached) = self.resolution_cache.get(&cache_key) {
            return Some(cached);
        }

        // Look up function and validate types
        if let Some(function) = self.get(name) {
            let sig = function.signature();
            if sig.matches(arg_types) {
                // Store in cache for future lookups
                let func_arc = Arc::new(function.clone());
                self.resolution_cache.insert(cache_key, func_arc.clone());
                return Some(func_arc);
            }
        }
        None
    }

    /// Get function implementation by name (new hybrid approach)
    pub fn get_impl(&self, name: &str) -> Option<&FunctionImpl> {
        self.functions.get(name)
    }

    /// Evaluate a function with the hybrid system and caching
    pub fn evaluate_function(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Try fast validation first using compiled signatures
        if let Ok(compiled) = self.compiled_signatures.lock() {
            if !compiled.validates_fast(name, args) {
                // Use compiled signatures for detailed error reporting
                return compiled
                    .validate_with_errors(name, args)
                    .map(|_| unreachable!());
            }
        }

        // Get argument types for cache lookup
        let arg_types: Vec<TypeInfo> = args.iter().map(|v| v.to_type_info()).collect();

        // Get function with type-based caching
        let function = self
            .get_function_for_types(name, &arg_types)
            .ok_or_else(|| FunctionError::EvaluationError {
                name: name.to_string(),
                message: "Function not found or type mismatch".to_string(),
            })?;

        // For pure functions, check result cache
        if self.is_pure_function(name) && self.cache_config.enable_result_caching {
            let result_key = crate::registry::cache::generate_result_cache_key(
                name, args, 0, // TODO: proper context hash
            );

            if let Some(cached_result) = self.result_cache.get(&result_key) {
                return Ok(cached_result);
            }

            // Evaluate and cache the result
            let result = function.evaluate(args, context)?;
            self.result_cache.insert(result_key, result.clone());
            Ok(result)
        } else {
            // Evaluate without result caching
            function.evaluate(args, context)
        }
    }

    /// Evaluate a function asynchronously with the hybrid system and caching
    pub async fn evaluate_function_async(
        &self,
        name: &str,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        // Try fast validation first using compiled signatures
        if let Ok(compiled) = self.compiled_signatures.lock() {
            if !compiled.validates_fast(name, args) {
                // Use compiled signatures for detailed error reporting
                return compiled
                    .validate_with_errors(name, args)
                    .map(|_| unreachable!());
            }
        }

        // Get argument types for cache lookup
        let arg_types: Vec<TypeInfo> = args.iter().map(|v| v.to_type_info()).collect();

        // Get function with type-based caching
        let function = self
            .get_function_for_types(name, &arg_types)
            .ok_or_else(|| FunctionError::EvaluationError {
                name: name.to_string(),
                message: "Function not found or type mismatch".to_string(),
            })?;

        // For pure functions, check result cache
        if self.is_pure_function(name) && self.cache_config.enable_result_caching {
            let result_key = crate::registry::cache::generate_result_cache_key(
                name, args, 0, // TODO: proper context hash
            );

            if let Some(cached_result) = self.result_cache.get(&result_key) {
                return Ok(cached_result);
            }

            // Evaluate and cache the result
            let result = function.evaluate_async(args, context).await?;
            self.result_cache.insert(result_key, result.clone());
            Ok(result)
        } else {
            // Evaluate without result caching
            function.evaluate_async(args, context).await
        }
    }

    /// Get best matching signature for given argument types
    pub fn get_best_signature(
        &self,
        name: &str,
        arg_types: &[TypeInfo],
    ) -> Option<&FunctionSignature> {
        self.get_signatures(name)?
            .iter()
            .find(|sig| sig.matches(arg_types))
    }

    /// Check if a function is pure (deterministic with no side effects)
    pub fn is_pure_function(&self, name: &str) -> bool {
        if let Some(function) = self.get(name) {
            function.is_pure()
        } else {
            false
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (String, String) {
        (
            self.resolution_cache.stats().summary(),
            self.result_cache.stats().summary(),
        )
    }

    /// Get compiled signature statistics
    pub fn compiled_signature_stats(&self) -> Option<CompilationStats> {
        self.compiled_signatures
            .lock()
            .ok()
            .map(|registry| registry.compilation_stats())
    }

    /// Warm compiled signature dispatch table with common type combinations
    pub fn warm_compiled_signatures(&self, common_combinations: Vec<(String, Vec<TypeInfo>)>) {
        if let Ok(mut compiled) = self.compiled_signatures.lock() {
            compiled.warm_dispatch_table(common_combinations);
        }
    }

    /// Fast validation using compiled signatures (useful for pre-validation)
    pub fn validates_fast(&self, name: &str, args: &[FhirPathValue]) -> bool {
        self.compiled_signatures
            .lock()
            .map(|registry| registry.validates_fast(name, args))
            .unwrap_or(false)
    }

    /// Generate documentation from function traits
    pub fn generate_function_docs(&self) -> String {
        let mut docs = String::new();
        docs.push_str("# FHIRPath Function Documentation\n\n");

        // Sort functions by name for consistent output
        let mut functions: Vec<_> = self.functions.iter().collect();
        functions.sort_by_key(|(name, _)| *name);

        for (name, function) in functions {
            docs.push_str(&format!("## {}\n\n", function.human_friendly_name()));
            docs.push_str(&format!("**Function Name:** `{name}`\n\n"));

            // Add signature information
            let sig = function.signature();
            docs.push_str("**Signature:**\n");
            docs.push_str(&format!(
                "- **Arity:** {}{}\n",
                sig.min_arity,
                if let Some(max) = sig.max_arity {
                    format!("-{max}")
                } else {
                    "+".to_string()
                }
            ));
            docs.push_str(&format!("- **Return Type:** {}\n", sig.return_type));

            if !sig.parameters.is_empty() {
                docs.push_str("- **Parameters:**\n");
                for param in sig.parameters.iter() {
                    docs.push_str(&format!(
                        "  - `{}` ({}): {}\n",
                        param.name,
                        param.param_type,
                        if param.optional {
                            "optional"
                        } else {
                            "required"
                        }
                    ));
                }
            }

            // Add documentation
            let doc = function.documentation();
            if !doc.is_empty() {
                docs.push_str("\n**Description:**\n");
                docs.push_str(doc);
                docs.push('\n');
            }

            // Add purity information
            if function.is_pure() {
                docs.push_str("\n*This function is pure (deterministic with no side effects) and results may be cached.*\n");
            }

            docs.push_str("\n---\n\n");
        }

        docs
    }

    /// Generate markdown documentation for a specific function
    pub fn generate_function_doc(&self, name: &str) -> Option<String> {
        let function = self.get(name)?;
        let sig = function.signature();

        let mut doc = String::new();
        doc.push_str(&format!("# {}\n\n", function.human_friendly_name()));
        doc.push_str(&format!("**Function:** `{name}`\n\n"));

        // Signature details
        doc.push_str("## Signature\n\n");
        doc.push_str(&format!(
            "- **Arity:** {}{}\n",
            sig.min_arity,
            if let Some(max) = sig.max_arity {
                format!("-{max}")
            } else {
                "+".to_string()
            }
        ));
        doc.push_str(&format!("- **Return Type:** {}\n\n", sig.return_type));

        if !sig.parameters.is_empty() {
            doc.push_str("### Parameters\n\n");
            for param in &sig.parameters {
                doc.push_str(&format!(
                    "- **{}** ({}): {}\n",
                    param.name,
                    param.param_type,
                    if param.optional {
                        "optional"
                    } else {
                        "required"
                    }
                ));
            }
            doc.push('\n');
        }

        // Description
        let description = function.documentation();
        if !description.is_empty() {
            doc.push_str("## Description\n\n");
            doc.push_str(description);
            doc.push_str("\n\n");
        }

        // Properties
        doc.push_str("## Properties\n\n");
        if function.is_pure() {
            doc.push_str("- **Pure Function**: Yes (deterministic, results may be cached)\n");
        } else {
            doc.push_str(
                "- **Pure Function**: No (may have side effects or non-deterministic behavior)\n",
            );
        }

        Some(doc)
    }

    /// Generate JSON documentation for all functions
    pub fn generate_function_docs_json(&self) -> serde_json::Value {
        use serde_json::{Map, Value, json};

        let functions: Map<String, Value> = self
            .functions
            .iter()
            .map(|(name, function)| {
                let sig = function.signature();
                let params: Vec<Value> = sig
                    .parameters
                    .iter()
                    .map(|p| {
                        json!({
                            "name": p.name,
                            "type": p.param_type.to_string(),
                            "optional": p.optional
                        })
                    })
                    .collect();

                let function_info = json!({
                    "name": name,
                    "friendly_name": function.human_friendly_name(),
                    "signature": {
                        "min_arity": sig.min_arity,
                        "max_arity": sig.max_arity,
                        "parameters": params,
                        "return_type": sig.return_type.to_string()
                    },
                    "documentation": function.documentation(),
                    "is_pure": function.is_pure()
                });

                (name.clone(), function_info)
            })
            .collect();

        json!({
            "functions": functions,
            "total_count": self.functions.len(),
            "generated_at": chrono::Utc::now().to_rfc3339()
        })
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Register all built-in FHIRPath functions
pub fn register_builtin_functions(registry: &mut FunctionRegistry) {
    // Collection functions - async converted
    registry.register_async(CountFunction);
    registry.register_async(EmptyFunction);
    registry.register_async(DescendantsFunction);
    registry.register_async(ChildrenFunction);
    registry.register_async(FirstFunction);
    registry.register_async(LastFunction);
    registry.register_async(LengthFunction);
    registry.register_async(DistinctFunction);
    registry.register_async(SingleFunction);
    registry.register_async(IntersectFunction);
    registry.register_async(ExcludeFunction);
    registry.register_async(CombineFunction);
    registry.register_async(TailFunction);
    registry.register_async(SubsetOfFunction);
    registry.register_async(SupersetOfFunction);

    // Collection functions - still using old trait (lambda functions)
    registry.register(ExistsFunction);
    registry.register(AggregateFunction);
    registry.register(SortFunction);
    registry.register_async(TakeFunction);
    registry.register_async(SkipFunction);

    // Boolean functions
    registry.register(AllFunction);
    registry.register_async(AllTrueFunction);
    registry.register_async(AnyFunction);
    registry.register_async(IsDistinctFunction);
    registry.register_async(NotFunction);

    // String functions
    registry.register_async(SubstringFunction);
    registry.register_async(StartsWithFunction);
    registry.register_async(EndsWithFunction);
    registry.register_async(ContainsFunction);
    registry.register_async(MatchesFunction);
    registry.register_async(MatchesFullFunction);
    registry.register_async(ReplaceFunction);
    registry.register_async(ReplaceMatchesFunction);
    registry.register_async(SplitFunction);
    registry.register_async(JoinFunction);
    registry.register_async(TrimFunction);
    registry.register_async(ToCharsFunction);
    registry.register_async(IndexOfFunction);
    registry.register_async(UpperFunction);
    registry.register_async(LowerFunction);
    registry.register_async(EncodeFunction);
    registry.register_async(DecodeFunction);
    registry.register_async(EscapeFunction);
    registry.register_async(UnescapeFunction);

    // Math functions
    registry.register_async(AbsFunction);
    registry.register_async(CeilingFunction);
    registry.register_async(FloorFunction);
    registry.register_async(RoundFunction);
    registry.register_async(SqrtFunction);
    registry.register_async(TruncateFunction);
    registry.register_async(ExpFunction);
    registry.register_async(LnFunction);
    registry.register_async(LogFunction);
    registry.register_async(PowerFunction);
    registry.register_async(PrecisionFunction);

    // Aggregate functions
    registry.register_async(SumFunction);
    registry.register_async(AvgFunction);
    registry.register_async(MinFunction);
    registry.register_async(MaxFunction);

    // Type conversion functions
    registry.register_async(AsFunction);
    registry.register_async(ToStringFunction);
    registry.register_async(ToIntegerFunction);
    registry.register_async(ToDecimalFunction);
    registry.register_async(ToBooleanFunction);
    registry.register_async(TypeFunction);
    registry.register(ConvertsToIntegerFunction);
    registry.register(ConvertsToDecimalFunction);
    registry.register(ConvertsToStringFunction);
    registry.register_async(ConvertsToBooleanFunction);
    registry.register(ConvertsToDateFunction);
    registry.register(ConvertsToDateTimeFunction);
    registry.register(ConvertsToTimeFunction);
    registry.register_async(ToQuantityFunction);
    registry.register(ConvertsToQuantityFunction);

    // Filtering functions
    registry.register(WhereFunction);
    registry.register(SelectFunction);
    registry.register_async(OfTypeFunction);

    // DateTime functions
    registry.register_async(NowFunction);
    registry.register_async(TodayFunction);
    registry.register_async(LowBoundaryFunction);
    registry.register_async(HighBoundaryFunction);

    // Utility functions
    registry.register_async(IifFunction);
    registry.register_async(TraceFunction);
    registry.register_async(ConformsToFunction::new());
    registry.register_async(DefineVariableFunction);
    registry.register_async(HasValueFunction);
    registry.register_async(RepeatFunction);

    // FHIR type functions
    registry.register_async(IsFunction);
    registry.register_async(ComparableFunction);
    registry.register_async(ExtensionFunction);
    registry.register_async(ResolveFunction);

    // CDA functions
    registry.register(HasTemplateIdOfFunction);

    // Warm cache with common function lookups if enabled
    if registry.cache_config.warm_cache_on_init {
        warm_function_cache(registry);
        warm_compiled_signatures(registry);
    }
}

/// Warm the function cache with common function lookups
fn warm_function_cache(registry: &FunctionRegistry) {
    // Common type combinations for frequently used functions
    let common_types = vec![
        vec![TypeInfo::String],
        vec![TypeInfo::Integer],
        vec![TypeInfo::Decimal],
        vec![TypeInfo::Boolean],
        vec![TypeInfo::Collection(Box::new(TypeInfo::String))],
        vec![TypeInfo::Collection(Box::new(TypeInfo::Integer))],
        vec![TypeInfo::String, TypeInfo::String],
        vec![TypeInfo::Integer, TypeInfo::Integer],
    ];

    // Most frequently used functions to pre-cache
    let frequent_functions = [
        "count",
        "length",
        "empty",
        "exists",
        "first",
        "last",
        "substring",
        "startsWith",
        "endsWith",
        "contains",
        "toString",
        "toInteger",
        "abs",
        "floor",
        "ceiling",
        "where",
        "select",
        "all",
        "any",
        "distinct",
    ];

    for function_name in &frequent_functions {
        for arg_types in &common_types {
            // Attempt to cache the function lookup
            let _ = registry.get_function_for_types(function_name, arg_types);
        }
    }
}

/// Warm the compiled signature dispatch table with common type combinations
fn warm_compiled_signatures(registry: &FunctionRegistry) {
    // Common type combinations for frequently used functions
    let common_combinations = vec![
        // Collection functions (nullary)
        ("count".to_string(), vec![]),
        ("empty".to_string(), vec![]),
        ("exists".to_string(), vec![]),
        ("first".to_string(), vec![]),
        ("last".to_string(), vec![]),
        ("length".to_string(), vec![]),
        // Unary functions with various types
        ("toString".to_string(), vec![TypeInfo::Integer]),
        ("toString".to_string(), vec![TypeInfo::Decimal]),
        ("toString".to_string(), vec![TypeInfo::Boolean]),
        ("toInteger".to_string(), vec![TypeInfo::String]),
        ("toInteger".to_string(), vec![TypeInfo::Decimal]),
        ("abs".to_string(), vec![TypeInfo::Integer]),
        ("abs".to_string(), vec![TypeInfo::Decimal]),
        ("floor".to_string(), vec![TypeInfo::Decimal]),
        ("ceiling".to_string(), vec![TypeInfo::Decimal]),
        // Binary string functions
        (
            "substring".to_string(),
            vec![TypeInfo::String, TypeInfo::Integer],
        ),
        (
            "substring".to_string(),
            vec![TypeInfo::String, TypeInfo::Integer, TypeInfo::Integer],
        ),
        (
            "startsWith".to_string(),
            vec![TypeInfo::String, TypeInfo::String],
        ),
        (
            "endsWith".to_string(),
            vec![TypeInfo::String, TypeInfo::String],
        ),
        (
            "contains".to_string(),
            vec![TypeInfo::String, TypeInfo::String],
        ),
        // Collection with various element types
        (
            "count".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::String))],
        ),
        (
            "count".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::Integer))],
        ),
        (
            "first".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::Any))],
        ),
        (
            "last".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::Any))],
        ),
        // Math operations
        ("abs".to_string(), vec![TypeInfo::Integer]),
        ("abs".to_string(), vec![TypeInfo::Decimal]),
        // Boolean operations
        (
            "all".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::Boolean))],
        ),
        (
            "any".to_string(),
            vec![TypeInfo::Collection(Box::new(TypeInfo::Boolean))],
        ),
    ];

    registry.warm_compiled_signatures(common_combinations);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FhirPathValue, TypeInfo};
    use crate::registry::signature::{FunctionSignature, ParameterInfo};

    // Example trait-based function for testing
    #[derive(Debug)]
    struct TestTraitFunction;

    impl FhirPathFunction for TestTraitFunction {
        fn name(&self) -> &str {
            "testTrait"
        }

        fn human_friendly_name(&self) -> &str {
            "Test Trait Function"
        }

        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIGNATURE.get_or_init(|| FunctionSignature {
                name: "testTrait".to_string(),
                min_arity: 1,
                max_arity: Some(1),
                parameters: vec![ParameterInfo::required("input", TypeInfo::Integer)],
                return_type: TypeInfo::Integer,
            })
        }

        fn evaluate(
            &self,
            args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            if let Some(FhirPathValue::Integer(n)) = args.first() {
                Ok(FhirPathValue::Integer(n * 2))
            } else {
                Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Expected integer argument".to_string(),
                })
            }
        }

        fn documentation(&self) -> &str {
            "Doubles the input integer using trait-based implementation"
        }
    }

    #[test]
    fn test_hybrid_registration_trait_function() {
        let mut registry = FunctionRegistry::new();

        // Register trait-based function
        registry.register(TestTraitFunction);

        // Verify registration
        assert!(registry.contains("testTrait"));

        let function = registry.get("testTrait").unwrap();
        assert_eq!(function.name(), "testTrait");
        assert_eq!(function.human_friendly_name(), "Test Trait Function");
        assert_eq!(
            function.documentation(),
            "Doubles the input integer using trait-based implementation"
        );

        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(21)];

        let result = registry
            .evaluate_function("testTrait", &args, &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_hybrid_registration_closure_function() {
        let mut registry = FunctionRegistry::new();

        // Register closure-based function
        let signature = FunctionSignature {
            name: "testClosure".to_string(),
            min_arity: 1,
            max_arity: Some(1),
            parameters: vec![ParameterInfo::required("input", TypeInfo::Integer)],
            return_type: TypeInfo::Integer,
        };

        registry.register_closure(
            "testClosure",
            "Test Closure Function",
            signature,
            "Triples the input integer using closure-based implementation",
            |args, _context| {
                if let Some(FhirPathValue::Integer(n)) = args.first() {
                    Ok(FhirPathValue::Integer(n * 3))
                } else {
                    Err(FunctionError::EvaluationError {
                        name: "testClosure".to_string(),
                        message: "Expected integer argument".to_string(),
                    })
                }
            },
        );

        // Verify registration
        assert!(registry.contains("testClosure"));

        let function = registry.get("testClosure").unwrap();
        assert_eq!(function.name(), "testClosure");
        assert_eq!(function.human_friendly_name(), "Test Closure Function");
        assert_eq!(
            function.documentation(),
            "Triples the input integer using closure-based implementation"
        );

        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(14)];

        let result = registry
            .evaluate_function("testClosure", &args, &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_hybrid_registration_simple_function() {
        let mut registry = FunctionRegistry::new();

        // Register simple closure-based function with minimal boilerplate
        registry.register_simple("add10", 1, Some(1), |args, _context| {
            if let Some(FhirPathValue::Integer(n)) = args.first() {
                Ok(FhirPathValue::Integer(n + 10))
            } else {
                Err(FunctionError::EvaluationError {
                    name: "add10".to_string(),
                    message: "Expected integer argument".to_string(),
                })
            }
        });

        // Verify registration
        assert!(registry.contains("add10"));

        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(32)];

        let result = registry
            .evaluate_function("add10", &args, &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_mixed_function_types() {
        let mut registry = FunctionRegistry::new();

        // Register both trait-based and closure-based functions
        registry.register(TestTraitFunction);
        registry.register_simple("multiply5", 1, Some(1), |args, _context| {
            if let Some(FhirPathValue::Integer(n)) = args.first() {
                Ok(FhirPathValue::Integer(n * 5))
            } else {
                Ok(FhirPathValue::Empty)
            }
        });

        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(8)];

        // Test trait-based function
        let result1 = registry
            .evaluate_function("testTrait", &args, &context)
            .unwrap();
        assert_eq!(result1, FhirPathValue::Integer(16)); // 8 * 2

        // Test closure-based function
        let result2 = registry
            .evaluate_function("multiply5", &args, &context)
            .unwrap();
        assert_eq!(result2, FhirPathValue::Integer(40)); // 8 * 5
    }

    #[test]
    fn test_documentation_generation() {
        let mut registry = FunctionRegistry::new();

        // Register a test function
        registry.register(TestTraitFunction);

        // Test markdown documentation generation
        let docs = registry.generate_function_docs();
        assert!(docs.contains("# FHIRPath Function Documentation"));
        assert!(docs.contains("## Test Trait Function"));
        assert!(docs.contains("**Function Name:** `testTrait`"));
        assert!(docs.contains("Doubles the input integer using trait-based implementation"));

        // Test individual function documentation
        let single_doc = registry.generate_function_doc("testTrait").unwrap();
        assert!(single_doc.contains("# Test Trait Function"));
        assert!(single_doc.contains("**Function:** `testTrait`"));
        assert!(single_doc.contains("## Signature"));
        assert!(single_doc.contains("## Description"));

        // Test JSON documentation generation
        let json_docs = registry.generate_function_docs_json();
        let functions = json_docs["functions"].as_object().unwrap();
        let test_trait = &functions["testTrait"];

        assert_eq!(test_trait["name"], "testTrait");
        assert_eq!(test_trait["friendly_name"], "Test Trait Function");
        assert_eq!(
            test_trait["documentation"],
            "Doubles the input integer using trait-based implementation"
        );
        assert_eq!(test_trait["signature"]["min_arity"], 1);
        assert_eq!(test_trait["signature"]["max_arity"], 1);
        assert_eq!(test_trait["signature"]["return_type"], "Integer");
    }

    #[test]
    fn test_documentation_generation_empty_registry() {
        let registry = FunctionRegistry::new();

        // Test empty registry
        let docs = registry.generate_function_docs();
        assert_eq!(docs, "# FHIRPath Function Documentation\n\n");

        let json_docs = registry.generate_function_docs_json();
        assert_eq!(json_docs["total_count"], 0);

        // Test non-existent function
        let single_doc = registry.generate_function_doc("nonexistent");
        assert!(single_doc.is_none());
    }

    // Test const generic arity-specific functions
    #[derive(Debug)]
    struct TestNullaryFunction;

    impl NullaryFunction for TestNullaryFunction {
        fn name(&self) -> &str {
            "testNullary"
        }
        fn human_friendly_name(&self) -> &str {
            "Test Nullary Function"
        }
        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIGNATURE.get_or_init(|| FunctionSignature {
                name: "testNullary".to_string(),
                min_arity: 0,
                max_arity: Some(0),
                parameters: vec![],
                return_type: TypeInfo::Integer,
            })
        }
        fn evaluate(&self, _context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
            Ok(FhirPathValue::Integer(42))
        }
        fn documentation(&self) -> &str {
            "Returns 42"
        }
        fn is_pure(&self) -> bool {
            true
        }
    }

    #[derive(Debug)]
    struct TestUnaryFunction;

    impl UnaryFunction for TestUnaryFunction {
        fn name(&self) -> &str {
            "testUnary"
        }
        fn human_friendly_name(&self) -> &str {
            "Test Unary Function"
        }
        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIGNATURE.get_or_init(|| FunctionSignature {
                name: "testUnary".to_string(),
                min_arity: 1,
                max_arity: Some(1),
                parameters: vec![ParameterInfo::required("x", TypeInfo::Integer)],
                return_type: TypeInfo::Integer,
            })
        }
        fn evaluate(
            &self,
            arg: FhirPathValue,
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            if let FhirPathValue::Integer(n) = arg {
                Ok(FhirPathValue::Integer(n + 1))
            } else {
                Err(FunctionError::EvaluationError {
                    name: UnaryFunction::name(self).to_string(),
                    message: "Expected integer".to_string(),
                })
            }
        }
        fn documentation(&self) -> &str {
            "Adds 1 to the input"
        }
        fn is_pure(&self) -> bool {
            true
        }
    }

    #[derive(Debug)]
    struct TestBinaryFunction;

    impl BinaryFunction for TestBinaryFunction {
        fn name(&self) -> &str {
            "testBinary"
        }
        fn human_friendly_name(&self) -> &str {
            "Test Binary Function"
        }
        fn signature(&self) -> &FunctionSignature {
            static SIGNATURE: std::sync::OnceLock<FunctionSignature> = std::sync::OnceLock::new();
            SIGNATURE.get_or_init(|| FunctionSignature {
                name: "testBinary".to_string(),
                min_arity: 2,
                max_arity: Some(2),
                parameters: vec![
                    ParameterInfo::required("x", TypeInfo::Integer),
                    ParameterInfo::required("y", TypeInfo::Integer),
                ],
                return_type: TypeInfo::Integer,
            })
        }
        fn evaluate(
            &self,
            arg1: FhirPathValue,
            arg2: FhirPathValue,
            _context: &EvaluationContext,
        ) -> FunctionResult<FhirPathValue> {
            if let (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) = (arg1, arg2) {
                Ok(FhirPathValue::Integer(a + b))
            } else {
                Err(FunctionError::EvaluationError {
                    name: BinaryFunction::name(self).to_string(),
                    message: "Expected integers".to_string(),
                })
            }
        }
        fn documentation(&self) -> &str {
            "Adds two integers"
        }
        fn is_pure(&self) -> bool {
            true
        }
    }

    #[test]
    fn test_const_generic_nullary_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_nullary(TestNullaryFunction);

        assert!(registry.contains("testNullary"));

        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Test valid call (no arguments)
        let result = registry
            .evaluate_function("testNullary", &[], &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test invalid call (with arguments)
        let invalid_result =
            registry.evaluate_function("testNullary", &[FhirPathValue::Integer(1)], &context);
        assert!(invalid_result.is_err());
        if let Err(FunctionError::InvalidArity {
            min, max, actual, ..
        }) = invalid_result
        {
            assert_eq!(min, 0);
            assert_eq!(max, Some(0));
            assert_eq!(actual, 1);
        }
    }

    #[test]
    fn test_const_generic_unary_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_unary(TestUnaryFunction);

        assert!(registry.contains("testUnary"));

        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Test valid call
        let result = registry
            .evaluate_function("testUnary", &[FhirPathValue::Integer(41)], &context)
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test invalid arity (no arguments)
        let invalid_result = registry.evaluate_function("testUnary", &[], &context);
        assert!(invalid_result.is_err());

        // Test invalid arity (too many arguments)
        let invalid_result2 = registry.evaluate_function(
            "testUnary",
            &[FhirPathValue::Integer(1), FhirPathValue::Integer(2)],
            &context,
        );
        assert!(invalid_result2.is_err());
    }

    #[test]
    fn test_const_generic_binary_function() {
        let mut registry = FunctionRegistry::new();
        registry.register_binary(TestBinaryFunction);

        assert!(registry.contains("testBinary"));

        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Test valid call
        let result = registry
            .evaluate_function(
                "testBinary",
                &[FhirPathValue::Integer(20), FhirPathValue::Integer(22)],
                &context,
            )
            .unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));

        // Test invalid arity (one argument)
        let invalid_result =
            registry.evaluate_function("testBinary", &[FhirPathValue::Integer(1)], &context);
        assert!(invalid_result.is_err());

        // Test invalid arity (three arguments)
        let invalid_result2 = registry.evaluate_function(
            "testBinary",
            &[
                FhirPathValue::Integer(1),
                FhirPathValue::Integer(2),
                FhirPathValue::Integer(3),
            ],
            &context,
        );
        assert!(invalid_result2.is_err());
    }

    #[test]
    fn test_arity_wrapper_functionality() {
        let nullary_func = TestNullaryFunction;
        let wrapper = ArityWrapper::<_, 0>::new(nullary_func);

        // Test that wrapper implements FhirPathFunction correctly
        assert_eq!(wrapper.name(), "testNullary");
        assert_eq!(wrapper.human_friendly_name(), "Test Nullary Function");
        assert_eq!(wrapper.documentation(), "Returns 42");
        assert!(wrapper.is_pure());

        let context = EvaluationContext::new(FhirPathValue::Empty);
        let result = wrapper.evaluate(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_mixed_arity_functions() {
        let mut registry = FunctionRegistry::new();

        // Register functions with different arities
        registry.register_nullary(TestNullaryFunction);
        registry.register_unary(TestUnaryFunction);
        registry.register_binary(TestBinaryFunction);

        let context = EvaluationContext::new(FhirPathValue::Empty);

        // Test that all functions work correctly
        let result0 = registry
            .evaluate_function("testNullary", &[], &context)
            .unwrap();
        assert_eq!(result0, FhirPathValue::Integer(42));

        let result1 = registry
            .evaluate_function("testUnary", &[FhirPathValue::Integer(10)], &context)
            .unwrap();
        assert_eq!(result1, FhirPathValue::Integer(11));

        let result2 = registry
            .evaluate_function(
                "testBinary",
                &[FhirPathValue::Integer(5), FhirPathValue::Integer(7)],
                &context,
            )
            .unwrap();
        assert_eq!(result2, FhirPathValue::Integer(12));

        // Test that arity checking still works correctly
        assert!(
            registry
                .evaluate_function("testNullary", &[FhirPathValue::Integer(1)], &context)
                .is_err()
        );
        assert!(
            registry
                .evaluate_function("testUnary", &[], &context)
                .is_err()
        );
        assert!(
            registry
                .evaluate_function("testBinary", &[FhirPathValue::Integer(1)], &context)
                .is_err()
        );
    }
}
