//! Filtering and selection functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext, LambdaEvaluationContext, LambdaFunction};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};
use fhirpath_ast::ExpressionNode;

/// where() function - filters collection based on criteria
pub struct WhereFunction;

impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str { "where" }
    fn human_friendly_name(&self) -> &str { "Where" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "where",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // For now, just return the input collection as-is since we don't have lambda support
        Ok(context.input.clone())
    }
}

/// select() function - transforms collection using expression
pub struct SelectFunction;

impl FhirPathFunction for SelectFunction {
    fn name(&self) -> &str { "select" }
    fn human_friendly_name(&self) -> &str { "Select" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "select",
                vec![ParameterInfo::required("expression", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // This should not be called for lambda functions - use evaluate_with_lambda instead
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "select() should use lambda evaluation".to_string(),
        })
    }
}

impl LambdaFunction for SelectFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.is_empty() {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 1,
                max: Some(1),
                actual: 0,
            });
        }

        let expression = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])), // Empty collection returns empty
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Apply expression to each item and collect results
        for item in items {
            let result = (context.evaluator)(expression, item)?;

            // Add result to collection, flattening collections
            match result {
                FhirPathValue::Collection(coll) => {
                    for item in coll {
                        results.push(item);
                    }
                }
                FhirPathValue::Empty => {
                    // Skip empty results
                }
                other => results.push(other),
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

/// take() function - takes first n elements
pub struct TakeFunction;

impl FhirPathFunction for TakeFunction {
    fn name(&self) -> &str { "take" }
    fn human_friendly_name(&self) -> &str { "Take" }
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
        let num = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Integer".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };

        let items = context.input.clone().to_collection();
        let result: Vec<FhirPathValue> = items.into_iter().take(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}

/// skip() function - skips first n elements
pub struct SkipFunction;

impl FhirPathFunction for SkipFunction {
    fn name(&self) -> &str { "skip" }
    fn human_friendly_name(&self) -> &str { "Skip" }
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
        let num = match &args[0] {
            FhirPathValue::Integer(n) => *n as usize,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Integer".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };

        let items = context.input.clone().to_collection();
        let result: Vec<FhirPathValue> = items.into_iter().skip(num).collect();
        Ok(FhirPathValue::collection(result))
    }
}
