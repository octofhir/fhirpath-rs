// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Enhanced sort() function with proper lambda expression support

use crate::expression_argument::{ExpressionArgument, VariableScope};
use crate::lambda_function::{LambdaEvaluationContext, LambdaFhirPathFunction};
use crate::function::{FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use crate::unified_function::UnifiedFhirPathFunction;
use crate::enhanced_metadata::EnhancedFunctionMetadata;
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};
use std::cmp::Ordering;

/// Enhanced sort() function with lambda expression support
///
/// Implements the sort() function correctly by:
/// 1. Receiving the sort criteria expression as an AST node (not pre-evaluated)
/// 2. Evaluating the expression for each collection item with proper variable scoping
/// 3. Supporting $this variable for accessing the current item
pub struct EnhancedSortFunction {
    signature: FunctionSignature,
    metadata: EnhancedFunctionMetadata,
}

impl EnhancedSortFunction {
    pub fn new() -> Self {
        let signature = FunctionSignature {
            name: "sort".to_string(),
            min_arity: 0,
            max_arity: None, // Support unlimited arguments for multi-criteria sorting
            parameters: vec![
                ParameterInfo::optional("criteria", TypeInfo::Any), // Expression, not pre-evaluated
            ],
            return_type: TypeInfo::Any,
        };

        let metadata = crate::metadata_builder::MetadataBuilder::new("sort", crate::function::FunctionCategory::Collections)
            .display_name("Sort")
            .description("Sorts the collection elements by natural order or specified criteria expressions")
            .example("(3 | 1 | 2).sort()")
            .example("(1 | 2 | 3).sort(-$this)")
            .example("Patient.name.sort($this.family)")
            .example("Patient.name.sort(-$this.family, -$this.given.first())")
            .execution_mode(crate::unified_function::ExecutionMode::Sync)
            .input_types(vec![crate::enhanced_metadata::TypePattern::CollectionOf(Box::new(crate::enhanced_metadata::TypePattern::Any))])
            .output_type(crate::enhanced_metadata::TypePattern::CollectionOf(Box::new(crate::enhanced_metadata::TypePattern::Any)))
            .supports_collections(true)
            .requires_collection(false)
            .pure(true)
            .complexity(crate::enhanced_metadata::PerformanceComplexity::Logarithmic)
            .memory_usage(crate::enhanced_metadata::MemoryUsage::Linear)
            .lsp_snippet("sort(${1:criteria})")
            .completion_visibility(crate::function::CompletionVisibility::Contextual)
            .keywords(vec!["sort", "order", "arrange", "sequence", "lambda"])
            .build();

        Self { signature, metadata }
    }
}

#[async_trait]
impl UnifiedFhirPathFunction for EnhancedSortFunction {
    fn name(&self) -> &str {
        "sort"
    }

    fn metadata(&self) -> &EnhancedFunctionMetadata {
        &self.metadata
    }

    fn execution_mode(&self) -> crate::unified_function::ExecutionMode {
        crate::unified_function::ExecutionMode::Sync
    }

    fn supports_lambda_expressions(&self) -> bool {
        true // Enhanced sort function supports lambda expressions for sort criteria
    }

    async fn evaluate_lambda(
        &self,
        args: &[crate::expression_argument::ExpressionArgument],
        context: &crate::lambda_function::LambdaEvaluationContext<'_>,
    ) -> crate::function::FunctionResult<FhirPathValue> {
        // Delegate to the lambda function implementation
        self.evaluate_with_expressions(args, context).await
    }

    fn evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &crate::function::EvaluationContext,
    ) -> crate::function::FunctionResult<FhirPathValue> {
        // For sync evaluation, only support simple sort without criteria
        if args.is_empty() {
            // Natural order sort
            let input_collection = match &context.input {
                FhirPathValue::Collection(items) => items.iter().cloned().collect::<Vec<_>>(),
                FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
                single_item => vec![single_item.clone()],
            };

            let mut sorted_collection = input_collection;
            sorted_collection.sort_by(|a, b| self.compare_values(a, b));
            Ok(FhirPathValue::collection(sorted_collection))
        } else {
            // Sort with criteria requires lambda expression evaluation
            Err(FunctionError::EvaluationError {
                name: "sort".to_string(),
                message: "Sort with criteria requires lambda expression evaluation. Use async evaluation instead.".to_string(),
            })
        }
    }

    async fn evaluate_async(
        &self,
        args: &[FhirPathValue],
        context: &crate::function::EvaluationContext,
    ) -> crate::function::FunctionResult<FhirPathValue> {
        // For async evaluation, delegate to sync for simple cases
        if args.is_empty() {
            self.evaluate_sync(args, context)
        } else {
            // Complex sort with criteria - this should be handled by lambda evaluation instead
            Err(FunctionError::EvaluationError {
                name: "sort".to_string(),
                message: "Sort with criteria should be handled by lambda evaluation system".to_string(),
            })
        }
    }
}

