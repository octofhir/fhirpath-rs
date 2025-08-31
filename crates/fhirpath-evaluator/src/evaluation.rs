//! Core FHIRPath expression evaluation logic
//!
//! This module contains the primary expression evaluation engine that dispatches
//! to specialized evaluators based on expression node types. It handles recursion
//! depth checking, performance monitoring, and delegates to appropriate evaluators.

use crate::context::EvaluationContext as LocalEvaluationContext;
use crate::parsing::{parse_fhirpath_date, parse_fhirpath_datetime, parse_fhirpath_time};
use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::{Collection, FhirPathValue};
use std::str::FromStr;

/// Core evaluation methods for the FHIRPath engine
impl crate::FhirPathEngine {
    /// Main async evaluation method for expression nodes
    /// Handles recursion depth checking and performance monitoring
    pub fn evaluate_node_async<'a>(
        &'a self,
        node: &'a ExpressionNode,
        input: FhirPathValue,
        context: &'a LocalEvaluationContext,
        depth: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = EvaluationResult<FhirPathValue>> + Send + 'a>,
    > {
        Box::pin(async move {
            // Recursion depth check
            if depth > self.config().max_recursion_depth {
                return Err(EvaluationError::InvalidOperation {
                    message: format!(
                        "Recursion depth exceeded: max depth is {}",
                        self.config().max_recursion_depth
                    ),
                });
            }

            // Performance monitoring hook
            let start_time = std::time::Instant::now();
            let result = self
                .evaluate_node_internal(node, input, context, depth)
                .await;
            let duration = start_time.elapsed();

            // Log slow evaluations (optional)
            if duration.as_millis() > 1000 {
                // TODO: Add logging when log crate is available
                eprintln!("Slow evaluation: took {}ms", duration.as_millis());
            }

            result
        })
    }

    /// Internal node evaluation with pattern matching
    /// Dispatches to appropriate specialized evaluators based on node type
    pub async fn evaluate_node_internal(
        &self,
        node: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::ExpressionNode;

        match node {
            // Simple cases - direct evaluation
            ExpressionNode::Literal(lit) => self.evaluate_literal(lit),

            ExpressionNode::Identifier(id) => {
                return self.evaluate_identifier_async(id, &input, context).await;
            }

            ExpressionNode::Index { base, index } => {
                self.evaluate_index(base, index, input, context, depth)
                    .await
            }

            ExpressionNode::Path { base, path } => {
                self.evaluate_path(base, path, input, context, depth).await
            }

            // Complex cases - delegate to specialized methods
            ExpressionNode::FunctionCall(func_data) => {
                if self.is_lambda_function(&func_data.name).await {
                    self.evaluate_lambda_function(func_data, input, context, depth)
                        .await
                } else {
                    self.evaluate_standard_function(func_data, input, context, depth)
                        .await
                }
            }

            ExpressionNode::BinaryOp(op_data) => {
                self.evaluate_binary_operation(op_data, input, context, depth)
                    .await
            }

            ExpressionNode::UnaryOp { op, operand } => {
                self.evaluate_unary_operation(op, operand, input, context, depth)
                    .await
            }

            ExpressionNode::MethodCall(method_data) => {
                self.evaluate_method_call(method_data, input, context, depth)
                    .await
            }

            ExpressionNode::Lambda(lambda_data) => {
                self.evaluate_lambda_expression(lambda_data, input, context, depth)
                    .await
            }

            ExpressionNode::Conditional(cond_data) => {
                self.evaluate_conditional(cond_data, input, context, depth)
                    .await
            }

            ExpressionNode::Variable(var_name) => self.evaluate_variable(var_name, context),

            ExpressionNode::Filter { base, condition } => {
                self.evaluate_filter(base, condition, input, context, depth)
                    .await
            }

            ExpressionNode::Union { left, right } => {
                self.evaluate_union(left, right, input, context, depth)
                    .await
            }

            ExpressionNode::TypeCheck {
                expression,
                type_name,
            } => {
                self.evaluate_type_check(expression, type_name, input, context, depth)
                    .await
            }

            ExpressionNode::TypeCast {
                expression,
                type_name,
            } => {
                self.evaluate_type_cast(expression, type_name, input, context, depth)
                    .await
            }
        }
    }

