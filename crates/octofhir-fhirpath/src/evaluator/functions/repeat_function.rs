//! Repeat function implementation
//!
//! The repeat function applies an expression repeatedly to a collection until no new items are added.
//! Syntax: collection.repeat(expression)

use std::collections::HashSet;
use std::sync::Arc;

use crate::ast::{ExpressionNode, LiteralNode, LiteralValue};
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// Repeat function evaluator
pub struct RepeatFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RepeatFunctionEvaluator {
    /// Create a new repeat function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "repeat".to_string(),
                description: "Applies an expression repeatedly until no new items are added"
                    .to_string(),
                signature: FunctionSignature {
                    input_type: "Any".to_string(),
                    parameters: vec![FunctionParameter {
                        name: "expression".to_string(),
                        parameter_type: vec!["Expression".to_string()],
                        optional: false,
                        is_expression: true,
                        description: "The expression to apply repeatedly".to_string(),
                        default_value: None,
                    }],
                    return_type: "Any".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: true,
                category: FunctionCategory::Aggregate,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }
}

#[async_trait::async_trait]
impl LazyFunctionEvaluator for RepeatFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0053,
                format!("repeat function expects 1 argument, got {}", args.len()),
            ));
        }

        let repeat_expr = &args[0];
        let mut result_values = Vec::new();
        let mut result_keys: HashSet<String> = HashSet::new();
        let mut seen = HashSet::new();
        let original_input = input.clone();
        let mut current_items = input;
        // Track type tags for decision about seed placement
        let seed_type_tags: std::collections::HashSet<&'static str> =
            original_input.iter().map(|v| Self::type_tag(v)).collect();
        let mut produced_type_tags: std::collections::HashSet<&'static str> = Default::default();

        // Track original items to avoid infinite loops
        for item in &current_items {
            let item_key = self.create_item_hash(item);
            seen.insert(item_key);
        }

        // Check for infinite constant repetition before starting
        // If the expression is a constant that equals any input item, it will loop infinitely
        if let ExpressionNode::Literal(literal) = repeat_expr
            && let Some(literal_value) = self.literal_to_fhirpath_value(literal)
        {
            for item in &current_items {
                if self.values_equal(item, &literal_value) {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0061,
                        format!(
                            "repeat function with constant '{}' would create infinite loop",
                            self.value_to_string(&literal_value)
                        ),
                    ));
                }
            }
        }

        // Safety: detect simple arithmetic on non-numeric $this that would silently yield empty
        if self.is_simple_arithmetic_on_this(repeat_expr) {
            let has_non_numeric_seed = original_input.iter().any(|v| {
                !matches!(
                    v,
                    FhirPathValue::Integer(_, _, _)
                        | FhirPathValue::Decimal(_, _, _)
                        | FhirPathValue::Quantity { .. }
                        | FhirPathValue::Date(_, _, _)
                        | FhirPathValue::DateTime(_, _, _)
                        | FhirPathValue::Time(_, _, _)
                )
            });
            if has_non_numeric_seed {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    "repeat function projection uses arithmetic on non-numeric input type",
                ));
            }
        }

        // Maximum iterations to prevent infinite loops
        const MAX_ITERATIONS: usize = 1000;

        // Maximum total items to prevent excessive memory usage
        const MAX_ITEMS: usize = 10000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!(
                        "repeat function exceeded maximum iterations ({MAX_ITERATIONS}) - potential infinite loop"
                    ),
                ));
            }

            if result_values.len() > MAX_ITEMS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!(
                        "repeat function exceeded maximum items ({MAX_ITEMS}) - preventing excessive memory usage"
                    ),
                ));
            }

            let mut new_items = Vec::new();

            // Apply the repeat expression to each current item
            for item in &current_items {
                // Create a proper context with this item as the input
                let single_item_collection = vec![item.clone()];
                let item_context = EvaluationContext::new(
                    crate::core::Collection::from(single_item_collection),
                    context.model_provider().clone(),
                    context.terminology_provider().cloned(),
                    context.validation_provider().cloned(),
                    context.trace_provider().cloned(),
                )
                .await;

                let item_result = evaluator.evaluate(repeat_expr, &item_context).await?;
                for new_item in item_result.value.into_iter() {
                    let item_key = self.create_item_hash(&new_item);
                    if seen.insert(item_key.clone()) {
                        // Track in result keys and values
                        result_keys.insert(item_key);
                        // Track type tags for produced items
                        produced_type_tags.insert(Self::type_tag(&new_item));
                        new_items.push(new_item.clone());
                        result_values.push(new_item);
                    }
                }
            }

            // If no new items were found, we're done
            if new_items.is_empty() {
                break;
            }

            current_items = new_items;
        }

        // Place seeds only for numeric/temporal/quantity sequences; never include seeds for complex/object traversals
        let eligible_seed_types: std::collections::HashSet<&'static str> =
            ["Integer", "Decimal", "Date", "DateTime", "Time", "Quantity"]
                .into_iter()
                .collect();
        let seeds_all_eligible = seed_type_tags
            .iter()
            .all(|t| eligible_seed_types.contains(t));
        let produced_all_eligible = produced_type_tags
            .iter()
            .all(|t| eligible_seed_types.contains(t));
        let allow_seed_inclusion =
            seeds_all_eligible && produced_all_eligible && !produced_type_tags.is_empty();

        if allow_seed_inclusion {
            // Seeds first, followed by discovered items (without duplicates)
            let mut final_values = Vec::with_capacity(result_values.len() + original_input.len());
            for item in original_input {
                let key = self.create_item_hash(&item);
                if !result_keys.contains(&key) {
                    result_keys.insert(key);
                    final_values.push(item);
                }
            }
            final_values.extend(result_values);
            Ok(EvaluationResult {
                value: crate::core::Collection::from(final_values),
            })
        } else {
            // Do not include seeds for non-eligible traversals (e.g., property projection producing complex types or strings)
            Ok(EvaluationResult {
                value: crate::core::Collection::from(result_values),
            })
        }
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl RepeatFunctionEvaluator {
    /// Return a static type tag for a FhirPathValue variant (used for ordering heuristics)
    fn type_tag(value: &FhirPathValue) -> &'static str {
        match value {
            FhirPathValue::String(_, _, _) => "String",
            FhirPathValue::Integer(_, _, _) => "Integer",
            FhirPathValue::Decimal(_, _, _) => "Decimal",
            FhirPathValue::Boolean(_, _, _) => "Boolean",
            FhirPathValue::Date(_, _, _) => "Date",
            FhirPathValue::DateTime(_, _, _) => "DateTime",
            FhirPathValue::Time(_, _, _) => "Time",
            FhirPathValue::Resource(_, _, _) => "Resource",
            FhirPathValue::Quantity { .. } => "Quantity",
            FhirPathValue::Collection(_) => "Collection",
            FhirPathValue::Empty => "Empty",
        }
    }

    /// Simple pattern check: expression is a direct arithmetic op on $this
    fn is_simple_arithmetic_on_this(&self, expr: &ExpressionNode) -> bool {
        use crate::ast::operator::BinaryOperator;
        use ExpressionNode as EN;
        match expr {
            EN::BinaryOperation(bin) => {
                let is_arith = matches!(
                    bin.operator,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::IntegerDivide
                        | BinaryOperator::Modulo
                );
                if !is_arith {
                    return false;
                }
                // $this on one side and the other side does not reference $this
                let left_is_this =
                    matches!(*bin.left.clone(), EN::Variable(ref v) if v.name == "this");
                let right_is_this =
                    matches!(*bin.right.clone(), EN::Variable(ref v) if v.name == "this");
                if left_is_this ^ right_is_this {
                    // Additionally require the other side to be a literal to avoid flagging complex expressions
                    let other_is_literal = if left_is_this {
                        matches!(*bin.right.clone(), EN::Literal(_))
                    } else {
                        matches!(*bin.left.clone(), EN::Literal(_))
                    };
                    return other_is_literal;
                }
                false
            }
            _ => false,
        }
    }

    /// Create a better hash for item deduplication
    fn create_item_hash(&self, item: &FhirPathValue) -> String {
        match item {
            FhirPathValue::String(s, _, _) => format!("str:{s}"),
            FhirPathValue::Integer(i, _, _) => format!("int:{i}"),
            FhirPathValue::Decimal(d, _, _) => format!("dec:{d}"),
            FhirPathValue::Boolean(b, _, _) => format!("bool:{b}"),
            FhirPathValue::Date(d, _, _) => format!("date:{d:?}"),
            FhirPathValue::DateTime(dt, _, _) => format!("datetime:{dt:?}"),
            FhirPathValue::Time(t, _, _) => format!("time:{t:?}"),
            FhirPathValue::Resource(json, _type_info, _) => {
                // For top-level FHIR resources, use resourceType/id; otherwise fall back to content hash
                let resource_type = json.get("resourceType").and_then(|v| v.as_str());
                let id = json.get("id").and_then(|v| v.as_str());
                if let (Some(rt), Some(id)) = (resource_type, id) {
                    format!("resource:{rt}:{id}")
                } else {
                    // Not a top-level resource or missing identifiers — derive a stable key from content
                    match serde_json::to_string(json.as_ref()) {
                        Ok(s) => format!("json:{s}"),
                        Err(_) => format!("json:{:?}", json),
                    }
                }
            }
            FhirPathValue::Quantity { value, unit, .. } => {
                let unit_str = unit.as_deref().unwrap_or("no-unit");
                format!("quantity:{value}:{unit_str}")
            }
            FhirPathValue::Collection(collection) => {
                format!("collection:{}", collection.len())
            }
            FhirPathValue::Empty => "empty".to_string(),
        }
    }

    /// Convert literal AST node to FhirPathValue for comparison
    fn literal_to_fhirpath_value(&self, literal: &LiteralNode) -> Option<FhirPathValue> {
        match &literal.value {
            LiteralValue::String(s) => Some(FhirPathValue::string(s.clone())),
            LiteralValue::Integer(i) => Some(FhirPathValue::integer(*i)),
            LiteralValue::Decimal(d) => Some(FhirPathValue::decimal(*d)),
            LiteralValue::Boolean(b) => Some(FhirPathValue::boolean(*b)),
            _ => None,
        }
    }

    /// Check if two values are equal (for infinite loop detection)
    fn values_equal(&self, a: &FhirPathValue, b: &FhirPathValue) -> bool {
        match (a, b) {
            (FhirPathValue::String(s1, _, _), FhirPathValue::String(s2, _, _)) => s1 == s2,
            (FhirPathValue::Integer(i1, _, _), FhirPathValue::Integer(i2, _, _)) => i1 == i2,
            (FhirPathValue::Decimal(d1, _, _), FhirPathValue::Decimal(d2, _, _)) => d1 == d2,
            (FhirPathValue::Boolean(b1, _, _), FhirPathValue::Boolean(b2, _, _)) => b1 == b2,
            _ => false,
        }
    }

    /// Convert value to string for error messages
    fn value_to_string(&self, value: &FhirPathValue) -> String {
        match value {
            FhirPathValue::String(s, _, _) => format!("'{s}'"),
            FhirPathValue::Integer(i, _, _) => i.to_string(),
            FhirPathValue::Decimal(d, _, _) => d.to_string(),
            FhirPathValue::Boolean(b, _, _) => b.to_string(),
            _ => "unknown".to_string(),
        }
    }
}
