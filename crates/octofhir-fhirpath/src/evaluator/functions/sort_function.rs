//! Sort function implementation
//!
//! The sort function sorts a collection based on one or more sort criteria.
//! Syntax: collection.sort([criteria1, criteria2, ...])
//! If no criteria are provided, sorts by the items themselves.

use std::cmp::Ordering;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata, FunctionParameter,
    FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Sort function evaluator
pub struct SortFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl SortFunctionEvaluator {
    /// Create a new sort function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "sort".to_string(),
                description: "Sorts a collection based on one or more sort criteria. If no criteria are provided, sorts by the items themselves.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "criteria".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: true,
                            is_expression: true,
                            description: "Expression(s) to evaluate for sorting. Multiple criteria can be provided.".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 0,
                    max_params: None, // Unlimited parameters for multicriteria sort
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::NoPropagation,
                deterministic: true,
                category: FunctionCategory::Subsetting,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[derive(Debug, Clone)]
struct SortKey {
    values: Vec<FhirPathValue>,
    descending: bool,
}

impl SortKey {
    fn compare(&self, other: &SortKey) -> Ordering {
        let ordering = self.compare_values(&self.values, &other.values);
        if self.descending {
            ordering.reverse()
        } else {
            ordering
        }
    }

    fn compare_values(&self, a: &[FhirPathValue], b: &[FhirPathValue]) -> Ordering {
        // Handle empty collections - empty sorts before non-empty in ascending, after in descending
        match (a.is_empty(), b.is_empty()) {
            (true, true) => Ordering::Equal,
            (true, false) => {
                if self.descending {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (false, true) => {
                if self.descending {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (false, false) => {
                // Both non-empty, compare element by element
                for (val_a, val_b) in a.iter().zip(b.iter()) {
                    let cmp = self.compare_single_value(val_a, val_b);
                    if cmp != Ordering::Equal {
                        return cmp;
                    }
                }
                // If all compared elements are equal, compare by length
                a.len().cmp(&b.len())
            }
        }
    }

    fn compare_single_value(&self, a: &FhirPathValue, b: &FhirPathValue) -> Ordering {
        // Use the existing PartialOrd implementation
        a.partial_cmp(b).unwrap_or(Ordering::Equal)
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for SortFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // If input is empty, return empty
        if input.is_empty() {
            return Ok(EvaluationResult {
                value: crate::core::Collection::from(Vec::<FhirPathValue>::new()),
            });
        }

        // If no arguments, sort by the items themselves
        if args.is_empty() {
            let mut sorted_items = input;
            sorted_items.sort_by(|a, b| {
                let key = SortKey {
                    values: vec![a.clone()],
                    descending: false,
                };
                key.compare_single_value(a, b)
            });

            return Ok(EvaluationResult {
                value: crate::core::Collection::from(sorted_items),
            });
        }

        // Create sort keys for each input item
        let mut items_with_keys = Vec::new();

        for (index, item) in input.iter().enumerate() {
            // Create single-element collection for this item (focus)
            let single_item_collection = vec![item.clone()];

            // Create nested context for this iteration
            let mut iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection.clone()),
                context.model_provider().clone(),
                context.terminology_provider().cloned(),
                context.validation_provider().cloned(),
                context.trace_provider().cloned(),
            )
            .await;

            iteration_context.set_variable("$this".to_string(), item.clone());
            iteration_context.set_variable("$index".to_string(), FhirPathValue::integer(index as i64));
            iteration_context.set_variable("$total".to_string(), FhirPathValue::integer(input.len() as i64));

            let mut sort_keys = Vec::new();

            // Evaluate each sort criterion
            for arg in &args {
                // Check if this is a unary minus expression for descending sort
                let (expr_to_eval, descending) = match arg {
                    ExpressionNode::UnaryOperation(unary_op) => match unary_op.operator {
                        crate::ast::UnaryOperator::Negate => (unary_op.operand.as_ref(), true),
                        _ => (arg, false),
                    },
                    _ => (arg, false),
                };

                // Evaluate the sort expression
                let result = evaluator.evaluate(expr_to_eval, &iteration_context).await?;

                sort_keys.push(SortKey {
                    values: result.value.iter().cloned().collect(),
                    descending,
                });
            }

            items_with_keys.push((item.clone(), sort_keys));
        }

        // Sort items by their keys
        items_with_keys.sort_by(|a, b| {
            // Compare by each sort key in order
            for (key_a, key_b) in a.1.iter().zip(b.1.iter()) {
                let cmp = key_a.compare(key_b);
                if cmp != Ordering::Equal {
                    return cmp;
                }
            }
            Ordering::Equal
        });

        // Extract the sorted items
        let sorted_items: Vec<FhirPathValue> =
            items_with_keys.into_iter().map(|(item, _)| item).collect();

        Ok(EvaluationResult {
            value: crate::core::Collection::from(sorted_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}
