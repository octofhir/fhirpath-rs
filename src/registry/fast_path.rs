//! Fast path optimizations for common FHIRPath functions
//!
//! This module provides optimized implementations for frequently used functions
//! that can bypass the general evaluation pipeline for better performance.

use crate::ast::ExpressionNode;
use crate::model::FhirPathValue;
use crate::registry::function::{EvaluationContext, FunctionResult};

/// Trait for functions that can provide fast path optimizations
pub trait FastPathFunction {
    /// Check if this function call can use a fast path optimization
    fn can_fast_path(&self, args: &[ExpressionNode], context: &EvaluationContext) -> bool;

    /// Execute the fast path optimization
    fn fast_evaluate(
        &self,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue>;
}

/// Fast path implementation for the count() function
pub struct FastCountFunction;

impl FastCountFunction {
    /// Create a new fast count function implementation
    pub fn new() -> Self {
        Self
    }

    /// Optimized count implementation using Arc-based collections
    pub fn fast_count(input: &FhirPathValue) -> FhirPathValue {
        let count = match input {
            FhirPathValue::Collection(items) => items.len(),
            FhirPathValue::Empty => 0,
            _ => 1,
        };
        FhirPathValue::Integer(count as i64)
    }
}

impl FastPathFunction for FastCountFunction {
    fn can_fast_path(&self, args: &[ExpressionNode], _context: &EvaluationContext) -> bool {
        // count() takes no arguments, so always eligible for fast path
        args.is_empty()
    }

    fn fast_evaluate(
        &self,
        _args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        Ok(Self::fast_count(&context.input))
    }
}

/// Fast path implementation for the exists() function
pub struct FastExistsFunction;

impl FastExistsFunction {
    /// Create a new fast exists function implementation
    pub fn new() -> Self {
        Self
    }

    /// Optimized exists implementation
    pub fn fast_exists(input: &FhirPathValue) -> FhirPathValue {
        let exists = match input {
            FhirPathValue::Empty => false,
            FhirPathValue::Collection(items) => !items.is_empty(),
            _ => true,
        };
        FhirPathValue::Boolean(exists)
    }
}

impl FastPathFunction for FastExistsFunction {
    fn can_fast_path(&self, args: &[ExpressionNode], _context: &EvaluationContext) -> bool {
        // Only fast path exists() without conditions
        args.is_empty()
    }

    fn fast_evaluate(
        &self,
        _args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        Ok(Self::fast_exists(&context.input))
    }
}

/// Fast path implementation for the first() function
pub struct FastFirstFunction;

impl FastFirstFunction {
    /// Create a new fast first function implementation
    pub fn new() -> Self {
        Self
    }

    /// Optimized first implementation using zero-copy operations
    pub fn fast_first(input: &FhirPathValue) -> FhirPathValue {
        match input {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    // Zero-copy access to first element
                    items.iter().next().unwrap().clone()
                }
            }
            other => other.clone(),
        }
    }
}

impl FastPathFunction for FastFirstFunction {
    fn can_fast_path(&self, args: &[ExpressionNode], _context: &EvaluationContext) -> bool {
        args.is_empty()
    }

    fn fast_evaluate(
        &self,
        _args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        Ok(Self::fast_first(&context.input))
    }
}

/// Fast path implementation for the last() function
pub struct FastLastFunction;

impl FastLastFunction {
    /// Create a new fast last function implementation  
    pub fn new() -> Self {
        Self
    }

    /// Optimized last implementation using zero-copy operations
    pub fn fast_last(input: &FhirPathValue) -> FhirPathValue {
        match input {
            FhirPathValue::Empty => FhirPathValue::Empty,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    FhirPathValue::Empty
                } else {
                    // Zero-copy access to last element
                    items.iter().last().unwrap().clone()
                }
            }
            other => other.clone(),
        }
    }
}

