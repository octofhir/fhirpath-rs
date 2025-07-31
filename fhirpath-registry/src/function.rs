//! Function registry and built-in functions

// Import from modular structure
use crate::functions::math::*;
use crate::functions::string::*;
use crate::functions::collection::*;
use crate::functions::boolean::*;
use crate::functions::type_conversion::*;
use crate::functions::filtering::*;
use crate::functions::utility::*;
use crate::functions::fhir_types::*;
use crate::functions::datetime::*;
use crate::functions::cda::*;
use crate::signature::{FunctionSignature, ParameterInfo};
use crate::cache::{CacheConfig, FunctionCacheKey, FunctionResolutionCache, FunctionResultCache};

// Re-export commonly used function types for external crates
// Note: Lambda evaluation is not yet fully implemented
// pub use crate::functions::boolean::{AllFunction, AnyFunction};
// pub use crate::functions::collection::ExistsFunction;
use fhirpath_model::{FhirPathValue, TypeInfo};
use rustc_hash::FxHashMap;
use std::hash::BuildHasherDefault;
use std::sync::Arc;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;
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
pub type LambdaEvaluator<'a> =
    dyn Fn(&ExpressionNode, &FhirPathValue) -> Result<FhirPathValue, FunctionError> + 'a;

/// Enhanced lambda evaluator type that supports additional variables injection  
pub type EnhancedLambdaEvaluator<'a> = dyn for<'r> Fn(
        &'r ExpressionNode,
        &'r FhirPathValue,
        &'r VarMap,
    ) -> Result<FhirPathValue, FunctionError>
    + 'a;

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

/// Trait for implementing FHIRPath functions
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

/// Trait for functions that need to evaluate lambda expressions
pub trait LambdaFunction: FhirPathFunction {
    /// Evaluate function with lambda expressions
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
}

/// Hybrid function implementation supporting both trait-based and closure-based functions
#[derive(Clone)]
pub enum FunctionImpl {
    /// Traditional trait-based function
    Trait(Arc<dyn FhirPathFunction>),
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
        func: Arc<dyn Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue> + Send + Sync>,
    },
}

impl std::fmt::Debug for FunctionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FunctionImpl::Trait(func) => {
                f.debug_struct("Trait")
                    .field("name", &func.name())
                    .finish()
            }
            FunctionImpl::Closure { name, friendly_name, .. } => {
                f.debug_struct("Closure")
                    .field("name", name)
                    .field("friendly_name", friendly_name)
                    .finish()
            }
        }
    }
}

