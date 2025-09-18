//! FHIRPath expression evaluator implementation
//!
//! This module provides the main Evaluator struct that replaces the stub implementation
//! with a registry-based architecture for operators and functions.

use std::sync::Arc;

use super::context::EvaluationContext;
use crate::ast::ExpressionNode;
use crate::core::trace::SharedTraceProvider;
use crate::core::{Collection, FhirPathError, FhirPathValue, ModelProvider, Result};
use crate::evaluator::operations::union_operator::UnionOperatorEvaluator;
use crate::evaluator::operator_registry::OperationEvaluator;
use crate::evaluator::{EvaluationResult, EvaluationResultWithMetadata};
use octofhir_fhir_model::TerminologyProvider;

use super::function_registry::FunctionRegistry;
use super::operator_registry::OperatorRegistry;

/// Main FHIRPath expression evaluator with registry-based architecture
pub struct Evaluator {
    /// Registry for operators (=, +, -, etc.)
    operator_registry: Arc<OperatorRegistry>,
    /// Registry for functions (count(), where(), select(), etc.)
    function_registry: Arc<FunctionRegistry>,
    /// Model provider for type information
    model_provider: Arc<dyn ModelProvider>,
    /// Optional terminology provider for terminology functions
    terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    /// Optional trace provider for trace function
    trace_provider: Option<SharedTraceProvider>,
}

impl Evaluator {
    /// Create a new evaluator with the provided registries and providers
    pub fn new(
        operator_registry: Arc<OperatorRegistry>,
        function_registry: Arc<FunctionRegistry>,
        model_provider: Arc<dyn ModelProvider>,
        terminology_provider: Option<Arc<dyn TerminologyProvider>>,
    ) -> Self {
        Self {
            operator_registry,
            function_registry,
            model_provider,
            terminology_provider,
            trace_provider: None,
        }
    }

    /// Get the function registry
    pub fn function_registry(&self) -> &Arc<FunctionRegistry> {
        &self.function_registry
    }

    /// Get the operator registry
    pub fn operator_registry(&self) -> &Arc<OperatorRegistry> {
        &self.operator_registry
    }

    /// Get the model provider
    pub fn model_provider(&self) -> Arc<dyn ModelProvider> {
        self.model_provider.clone()
    }

    /// Get the terminology provider
    pub fn terminology_provider(&self) -> Option<Arc<dyn TerminologyProvider>> {
        self.terminology_provider.clone()
    }

    /// Add terminology provider to the evaluator
    pub fn with_terminology_provider(mut self, provider: Arc<dyn TerminologyProvider>) -> Self {
        self.terminology_provider = Some(provider);
        self
    }

    /// Get the trace provider
    pub fn trace_provider(&self) -> Option<SharedTraceProvider> {
        self.trace_provider.clone()
    }

    /// Add trace provider to the evaluator
    pub fn with_trace_provider(mut self, provider: SharedTraceProvider) -> Self {
        self.trace_provider = Some(provider);
        self
    }

    /// Evaluate an AST node within the given context
    pub async fn evaluate_node(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        Box::pin(self.evaluate_node_inner(node, context)).await
    }

    /// Inner evaluation method to handle recursion
    async fn evaluate_node_inner(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        match node {
            ExpressionNode::Literal(literal_node) => {
                // Convert literal to FhirPathValue
                let value = self.evaluate_literal(&literal_node.value)?;
                Ok(EvaluationResult {
                    value: Collection::single(value),
                })
            }
            ExpressionNode::Identifier(identifier_node) => {
                // Navigate to property on input collection
                self.evaluate_path(&identifier_node.name, context).await
            }
            ExpressionNode::BinaryOperation(binary_op) => {
                // Evaluate both operands first
                let left_result =
                    Box::pin(self.evaluate_node_inner(&binary_op.left, context)).await?;
                let right_result =
                    Box::pin(self.evaluate_node_inner(&binary_op.right, context)).await?;

                // Dispatch to operator registry
                self.evaluate_binary_operation(
                    &binary_op.operator,
                    left_result.value,
                    right_result.value,
                    context,
                )
                .await
            }
            ExpressionNode::UnaryOperation(unary_op) => {
                // Evaluate operand first
                let operand_result =
                    Box::pin(self.evaluate_node_inner(&unary_op.operand, context)).await?;

                // Dispatch to operator registry for unary operations
                self.evaluate_unary_operation(&unary_op.operator, operand_result.value, context)
                    .await
            }
            ExpressionNode::FunctionCall(function_call) => {
                // Dispatch to function registry
                self.evaluate_function_call(&function_call.name, &function_call.arguments, context)
                    .await
            }
            ExpressionNode::IndexAccess(index_access) => {
                // Evaluate collection first, then apply index
                let collection_result =
                    Box::pin(self.evaluate_node_inner(&index_access.object, context)).await?;
                let index_result =
                    Box::pin(self.evaluate_node_inner(&index_access.index, context)).await?;

                self.evaluate_index_operation(collection_result.value, index_result.value)
                    .await
            }
            ExpressionNode::PropertyAccess(property_access) => {
                // Evaluate object first, then navigate to member
                let object_result =
                    Box::pin(self.evaluate_node_inner(&property_access.object, context)).await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                self.evaluate_path(&property_access.property, &new_context)
                    .await
            }
            ExpressionNode::MethodCall(method_call) => {
                // Evaluate object first, then call method
                let object_result =
                    Box::pin(self.evaluate_node_inner(&method_call.object, context)).await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                self.evaluate_function_call(
                    &method_call.method,
                    &method_call.arguments,
                    &new_context,
                )
                .await
            }
            ExpressionNode::Collection(collection_node) => {
                // Evaluate collection literal
                self.evaluate_collection(&collection_node.elements, context)
                    .await
            }
            ExpressionNode::Variable(variable_node) => {
                // Evaluate variable access ($this, $index, $total, user variables)
                self.evaluate_variable(&variable_node.name, context).await
            }
            ExpressionNode::Parenthesized(expr) => {
                Box::pin(self.evaluate_node_inner(expr, context)).await
            }
            ExpressionNode::Union(union_node) => {
                let left_result =
                    Box::pin(self.evaluate_node_inner(&union_node.left, context)).await?;
                let right_result =
                    Box::pin(self.evaluate_node_inner(&union_node.right, context)).await?;

                self.evaluate_union_operator(left_result.value, right_result.value, context)
                    .await
            }
            ExpressionNode::TypeCheck(type_check) => {
                // Evaluate the expression being type-checked
                let expression_result =
                    Box::pin(self.evaluate_node_inner(&type_check.expression, context)).await?;

                // Create a new context with the expression result as input
                let new_context = EvaluationContext::new(
                    expression_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                // Create an identifier node for the type name to pass to the is function
                let type_arg = ExpressionNode::Identifier(crate::ast::expression::IdentifierNode {
                    name: type_check.target_type.clone(),
                    location: None,
                });

                // Delegate to the is function
                self.evaluate_function_call("is", &[type_arg], &new_context)
                    .await
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Expression type not yet implemented: {:?}", node),
            )),
        }
    }