impl FastPathFunction for FastLastFunction {
    fn can_fast_path(&self, args: &[ExpressionNode], _context: &EvaluationContext) -> bool {
        args.is_empty()
    }

    fn fast_evaluate(
        &self,
        _args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        Ok(Self::fast_last(&context.input))
    }
}

// Note: where() and select() fast paths are more complex and would require
// full expression analysis. For now, we focus on the simpler functions.

/// Registry for fast path functions
pub struct FastPathRegistry {
    count: FastCountFunction,
    exists: FastExistsFunction,
    first: FastFirstFunction,
    last: FastLastFunction,
}

impl FastPathRegistry {
    /// Create a new fast path registry
    pub fn new() -> Self {
        Self {
            count: FastCountFunction::new(),
            exists: FastExistsFunction::new(),
            first: FastFirstFunction::new(),
            last: FastLastFunction::new(),
        }
    }

    /// Try to use fast path for a function call
    pub fn try_fast_path(
        &self,
        function_name: &str,
        args: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> Option<FunctionResult<FhirPathValue>> {
        match function_name {
            "count" if self.count.can_fast_path(args, context) => {
                Some(self.count.fast_evaluate(args, context))
            }
            "exists" if self.exists.can_fast_path(args, context) => {
                Some(self.exists.fast_evaluate(args, context))
            }
            "first" if self.first.can_fast_path(args, context) => {
                Some(self.first.fast_evaluate(args, context))
            }
            "last" if self.last.can_fast_path(args, context) => {
                Some(self.last.fast_evaluate(args, context))
            }
            _ => None,
        }
    }
}

impl Default for FastPathRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FhirPathValue;

    #[test]
    fn test_fast_count() {
        let empty = FhirPathValue::Empty;
        assert_eq!(
            FastCountFunction::fast_count(&empty),
            FhirPathValue::Integer(0)
        );

        let single = FhirPathValue::Integer(42);
        assert_eq!(
            FastCountFunction::fast_count(&single),
            FhirPathValue::Integer(1)
        );

        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        assert_eq!(
            FastCountFunction::fast_count(&collection),
            FhirPathValue::Integer(3)
        );
    }

    #[test]
    fn test_fast_exists() {
        let empty = FhirPathValue::Empty;
        assert_eq!(
            FastExistsFunction::fast_exists(&empty),
            FhirPathValue::Boolean(false)
        );

        let single = FhirPathValue::Integer(42);
        assert_eq!(
            FastExistsFunction::fast_exists(&single),
            FhirPathValue::Boolean(true)
        );

        let empty_collection = FhirPathValue::collection(vec![]);
        assert_eq!(
            FastExistsFunction::fast_exists(&empty_collection),
            FhirPathValue::Boolean(false)
        );

        let collection = FhirPathValue::collection(vec![FhirPathValue::Integer(1)]);
        assert_eq!(
            FastExistsFunction::fast_exists(&collection),
            FhirPathValue::Boolean(true)
        );
    }

    #[test]
    fn test_fast_first() {
        let empty = FhirPathValue::Empty;
        assert_eq!(FastFirstFunction::fast_first(&empty), FhirPathValue::Empty);

        let single = FhirPathValue::Integer(42);
        assert_eq!(
            FastFirstFunction::fast_first(&single),
            FhirPathValue::Integer(42)
        );

        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        assert_eq!(
            FastFirstFunction::fast_first(&collection),
            FhirPathValue::Integer(1)
        );
    }

    #[test]
    fn test_fast_last() {
        let empty = FhirPathValue::Empty;
        assert_eq!(FastLastFunction::fast_last(&empty), FhirPathValue::Empty);

        let single = FhirPathValue::Integer(42);
        assert_eq!(
            FastLastFunction::fast_last(&single),
            FhirPathValue::Integer(42)
        );

        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);
        assert_eq!(
            FastLastFunction::fast_last(&collection),
            FhirPathValue::Integer(3)
        );
    }
}
