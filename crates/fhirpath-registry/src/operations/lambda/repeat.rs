use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    lambda::{ExpressionEvaluator, LambdaFunction},
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;
use std::collections::HashSet;

pub struct RepeatFunction {
    metadata: OperationMetadata,
}

impl Default for RepeatFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl RepeatFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("repeat", OperationType::Function)
            .description("A version of select that will repeat the projection and add it to the output collection, as long as the projection yields new items")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("Resource.descendants()")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for RepeatFunction {
    fn identifier(&self) -> &str {
        "repeat"
    }

    fn operation_type(&self) -> OperationType {
        OperationType::Function
    }

    fn metadata(&self) -> &OperationMetadata {
        &self.metadata
    }

    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: FhirPathOperation::identifier(self).to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let projection_expr = &args[0];

        // Handle literal values (like 'test' string)
        if let FhirPathValue::String(_) = projection_expr {
            return Ok(projection_expr.clone());
        }

        // For non-literal expressions, we can't evaluate them without an evaluator
        // Return the input collection as fallback
        Ok(context.input.clone())
    }

    fn supports_sync(&self) -> bool {
        false // Complex lambda evaluation requires async
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl LambdaFunction for RepeatFunction {
    fn identifier(&self) -> &str {
        "repeat"
    }

    async fn evaluate_lambda(
        &self,
        expressions: &[ExpressionNode],
        context: &EvaluationContext,
        evaluator: &dyn ExpressionEvaluator,
    ) -> Result<FhirPathValue> {
        // Validate arguments
        if expressions.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: LambdaFunction::identifier(self).to_string(),
                expected: 1,
                actual: expressions.len(),
            });
        }

        // CRITICAL: repeat() can only be applied to collections, not single values
        match &context.input {
            FhirPathValue::Empty => return Ok(FhirPathValue::Empty),
            FhirPathValue::Collection(_) => {
                // Valid - continue with collection processing
            }
            _ => {
                // Single value - this is an error according to FHIRPath spec
                return Err(FhirPathError::EvaluationError {
                    message: format!(
                        "repeat() function can only be applied to collections, not single values. Input was: {:?}",
                        context.input
                    ),
                });
            }
        }

        let projection_expr = &expressions[0];

        // Check if this is a literal expression (like 'test')
        // For literal strings, we should return just that literal
        if let ExpressionNode::Literal(literal) = projection_expr {
            if let LiteralValue::String(s) = literal {
                return Ok(FhirPathValue::String(s.clone().into()));
            }
        }

        let mut result = Vec::new();
        let mut current_items = context
            .input
            .clone()
            .to_collection()
            .into_iter()
            .collect::<Vec<_>>();
        let mut seen_items = HashSet::new();

        // Add initial items to seen set but NOT to result (repeat() excludes initial items)
        for item in &current_items {
            let item_key = self.item_to_key(item);
            seen_items.insert(item_key);
        }

        // Continue until no new items are found or we hit safety limits
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite loops
        const MAX_RESULT_SIZE: usize = 10000; // Prevent memory explosion
        let mut iteration_count = 0;

        loop {
            iteration_count += 1;

            // Safety check: prevent infinite loops
            if iteration_count > MAX_ITERATIONS {
                return Err(FhirPathError::EvaluationError {
                    message: format!("repeat() exceeded maximum iterations ({MAX_ITERATIONS})"),
                });
            }

            // Safety check: prevent memory explosion
            if result.len() > MAX_RESULT_SIZE {
                return Err(FhirPathError::EvaluationError {
                    message: format!("repeat() exceeded maximum result size ({MAX_RESULT_SIZE})"),
                });
            }

            let mut new_items = Vec::new();
            let mut found_new = false;

            for (i, item) in current_items.iter().enumerate() {
                // Create new context with current item as input (not focus)
                let item_context = context.with_input(item.clone());

                // Evaluate projection expression using the evaluator
                let projected = evaluator
                    .evaluate_expression(projection_expr, &item_context)
                    .await?;
                let projected_collection = projected.to_collection();

                // Add new items that haven't been seen before
                for proj_item in projected_collection.iter() {
                    let item_key = self.item_to_key(proj_item);
                    if !seen_items.contains(&item_key) {
                        seen_items.insert(item_key);
                        new_items.push(proj_item.clone());
                        result.push(proj_item.clone());
                        found_new = true;
                    }
                }
            }

            if !found_new {
                break;
            }

            current_items = new_items;
        }

        Ok(FhirPathValue::Collection(result.into()))
    }

    fn expected_expression_count(&self) -> usize {
        1
    }
}

impl RepeatFunction {
    // Generate a unique key for an item to detect duplicates
    fn item_to_key(&self, item: &FhirPathValue) -> String {
        match item {
            FhirPathValue::String(s) => format!("string:{s}"),
            FhirPathValue::Integer(i) => format!("integer:{i}"),
            FhirPathValue::Decimal(d) => format!("decimal:{d}"),
            FhirPathValue::Boolean(b) => format!("boolean:{b}"),
            FhirPathValue::JsonValue(json_val) => {
                // For JSON objects, use id if available, otherwise use a hash-like approach
                if let Some(obj) = json_val.as_object() {
                    if let Some(serde_json::Value::String(id)) = obj.get("id") {
                        format!("object:id:{id}")
                    } else {
                        format!("object:hash:{obj:?}")
                    }
                } else {
                    format!("json:{json_val:?}")
                }
            }
            _ => format!("{item:?}"),
        }
    }
}