    /// Evaluate an AST node with metadata collection
    pub async fn evaluate_node_with_metadata(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        // Create metadata collector for this evaluation session
        let metadata_collector = Arc::new(super::metadata_collector::MetadataCollector::new());

        // Evaluate with metadata collection
        let result =
            Box::pin(self.evaluate_node_with_collector(node, context, &metadata_collector, 0))
                .await?;

        // Build comprehensive metadata
        let metadata = crate::evaluator::stub::EvaluationMetadata {
            execution_time: metadata_collector.execution_time(),
            node_evaluations: metadata_collector.node_evaluations(),
            type_resolutions: metadata_collector.type_resolutions(),
            cache_stats: metadata_collector.cache_stats(),
            trace_events: metadata_collector.trace_events(),
            performance_metrics: metadata_collector.performance_metrics(),
            session_id: metadata_collector.session_id().to_string(),
        };

        Ok(EvaluationResultWithMetadata {
            value: result.value,
            metadata,
        })
    }

    /// Evaluate an AST node with metadata collection tracking
    async fn evaluate_node_with_collector(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
        collector: &Arc<super::metadata_collector::MetadataCollector>,
        depth: usize,
    ) -> Result<EvaluationResult> {
        use super::metadata_collector::{NodeEvaluationInfo, TraceEvent};
        use std::time::Instant;

        let start_time = Instant::now();
        let node_type = format!("{:?}", node)
            .split('(')
            .next()
            .unwrap_or("Unknown")
            .to_string();
        let evaluation_id = collector.node_evaluations().len();

        // Record evaluation start
        collector.record_trace_event(TraceEvent::EvaluationStart {
            node_type: node_type.clone(),
            input_count: context.input_collection().len(),
            depth,
            timestamp: start_time.elapsed(),
        });

        // Update depth statistics
        collector.update_depth_stats(depth);

        // Perform the actual evaluation
        let result = match node {
            ExpressionNode::Literal(literal_node) => {
                let value = self.evaluate_literal(&literal_node.value)?;
                Ok(EvaluationResult {
                    value: Collection::single(value),
                })
            }
            ExpressionNode::Identifier(identifier_node) => {
                // Record property access timing
                let prop_start = Instant::now();
                let result = self.evaluate_path(&identifier_node.name, context).await;
                let prop_time = prop_start.elapsed();
                collector.record_property_timing(
                    &identifier_node.name,
                    prop_time,
                    std::time::Duration::ZERO,
                );

                // Record property access trace
                collector.record_trace_event(TraceEvent::PropertyAccess {
                    property_name: identifier_node.name.clone(),
                    input_count: context.input_collection().len(),
                    timestamp: start_time.elapsed(),
                });

                result
            }
            ExpressionNode::BinaryOperation(binary_op) => {
                let left_result = Box::pin(self.evaluate_node_with_collector(
                    &binary_op.left,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;
                let right_result = Box::pin(self.evaluate_node_with_collector(
                    &binary_op.right,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;

                // Record counts before moving values
                let left_count = left_result.value.len();
                let right_count = right_result.value.len();

                // Record operator timing
                let op_start = Instant::now();
                let result = self
                    .evaluate_binary_operation(
                        &binary_op.operator,
                        left_result.value,
                        right_result.value,
                        context,
                    )
                    .await;
                let op_time = op_start.elapsed();
                collector.record_operator_timing(&format!("{:?}", binary_op.operator), op_time);

                // Record operator trace
                collector.record_trace_event(TraceEvent::OperatorEvaluation {
                    operator: format!("{:?}", binary_op.operator),
                    left_count,
                    right_count,
                    timestamp: start_time.elapsed(),
                });

                result
            }
            ExpressionNode::UnaryOperation(unary_op) => {
                let operand_result = Box::pin(self.evaluate_node_with_collector(
                    &unary_op.operand,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;

                // Record operator timing
                let op_start = Instant::now();
                let result = self
                    .evaluate_unary_operation(&unary_op.operator, operand_result.value, context)
                    .await;
                let op_time = op_start.elapsed();
                collector.record_operator_timing(&format!("{:?}", unary_op.operator), op_time);

                result
            }
            ExpressionNode::FunctionCall(function_call) => {
                // Record function timing
                let func_start = Instant::now();
                let result = self
                    .evaluate_function_call(&function_call.name, &function_call.arguments, context)
                    .await;
                let func_time = func_start.elapsed();
                collector.record_function_timing(&function_call.name, func_time);

                // Record function call trace
                collector.record_trace_event(TraceEvent::FunctionCall {
                    function_name: function_call.name.clone(),
                    input_count: context.input_collection().len(),
                    parameter_count: function_call.arguments.len(),
                    timestamp: start_time.elapsed(),
                });

                result
            }
            ExpressionNode::IndexAccess(index_access) => {
                let collection_result = Box::pin(self.evaluate_node_with_collector(
                    &index_access.object,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;
                let index_result = Box::pin(self.evaluate_node_with_collector(
                    &index_access.index,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;

                self.evaluate_index_operation(collection_result.value, index_result.value)
                    .await
            }
            ExpressionNode::PropertyAccess(property_access) => {
                let object_result = Box::pin(self.evaluate_node_with_collector(
                    &property_access.object,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                // Record property access timing
                let prop_start = Instant::now();
                let result = self
                    .evaluate_path(&property_access.property, &new_context)
                    .await;
                let prop_time = prop_start.elapsed();
                collector.record_property_timing(
                    &property_access.property,
                    prop_time,
                    std::time::Duration::ZERO,
                );

                result
            }
            ExpressionNode::MethodCall(method_call) => {
                let object_result = Box::pin(self.evaluate_node_with_collector(
                    &method_call.object,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                // Record method call timing
                let method_start = Instant::now();
                let result = self
                    .evaluate_function_call(
                        &method_call.method,
                        &method_call.arguments,
                        &new_context,
                    )
                    .await;
                let method_time = method_start.elapsed();
                collector.record_function_timing(&method_call.method, method_time);

                result
            }
            ExpressionNode::Collection(collection_node) => {
                self.evaluate_collection(&collection_node.elements, context)
                    .await
            }
            ExpressionNode::Variable(variable_node) => {
                self.evaluate_variable(&variable_node.name, context).await
            }
            ExpressionNode::Parenthesized(expr) => {
                Box::pin(self.evaluate_node_with_collector(expr, context, collector, depth)).await
            }
            ExpressionNode::Union(union_node) => {
                let left_result = Box::pin(self.evaluate_node_with_collector(
                    &union_node.left,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;
                let right_result = Box::pin(self.evaluate_node_with_collector(
                    &union_node.right,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;

                self.evaluate_union_operator(left_result.value, right_result.value, context)
                    .await
            }
            ExpressionNode::TypeCheck(type_check) => {
                // Evaluate the expression being type-checked
                let expression_result = Box::pin(self.evaluate_node_with_collector(
                    &type_check.expression,
                    context,
                    collector,
                    depth + 1,
                ))
                .await?;

                // Create a new context with the expression result as input
                let new_context = EvaluationContext::new(
                    expression_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                    self.trace_provider.clone(),
                )
                .await;

                // Create an identifier node for the type name to pass to the is function
                let type_arg = ExpressionNode::Identifier(crate::ast::expression::IdentifierNode {
                    name: type_check.target_type.clone(),
                    location: None,
                });

                // Record type check timing
                let type_check_start = Instant::now();
                let result = self
                    .evaluate_function_call("is", &[type_arg], &new_context)
                    .await;
                let type_check_time = type_check_start.elapsed();
                collector.record_function_timing("is", type_check_time);

                result
            }
            _ => Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Expression type not yet implemented: {:?}", node),
            )),
        };

        // Record execution time and result
        let execution_time = start_time.elapsed();
        let (output_count, error_msg) = match &result {
            Ok(res) => (res.value.len(), None),
            Err(err) => (0, Some(err.to_string())),
        };

        // Record node evaluation info
        collector.record_node_evaluation(NodeEvaluationInfo {
            node_type: node_type.clone(),
            node_location: None, // TODO: Add source location when parser provides it
            input_count: context.input_collection().len(),
            output_count,
            execution_time,
            error: error_msg,
            input_types: context
                .input_collection()
                .iter()
                .map(|v| {
                    format!("{:?}", v)
                        .split('(')
                        .next()
                        .unwrap_or("Unknown")
                        .to_string()
                })
                .collect(),
            output_types: result
                .as_ref()
                .map(|res| {
                    res.value
                        .iter()
                        .map(|v| {
                            format!("{:?}", v)
                                .split('(')
                                .next()
                                .unwrap_or("Unknown")
                                .to_string()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            depth,
            evaluation_id,
        });

        // Record evaluation end
        collector.record_trace_event(TraceEvent::EvaluationEnd {
            node_type,
            execution_time,
            success: result.is_ok(),
            output_count,
            timestamp: start_time.elapsed(),
        });

        result
    }

    /// Evaluate a literal value
    fn evaluate_literal(&self, literal: &crate::ast::LiteralValue) -> Result<FhirPathValue> {
        use crate::ast::LiteralValue;

        match literal {
            LiteralValue::Boolean(b) => Ok(FhirPathValue::boolean(*b)),
            LiteralValue::Integer(i) => Ok(FhirPathValue::integer(*i)),
            LiteralValue::Decimal(d) => Ok(FhirPathValue::decimal(*d)),
            LiteralValue::String(s) => Ok(FhirPathValue::string(s.clone())),
            LiteralValue::Date(date) => Ok(FhirPathValue::date(date.clone())),
            LiteralValue::DateTime(datetime) => Ok(FhirPathValue::datetime(datetime.clone())),
            LiteralValue::Time(time) => Ok(FhirPathValue::time(time.clone())),
            LiteralValue::Quantity { value, unit } => {
                Ok(FhirPathValue::quantity(*value, unit.clone()))
            }
        }
    }

    /// Evaluate a path navigation (property access) with enhanced ModelProvider integration
    async fn evaluate_path(
        &self,
        identifier: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Check if identifier starts with capital letter (potential resource type)
        let is_resource_type_check = identifier
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);

        let mut result_values = Vec::new();

        // Navigate each item in the input collection
        for item in context.input_collection().iter() {
            match item {
                FhirPathValue::Resource(json, type_info, _) => {
                    // Handle resource type validation when identifier starts with capital letter
                    if is_resource_type_check {
                        // Extract resourceType from JSON
                        let actual_resource_type = json
                            .get("resourceType")
                            .and_then(|rt| rt.as_str())
                            .ok_or_else(|| {
                                FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0054,
                                    "Resource does not have a resourceType field".to_string(),
                                )
                            })?;

                        // Validate that the resource type matches the identifier
                        if actual_resource_type == identifier {
                            // Resource type matches - return the resource with proper type info
                            let resource_type_info = self
                                .model_provider
                                .get_type(identifier)
                                .await
                                .map_err(|e| {
                                    FhirPathError::evaluation_error(
                                        crate::core::error_code::FP0054,
                                        format!(
                                            "ModelProvider error getting type '{}': {}",
                                            identifier, e
                                        ),
                                    )
                                })?
                                .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                                    type_name: identifier.to_string(),
                                    singleton: Some(true),
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(identifier.to_string()),
                                    is_empty: Some(false),
                                });

                            let resource_value = FhirPathValue::wrap_value(
                                crate::core::value::utils::json_to_fhirpath_value((**json).clone()),
                                resource_type_info,
                                None,
                            );
                            result_values.push(resource_value);
                        } else {
                            // Resource type mismatch - return empty per FHIRPath spec
                            // Semantic analysis will catch this during development, but runtime should be lenient
                            continue;
                        }
                        continue;
                    }

                    // Use ModelProvider to get element type information
                    let property_type_info = if let Some(element_type) = self
                        .model_provider
                        .get_element_type(&type_info, identifier)
                        .await
                        .map_err(|e| {
                            FhirPathError::evaluation_error(
                                crate::core::error_code::FP0054,
                                format!("Model provider error getting element type: {}", e),
                            )
                        })? {
                        element_type
                    } else {
                        // Fallback type info
                        crate::core::model_provider::TypeInfo {
                            type_name: "Unknown".to_string(),
                            singleton: Some(true),
                            namespace: Some("FHIR".to_string()),
                            name: Some(identifier.to_string()),
                            is_empty: Some(false),
                        }
                    };

                    // Extract the value directly from JSON
                    if let Some(property_value) = json.get(identifier) {
                        let flattened_values = self
                            .navigate_property_with_flattening(property_value, &property_type_info)
                            .await?;
                        result_values.extend(flattened_values);
                    } else {
                        // Check for choice types (valueX properties)
                        if identifier.starts_with("value") && identifier.len() > 5 {
                            let choice_results = self
                                .navigate_choice_property(json, identifier, &type_info.type_name)
                                .await?;
                            result_values.extend(choice_results);
                        }
                        // Check for extension access
                        else if identifier.starts_with("extension") {
                            let extension_results =
                                self.navigate_extension_property(json, identifier).await?;
                            result_values.extend(extension_results);
                        }
                        // Check for contained resource navigation
                        else if identifier == "contained" {
                            let contained_results = self.navigate_contained_resources(json).await?;
                            result_values.extend(contained_results);
                        }
                        // Check for Bundle resource navigation patterns
                        else if identifier == "resource" && type_info.type_name == "Bundle" {
                            let bundle_results = self.navigate_bundle_resources(json).await?;
                            result_values.extend(bundle_results);
                        }
                        // Fallback to direct JSON navigation for unknown properties
                        else if let Some(property_value) = json.get(identifier) {
                            let fallback_type_info = crate::core::model_provider::TypeInfo {
                                type_name: "Unknown".to_string(),
                                singleton: Some(true),
                                namespace: Some("FHIR".to_string()),
                                name: Some(identifier.to_string()),
                                is_empty: Some(false),
                            };
                            let flattened_values = self
                                .navigate_property_with_flattening(
                                    property_value,
                                    &fallback_type_info,
                                )
                                .await?;
                            result_values.extend(flattened_values);
                        }
                        // Property not found - return empty collection (standard FHIRPath)
                        else {
                            // Standard FHIRPath behavior: unknown properties return empty
                        }
                    }
                }
                FhirPathValue::Collection(collection) => {
                    // Navigate into each item of the collection
                    for sub_item in collection.iter() {
                        if let FhirPathValue::Resource(json, type_info, _) = sub_item {
                            // Use ModelProvider to get element type for collection items
                            let property_type_info = if let Some(element_type) = self
                                .model_provider
                                .get_element_type(&type_info, identifier)
                                .await
                                .map_err(|e| {
                                    FhirPathError::evaluation_error(
                                        crate::core::error_code::FP0054,
                                        format!("Model provider error getting element type: {}", e),
                                    )
                                })? {
                                element_type
                            } else {
                                // Fallback type info
                                crate::core::model_provider::TypeInfo {
                                    type_name: "Unknown".to_string(),
                                    singleton: Some(true),
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(identifier.to_string()),
                                    is_empty: Some(false),
                                }
                            };

                            // Extract the value directly from JSON
                            if let Some(property_value) = json.get(identifier) {
                                let flattened_values = self
                                    .navigate_property_with_flattening(
                                        property_value,
                                        &property_type_info,
                                    )
                                    .await?;
                                result_values.extend(flattened_values);
                            } else {
                                // Apply same fallback logic as above
                                if identifier.starts_with("value") && identifier.len() > 5 {
                                    let choice_results = self
                                        .navigate_choice_property(
                                            json,
                                            identifier,
                                            &type_info.type_name,
                                        )
                                        .await?;
                                    result_values.extend(choice_results);
                                } else if identifier.starts_with("extension") {
                                    let extension_results =
                                        self.navigate_extension_property(json, identifier).await?;
                                    result_values.extend(extension_results);
                                } else if let Some(property_value) = json.get(identifier) {
                                    let fallback_type_info =
                                        crate::core::model_provider::TypeInfo {
                                            type_name: "Unknown".to_string(),
                                            singleton: Some(true),
                                            namespace: Some("FHIR".to_string()),
                                            name: Some(identifier.to_string()),
                                            is_empty: Some(false),
                                        };
                                    let flattened_values = self
                                        .navigate_property_with_flattening(
                                            property_value,
                                            &fallback_type_info,
                                        )
                                        .await?;
                                    result_values.extend(flattened_values);
                                } else {
                                    // Check if this property is valid for the current type
                                    match self
                                        .model_provider
                                        .get_element_type(type_info, identifier)
                                        .await
                                    {
                                        Ok(Some(_)) => {
                                            // Property exists but has no value - return empty (standard FHIRPath)
                                        }
                                        Ok(None) => {
                                            // Property is known but not present - return empty (standard FHIRPath)
                                        }
                                        Err(_) => {
                                            // Property is completely unknown for this type - semantic error
                                            return Err(FhirPathError::evaluation_error(
                                                crate::core::error_code::FP0054,
                                                format!(
                                                    "Unknown property '{}' on type '{}'",
                                                    identifier, type_info.type_name
                                                ),
                                            ));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                    // Other types don't have navigable properties
                    // Return empty result for this item
                }
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(result_values),
        })
    }

    /// Navigate choice type properties (valueX patterns) with enhanced ModelProvider integration
    async fn navigate_choice_property(
        &self,
        json: &serde_json::Value,
        base_property: &str,
        parent_type: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        // Use ModelProvider to get choice type metadata
        if let Some(choice_types) = self
            .model_provider
            .get_choice_types(parent_type, base_property)
            .await
            .map_err(|e| {
                FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    format!("ModelProvider error getting choice types: {}", e),
                )
            })?
        {
            for choice in choice_types {
                let property_name = format!("{}{}", base_property, choice.suffix);

                if let Some(property_value) = json.get(&property_name) {
                    // Get precise TypeInfo from ModelProvider
                    let choice_type_info = self
                        .model_provider
                        .get_type(&choice.type_name)
                        .await
                        .map_err(|e| {
                            FhirPathError::evaluation_error(
                                crate::core::error_code::FP0054,
                                format!(
                                    "ModelProvider error getting type '{}': {}",
                                    choice.type_name, e
                                ),
                            )
                        })?
                        .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                            type_name: choice.type_name.clone(),
                            singleton: Some(true),
                            namespace: Some("FHIR".to_string()),
                            name: Some(choice.type_name.clone()),
                            is_empty: Some(false),
                        });

                    // Handle array vs single values
                    match property_value {
                        serde_json::Value::Array(array) => {
                            for item in array {
                                let wrapped_value = self
                                    .wrap_json_with_type(
                                        item.clone(),
                                        &choice_type_info,
                                        &property_name,
                                        json,
                                    )
                                    .await?;
                                results.push(wrapped_value);
                            }
                        }
                        _ => {
                            let wrapped_value = self
                                .wrap_json_with_type(
                                    property_value.clone(),
                                    &choice_type_info,
                                    &property_name,
                                    json,
                                )
                                .await?;
                            results.push(wrapped_value);
                        }
                    }
                }
            }
        } else {
            // Fallback: look for common valueX patterns if ModelProvider doesn't have info
            if base_property == "value" {
                let common_types = vec![
                    "String",
                    "Integer",
                    "Decimal",
                    "Boolean",
                    "Date",
                    "DateTime",
                    "Time",
                    "Code",
                    "CodeableConcept",
                    "Coding",
                    "Quantity",
                    "Reference",
                ];

                for type_name in common_types {
                    let property_name = format!("value{}", type_name);
                    if let Some(property_value) = json.get(&property_name) {
                        let type_info = crate::core::model_provider::TypeInfo {
                            type_name: type_name.to_string(),
                            singleton: Some(true),
                            namespace: Some("System".to_string()),
                            name: Some(type_name.to_string()),
                            is_empty: Some(false),
                        };

                        let wrapped_value = self
                            .wrap_json_with_type(
                                property_value.clone(),
                                &type_info,
                                &property_name,
                                json,
                            )
                            .await?;
                        results.push(wrapped_value);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Enhanced extension handling with proper URL filtering and nested support
    async fn navigate_extension_property(
        &self,
        json: &serde_json::Value,
        property_name: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        // Handle different extension access patterns
        if property_name == "extension" {
            // Access all extensions
            results.extend(self.get_extensions(json, "extension").await?);
        } else if property_name == "modifierExtension" {
            // Access all modifier extensions
            results.extend(self.get_extensions(json, "modifierExtension").await?);
        } else if property_name.starts_with("extension(") && property_name.ends_with(')') {
            // Access extension by URL: extension('http://example.com/ext')
            let url = &property_name[10..property_name.len() - 1]
                .trim_matches('\'')
                .trim_matches('"');
            results.extend(self.filter_extensions_by_url(json, url).await?);
        }

        Ok(results)
    }

    /// Get all extensions from a property
    async fn get_extensions(
        &self,
        json: &serde_json::Value,
        extension_property: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        if let Some(extensions) = json.get(extension_property).and_then(|e| e.as_array()) {
            for ext in extensions {
                let extension_value = self.wrap_extension(ext.clone()).await?;
                results.push(extension_value);
            }
        }

        Ok(results)
    }

    /// Filter extensions by URL with support for nested extensions
    async fn filter_extensions_by_url(
        &self,
        json: &serde_json::Value,
        target_url: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        // Check main extensions
        if let Some(extensions) = json.get("extension").and_then(|e| e.as_array()) {
            results.extend(self.find_extensions_by_url(extensions, target_url).await?);
        }

        // Check modifier extensions
        if let Some(modifier_extensions) = json.get("modifierExtension").and_then(|e| e.as_array())
        {
            results.extend(
                self.find_extensions_by_url(modifier_extensions, target_url)
                    .await?,
            );
        }

        // Check primitive element extensions (e.g., _value.extension)
        for (key, value) in json.as_object().unwrap_or(&serde_json::Map::new()) {
            if key.starts_with('_') {
                if let Some(primitive_extensions) =
                    value.get("extension").and_then(|e| e.as_array())
                {
                    results.extend(
                        self.find_extensions_by_url(primitive_extensions, target_url)
                            .await?,
                    );
                }
            }
        }

        Ok(results)
    }

    /// Find extensions with matching URL in an array, including nested extensions
    async fn find_extensions_by_url(
        &self,
        extensions: &[serde_json::Value],
        target_url: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        for ext in extensions {
            if let Some(ext_obj) = ext.as_object() {
                if let Some(url) = ext_obj.get("url").and_then(|u| u.as_str()) {
                    if url == target_url {
                        let extension_value = self.wrap_extension(ext.clone()).await?;
                        results.push(extension_value);
                    }
                }

                // Check nested extensions
                if let Some(nested_extensions) = ext_obj.get("extension").and_then(|e| e.as_array())
                {
                    let nested_results =
                        Box::pin(self.find_extensions_by_url(nested_extensions, target_url))
                            .await?;
                    results.extend(nested_results);
                }
            }
        }

        Ok(results)
    }

    /// Wrap an extension JSON as a FhirPathValue with proper type info
    async fn wrap_extension(&self, extension_json: serde_json::Value) -> Result<FhirPathValue> {
        let type_info = crate::core::model_provider::TypeInfo {
            type_name: "Extension".to_string(),
            singleton: Some(true),
            namespace: Some("FHIR".to_string()),
            name: Some("Extension".to_string()),
            is_empty: Some(false),
        };

        let base_value = crate::core::value::utils::json_to_fhirpath_value(extension_json);
        Ok(FhirPathValue::wrap_value(base_value, type_info, None))
    }

    /// Evaluate variable access ($this, $index, $total, user variables)
    async fn evaluate_variable(
        &self,
        variable_name: &str,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        match variable_name {
            "this" | "$this" | "%this" => {
                // Return $this variable
                if let Some(this_value) = context.get_system_this() {
                    Ok(EvaluationResult {
                        value: Collection::single(this_value.clone()),
                    })
                } else {
                    // If $this is not set, return current input collection
                    Ok(EvaluationResult {
                        value: context.input_collection().clone(),
                    })
                }
            }
            "index" | "$index" | "%index" => {
                // Return $index variable
                if let Some(index_value) = context.get_system_index() {
                    Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::integer(index_value)),
                    })
                } else {
                    // Return empty if $index is not set
                    Ok(EvaluationResult {
                        value: Collection::empty(),
                    })
                }
            }
            "total" | "$total" | "%total" => {
                // Return $total variable - check user variables first for aggregate function support
                if let Some(total_value) = context.get_variable("$total") {
                    Ok(EvaluationResult {
                        value: Collection::single(total_value.clone()),
                    })
                } else if let Some(total_value) = context.get_system_total() {
                    // Fallback to system $total (integer-only for backwards compatibility)
                    Ok(EvaluationResult {
                        value: Collection::single(FhirPathValue::integer(total_value)),
                    })
                } else {
                    // Return empty if $total is not set
                    Ok(EvaluationResult {
                        value: Collection::empty(),
                    })
                }
            }
            _ => {
                // Check for user-defined variables
                if let Some(user_variable) = context.get_variable(variable_name) {
                    Ok(EvaluationResult {
                        value: Collection::single(user_variable.clone()),
                    })
                } else {
                    // Variable not found
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        format!("Unknown variable: {}", variable_name),
                    ))
                }
            }
        }
    }

    /// Evaluate a collection literal (e.g., {1, 2, 3})
    async fn evaluate_collection(
        &self,
        elements: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let mut collection_values = Vec::new();

        // Evaluate each element in the collection
        for element in elements {
            let element_result = Box::pin(self.evaluate_node_inner(element, context)).await?;

            // Add all values from the element result to the collection
            // This handles both single values and collections properly
            for value in element_result.value.into_iter() {
                collection_values.push(value);
            }
        }

        // Create collection with proper ordering
        // Collection literals maintain the order of their elements
        Ok(EvaluationResult {
            value: Collection::from_values_with_ordering(collection_values, true),
        })
    }

    /// Navigate property with array flattening (following FHIRPath semantics)
    async fn navigate_property_with_flattening(
        &self,
        property_value: &serde_json::Value,
        type_info: &crate::core::model_provider::TypeInfo,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        match property_value {
            serde_json::Value::Array(arr) => {
                let element_type_info = crate::core::model_provider::TypeInfo {
                    type_name: type_info
                        .name
                        .as_deref()
                        .unwrap_or(&type_info.type_name)
                        .to_string(),
                    singleton: Some(true), // Each element is singular
                    namespace: type_info.namespace.clone(),
                    name: type_info.name.clone(),
                    is_empty: Some(false),
                };

                for element in arr {
                    let fhir_value = match element {
                        serde_json::Value::Object(_) => FhirPathValue::Resource(
                            std::sync::Arc::new(element.clone()),
                            element_type_info.clone(),
                            None,
                        ),
                        _ => FhirPathValue::wrap_value(
                            crate::core::value::utils::json_to_fhirpath_value(element.clone()),
                            element_type_info.clone(),
                            None,
                        ),
                    };
                    results.push(fhir_value);
                }
            }
            _ => {
                let element_type_info = crate::core::model_provider::TypeInfo {
                    type_name: type_info
                        .name
                        .as_deref()
                        .unwrap_or(&type_info.type_name)
                        .to_string(),
                    singleton: Some(true), // Each element is singular
                    namespace: type_info.namespace.clone(),
                    name: type_info.name.clone(),
                    is_empty: Some(false),
                };

                if property_value.is_object() && property_value.get("resourceType").is_some() {
                    // This is a true FHIR resource
                    let base_value =
                        crate::core::value::utils::json_to_fhirpath_value(property_value.clone());
                    results.push(FhirPathValue::wrap_value(
                        base_value,
                        element_type_info.clone(),
                        None,
                    ));
                } else if property_value.is_object() {
                    // This is a complex FHIR type - create Resource with correct type info from model provider
                    let fhir_value = FhirPathValue::Resource(
                        std::sync::Arc::new(property_value.clone()),
                        element_type_info.clone(),
                        None,
                    );
                    results.push(fhir_value);
                } else {
                    // Primitive values - use json conversion but wrap with correct type info
                    let base_value =
                        crate::core::value::utils::json_to_fhirpath_value(property_value.clone());
                    results.push(FhirPathValue::wrap_value(
                        base_value,
                        element_type_info.clone(),
                        None,
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Convert JSON to FhirPathValue with specific type information
    async fn convert_json_to_fhirpath_with_type(
        &self,
        json: serde_json::Value,
        type_info: &crate::core::model_provider::TypeInfo,
    ) -> Result<FhirPathValue> {
        // Convert JSON to basic FhirPathValue first
        let base_value = crate::core::value::utils::json_to_fhirpath_value(json);

        // Wrap with the provided type information
        Ok(FhirPathValue::wrap_value(
            base_value,
            type_info.clone(),
            None,
        ))
    }

    /// Convert JSON value to FhirPathValue using ModelProvider for type information
    async fn convert_json_with_type_info(
        &self,
        json: serde_json::Value,
        property_name: &str,
        parent_type_info: &crate::core::model_provider::TypeInfo,
    ) -> Result<FhirPathValue> {
        // Use ModelProvider to get property type information
        let property_type_info = self
            .model_provider
            .get_element_type(parent_type_info, property_name)
            .await
            .unwrap_or(None)
            .unwrap_or_else(|| {
                // Default type info if not found
                crate::core::model_provider::TypeInfo {
                    type_name: "Unknown".to_string(),
                    singleton: Some(true),
                    namespace: None,
                    name: Some(property_name.to_string()),
                    is_empty: Some(false),
                }
            });

        // Convert JSON to FhirPathValue using type information
        let value = crate::core::value::utils::json_to_fhirpath_value(json);

        // Wrap with proper type information
        Ok(FhirPathValue::wrap_value(value, property_type_info, None))
    }

    /// Wrap JSON with type info for choice types, handling primitive elements
    async fn wrap_json_with_type(
        &self,
        value: serde_json::Value,
        type_info: &crate::core::model_provider::TypeInfo,
        property_name: &str,
        parent_object: &serde_json::Value,
    ) -> Result<FhirPathValue> {
        // Get primitive element for extensions if it exists
        let primitive_element = self.get_primitive_element(parent_object, property_name);

        // Handle temporal parsing for date/datetime/time types
        let parsed_value = if self.is_temporal_type(&type_info.type_name) {
            self.parse_temporal_if_needed(value, type_info)?
        } else {
            value
        };

        // Convert to FhirPathValue and wrap with metadata
        let base_value = crate::core::value::utils::json_to_fhirpath_value(parsed_value);
        let wrapped_primitive = primitive_element.map(|pe| crate::core::WrappedPrimitiveElement {
            id: None,
            extensions: vec![],
        });
        Ok(FhirPathValue::wrap_value(
            base_value,
            type_info.clone(),
            wrapped_primitive,
        ))
    }

    /// Check if a type is temporal (date/datetime/time)
    fn is_temporal_type(&self, type_name: &str) -> bool {
        matches!(
            type_name.to_lowercase().as_str(),
            "date" | "datetime" | "time" | "instant"
        )
    }

    /// Parse temporal value if needed
    fn parse_temporal_if_needed(
        &self,
        value: serde_json::Value,
        type_info: &crate::core::model_provider::TypeInfo,
    ) -> Result<serde_json::Value> {
        match type_info.type_name.to_lowercase().as_str() {
            "date" => {
                if let Some(date_str) = value.as_str() {
                    // Basic date validation (YYYY-MM-DD format)
                    if date_str.len() >= 4 && date_str.chars().nth(4) == Some('-') {
                        Ok(value)
                    } else {
                        Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0054,
                            format!("Invalid date format: {}", date_str),
                        ))
                    }
                } else {
                    Ok(value)
                }
            }
            "datetime" | "instant" => {
                if let Some(datetime_str) = value.as_str() {
                    // Basic datetime validation (has T separator)
                    if datetime_str.contains('T') {
                        Ok(value)
                    } else {
                        Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0054,
                            format!("Invalid datetime format: {}", datetime_str),
                        ))
                    }
                } else {
                    Ok(value)
                }
            }
            "time" => {
                if let Some(time_str) = value.as_str() {
                    // Basic time validation (HH:MM format at minimum)
                    if time_str.len() >= 5 && time_str.chars().nth(2) == Some(':') {
                        Ok(value)
                    } else {
                        Err(FhirPathError::evaluation_error(
                            crate::core::error_code::FP0054,
                            format!("Invalid time format: {}", time_str),
                        ))
                    }
                } else {
                    Ok(value)
                }
            }
            _ => Ok(value),
        }
    }

    /// Get primitive element for a property (for extension support)
    fn get_primitive_element(
        &self,
        parent_object: &serde_json::Value,
        property_name: &str,
    ) -> Option<std::sync::Arc<serde_json::Value>> {
        // Check for _propertyName pattern for primitive extensions
        let primitive_element_name = format!("_{}", property_name);
        parent_object
            .get(&primitive_element_name)
            .map(|pe| std::sync::Arc::new(pe.clone()))
    }

    /// Navigate contained resources with proper type information
    async fn navigate_contained_resources(
        &self,
        json: &serde_json::Value,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        if let Some(contained) = json.get("contained").and_then(|c| c.as_array()) {
            for contained_resource in contained {
                if let Some(resource_obj) = contained_resource.as_object() {
                    if let Some(resource_type) =
                        resource_obj.get("resourceType").and_then(|rt| rt.as_str())
                    {
                        // Get precise type information from ModelProvider
                        let resource_type_info = self
                            .model_provider
                            .get_type(resource_type)
                            .await
                            .map_err(|e| {
                                FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0054,
                                    format!(
                                        "ModelProvider error getting type '{}': {}",
                                        resource_type, e
                                    ),
                                )
                            })?
                            .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                                type_name: resource_type.to_string(),
                                singleton: Some(true),
                                namespace: Some("FHIR".to_string()),
                                name: Some(resource_type.to_string()),
                                is_empty: Some(false),
                            });

                        let resource_value = FhirPathValue::Resource(
                            std::sync::Arc::new(contained_resource.clone()),
                            resource_type_info,
                            None,
                        );
                        results.push(resource_value);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Navigate Bundle entry resources with proper type information
    async fn navigate_bundle_resources(
        &self,
        json: &serde_json::Value,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        if let Some(entry) = json.get("entry").and_then(|e| e.as_array()) {
            for entry_item in entry {
                if let Some(resource) = entry_item.get("resource") {
                    if let Some(resource_obj) = resource.as_object() {
                        if let Some(resource_type) =
                            resource_obj.get("resourceType").and_then(|rt| rt.as_str())
                        {
                            // Get precise type information from ModelProvider
                            let resource_type_info = self
                                .model_provider
                                .get_type(resource_type)
                                .await
                                .map_err(|e| {
                                    FhirPathError::evaluation_error(
                                        crate::core::error_code::FP0054,
                                        format!(
                                            "ModelProvider error getting type '{}': {}",
                                            resource_type, e
                                        ),
                                    )
                                })?
                                .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                                    type_name: resource_type.to_string(),
                                    singleton: Some(true),
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(resource_type.to_string()),
                                    is_empty: Some(false),
                                });

                            let resource_value = FhirPathValue::Resource(
                                std::sync::Arc::new(resource.clone()),
                                resource_type_info,
                                None,
                            );
                            results.push(resource_value);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Filter resources by type with ModelProvider validation
    async fn filter_resources_by_type(
        &self,
        input: Vec<FhirPathValue>,
        resource_type: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut filtered = Vec::new();

        for value in input {
            if let FhirPathValue::Resource(json_obj, current_type_info, primitive_element) = &value
            {
                if let Some(rt) = json_obj.get("resourceType").and_then(|rt| rt.as_str()) {
                    if rt == resource_type {
                        // Re-type the resource with precise type information from ModelProvider
                        let precise_type_info = self
                            .model_provider
                            .get_type(resource_type)
                            .await
                            .map_err(|e| {
                                FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0054,
                                    format!(
                                        "ModelProvider error getting type '{}': {}",
                                        resource_type, e
                                    ),
                                )
                            })?
                            .unwrap_or_else(|| current_type_info.clone());

                        let retyped_value = FhirPathValue::Resource(
                            json_obj.clone(),
                            precise_type_info,
                            primitive_element.clone(),
                        );
                        filtered.push(retyped_value);
                    }
                }
            }
        }

        Ok(filtered)
    }

    /// Resolve reference with circular reference detection
    async fn resolve_reference(
        &self,
        reference_value: &FhirPathValue,
        context: &EvaluationContext,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<Vec<FhirPathValue>> {
        if let FhirPathValue::Resource(json_obj, _, _) = reference_value {
            if let Some(reference_url) = json_obj.get("reference").and_then(|r| r.as_str()) {
                // Check for circular references
                if visited.contains(reference_url) {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        format!("Circular reference detected: {}", reference_url),
                    ));
                }

                visited.insert(reference_url.to_string());

                // Basic reference resolution - look for contained resources first
                let mut results = Vec::new();

                // Check if reference is a local fragment reference (starts with #)
                if reference_url.starts_with('#') {
                    let local_id = &reference_url[1..];

                    // Look in the root context for contained resources
                    for root_value in context.get_root_context().iter() {
                        if let FhirPathValue::Resource(root_json, _, _) = root_value {
                            if let Some(contained) =
                                root_json.get("contained").and_then(|c| c.as_array())
                            {
                                for contained_resource in contained {
                                    if let Some(resource_id) =
                                        contained_resource.get("id").and_then(|id| id.as_str())
                                    {
                                        if resource_id == local_id {
                                            // Found the referenced resource
                                            if let Some(resource_type) = contained_resource
                                                .get("resourceType")
                                                .and_then(|rt| rt.as_str())
                                            {
                                                let resource_type_info = self
                                                    .model_provider
                                                    .get_type(resource_type)
                                                    .await
                                                    .unwrap_or(None)
                                                    .unwrap_or_else(|| {
                                                        crate::core::model_provider::TypeInfo {
                                                            type_name: resource_type.to_string(),
                                                            singleton: Some(true),
                                                            namespace: Some("FHIR".to_string()),
                                                            name: Some(resource_type.to_string()),
                                                            is_empty: Some(false),
                                                        }
                                                    });

                                                let resolved_resource = FhirPathValue::Resource(
                                                    std::sync::Arc::new(contained_resource.clone()),
                                                    resource_type_info,
                                                    None,
                                                );
                                                results.push(resolved_resource);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // For external references (like Patient/123), we would need a reference resolver
                // For now, just return empty for external references

                visited.remove(reference_url);
                return Ok(results);
            }
        }

        Ok(vec![])
    }

    /// Evaluate a binary operation using the operator registry
    async fn evaluate_binary_operation(
        &self,
        operator: &crate::ast::BinaryOperator,
        left: Collection,
        right: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        use crate::ast::BinaryOperator;

        match operator {
            // Special case: Union operator (|)
            // Both sides should be evaluated with fresh context from original input
            BinaryOperator::Union => self.evaluate_union_operator(left, right, context).await,

            // Special case: Type operators (is/as)
            // Right side is treated as type identifier, not expression
            BinaryOperator::Is => self.evaluate_is_operator(left, right, context).await,
            BinaryOperator::As => self.evaluate_as_operator(left, right, context).await,

            // Standard registry-based operators
            _ => {
                if let Some(evaluator) = self.operator_registry.get_binary_operator(operator) {
                    let input = Collection::empty(); // Binary operations don't use input collection
                    evaluator
                        .evaluate(input.into_vec(), context, left.into_vec(), right.into_vec())
                        .await
                } else {
                    Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0054,
                        format!("Unsupported binary operator: {:?}", operator),
                    ))
                }
            }
        }
    }

    /// Evaluate a unary operation using the operator registry
    async fn evaluate_unary_operation(
        &self,
        operator: &crate::ast::UnaryOperator,
        operand: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Get the operation evaluator from the registry
        if let Some(evaluator) = self.operator_registry.get_unary_operator(operator) {
            let input = Collection::empty(); // Unary operations don't use input collection
            let empty = Collection::empty();
            evaluator
                .evaluate(
                    input.into_vec(),
                    context,
                    operand.into_vec(),
                    empty.into_vec(),
                )
                .await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Unsupported unary operator: {:?}", operator),
            ))
        }
    }

    /// Evaluate a function call using the function registry
    async fn evaluate_function_call(
        &self,
        function_name: &str,
        arguments: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Get the function evaluator from the registry
        if let Some(evaluator) = self.function_registry.get_function(function_name) {
            // Check if this function needs special argument evaluation context
            let needs_original_context =
                matches!(function_name, "combine" | "union" | "intersect" | "exclude");

            if needs_original_context {
                return self
                    .evaluate_function_with_pre_evaluated_args(function_name, arguments, context)
                    .await;
            }

            // Create async node evaluator closure
            let async_evaluator = AsyncNodeEvaluator::new(self);

            evaluator
                .evaluate(
                    context.input_collection().values().to_vec(),
                    context,
                    arguments.to_vec(),
                    async_evaluator,
                )
                .await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Unknown function: {}", function_name),
            ))
        }
    }

    async fn evaluate_function_with_pre_evaluated_args(
        &self,
        function_name: &str,
        arguments: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // TARGETED FIX: For combine function, we need to evaluate arguments against the original
        // Patient resource, not the narrowed context. Since our context system doesn't preserve
        // the original resource properly, we'll implement a workaround specific to the combine
        // function by manually reconstructing what the original context should be.

        // For now, let's just implement the combine function directly without pre-evaluation
        // This bypasses the context issue entirely
        match function_name {
            "combine" => {
                return self
                    .evaluate_combine_function_directly(arguments, context)
                    .await;
            }
            _ => {
                // For other functions, fall back to normal evaluation
                let evaluator = self
                    .function_registry
                    .get_function(function_name)
                    .ok_or_else(|| {
                        FhirPathError::evaluation_error(
                            crate::core::error_code::FP0054,
                            format!("Unknown function: {}", function_name),
                        )
                    })?;

                let async_evaluator = AsyncNodeEvaluator::new(self);
                return evaluator
                    .evaluate(
                        context.input_collection().values().to_vec(),
                        context,
                        arguments.to_vec(),
                        async_evaluator,
                    )
                    .await;
            }
        }
    }

    /// Direct implementation of combine function to work around context issues
    async fn evaluate_combine_function_directly(
        &self,
        arguments: &[ExpressionNode],
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        if arguments.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::FP0053,
                "combine function requires exactly one argument".to_string(),
            ));
        }

        // Get the input collection (left side of combine)
        let left_collection: Vec<FhirPathValue> = context.input_collection().values().to_vec();

        // SPECIAL HANDLING: We need to evaluate the argument in the context of the original
        // Patient resource. Since our context system doesn't preserve this properly, we'll
        // use a different approach.

        // First, try to find a Resource value in the current context that looks like the Patient
        // by traversing up the context chain or looking for a Resource type
        let right_collection = match self
            .evaluate_expression_in_patient_context(&arguments[0], context)
            .await
        {
            Ok(result) => result.value.values().to_vec(),
            Err(_) => {
                // If that fails, fall back to evaluating in current context
                let result = Box::pin(self.evaluate_node_inner(&arguments[0], context)).await?;
                result.value.values().to_vec()
            }
        };

        // Combine the collections
        let mut combined = left_collection;
        combined.extend(right_collection);

        Ok(EvaluationResult {
            value: crate::core::Collection::from(combined),
        })
    }

    /// Evaluate argument in resource context following FHIR specification
    async fn evaluate_expression_in_patient_context(
        &self,
        expression: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        let root_input = if let Some(resource_var) = context.get_variable("%resource") {
            // Use the %resource variable (FHIR-defined)
            vec![resource_var]
        } else if let Some(context_var) = context.get_variable("%context") {
            vec![context_var]
        } else {
            // Last resort: use the root context
            context.get_root_context().values().to_vec()
        };

        // Create a new evaluation context with the resource input
        let resource_context = EvaluationContext::new(
            crate::core::Collection::from(root_input),
            context.model_provider().clone(),
            context.terminology_provider().clone(),
            context.trace_provider(),
        )
        .await;

        // Evaluate the expression in the resource context
        Box::pin(self.evaluate_node_inner(expression, &resource_context)).await
    }

    /// Evaluate an index operation (e.g., collection[0])
    async fn evaluate_index_operation(
        &self,
        collection: Collection,
        index: Collection,
    ) -> Result<EvaluationResult> {
        // Index should be a single integer
        if let Some(index_value) = index.first() {
            if let FhirPathValue::Integer(idx, _, _) = index_value {
                if *idx < 0 {
                    // Negative indices not supported
                    return Ok(EvaluationResult {
                        value: Collection::empty(),
                    });
                }

                let index_usize = *idx as usize;
                if let Some(item) = collection.get(index_usize) {
                    Ok(EvaluationResult {
                        value: Collection::single(item.clone()),
                    })
                } else {
                    Ok(EvaluationResult {
                        value: Collection::empty(),
                    })
                }
            } else {
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0051,
                    "Index must be an integer".to_string(),
                ))
            }
        } else {
            // Empty index returns empty result
            Ok(EvaluationResult {
                value: Collection::empty(),
            })
        }
    }

    /// Special handler for union operator (|)
    /// Both sides should be evaluated with fresh context and merged
    async fn evaluate_union_operator(
        &self,
        left: Collection,
        right: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Use our UnionOperatorEvaluator directly for proper deduplication
        let union_evaluator = UnionOperatorEvaluator::new();
        union_evaluator
            .evaluate(vec![], context, left.into_vec(), right.into_vec())
            .await
    }

    /// Special handler for 'is' type operator
    /// Right side is type identifier, not evaluated expression
    async fn evaluate_is_operator(
        &self,
        left: Collection,
        right: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Delegate to the type operator from registry
        if let Some(evaluator) = self
            .operator_registry
            .get_binary_operator(&crate::ast::BinaryOperator::Is)
        {
            let input = Collection::empty();
            evaluator
                .evaluate(input.into_vec(), context, left.into_vec(), right.into_vec())
                .await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "Is operator not registered".to_string(),
            ))
        }
    }

    /// Special handler for 'as' type operator
    /// Right side is type identifier, not evaluated expression
    async fn evaluate_as_operator(
        &self,
        left: Collection,
        right: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Delegate to the type operator from registry
        if let Some(evaluator) = self
            .operator_registry
            .get_binary_operator(&crate::ast::BinaryOperator::As)
        {
            let input = Collection::empty();
            evaluator
                .evaluate(input.into_vec(), context, left.into_vec(), right.into_vec())
                .await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                "As operator not registered".to_string(),
            ))
        }
    }
}

/// Async node evaluator wrapper for function evaluation
pub struct AsyncNodeEvaluator<'a> {
    evaluator: &'a Evaluator,
}

impl<'a> AsyncNodeEvaluator<'a> {
    fn new(evaluator: &'a Evaluator) -> Self {
        Self { evaluator }
    }

    /// Evaluate a node asynchronously within a given context
    pub async fn evaluate(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        self.evaluator.evaluate_node_inner(node, context).await
    }
}
