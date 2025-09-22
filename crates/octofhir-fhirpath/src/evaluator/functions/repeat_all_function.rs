//! RepeatAll function implementation
//!
//! The repeatAll function allows duplicate items in output collection.
//! Traverses tree and selects children without checking for duplicates.
//! Undefined order of returned items. Includes safety mechanisms to prevent infinite loops.
//! Syntax: collection.repeatAll(projection)

use std::sync::Arc;

use crate::ast::{ExpressionNode, LiteralNode, LiteralValue};
use crate::core::{FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// RepeatAll function evaluator
pub struct RepeatAllFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl RepeatAllFunctionEvaluator {
    /// Create a new repeatAll function evaluator
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self {
            metadata: FunctionMetadata {
                name: "repeatAll".to_string(),
                description: "Traverses tree and selects children without checking for duplicates. Allows duplicate items in output collection. Includes safety mechanisms to prevent infinite loops.".to_string(),
                signature: FunctionSignature {
                    input_type: "Collection".to_string(),
                    parameters: vec![
                        FunctionParameter {
                            name: "projection".to_string(),
                            parameter_type: vec!["Any".to_string()],
                            optional: false,
                            is_expression: true,
                            description: "Expression to evaluate for each item to find children".to_string(),
                            default_value: None,
                        }
                    ],
                    return_type: "Collection".to_string(),
                    polymorphic: true,
                    min_params: 1,
                    max_params: Some(1),
                },
                argument_evaluation: ArgumentEvaluationStrategy::Current,
                null_propagation: NullPropagationStrategy::Focus,
                empty_propagation: EmptyPropagation::Propagate,
                deterministic: false, // Order is undefined, duplicates allowed
                category: FunctionCategory::TreeNavigation,
                requires_terminology: false,
                requires_model: false,
            },
        })
    }

    /// Maximum depth to prevent infinite loops
    const MAX_DEPTH: usize = 1000;

    /// Maximum total items processed to prevent infinite expansion
    const MAX_ITEMS: usize = 10000;

    /// Maximum iterations for iterative approach
    const MAX_ITERATIONS: usize = 1000;

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
                let left_is_this = matches!(*bin.left.clone(), EN::Variable(ref v) if v.name == "this");
                let right_is_this = matches!(*bin.right.clone(), EN::Variable(ref v) if v.name == "this");
                if left_is_this ^ right_is_this {
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

    /// Check if a type name is valid using ModelProvider
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

#[async_trait::async_trait]
impl LazyFunctionEvaluator for RepeatAllFunctionEvaluator {
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
                format!("repeatAll function expects 1 argument, got {}", args.len()),
            ));
        }

        let projection_expr = &args[0];
        let mut results = Vec::new();
        let mut processed_count = 0;

        // Check for infinite constant repetition before starting
        // If the expression is a constant that equals any input item, it will loop infinitely
        if let ExpressionNode::Literal(literal) = projection_expr {
            if let Some(literal_value) = self.literal_to_fhirpath_value(literal) {
                for item in &input {
                    if self.values_equal(item, &literal_value) {
                        return Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0061,
                            format!(
                                "repeatAll function with constant '{}' would create infinite loop",
                                self.value_to_string(&literal_value)
                            ),
                        ));
                    }
                }
            }
        }

        // Safety: detect simple arithmetic on non-numeric $this that would silently yield empty
        if self.is_simple_arithmetic_on_this(projection_expr) {
            let has_non_numeric_seed = input.iter().any(|v| !matches!(
                v,
                FhirPathValue::Integer(_,_,_) | FhirPathValue::Decimal(_,_,_) | FhirPathValue::Quantity{..} | FhirPathValue::Date(_,_,_) | FhirPathValue::DateTime(_,_,_) | FhirPathValue::Time(_,_,_)
            ));
            if has_non_numeric_seed {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    "repeatAll function projection uses arithmetic on non-numeric input type",
                ));
            }
        }

        // Use iterative approach instead of recursion to prevent stack overflow
        self.repeat_all_iterative(
            input,
            projection_expr,
            context,
            &evaluator,
            &mut results,
            &mut processed_count,
        )
        .await?;

        Ok(EvaluationResult {
            value: crate::core::Collection::from(results),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl RepeatAllFunctionEvaluator {
    /// Iterative implementation to prevent stack overflow
    async fn repeat_all_iterative(
        &self,
        input: Vec<FhirPathValue>,
        projection_expr: &ExpressionNode,
        context: &EvaluationContext,
        evaluator: &AsyncNodeEvaluator<'_>,
        results: &mut Vec<FhirPathValue>,
        processed_count: &mut usize,
    ) -> Result<()> {
        use std::collections::{HashSet, VecDeque};

        // Use a queue for breadth-first traversal (prevents deep stack)
        let mut queue: VecDeque<(FhirPathValue, usize)> = VecDeque::new();
        let mut seen_values: HashSet<String> = HashSet::new();

        // Initialize queue with input items
        for item in input {
            queue.push_back((item, 0));
        }

        let mut iterations = 0;

        while let Some((current_item, depth)) = queue.pop_front() {
            iterations += 1;
            *processed_count += 1;

            // Safety checks
            if iterations > Self::MAX_ITERATIONS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!(
                        "repeatAll exceeded maximum iterations ({}) to prevent infinite loops",
                        Self::MAX_ITERATIONS
                    ),
                ));
            }

            if depth > Self::MAX_DEPTH {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!(
                        "repeatAll exceeded maximum depth ({}) to prevent infinite recursion",
                        Self::MAX_DEPTH
                    ),
                ));
            }

            if *processed_count > Self::MAX_ITEMS {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0061,
                    format!(
                        "repeatAll exceeded maximum item limit ({}) to prevent infinite expansion",
                        Self::MAX_ITEMS
                    ),
                ));
            }

            // Create a simple hash for cycle detection (basic but effective)
            let item_hash = format!("{current_item:?}");

            // Add current item to results (repeatAll includes all items, even duplicates)
            results.push(current_item.clone());


            // Only expand if we haven't seen this exact item at this depth (prevents immediate cycles)
            let item_key = format!("{item_hash}@{depth}");
            if seen_values.contains(&item_key) {
                continue; // Skip expansion to prevent immediate cycles
            }
            seen_values.insert(item_key);

            // Create evaluation context for this item
            let single_item_collection = vec![current_item.clone()];
            let iteration_context = EvaluationContext::new(
                crate::core::Collection::from(single_item_collection),
                context.model_provider().clone(),
                context.terminology_provider().cloned(),
                context.validation_provider().cloned(),
                context.trace_provider().cloned(),
            )
            .await;

            // Evaluate projection expression to get children
            let projection_result = evaluator
                .evaluate(projection_expr, &iteration_context)
                .await?;

            // Add children to queue for processing
            for child in projection_result.value.iter() {
                // If projection is a literal constant, only expand one level to avoid infinite recursion
                if let ExpressionNode::Literal(_) = projection_expr {
                    if depth == 0 {
                        queue.push_back((child.clone(), depth + 1));
                    }
                } else {
                    queue.push_back((child.clone(), depth + 1));
                }
            }
        }

        Ok(())
    }
}
