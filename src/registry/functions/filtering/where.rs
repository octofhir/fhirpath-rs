//! where() function - filters collection based on criteria

use crate::ast::ExpressionNode;
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use std::hash::BuildHasherDefault;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;

/// where() function - filters collection based on criteria
pub struct WhereFunction;

impl FhirPathFunction for WhereFunction {
    fn name(&self) -> &str {
        "where"
    }
    fn human_friendly_name(&self) -> &str {
        "Where"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "where",
                vec![ParameterInfo::required("criteria", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        // This should not be called for lambda functions - use evaluate_with_lambda instead
        Err(FunctionError::EvaluationError {
            name: self.name().to_string(),
            message: "where() should use lambda evaluation".to_string(),
        })
    }
}

impl LambdaFunction for WhereFunction {
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

        let criteria = &args[0];

        // Get the collection to iterate over
        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])), // Empty collection returns empty
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Apply criteria to each item with index support
        for (index, item) in items.iter().enumerate() {
            let result = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                // Use enhanced evaluator with $index variable injection
                let mut additional_vars: VarMap =
                    std::collections::HashMap::with_hasher(BuildHasherDefault::<
                        rustc_hash::FxHasher,
                    >::default());
                additional_vars.insert("$index".to_string(), FhirPathValue::Integer(index as i64));

                enhanced_evaluator(criteria, item, &additional_vars)?
            } else {
                // Fall back to regular evaluator
                (context.evaluator)(criteria, item)?
            };

            // Check if criteria evaluates to true
            let is_true = match result {
                FhirPathValue::Boolean(true) => true,
                FhirPathValue::Collection(coll) => {
                    // Collection is true if it contains at least one true value
                    coll.iter()
                        .any(|v| matches!(v, FhirPathValue::Boolean(true)))
                }
                FhirPathValue::Empty => false, // Empty is considered false
                _ => false,                    // All other values are considered false
            };

            if is_true {
                results.push((*item).clone());
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}