impl FunctionImpl {
    /// Get the function name
    pub fn name(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.name(),
            FunctionImpl::Closure { name, .. } => name,
        }
    }

    /// Get the human-friendly name
    pub fn human_friendly_name(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.human_friendly_name(),
            FunctionImpl::Closure { friendly_name, .. } => friendly_name,
        }
    }

    /// Get the function signature
    pub fn signature(&self) -> &FunctionSignature {
        match self {
            FunctionImpl::Trait(f) => f.signature(),
            FunctionImpl::Closure { signature, .. } => signature,
        }
    }

    /// Get function documentation
    pub fn documentation(&self) -> &str {
        match self {
            FunctionImpl::Trait(f) => f.documentation(),
            FunctionImpl::Closure { documentation, .. } => documentation,
        }
    }
    
    /// Check if this function is pure
    pub fn is_pure(&self) -> bool {
        match self {
            FunctionImpl::Trait(f) => f.is_pure(),
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
}

impl std::fmt::Debug for FunctionRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionRegistry")
            .field("function_count", &self.functions.len())
            .field("signature_count", &self.signatures.len())
            .field("resolution_cache", &self.resolution_cache)
            .field("result_cache", &self.result_cache)
            .field("cache_config", &self.cache_config)
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
        }
    }

    /// Register a trait-based function
    pub fn register<F: FhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        let signature = function.signature().clone();
        let func_impl = FunctionImpl::Trait(Arc::new(function));

        self.functions.insert(name.clone(), func_impl);
        self.signatures
            .entry(name)
            .or_insert_with(Vec::new)
            .push(signature);
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
        F: Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue> + Send + Sync + 'static,
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
            .entry(name)
            .or_insert_with(Vec::new)
            .push(signature);
    }

    /// Register a simple closure-based function with minimal boilerplate
    pub fn register_simple<F>(
        &mut self,
        name: impl Into<String>,
        min_arity: usize,
        max_arity: Option<usize>,
        func: F,
    ) where
        F: Fn(&[FhirPathValue], &EvaluationContext) -> FunctionResult<FhirPathValue> + Send + Sync + 'static,
    {
        let name_str = name.into();
        
        // Create parameter info for generic types based on arity
        let mut parameters = Vec::new();
        if let Some(max) = max_arity {
            for i in 0..max {
                let param_name = format!("arg{}", i);
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
                let param_name = format!("arg{}", i);
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
            format!("Auto-generated function: {}", name_str),
            func,
        );
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
        // Get argument types for cache lookup
        let arg_types: Vec<TypeInfo> = args.iter().map(|v| v.to_type_info()).collect();
        
        // Get function with type-based caching
        let function = self.get_function_for_types(name, &arg_types)
            .ok_or_else(|| FunctionError::EvaluationError {
                name: name.to_string(),
                message: "Function not found or type mismatch".to_string(),
            })?;

        // Validate arguments
        function.validate_args(args)?;

        // For pure functions, check result cache
        if self.is_pure_function(name) && self.cache_config.enable_result_caching {
            let result_key = crate::cache::generate_result_cache_key(
                name,
                args,
                0, // TODO: proper context hash
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

    /// Generate documentation from function traits
    pub fn generate_function_docs(&self) -> String {
        let mut docs = String::new();
        docs.push_str("# FHIRPath Function Documentation\n\n");
        
        // Sort functions by name for consistent output
        let mut functions: Vec<_> = self.functions.iter().collect();
        functions.sort_by_key(|(name, _)| *name);
        
        for (name, function) in functions {
            docs.push_str(&format!("## {}\n\n", function.human_friendly_name()));
            docs.push_str(&format!("**Function Name:** `{}`\n\n", name));
            
            // Add signature information
            let sig = function.signature();
            docs.push_str("**Signature:**\n");
            docs.push_str(&format!(
                "- **Arity:** {}{}\n",
                sig.min_arity,
                if let Some(max) = sig.max_arity {
                    format!("-{}", max)
                } else {
                    "+".to_string()
                }
            ));
            docs.push_str(&format!("- **Return Type:** {}\n", sig.return_type));
            
            if !sig.parameters.is_empty() {
                docs.push_str("- **Parameters:**\n");
                for (_i, param) in sig.parameters.iter().enumerate() {
                    docs.push_str(&format!(
                        "  - `{}` ({}): {}\n",
                        param.name,
                        param.param_type,
                        if param.optional { "optional" } else { "required" }
                    ));
                }
            }
            
            // Add documentation
            let doc = function.documentation();
            if !doc.is_empty() {
                docs.push_str("\n**Description:**\n");
                docs.push_str(doc);
                docs.push_str("\n");
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
        doc.push_str(&format!("**Function:** `{}`\n\n", name));
        
        // Signature details
        doc.push_str("## Signature\n\n");
        doc.push_str(&format!("- **Arity:** {}{}\n", 
            sig.min_arity,
            if let Some(max) = sig.max_arity {
                format!("-{}", max)
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
                    if param.optional { "optional" } else { "required" }
                ));
            }
            doc.push_str("\n");
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
            doc.push_str("- **Pure Function**: No (may have side effects or non-deterministic behavior)\n");
        }
        
        Some(doc)
    }

    /// Generate JSON documentation for all functions
    pub fn generate_function_docs_json(&self) -> serde_json::Value {
        use serde_json::{json, Map, Value};
        
        let functions: Map<String, Value> = self.functions
            .iter()
            .map(|(name, function)| {
                let sig = function.signature();
                let params: Vec<Value> = sig.parameters
                    .iter()
                    .map(|p| json!({
                        "name": p.name,
                        "type": p.param_type.to_string(),
                        "optional": p.optional
                    }))
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
    // Collection functions
    registry.register(CountFunction);
    registry.register(EmptyFunction);
    registry.register(ExistsFunction);
    registry.register(DescendantsFunction);
    registry.register(ChildrenFunction);
    registry.register(AggregateFunction);
    registry.register(FirstFunction);
    registry.register(LastFunction);
    registry.register(LengthFunction);
    registry.register(DistinctFunction);
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
    registry.register(AllFunction);
    registry.register(AllTrueFunction);
    registry.register(AnyFunction);
    registry.register(IsDistinctFunction);
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
    registry.register(AsFunction);
    registry.register(ToStringFunction);
    registry.register(ToIntegerFunction);
    registry.register(ToDecimalFunction);
    registry.register(ToBooleanFunction);
    registry.register(TypeFunction);
    registry.register(ConvertsToIntegerFunction);
    registry.register(ConvertsToDecimalFunction);
    registry.register(ConvertsToStringFunction);
    registry.register(ConvertsToBooleanFunction);
    registry.register(ConvertsToDateFunction);
    registry.register(ConvertsToDateTimeFunction);
    registry.register(ConvertsToTimeFunction);
    registry.register(ToQuantityFunction);
    registry.register(ConvertsToQuantityFunction);

    // Filtering functions
    registry.register(WhereFunction);
    registry.register(SelectFunction);
    registry.register(OfTypeFunction);

    // DateTime functions
    registry.register(NowFunction);
    registry.register(TodayFunction);
    registry.register(LowBoundaryFunction);
    registry.register(HighBoundaryFunction);

    // Utility functions
    registry.register(IifFunction);
    registry.register(TraceFunction);
    registry.register(ConformsToFunction);
    registry.register(DefineVariableFunction);
    registry.register(HasValueFunction);
    registry.register(RepeatFunction);

    // FHIR type functions
    registry.register(IsFunction);
    registry.register(ComparableFunction);
    registry.register(ExtensionFunction);
    registry.register(ResolveFunction);

    // CDA functions
    registry.register(HasTemplateIdOfFunction);
    
    // Warm cache with common function lookups if enabled
    if registry.cache_config.warm_cache_on_init {
        warm_function_cache(registry);
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
        "count", "length", "empty", "exists", "first", "last",
        "substring", "startsWith", "endsWith", "contains",
        "toString", "toInteger", "abs", "floor", "ceiling",
        "where", "select", "all", "any", "distinct"
    ];
    
    for function_name in &frequent_functions {
        for arg_types in &common_types {
            // Attempt to cache the function lookup
            let _ = registry.get_function_for_types(function_name, arg_types);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::{FunctionSignature, ParameterInfo};
    use fhirpath_model::{FhirPathValue, TypeInfo};

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
        assert_eq!(function.documentation(), "Doubles the input integer using trait-based implementation");
        
        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(21)];
        
        let result = registry.evaluate_function("testTrait", &args, &context).unwrap();
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
            }
        );
        
        // Verify registration
        assert!(registry.contains("testClosure"));
        
        let function = registry.get("testClosure").unwrap();
        assert_eq!(function.name(), "testClosure");
        assert_eq!(function.human_friendly_name(), "Test Closure Function");
        assert_eq!(function.documentation(), "Triples the input integer using closure-based implementation");
        
        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(14)];
        
        let result = registry.evaluate_function("testClosure", &args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Integer(42));
    }

    #[test]
    fn test_hybrid_registration_simple_function() {
        let mut registry = FunctionRegistry::new();
        
        // Register simple closure-based function with minimal boilerplate
        registry.register_simple(
            "add10",
            1,
            Some(1),
            |args, _context| {
                if let Some(FhirPathValue::Integer(n)) = args.first() {
                    Ok(FhirPathValue::Integer(n + 10))
                } else {
                    Err(FunctionError::EvaluationError {
                        name: "add10".to_string(),
                        message: "Expected integer argument".to_string(),
                    })
                }
            }
        );
        
        // Verify registration
        assert!(registry.contains("add10"));
        
        // Test evaluation
        let context = EvaluationContext::new(FhirPathValue::Empty);
        let args = vec![FhirPathValue::Integer(32)];
        
        let result = registry.evaluate_function("add10", &args, &context).unwrap();
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
        let result1 = registry.evaluate_function("testTrait", &args, &context).unwrap();
        assert_eq!(result1, FhirPathValue::Integer(16)); // 8 * 2
        
        // Test closure-based function
        let result2 = registry.evaluate_function("multiply5", &args, &context).unwrap();
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
        assert_eq!(test_trait["documentation"], "Doubles the input integer using trait-based implementation");
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
}
