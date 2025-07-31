//! sort() function implementation

use crate::ast::ExpressionNode;
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult, LambdaEvaluationContext,
    LambdaFunction,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use std::cmp::Ordering;
use std::hash::BuildHasherDefault;

type VarMap =
    std::collections::HashMap<String, FhirPathValue, BuildHasherDefault<rustc_hash::FxHasher>>;

/// Helper function to compare FhirPathValue instances for sorting
fn compare_values(a: &FhirPathValue, b: &FhirPathValue) -> Ordering {
    match (a, b) {
        (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
        (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.cmp(b),
        (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => {
            rust_decimal::Decimal::from(*a).cmp(b)
        }
        (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => {
            a.cmp(&rust_decimal::Decimal::from(*b))
        }
        (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),
        (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),
        (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a.cmp(b),
        (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a.cmp(b),
        (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a.cmp(b),
        (FhirPathValue::Quantity(a), FhirPathValue::Quantity(b)) => {
            // Compare quantities by value first, then by unit
            let value_cmp = a.value.cmp(&b.value);
            if value_cmp != Ordering::Equal {
                value_cmp
            } else {
                a.unit.cmp(&b.unit)
            }
        }
        (FhirPathValue::Empty, FhirPathValue::Empty) => Ordering::Equal,
        (FhirPathValue::Empty, _) => Ordering::Less,
        (_, FhirPathValue::Empty) => Ordering::Greater,
        // For different types, use type precedence
        _ => {
            let type_order = |v: &FhirPathValue| match v {
                FhirPathValue::Empty => 0,
                FhirPathValue::Boolean(_) => 1,
                FhirPathValue::Integer(_) => 2,
                FhirPathValue::Decimal(_) => 3,
                FhirPathValue::String(_) => 4,
                FhirPathValue::Date(_) => 5,
                FhirPathValue::DateTime(_) => 6,
                FhirPathValue::Time(_) => 7,
                FhirPathValue::Quantity(_) => 8,
                FhirPathValue::Resource(_) => 9,
                FhirPathValue::TypeInfoObject { .. } => 10,
                FhirPathValue::Collection(_) => 11,
            };
            type_order(a).cmp(&type_order(b))
        }
    }
}

/// sort() function - sorts the collection
pub struct SortFunction;

impl FhirPathFunction for SortFunction {
    fn name(&self) -> &str {
        "sort"
    }
    fn human_friendly_name(&self) -> &str {
        "Sort"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "sort",
                vec![ParameterInfo::optional("expression", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if !args.is_empty() {
            // Sort with selector requires lambda evaluation
            return Err(FunctionError::EvaluationError {
                name: self.name().to_string(),
                message: "sort() with selector should use lambda evaluation".to_string(),
            });
        }

        let items = context.input.clone().to_collection();
        let mut items_vec: Vec<FhirPathValue> = items.into_iter().collect();

        // Sort using our custom comparison function
        items_vec.sort_by(compare_values);

        Ok(FhirPathValue::collection(items_vec))
    }
}

impl LambdaFunction for SortFunction {
    fn evaluate_with_lambda(
        &self,
        args: &[ExpressionNode],
        context: &LambdaEvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        if args.len() > 1 {
            return Err(FunctionError::InvalidArity {
                name: self.name().to_string(),
                min: 0,
                max: Some(1),
                actual: args.len(),
            });
        }

        let items = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])),
            single => vec![single],
        };

        if args.is_empty() {
            // Simple sort without selector
            let mut items_vec: Vec<FhirPathValue> = items.into_iter().cloned().collect();
            items_vec.sort_by(compare_values);
            Ok(FhirPathValue::collection(items_vec))
        } else {
            // Sort with custom expression
            let sort_expr = &args[0];
            let mut items_with_keys: Vec<(FhirPathValue, FhirPathValue)> = Vec::new();

            // Evaluate sort key for each item
            for item in &items {
                let sort_key = if let Some(enhanced_evaluator) = context.enhanced_evaluator {
                    // Use enhanced evaluator with $this variable
                    let mut additional_vars: VarMap =
                        std::collections::HashMap::with_hasher(BuildHasherDefault::<
                            rustc_hash::FxHasher,
                        >::default(
                        ));

                    // Include all variables from outer context
                    for (name, value) in &context.context.variables {
                        additional_vars.insert(name.clone(), value.clone());
                    }

                    // Add $this variable for current item (parser strips $ prefix)
                    additional_vars.insert("this".to_string(), (*item).clone());

                    enhanced_evaluator(sort_expr, item, &additional_vars)?
                } else {
                    // Fall back to regular evaluator
                    (context.evaluator)(sort_expr, item)?
                };

                items_with_keys.push(((*item).clone(), sort_key));
            }

            // Sort by the evaluated keys
            items_with_keys.sort_by(|a, b| compare_values(&a.1, &b.1));

            // Extract the sorted items
            let sorted_items: Vec<FhirPathValue> =
                items_with_keys.into_iter().map(|(item, _)| item).collect();
            Ok(FhirPathValue::collection(sorted_items))
        }
    }
}
