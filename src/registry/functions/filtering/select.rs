//! select() function - transforms collection using expression

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

/// select() function - transforms collection using expression
pub struct SelectFunction;

impl FhirPathFunction for SelectFunction {
    fn name(&self) -> &str {
        "select"
    }
    fn human_friendly_name(&self) -> &str {
        "Select"
    }
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
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
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
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty), // Empty returns empty
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Apply expression to each item with index support
        for (index, item) in items.iter().enumerate() {
            let result = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                // Use enhanced evaluator with $index variable injection and outer context variables
                let mut additional_vars: VarMap =
                    std::collections::HashMap::with_hasher(BuildHasherDefault::<
                        rustc_hash::FxHasher,
                    >::default());

                // Include all variables from outer context
                for (name, value) in &context.context.variables {
                    additional_vars.insert(name.clone(), value.clone());
                }

                // Add $index variable for iteration (parser strips $ prefix)
                additional_vars.insert("index".to_string(), FhirPathValue::Integer(index as i64));

                enhanced_evaluator(expression, item, &additional_vars)?
            } else {
                // Fall back to regular evaluator
                (context.evaluator)(expression, item)?
            };

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
