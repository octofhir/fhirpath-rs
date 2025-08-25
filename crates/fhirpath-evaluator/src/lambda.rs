//! Lambda function implementations for FHIRPath
//!
//! This module contains implementations for FHIRPath lambda functions like where(),
//! select(), sort(), repeat(), aggregate(), and all(). Lambda functions receive
//! raw expressions and control their own evaluation context.

use crate::context::EvaluationContext as LocalEvaluationContext;
use octofhir_fhirpath_ast::FunctionCallData;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use std::collections::HashSet;

/// Lambda function implementations for the FHIRPath engine
impl crate::FhirPathEngine {
    /// Evaluate where lambda function
    pub async fn evaluate_where_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "where() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let predicate_expr = &func_data.args[0];

        match &input {
            FhirPathValue::Collection(items) => {
                let estimated_capacity = (items.len() / 4).clamp(4, 64);
                let mut filtered_items = Vec::with_capacity(estimated_capacity);

                for (index, item) in items.iter().enumerate() {
                    let lambda_context =
                        context.with_lambda_context(item.clone(), index, FhirPathValue::Empty);

                    let predicate_result = self
                        .evaluate_node_async(
                            predicate_expr,
                            item.clone(),
                            &lambda_context,
                            depth + 1,
                        )
                        .await?;

                    if self.is_truthy(&predicate_result) {
                        filtered_items.push(item.clone());
                    }
                }

                Ok(FhirPathValue::Collection(Collection::from(filtered_items)))
            }
            other => {
                let lambda_context =
                    context.with_lambda_context(other.clone(), 0, FhirPathValue::Empty);

                let predicate_result = self
                    .evaluate_node_async(predicate_expr, other.clone(), &lambda_context, depth + 1)
                    .await?;

                if self.is_truthy(&predicate_result) {
                    Ok(other.clone())
                } else {
                    Ok(FhirPathValue::Collection(Collection::from(vec![])))
                }
            }
        }
    }

    /// Evaluate select lambda function
    pub async fn evaluate_select_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "select() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let expr = &func_data.args[0];

        match &input {
            FhirPathValue::Collection(items) => {
                let mut result_items = Vec::new();

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context for each item
                    let lambda_context =
                        context.with_lambda_context(item.clone(), index, FhirPathValue::Empty);

                    // Evaluate expression for each item
                    let item_result = self
                        .evaluate_node_async(expr, item.clone(), &lambda_context, depth + 1)
                        .await?;

                    // Add result to collection - select() can return multiple values per item
                    match item_result {
                        FhirPathValue::Collection(sub_items) => {
                            result_items.extend(sub_items.into_iter());
                        }
                        FhirPathValue::Empty => {
                            // Skip empty results - this is correct FHIRPath behavior for select()
                        }
                        other => result_items.push(other),
                    }
                }

                Ok(FhirPathValue::Collection(Collection::from(result_items)))
            }
            other => {
                // For single items, select() acts like a simple transformation
                let lambda_context =
                    context.with_lambda_context(other.clone(), 0, FhirPathValue::Empty);

                self.evaluate_node_async(expr, other.clone(), &lambda_context, depth + 1)
                    .await
            }
        }
    }

    /// Evaluate sort lambda function
    pub async fn evaluate_sort_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        match &input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    return Ok(input);
                }

                let mut items_with_sort_keys: Vec<(FhirPathValue, Vec<(FhirPathValue, bool)>)> =
                    Vec::new();

                // If no sort expression provided, sort by the items themselves
                if func_data.args.is_empty() {
                    for item in items.iter() {
                        items_with_sort_keys.push((item.clone(), vec![(item.clone(), false)]));
                    }
                } else {
                    // Evaluate sort expressions for each item
                    for (index, item) in items.iter().enumerate() {
                        let lambda_context =
                            context.with_lambda_context(item.clone(), index, FhirPathValue::Empty);

                        let mut sort_keys = Vec::new();

                        for sort_expr in &func_data.args {
                            // Extract sort intent (detect descending sort with unary minus)
                            let (actual_expr, is_descending) = self.extract_sort_intent(sort_expr);

                            let sort_key = self
                                .evaluate_node_async(
                                    actual_expr,
                                    item.clone(),
                                    &lambda_context,
                                    depth + 1,
                                )
                                .await?;

                            sort_keys.push((sort_key, is_descending));
                        }

                        items_with_sort_keys.push((item.clone(), sort_keys));
                    }
                }

                // Sort items by their sort keys (supporting multiple criteria)
                items_with_sort_keys.sort_by(|(_, keys_a), (_, keys_b)| {
                    use std::cmp::Ordering;

                    // Find the first non-empty sort key for comparison
                    for ((key_a, desc_a), (key_b, desc_b)) in keys_a.iter().zip(keys_b.iter()) {
                        let is_empty_a = match key_a {
                            FhirPathValue::Collection(c) if c.is_empty() => true,
                            FhirPathValue::Empty => true,
                            _ => false,
                        };
                        let is_empty_b = match key_b {
                            FhirPathValue::Collection(c) if c.is_empty() => true,
                            FhirPathValue::Empty => true,
                            _ => false,
                        };

                        // If both keys are empty for this criterion, continue to next criterion
                        if is_empty_a && is_empty_b {
                            continue;
                        }

                        // If only one is empty, prefer the non-empty one
                        let ordering = if is_empty_a && !is_empty_b {
                            Ordering::Greater // Empty sorts after non-empty in ascending
                        } else if !is_empty_a && is_empty_b {
                            Ordering::Less // Non-empty sorts before empty in ascending
                        } else {
                            // Both non-empty, do normal comparison
                            self.compare_fhir_values(key_a, key_b)
                        };

                        // Apply descending sort if needed
                        let final_ordering = if *desc_a != *desc_b {
                            // If one is descending and one isn't, we have a logic error
                            // but let's handle it gracefully
                            if *desc_a {
                                ordering.reverse()
                            } else {
                                ordering
                            }
                        } else if *desc_a {
                            // Both descending
                            ordering.reverse()
                        } else {
                            // Both ascending
                            ordering
                        };

                        if final_ordering != Ordering::Equal {
                            return final_ordering;
                        }
                    }
                    Ordering::Equal
                });

                // Extract sorted items
                let sorted_items: Vec<FhirPathValue> = items_with_sort_keys
                    .into_iter()
                    .map(|(item, _)| item)
                    .collect();

                Ok(FhirPathValue::Collection(Collection::from(sorted_items)))
            }
            other => {
                // Single items are already "sorted"
                Ok(other.clone())
            }
        }
    }

    /// Evaluate repeat lambda function
    pub async fn evaluate_repeat_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "repeat() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let expr = &func_data.args[0];
        let mut all_results = Vec::new();
        let mut current_input = input;
        let mut seen_values = HashSet::new();

        // Prevent infinite loops with a reasonable iteration limit
        const MAX_ITERATIONS: usize = 1000;
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                return Err(EvaluationError::InvalidOperation {
                    message: "repeat() exceeded maximum iterations (1000)".to_string(),
                });
            }

            // Evaluate expression with current input
            let result = self
                .evaluate_node_async(expr, current_input.clone(), context, depth + 1)
                .await?;

            // Check if we got empty result (stop condition)
            match &result {
                FhirPathValue::Empty => break,
                FhirPathValue::Collection(items) if items.is_empty() => break,
                _ => {
                    // Check for cycles using a simple string representation
                    let result_key = self.item_to_key(&result);
                    if seen_values.contains(&result_key) {
                        break; // Cycle detected
                    }
                    seen_values.insert(result_key);

                    // Add result to collection
                    match result.clone() {
                        FhirPathValue::Collection(items) => {
                            all_results.extend(items.into_iter());
                        }
                        other => all_results.push(other),
                    }

                    // Set up for next iteration
                    current_input = result;
                }
            }
        }

        Ok(FhirPathValue::Collection(Collection::from(all_results)))
    }

    /// Evaluate aggregate lambda function
    pub async fn evaluate_aggregate_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // aggregate() requires 1 or 2 arguments: aggregate(iterator) or aggregate(iterator, init)
        if func_data.args.is_empty() || func_data.args.len() > 2 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "aggregate() requires 1 or 2 arguments, got {}",
                    func_data.args.len()
                ),
            });
        }

        let iterator_expr = &func_data.args[0];

        // Initial value (default to empty)
        let mut accumulator = if func_data.args.len() == 2 {
            self.evaluate_node_async(&func_data.args[1], input.clone(), context, depth + 1)
                .await?
        } else {
            FhirPathValue::Empty
        };

        match &input {
            FhirPathValue::Collection(items) => {
                for (index, item) in items.iter().enumerate() {
                    // Create lambda context with $this (current item) and $total (accumulator)
                    let lambda_context = context.with_lambda_context(
                        item.clone(),
                        index,
                        accumulator.clone(), // $total
                    );

                    // Evaluate iterator expression
                    accumulator = self
                        .evaluate_node_async(
                            iterator_expr,
                            item.clone(),
                            &lambda_context,
                            depth + 1,
                        )
                        .await?;
                }
            }
            other => {
                // Single item aggregation
                let lambda_context =
                    context.with_lambda_context(other.clone(), 0, accumulator.clone());

                accumulator = self
                    .evaluate_node_async(iterator_expr, other.clone(), &lambda_context, depth + 1)
                    .await?;
            }
        }

        Ok(accumulator)
    }

    /// Evaluate iif (if-then-else) lambda function
    pub async fn evaluate_iif_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        // Validate arguments: iif(condition, then_expr) or iif(condition, then_expr, else_expr)
        if func_data.args.len() < 2 || func_data.args.len() > 3 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "iif() requires 2 or 3 arguments, got {}",
                    func_data.args.len()
                ),
            });
        }

        // According to FHIRPath spec, iif() only works on single values, not collections
        // If input is a collection with multiple items, return empty collection
        match &input {
            FhirPathValue::Collection(col) if col.len() > 1 => {
                return Ok(FhirPathValue::collection(vec![]));
            }
            _ => {}
        }

        // CRITICAL: Do NOT create a new lambda context - use the existing one
        // This preserves $index, $this, and other lambda variables from select() or other outer lambdas
        
        // Evaluate condition using the SAME input and context as the lambda function
        let condition = self
            .evaluate_node_async(
                &func_data.args[0],
                input.clone(),
                context,  // Use existing context directly
                depth + 1,
            )
            .await?;

        // Convert condition to boolean using FHIRPath boolean conversion rules
        let boolean_result = self.to_boolean_fhirpath(&condition);

        match boolean_result {
            Some(true) => {
                // Evaluate then expression
                self.evaluate_node_async(&func_data.args[1], input, context, depth + 1)
                    .await
            }
            Some(false) => {
                if func_data.args.len() == 3 {
                    // Evaluate else expression
                    self.evaluate_node_async(&func_data.args[2], input, context, depth + 1)
                        .await
                } else {
                    // No else expression provided
                    Ok(FhirPathValue::collection(vec![]))
                }
            }
            None => {
                // Invalid condition (shouldn't happen with FHIRPath conversion) - return empty
                Ok(FhirPathValue::collection(vec![]))
            }
        }
    }

    /// Evaluate all lambda function
    pub async fn evaluate_all_lambda(
        &self,
        func_data: &FunctionCallData,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        if func_data.args.len() != 1 {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "all() requires exactly 1 argument, got {}",
                    func_data.args.len()
                ),
            });
        }

        let condition_expr = &func_data.args[0];

        match &input {
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    // Empty collection: all() returns true
                    return Ok(FhirPathValue::Boolean(true));
                }

                for (index, item) in items.iter().enumerate() {
                    // Create lambda context
                    let lambda_context =
                        context.with_lambda_context(item.clone(), index, FhirPathValue::Empty);

                    // Evaluate condition
                    let condition_result = self
                        .evaluate_node_async(
                            condition_expr,
                            item.clone(),
                            &lambda_context,
                            depth + 1,
                        )
                        .await?;

                    // If any condition is false, return false
                    if !self.is_truthy(&condition_result) {
                        return Ok(FhirPathValue::Boolean(false));
                    }
                }

                // All conditions passed
                Ok(FhirPathValue::Boolean(true))
            }
            other => {
                // Single item: evaluate condition
                let lambda_context =
                    context.with_lambda_context(other.clone(), 0, FhirPathValue::Empty);

                let condition_result = self
                    .evaluate_node_async(condition_expr, other.clone(), &lambda_context, depth + 1)
                    .await?;

                Ok(FhirPathValue::Boolean(self.is_truthy(&condition_result)))
            }
        }
    }
}
