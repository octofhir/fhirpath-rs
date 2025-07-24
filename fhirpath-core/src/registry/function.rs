//! Function registry for FHIRPath functions

use std::collections::HashMap;
use std::sync::Arc;
use crate::error::{FhirPathError, Result};
use crate::value_ext::FhirPathValue;
use crate::evaluator::EvaluationContext;
use super::TypeInfo;

/// Trait for implementing FHIRPath functions
pub trait FhirPathFunction: Send + Sync {
    /// Get the function name
    fn name(&self) -> &str;
    
    /// Get the minimum number of arguments
    fn min_arity(&self) -> usize;
    
    /// Get the maximum number of arguments (None for variadic)
    fn max_arity(&self) -> Option<usize>;
    
    /// Get the return type based on argument types
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo;
    
    /// Evaluate the function with given arguments
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue>;
    
    /// Get function documentation
    fn documentation(&self) -> &str {
        ""
    }
    
    /// Check if arguments are valid for this function
    fn validate_args(&self, args: &[FhirPathValue]) -> Result<()> {
        let arg_count = args.len();
        if arg_count < self.min_arity() {
            return Err(FhirPathError::invalid_arity(
                self.name(),
                self.min_arity(),
                self.max_arity(),
                arg_count,
            ));
        }
        
        if let Some(max) = self.max_arity() {
            if arg_count > max {
                return Err(FhirPathError::invalid_arity(
                    self.name(),
                    self.min_arity(),
                    Some(max),
                    arg_count,
                ));
            }
        }
        
        Ok(())
    }
}

/// Function signature for overload resolution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub name: String,
    pub arg_types: Vec<TypeInfo>,
    pub return_type: TypeInfo,
}

/// Registry for FHIRPath functions
#[derive(Clone)]
pub struct FunctionRegistry {
    functions: HashMap<String, Arc<dyn FhirPathFunction>>,
    signatures: HashMap<String, Vec<FunctionSignature>>,
}

impl FunctionRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            signatures: HashMap::new(),
        }
    }
    
    /// Register a function
    pub fn register<F: FhirPathFunction + 'static>(&mut self, function: F) {
        let name = function.name().to_string();
        self.functions.insert(name.clone(), Arc::new(function));
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
}

/// Register all built-in FHIRPath functions
pub fn register_builtin_functions(registry: &mut FunctionRegistry) {
    // Collection functions
    registry.register(WhereFunction);
    registry.register(SelectFunction);
    registry.register(FirstFunction);
    registry.register(LastFunction);
    registry.register(TailFunction);
    registry.register(SkipFunction);
    registry.register(TakeFunction);
    registry.register(CountFunction);
    registry.register(EmptyFunction);
    registry.register(ExistsFunction);
    registry.register(AllFunction);
    registry.register(DistinctFunction);
    
    // Type functions
    registry.register(OfTypeFunction);
    registry.register(IsFunction);
    registry.register(AsFunction);
    
    // String functions
    registry.register(StartsWithFunction);
    registry.register(EndsWithFunction);
    registry.register(ContainsFunction);
    registry.register(SubstringFunction);
    registry.register(LengthFunction);
    
    // Math functions
    registry.register(AbsFunction);
    registry.register(CeilingFunction);
    registry.register(FloorFunction);
    registry.register(RoundFunction);
    registry.register(SqrtFunction);
    
    // Date/Time functions
    registry.register(TodayFunction);
    registry.register(NowFunction);
}

// Collection function implementations

struct WhereFunction;
impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str { "where" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        if let Some(input_type) = arg_types.first() {
            input_type.clone()
        } else {
            TypeInfo::Any
        }
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement where function logic
        Ok(FhirPathValue::Empty)
    }
}

struct SelectFunction;
impl FhirPathFunction for SelectFunction {
    fn name(&self) -> &str { "select" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Collection(Box::new(TypeInfo::Any))
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement select function logic
        Ok(FhirPathValue::Empty)
    }
}

