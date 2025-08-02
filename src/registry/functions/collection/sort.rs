//! sort() function implementation

use crate::ast::{ExpressionNode, UnaryOperator};
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

/// Helper function to determine if an expression is a reverse sort (e.g., -$this)
fn is_reverse_sort(expr: &ExpressionNode) -> bool {
    matches!(
        expr,
        ExpressionNode::UnaryOp {
            op: UnaryOperator::Minus,
            ..
        }
    )
}

/// Helper function to get the inner expression from a reverse sort (strip the minus)
fn get_inner_expression(expr: &ExpressionNode) -> &ExpressionNode {
    match expr {
        ExpressionNode::UnaryOp { operand, .. } => operand,
        _ => expr,
    }
}

/// Helper function to compare FhirPathValue instances for sorting with reverse handling
/// In FHIRPath:
/// - In ascending order, null/empty values come last  
/// - In descending order, null/empty values come first
fn compare_values_with_reverse(a: &FhirPathValue, b: &FhirPathValue, reverse: bool) -> Ordering {
    match (a, b) {
        (FhirPathValue::Empty, FhirPathValue::Empty) => Ordering::Equal,
        (FhirPathValue::Empty, _) => {
            // In descending order, empty comes first (is "less")
            // In ascending order, empty comes last (is "greater")
            if reverse {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (_, FhirPathValue::Empty) => {
            // In descending order, empty comes first, so non-empty is "greater"
            // In ascending order, empty comes last, so non-empty is "less"
            if reverse {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        _ => {
            // For non-empty values, use standard comparison with reverse logic
            let cmp = compare_values(a, b);
            if reverse { cmp.reverse() } else { cmp }
        }
    }
}

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
            FunctionSignature::variadic(
                "sort",
                vec![ParameterInfo::optional("expressions", TypeInfo::Any)],
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
            // Sort with custom expressions (can be multiple)
            let mut items_with_keys: Vec<(FhirPathValue, Vec<(FhirPathValue, bool)>)> = Vec::new();

            // Evaluate sort keys for each item
            for item in &items {
                let mut sort_keys = Vec::new();

                for sort_expr in args {
                    let is_reverse = is_reverse_sort(sort_expr);
                    let inner_expr = get_inner_expression(sort_expr);

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

                        enhanced_evaluator(inner_expr, item, &additional_vars)?
                    } else {
                        // Fall back to regular evaluator
                        (context.evaluator)(inner_expr, item)?
                    };

                    // Take first value from collection if it's a collection
                    let key_value = match sort_key {
                        FhirPathValue::Collection(ref items) if !items.is_empty() => {
                            items.iter().next().unwrap().clone()
                        }
                        FhirPathValue::Collection(_) => {
                            // Empty collection should be treated as Empty for sorting
                            FhirPathValue::Empty
                        }
                        other => other,
                    };

                    sort_keys.push((key_value, is_reverse));
                }

                items_with_keys.push(((*item).clone(), sort_keys));
            }

            // Sort by the evaluated keys (handling multiple keys and reverse order)
            items_with_keys.sort_by(|a, b| {
                for ((key_a, reverse_a), (key_b, reverse_b)) in a.1.iter().zip(b.1.iter()) {
                    // Both keys should have the same reverse flag for a given sort expression
                    // since they come from the same expression
                    debug_assert_eq!(
                        reverse_a, reverse_b,
                        "Sort keys from same expression should have same reverse flag"
                    );
                    let cmp = compare_values_with_reverse(key_a, key_b, *reverse_a);

                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                Ordering::Equal
            });

            // Extract the sorted items
            let sorted_items: Vec<FhirPathValue> =
                items_with_keys.into_iter().map(|(item, _)| item).collect();
            Ok(FhirPathValue::collection(sorted_items))
        }
    }
}