#[async_trait]
impl LambdaFhirPathFunction for EnhancedSortFunction {
    fn name(&self) -> &str {
        "sort"
    }

    fn human_friendly_name(&self) -> &str {
        "Sort Function"
    }

    fn signature(&self) -> &FunctionSignature {
        &self.signature
    }

    fn lambda_argument_indices(&self) -> Vec<usize> {
        // All arguments are sort criteria expressions and should not be pre-evaluated
        (0..=10).collect() // Support up to 10 sort criteria for practical purposes
    }

    async fn evaluate_with_expressions(
        &self,
        args: &[ExpressionArgument],
        context: &LambdaEvaluationContext<'_>,
    ) -> FunctionResult<FhirPathValue> {
        // No argument validation needed - support unlimited arguments for multi-criteria sorting

        // Get the input collection
        let input_collection = match &context.context.input {
            FhirPathValue::Collection(items) => items.iter().cloned().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            single_item => vec![single_item.clone()],
        };

        if input_collection.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        // If no sort criteria provided, use natural order
        if args.is_empty() {
            let mut sorted_collection = input_collection;
            sorted_collection.sort_by(|a, b| self.compare_values(a, b));
            return Ok(FhirPathValue::collection(sorted_collection));
        }

        // Get all sort criteria expressions
        let mut criteria_exprs = Vec::new();
        for arg in args {
            let expr = arg.as_expression().ok_or_else(|| {
                FunctionError::EvaluationError {
                    name: "sort".to_string(),
                    message: "All arguments must be expressions".to_string(),
                }
            })?;
            criteria_exprs.push(expr);
        }

        // Create vector with items and their sort keys (multiple keys per item)
        let mut items_with_keys = Vec::new();
        for (index, item) in input_collection.iter().enumerate() {
            // Create variable scope with $this for current item
            let scope = VariableScope::from_variables(context.context.variables.clone())
                .with_this(item.clone())
                .with_index(index as i32);

            // Evaluate all sort criteria expressions for this item
            let mut sort_keys = Vec::new();
            for criteria_expr in &criteria_exprs {
                let sort_key = (context.evaluator)(criteria_expr, &scope, context.context).await?;
                sort_keys.push(sort_key);
            }

            items_with_keys.push((item.clone(), sort_keys));
        }

        // Sort by the evaluated sort keys (multi-criteria)
        items_with_keys.sort_by(|a, b| self.compare_multi_criteria(&a.1, &b.1));

        // Extract the sorted items
        let sorted_collection = items_with_keys.into_iter().map(|(item, _)| item).collect();

        Ok(FhirPathValue::collection(sorted_collection))
    }

    fn documentation(&self) -> &str {
        r#"
Sorts a collection based on optional criteria expression evaluated for each item.

Within the criteria expression, the following variables are available:
- $this: The current item being evaluated for sorting
- $index: The 0-based index of the current item

Examples:
- (3 | 1 | 2).sort() sorts by natural order → [1, 2, 3]
- (1 | 2 | 3).sort(-$this) sorts by negative values → [3, 2, 1]
- Patient.name.sort($this.family) sorts names by family name
- items.sort($this.value) sorts items by their value property

The function returns a collection with items sorted according to the criteria.
Items with equal sort keys maintain their relative order (stable sort).
"#
    }

    fn is_pure(&self) -> bool {
        // The sort function itself is pure, but the criteria expression may not be
        false
    }

    fn supports_lambda_expressions(&self) -> bool {
        true
    }
}