struct FirstFunction;
impl FhirPathFunction for FirstFunction {
    fn name(&self) -> &str { "first" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        if let Some(TypeInfo::Collection(elem)) = arg_types.first() {
            TypeInfo::Optional(elem.clone())
        } else {
            TypeInfo::Any
        }
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
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

struct LastFunction;
impl FhirPathFunction for LastFunction {
    fn name(&self) -> &str { "last" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        if let Some(TypeInfo::Collection(elem)) = arg_types.first() {
            TypeInfo::Optional(elem.clone())
        } else {
            TypeInfo::Any
        }
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
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

struct TailFunction;
impl FhirPathFunction for TailFunction {
    fn name(&self) -> &str { "tail" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        arg_types.first().cloned().unwrap_or(TypeInfo::Any)
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        match &context.input {
            FhirPathValue::Collection(items) => {
                if items.len() > 1 {
                    Ok(FhirPathValue::collection(items.iter().skip(1).cloned().collect()))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Empty => Ok(FhirPathValue::Empty),
            _single => Ok(FhirPathValue::Empty),
        }
    }
}

struct SkipFunction;
impl FhirPathFunction for SkipFunction {
    fn name(&self) -> &str { "skip" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        arg_types.first().cloned().unwrap_or(TypeInfo::Any)
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement skip function logic
        Ok(FhirPathValue::Empty)
    }
}

struct TakeFunction;
impl FhirPathFunction for TakeFunction {
    fn name(&self) -> &str { "take" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        arg_types.first().cloned().unwrap_or(TypeInfo::Any)
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement take function logic
        Ok(FhirPathValue::Empty)
    }
}

struct CountFunction;
impl FhirPathFunction for CountFunction {
    fn name(&self) -> &str { "count" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        let count = match &context.input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        
        Ok(FhirPathValue::Integer(count as i64))
    }
}

struct EmptyFunction;
impl FhirPathFunction for EmptyFunction {
    fn name(&self) -> &str { "empty" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        Ok(FhirPathValue::Boolean(context.input.is_empty()))
    }
}

struct ExistsFunction;
impl FhirPathFunction for ExistsFunction {
    fn name(&self) -> &str { "exists" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
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

struct AllFunction;
impl FhirPathFunction for AllFunction {
    fn name(&self) -> &str { "all" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement all function logic
        Ok(FhirPathValue::Boolean(true))
    }
}

struct DistinctFunction;
impl FhirPathFunction for DistinctFunction {
    fn name(&self) -> &str { "distinct" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        arg_types.first().cloned().unwrap_or(TypeInfo::Any)
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement distinct function logic
        Ok(context.input.clone())
    }
}

// Type function implementations

struct OfTypeFunction;
impl FhirPathFunction for OfTypeFunction {
    fn name(&self) -> &str { "ofType" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Collection(Box::new(TypeInfo::Any))
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement ofType function logic
        Ok(FhirPathValue::Empty)
    }
}

struct IsFunction;
impl FhirPathFunction for IsFunction {
    fn name(&self) -> &str { "is" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement is function logic
        Ok(FhirPathValue::Boolean(false))
    }
}

struct AsFunction;
impl FhirPathFunction for AsFunction {
    fn name(&self) -> &str { "as" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Any
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement as function logic
        Ok(context.input.clone())
    }
}

// String function implementations

struct StartsWithFunction;
impl FhirPathFunction for StartsWithFunction {
    fn name(&self) -> &str { "startsWith" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement startsWith function logic
        Ok(FhirPathValue::Boolean(false))
    }
}

struct EndsWithFunction;
impl FhirPathFunction for EndsWithFunction {
    fn name(&self) -> &str { "endsWith" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement endsWith function logic
        Ok(FhirPathValue::Boolean(false))
    }
}

struct ContainsFunction;
impl FhirPathFunction for ContainsFunction {
    fn name(&self) -> &str { "contains" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Boolean
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement contains function logic
        Ok(FhirPathValue::Boolean(false))
    }
}

struct SubstringFunction;
impl FhirPathFunction for SubstringFunction {
    fn name(&self) -> &str { "substring" }
    fn min_arity(&self) -> usize { 1 }
    fn max_arity(&self) -> Option<usize> { Some(2) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::String
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement substring function logic
        Ok(FhirPathValue::String(String::new()))
    }
}

struct LengthFunction;
impl FhirPathFunction for LengthFunction {
    fn name(&self) -> &str { "length" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        match &context.input {
            FhirPathValue::String(s) => Ok(FhirPathValue::Integer(s.len() as i64)),
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

// Math function implementations

struct AbsFunction;
impl FhirPathFunction for AbsFunction {
    fn name(&self) -> &str { "abs" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, arg_types: &[TypeInfo]) -> TypeInfo {
        arg_types.first().cloned().unwrap_or(TypeInfo::Any)
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        match &context.input {
            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(i.abs())),
            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(d.abs())),
            FhirPathValue::Quantity { value, unit, .. } => {
                Ok(FhirPathValue::quantity(value.abs(), unit.clone()))
            }
            _ => Ok(FhirPathValue::Empty),
        }
    }
}

struct CeilingFunction;
impl FhirPathFunction for CeilingFunction {
    fn name(&self) -> &str { "ceiling" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement ceiling function logic
        Ok(FhirPathValue::Empty)
    }
}

struct FloorFunction;
impl FhirPathFunction for FloorFunction {
    fn name(&self) -> &str { "floor" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Integer
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement floor function logic
        Ok(FhirPathValue::Empty)
    }
}

struct RoundFunction;
impl FhirPathFunction for RoundFunction {
    fn name(&self) -> &str { "round" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(1) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Decimal
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement round function logic
        Ok(FhirPathValue::Empty)
    }
}

struct SqrtFunction;
impl FhirPathFunction for SqrtFunction {
    fn name(&self) -> &str { "sqrt" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Decimal
    }
    
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        // TODO: Implement sqrt function logic
        Ok(FhirPathValue::Empty)
    }
}

// Date/Time function implementations

struct TodayFunction;
impl FhirPathFunction for TodayFunction {
    fn name(&self) -> &str { "today" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::Date
    }
    
    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        let today = chrono::Local::now().date_naive();
        Ok(FhirPathValue::Date(today))
    }
}

struct NowFunction;
impl FhirPathFunction for NowFunction {
    fn name(&self) -> &str { "now" }
    fn min_arity(&self) -> usize { 0 }
    fn max_arity(&self) -> Option<usize> { Some(0) }
    
    fn return_type(&self, _arg_types: &[TypeInfo]) -> TypeInfo {
        TypeInfo::DateTime
    }
    
    fn evaluate(&self, args: &[FhirPathValue], _context: &EvaluationContext) -> Result<FhirPathValue> {
        self.validate_args(args)?;
        
        let now = chrono::Utc::now();
        Ok(FhirPathValue::DateTime(now))
    }
}