    /// Evaluate literal values (strings, numbers, dates, etc.)
    pub fn evaluate_literal(&self, literal: &LiteralValue) -> EvaluationResult<FhirPathValue> {
        use octofhir_fhirpath_ast::LiteralValue;

        match literal {
            LiteralValue::String(s) => Ok(FhirPathValue::String(s.clone().into())),
            LiteralValue::Integer(i) => Ok(FhirPathValue::Integer(*i)),
            LiteralValue::Decimal(d) => Ok(FhirPathValue::Decimal(
                rust_decimal::Decimal::from_str(d).map_err(|_| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid decimal value: {d}"),
                    }
                })?,
            )),
            LiteralValue::Boolean(b) => Ok(FhirPathValue::Boolean(*b)),
            LiteralValue::Date(date_str) => match parse_fhirpath_date(date_str) {
                Ok(date) => Ok(FhirPathValue::Date(date)),
                Err(e) => Err(EvaluationError::InvalidOperation {
                    message: format!("Invalid date literal '{date_str}': {e}"),
                }),
            },
            LiteralValue::DateTime(datetime_str) => match parse_fhirpath_datetime(datetime_str) {
                Ok(datetime) => Ok(FhirPathValue::DateTime(datetime)),
                Err(e) => Err(EvaluationError::InvalidOperation {
                    message: format!("Invalid datetime literal '{datetime_str}': {e}"),
                }),
            },
            LiteralValue::Time(time_str) => match parse_fhirpath_time(time_str) {
                Ok(time) => Ok(FhirPathValue::Time(time)),
                Err(e) => Err(EvaluationError::InvalidOperation {
                    message: format!("Invalid time literal '{time_str}': {e}"),
                }),
            },
            LiteralValue::Quantity { value, unit } => {
                let decimal_value = rust_decimal::Decimal::from_str(value).map_err(|_| {
                    EvaluationError::InvalidOperation {
                        message: format!("Invalid quantity value: {value}"),
                    }
                })?;
                let quantity =
                    octofhir_fhirpath_model::Quantity::new(decimal_value, Some(unit.clone()));
                Ok(FhirPathValue::Quantity(std::sync::Arc::new(quantity)))
            }
            LiteralValue::Null => Ok(FhirPathValue::Empty),
        }
    }

    /// Evaluate identifier expressions (property access)
    /// Async identifier evaluation with polymorphic navigation support
    pub async fn evaluate_identifier_async(
        &self,
        identifier: &str,
        input: &FhirPathValue,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // If bridge navigation is enabled, use it first (priority over polymorphic)
        if let Some(bridge_navigator) = &self.bridge_navigator {
            // Clone the navigator to avoid mutable borrow issues
            let mut navigator = bridge_navigator.clone();
            match navigator
                .navigate_property(input, identifier, context)
                .await
            {
                Ok(result) => {
                    // Bridge navigation succeeded
                    return Ok(result);
                }
                Err(_) => {
                    // Bridge navigation failed, continue with fallback methods
                }
            }
        }

        // If polymorphic navigation is enabled, try it as fallback
        if let Some(polymorphic_engine) = self.polymorphic_engine() {
            if let Ok(nav_result) = polymorphic_engine.navigate_path(input, identifier).await {
                if nav_result.used_choice_resolution || !nav_result.values.is_empty() {
                    return Ok(FhirPathValue::normalize_collection_result(
                        nav_result.values,
                    ));
                }
            }
        }

        // Fall back to standard identifier evaluation
        self.evaluate_identifier(identifier, input, context)
    }

    pub fn evaluate_identifier(
        &self,
        identifier: &str,
        input: &FhirPathValue,
        _context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        match input {
            FhirPathValue::JsonValue(json_val) => {
                // Check if identifier is a resource type that matches the current context
                if let Some(resource_type_value) = json_val.get_property("resourceType") {
                    if let Some(resource_type) = resource_type_value.as_str() {
                        if resource_type == identifier {
                            // Return the current context (self-reference)
                            return Ok(input.clone());
                        }
                    }
                }

                // Access property from JSON object
                if let Some(prop_value) = json_val.get_property(identifier) {
                    // Convert JsonValue property to proper FhirPathValue type
                    // For FHIR primitives, preserve context by creating a JsonValue wrapper for scalar types
                    let inner = prop_value.as_inner();
                    if inner.is_boolean()
                        || inner.is_number()
                        || (inner.is_string() && !inner.as_str().unwrap_or("").starts_with('@'))
                    {
                        // Primitive FHIR types - preserve context with JsonValue wrapper
                        Ok(FhirPathValue::JsonValue(prop_value))
                    } else {
                        // Arrays and complex objects - convert normally to enable proper collection flattening
                        Ok(FhirPathValue::from(prop_value.as_inner().clone()))
                    }
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Resource(resource) => {
                // Check if identifier is a resource type that matches the current context
                if let Some(resource_type_value) = resource.get_property("resourceType") {
                    if let Some(resource_type) = resource_type_value.as_str() {
                        if resource_type == identifier {
                            // Return the current context (self-reference)
                            return Ok(input.clone());
                        }
                    }
                }

                // Access property from FHIR Resource
                if let Some(prop_value) = resource.get_property(identifier) {
                    let result = FhirPathValue::from(prop_value);
                    // Collections are already flattened in the From conversion
                    Ok(result)
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            FhirPathValue::Collection(items) => {
                let mut results = Vec::new();
                for item in items.iter() {
                    match self.evaluate_identifier(identifier, item, _context)? {
                        FhirPathValue::Empty => {} // Skip empty results
                        FhirPathValue::Collection(sub_items) => {
                            results.extend(sub_items.into_iter());
                        }
                        other => results.push(other),
                    }
                }
                Ok(FhirPathValue::Collection(Collection::from(results)))
            }
            // Handle TypeInfoObject property access for .namespace and .name
            FhirPathValue::TypeInfoObject { namespace, name } => match identifier {
                "namespace" => Ok(FhirPathValue::String(namespace.clone())),
                "name" => Ok(FhirPathValue::String(name.clone())),
                _ => Ok(FhirPathValue::Empty),
            },
            // Handle Quantity property access
            FhirPathValue::Quantity(quantity) => match identifier {
                "value" => Ok(FhirPathValue::Decimal(quantity.value)),
                "unit" => {
                    if let Some(ref unit) = quantity.unit {
                        Ok(FhirPathValue::String(unit.clone().into()))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                _ => Ok(FhirPathValue::Empty),
            },
            _ => Ok(FhirPathValue::Empty), // Non-object types don't have properties
        }
    }

    /// Evaluate variable access
    pub fn evaluate_variable(
        &self,
        var_name: &str,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        if let Some(value) = context.variable_scope.get_variable(var_name) {
            Ok(value.clone())
        } else {
            // Check for built-in environment variables
            match var_name {
                "context" => Ok(context.root.as_ref().clone()),
                "resource" => Ok(context.root.as_ref().clone()),
                "rootResource" => Ok(context.root.as_ref().clone()),
                "sct" => Ok(FhirPathValue::String("http://snomed.info/sct".into())),
                "loinc" => Ok(FhirPathValue::String("http://loinc.org".into())),
                "ucum" => Ok(FhirPathValue::String("http://unitsofmeasure.org".into())),
                // Special lambda/iteration variables - use focus as fallback
                "this" => {
                    // $this refers to the current focus item being processed
                    // Check lambda metadata first, then fall back to input
                    if let Some(lambda_meta) = &context.variable_scope.lambda_metadata {
                        Ok(lambda_meta.current_item.clone())
                    } else {
                        Ok(context.input.clone())
                    }
                }
                "index" => {
                    // $index refers to the current iteration index (0-based)
                    if let Some(lambda_meta) = &context.variable_scope.lambda_metadata {
                        Ok(lambda_meta.current_index.clone())
                    } else {
                        Ok(FhirPathValue::Integer(0))
                    }
                }
                "total" => {
                    // $total refers to accumulator in aggregate functions
                    if let Some(lambda_meta) = &context.variable_scope.lambda_metadata {
                        Ok(lambda_meta.total_value.clone())
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                _ => Err(EvaluationError::InvalidOperation {
                    message: format!("Variable '{var_name}' not found"),
                }),
            }
        }
    }

    /// Evaluate index expressions (e.g., collection[0])
    pub async fn evaluate_index(
        &self,
        base: &ExpressionNode,
        index_expr: &ExpressionNode,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        let base_result = self
            .evaluate_node_async(base, input.clone(), context, depth + 1)
            .await?;

        let index_result = self
            .evaluate_node_async(index_expr, input, context, depth + 1)
            .await?;

        match (&base_result, &index_result) {
            (FhirPathValue::Collection(items), FhirPathValue::Integer(idx)) => {
                let index = if *idx < 0 {
                    // Negative indices count from the end
                    let len = items.len() as i64;
                    (len + idx) as usize
                } else {
                    *idx as usize
                };

                if index < items.len() {
                    Ok(items.get(index).cloned().unwrap_or(FhirPathValue::Empty))
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            (_, FhirPathValue::Integer(_)) => {
                // Single value indexing - only index 0 is valid
                if let FhirPathValue::Integer(0) = index_result {
                    Ok(base_result)
                } else {
                    Ok(FhirPathValue::Empty)
                }
            }
            _ => Ok(FhirPathValue::Empty), // Invalid index type
        }
    }

    /// Evaluate path expressions (e.g., Patient.name.given)
    pub async fn evaluate_path(
        &self,
        base: &ExpressionNode,
        path: &str,
        input: FhirPathValue,
        context: &LocalEvaluationContext,
        depth: usize,
    ) -> EvaluationResult<FhirPathValue> {
        let base_result = self
            .evaluate_node_async(base, input, context, depth + 1)
            .await?;

        self.evaluate_identifier_async(path, &base_result, context)
            .await
    }

    /// Evaluate the 'is' binary operator for type checking
    pub async fn evaluate_is_operator(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
        context: &LocalEvaluationContext,
    ) -> EvaluationResult<FhirPathValue> {
        // Get the Is operation from the registry and delegate to it
        if self.registry().has_function("is").await {
            let registry_context = octofhir_fhirpath_registry::traits::EvaluationContext {
                input: left.clone(),
                root: context.root.clone(),
                variables: Default::default(),
                model_provider: self.model_provider().clone(),
            };

            // Call the Is operation with only the type identifier as argument (function-style)
            // The value to check is already in the registry_context.input
            let result = self
                .registry()
                .evaluate("is", &[right.clone()], &registry_context)
                .await
                .map_err(|e| EvaluationError::InvalidOperation {
                    message: format!("is operator error: {e}"),
                })?;

            // Extract the boolean result from the collection wrapper
            match result {
                FhirPathValue::Collection(items) => {
                    if let Some(FhirPathValue::Boolean(result)) = items.first() {
                        Ok(FhirPathValue::Boolean(*result))
                    } else if items.is_empty() {
                        Ok(FhirPathValue::Boolean(false))
                    } else {
                        Ok(FhirPathValue::Boolean(false))
                    }
                }
                FhirPathValue::Boolean(result) => Ok(FhirPathValue::Boolean(result)),
                _ => Ok(FhirPathValue::Boolean(false)),
            }
        } else {
            Err(EvaluationError::InvalidOperation {
                message: "is operation not found in registry".to_string(),
            })
        }
    }
}