impl EnhancedSortFunction {
    /// Compare two FhirPathValues for natural ordering
    fn compare_values(&self, a: &FhirPathValue, b: &FhirPathValue) -> Ordering {
        match (a, b) {
            // String comparison
            (FhirPathValue::String(a_str), FhirPathValue::String(b_str)) => {
                a_str.cmp(b_str)
            }

            // Integer comparison
            (FhirPathValue::Integer(a_int), FhirPathValue::Integer(b_int)) => {
                a_int.cmp(b_int)
            }

            // Decimal comparison
            (FhirPathValue::Decimal(a_dec), FhirPathValue::Decimal(b_dec)) => {
                a_dec.cmp(b_dec)
            }

            // Mixed numeric comparisons
            (FhirPathValue::Integer(a_int), FhirPathValue::Decimal(b_dec)) => {
                use rust_decimal::Decimal;
                Decimal::from(*a_int).cmp(b_dec)
            }
            (FhirPathValue::Decimal(a_dec), FhirPathValue::Integer(b_int)) => {
                use rust_decimal::Decimal;
                a_dec.cmp(&Decimal::from(*b_int))
            }

            // Boolean comparison
            (FhirPathValue::Boolean(a_bool), FhirPathValue::Boolean(b_bool)) => {
                a_bool.cmp(b_bool)
            }

            // Date/DateTime/Time comparison
            (FhirPathValue::Date(a_date), FhirPathValue::Date(b_date)) => {
                a_date.cmp(b_date)
            }
            (FhirPathValue::DateTime(a_dt), FhirPathValue::DateTime(b_dt)) => {
                a_dt.cmp(b_dt)
            }
            (FhirPathValue::Time(a_time), FhirPathValue::Time(b_time)) => {
                a_time.cmp(b_time)
            }

            // Collection comparison - compare by first element
            (FhirPathValue::Collection(a_items), FhirPathValue::Collection(b_items)) => {
                match (a_items.iter().next(), b_items.iter().next()) {
                    (Some(a_first), Some(b_first)) => self.compare_values(a_first, b_first),
                    (Some(_), None) => Ordering::Greater,
                    (None, Some(_)) => Ordering::Less,
                    (None, None) => Ordering::Equal,
                }
            }
            (FhirPathValue::Collection(a_items), b) => {
                match a_items.iter().next() {
                    Some(a_first) => self.compare_values(a_first, b),
                    None => Ordering::Less,
                }
            }
            (a, FhirPathValue::Collection(b_items)) => {
                match b_items.iter().next() {
                    Some(b_first) => self.compare_values(a, b_first),
                    None => Ordering::Greater,
                }
            }

            // Empty values come first
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ordering::Equal,
            (FhirPathValue::Empty, _) => Ordering::Less,
            (_, FhirPathValue::Empty) => Ordering::Greater,

            // Different types - use type ordering
            _ => self.type_order(a).cmp(&self.type_order(b))
        }
    }

    /// Get type ordering for mixed-type sorting
    fn type_order(&self, value: &FhirPathValue) -> u8 {
        match value {
            FhirPathValue::Empty => 0,
            FhirPathValue::Boolean(_) => 1,
            FhirPathValue::Integer(_) => 2,
            FhirPathValue::Decimal(_) => 3,
            FhirPathValue::String(_) => 4,
            FhirPathValue::Date(_) => 5,
            FhirPathValue::DateTime(_) => 6,
            FhirPathValue::Time(_) => 7,
            FhirPathValue::Quantity(_) => 8,
            FhirPathValue::Collection(_) => 9,
            FhirPathValue::Resource(_) => 10,
            FhirPathValue::JsonValue(_) => 11,
            FhirPathValue::TypeInfoObject { .. } => 12,
        }
    }

    /// Compare two multi-criteria sort key vectors
    fn compare_multi_criteria(&self, a_keys: &[FhirPathValue], b_keys: &[FhirPathValue]) -> Ordering {
        // Compare each criterion in order until we find a difference
        for (a_key, b_key) in a_keys.iter().zip(b_keys.iter()) {
            let result = self.compare_values(a_key, b_key);
            if result != Ordering::Equal {
                return result;
            }
        }

        // If all compared criteria are equal, check if one has more criteria
        a_keys.len().cmp(&b_keys.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression_argument::ExpressionArgument;
    use crate::lambda_function::{LambdaEvaluationContext, create_simple_lambda_evaluator};
    use crate::function::EvaluationContext;
    use octofhir_fhirpath_ast::{ExpressionNode, UnaryOperator};

    /// Create a test lambda evaluator that handles -$this expressions
    fn create_test_lambda_evaluator() -> Box<crate::lambda_function::LambdaExpressionEvaluator> {
        Box::new(|expr, scope, _context| {
            Box::pin(async move {
                match expr {
                    ExpressionNode::Variable(name) if name == "this" => {
                        Ok(scope.get("this").cloned().unwrap_or(FhirPathValue::Empty))
                    }
                    ExpressionNode::UnaryOp { op: UnaryOperator::Minus, operand } => {
                        // Handle -$this pattern
                        let operand_val = match operand.as_ref() {
                            ExpressionNode::Variable(name) if name == "this" => {
                                scope.get("this").cloned().unwrap_or(FhirPathValue::Empty)
                            }
                            _ => FhirPathValue::Empty,
                        };

                        // Negate the value
                        match operand_val {
                            FhirPathValue::Integer(i) => Ok(FhirPathValue::Integer(-i)),
                            FhirPathValue::Decimal(d) => Ok(FhirPathValue::Decimal(-d)),
                            FhirPathValue::String(s) => {
                                // For strings, reverse alphabetical order by returning the reverse string comparison value
                                // This is a simplified approach for testing
                                Ok(FhirPathValue::String(s)) // The comparison logic will handle this
                            }
                            _ => Err(FunctionError::EvaluationError {
                                name: "test_evaluator".to_string(),
                                message: "Cannot negate non-numeric value in test".to_string(),
                            }),
                        }
                    }
                    _ => Err(FunctionError::EvaluationError {
                        name: "test_evaluator".to_string(),
                        message: "Unsupported expression in test evaluator".to_string(),
                    }),
                }
            })
        })
    }

    #[tokio::test]
    async fn test_enhanced_sort_natural_order() {
        let func = EnhancedSortFunction::new();

        // Test collection: [3, 1, 2]
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(3),
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
        ]);

        let eval_context = EvaluationContext::new(collection);
        let evaluator = create_test_lambda_evaluator();
        let lambda_context = LambdaEvaluationContext {
            context: &eval_context,
            evaluator: evaluator.as_ref(),
        };

        // No arguments - natural order sort
        let args = vec![];

        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();

        // Should return [1, 2, 3]
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items.get(0), Some(&FhirPathValue::Integer(1)));
                assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
                assert_eq!(items.get(2), Some(&FhirPathValue::Integer(3)));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_enhanced_sort_with_negative_criteria() {
        let func = EnhancedSortFunction::new();

        // Test collection: [1, 2, 3]
        let collection = FhirPathValue::collection(vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ]);

        let eval_context = EvaluationContext::new(collection);
        let evaluator = create_test_lambda_evaluator();
        let lambda_context = LambdaEvaluationContext {
            context: &eval_context,
            evaluator: evaluator.as_ref(),
        };

        // Create expression: -$this
        let this_var = ExpressionNode::Variable("this".to_string());
        let minus_expr = ExpressionNode::UnaryOp {
            op: UnaryOperator::Minus,
            operand: Box::new(this_var),
        };

        let args = vec![ExpressionArgument::expression(minus_expr)];

        let result = func.evaluate_with_expressions(&args, &lambda_context).await.unwrap();

        // Should return [3, 2, 1] (sorted by negative values: -1, -2, -3 → reversed)
        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items.get(0), Some(&FhirPathValue::Integer(3)));
                assert_eq!(items.get(1), Some(&FhirPathValue::Integer(2)));
                assert_eq!(items.get(2), Some(&FhirPathValue::Integer(1)));
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[test]
    fn test_metadata() {
        let func = EnhancedSortFunction::new();
        assert_eq!(func.name(), "sort");
        assert_eq!(func.signature().min_arity, 0);
        assert_eq!(func.signature().max_arity, Some(1));
        assert!(func.supports_lambda_expressions());
        assert_eq!(func.lambda_argument_indices(), vec![0]);
    }
}
