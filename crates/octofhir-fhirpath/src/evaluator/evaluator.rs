//! FHIRPath expression evaluator implementation
//!
//! This module provides the main Evaluator struct that replaces the stub implementation
//! with a registry-based architecture for operators and functions.

use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathValue, ModelProvider, Result, FhirPathError};
use crate::evaluator::{EvaluationResult, EvaluationResultWithMetadata};
use super::context::EvaluationContext;
use octofhir_fhir_model::TerminologyProvider;

use super::operator_registry::OperatorRegistry;
use super::function_registry::FunctionRegistry;

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
                let left_result = Box::pin(self.evaluate_node_inner(&binary_op.left, context)).await?;
                let right_result = Box::pin(self.evaluate_node_inner(&binary_op.right, context)).await?;

                // Dispatch to operator registry
                self.evaluate_binary_operation(
                    &binary_op.operator,
                    left_result.value,
                    right_result.value,
                    context,
                ).await
            }
            ExpressionNode::UnaryOperation(unary_op) => {
                // Evaluate operand first
                let operand_result = Box::pin(self.evaluate_node_inner(&unary_op.operand, context)).await?;

                // Dispatch to operator registry for unary operations
                self.evaluate_unary_operation(&unary_op.operator, operand_result.value, context).await
            }
            ExpressionNode::FunctionCall(function_call) => {
                // Dispatch to function registry
                self.evaluate_function_call(&function_call.name, &function_call.arguments, context).await
            }
            ExpressionNode::IndexAccess(index_access) => {
                // Evaluate collection first, then apply index
                let collection_result = Box::pin(self.evaluate_node_inner(&index_access.object, context)).await?;
                let index_result = Box::pin(self.evaluate_node_inner(&index_access.index, context)).await?;

                self.evaluate_index_operation(collection_result.value, index_result.value).await
            }
            ExpressionNode::PropertyAccess(property_access) => {
                // Evaluate object first, then navigate to member
                let object_result = Box::pin(self.evaluate_node_inner(&property_access.object, context)).await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                ).await;

                self.evaluate_path(&property_access.property, &new_context).await
            }
            ExpressionNode::MethodCall(method_call) => {
                // Evaluate object first, then call method
                let object_result = Box::pin(self.evaluate_node_inner(&method_call.object, context)).await?;
                let new_context = EvaluationContext::new(
                    object_result.value,
                    self.model_provider.clone(),
                    self.terminology_provider.clone(),
                ).await;

                self.evaluate_function_call(&method_call.method, &method_call.arguments, &new_context).await
            }
            ExpressionNode::Collection(collection_node) => {
                // Evaluate collection literal
                self.evaluate_collection(&collection_node.elements, context).await
            }
            ExpressionNode::Variable(variable_node) => {
                // Evaluate variable access ($this, $index, $total, user variables)
                self.evaluate_variable(&variable_node.name, context).await
            }
            ExpressionNode::Parenthesized(expr) => {
                // Just evaluate the inner expression
                Box::pin(self.evaluate_node_inner(expr, context)).await
            }
            _ => {
                // TODO: Implement other expression types in future phases
                Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0054,
                    format!("Expression type not yet implemented: {:?}", node),
                ))
            }
        }
    }

    /// Evaluate an AST node with metadata collection
    pub async fn evaluate_node_with_metadata(
        &self,
        node: &ExpressionNode,
        context: &EvaluationContext,
    ) -> Result<EvaluationResultWithMetadata> {
        // For now, use the basic evaluation and add empty metadata
        // TODO: In Phase 6, this will collect detailed execution metadata
        let result = self.evaluate_node_inner(node, context).await?;

        Ok(EvaluationResultWithMetadata {
            value: result.value,
            metadata: crate::evaluator::stub::EvaluationMetadata::default(),
        })
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
    async fn evaluate_path(&self, identifier: &str, context: &EvaluationContext) -> Result<EvaluationResult> {
        // Check if identifier starts with capital letter (potential resource type)
        let is_resource_type_check = identifier.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);

        let mut result_values = Vec::new();

        // Navigate each item in the input collection
        for item in context.input_collection().iter() {
            match item {
                FhirPathValue::Resource(json, type_info, _) => {
                    // Handle resource type validation when identifier starts with capital letter
                    if is_resource_type_check {
                        // Extract resourceType from JSON
                        let actual_resource_type = json.get("resourceType")
                            .and_then(|rt| rt.as_str())
                            .ok_or_else(|| FhirPathError::evaluation_error(
                                crate::core::error_code::FP0054,
                                "Resource does not have a resourceType field".to_string()
                            ))?;

                        // Validate that the resource type matches the identifier
                        if actual_resource_type == identifier {
                            // Resource type matches - return the resource with proper type info
                            let resource_type_info = self.model_provider
                                .get_type(identifier)
                                .await
                                .map_err(|e| FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0054,
                                    format!("ModelProvider error getting type '{}': {}", identifier, e)
                                ))?
                                .unwrap_or_else(|| crate::core::model_provider::TypeInfo {
                                    type_name: identifier.to_string(),
                                    singleton: true,
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(identifier.to_string()),
                                    is_empty: Some(false),
                                    is_union_type: Some(false),
                                    union_choices: None,
                                });

                            let resource_value = FhirPathValue::wrap_value(
                                crate::core::value::utils::json_to_fhirpath_value((**json).clone()),
                                resource_type_info,
                                None
                            );
                            result_values.push(resource_value);
                        } else {
                            // Resource type mismatch - return empty per FHIRPath spec
                            // Semantic analysis will catch this during development, but runtime should be lenient
                            continue;
                        }
                        continue;
                    }

                    // Use ModelProvider for enhanced property navigation
                    let navigation_result = self.model_provider
                        .navigate_with_data(&type_info.type_name, identifier, json)
                        .await
                        .map_err(|e| FhirPathError::evaluation_error(
                            crate::core::error_code::FP0054,
                            format!("Model provider navigation error: {}", e)
                        ))?;

                    if navigation_result.success {
                        // Property found by ModelProvider - extract the value directly from JSON
                        if let Some(property_value) = json.get(identifier) {
                            let property_type_info = navigation_result.result_type.unwrap_or_else(|| {
                                crate::core::model_provider::TypeInfo {
                                    type_name: "Unknown".to_string(),
                                    singleton: true,
                                    namespace: Some("FHIR".to_string()),
                                    name: Some(identifier.to_string()),
                                    is_empty: Some(false),
                                    is_union_type: Some(false),
                                    union_choices: None,
                                }
                            });

                            let flattened_values = self.navigate_property_with_flattening(
                                property_value,
                                &property_type_info,
                            ).await?;
                            result_values.extend(flattened_values);
                        }
                    } else {
                        // Check for choice types (valueX properties)
                        if let Ok(true) = self.model_provider
                            .is_polymorphic_property(&type_info.type_name, identifier)
                            .await
                        {
                            // Handle choice type navigation
                            let choice_results = self.navigate_choice_property(
                                json,
                                identifier,
                                &type_info.type_name
                            ).await?;
                            result_values.extend(choice_results);
                        }
                        // Check for extension access
                        else if identifier.starts_with("extension") {
                            let extension_results = self.navigate_extension_property(
                                json,
                                identifier
                            ).await?;
                            result_values.extend(extension_results);
                        }
                        // Fallback to direct JSON navigation for unknown properties
                        else if let Some(property_value) = json.get(identifier) {
                            let fallback_type_info = crate::core::model_provider::TypeInfo {
                                type_name: "Unknown".to_string(),
                                singleton: true,
                                namespace: Some("FHIR".to_string()),
                                name: Some(identifier.to_string()),
                                is_empty: Some(false),
                                is_union_type: Some(false),
                                union_choices: None,
                            };
                            let flattened_values = self.navigate_property_with_flattening(
                                property_value,
                                &fallback_type_info,
                            ).await?;
                            result_values.extend(flattened_values);
                        }
                        // Property not found - for now just return empty collection (standard FHIRPath)
                        // TODO: Add strict mode validation that can throw errors for unknown properties
                        else {
                            // Standard FHIRPath behavior: unknown properties return empty
                        }
                    }
                }
                FhirPathValue::Collection(collection) => {
                    // Navigate into each item of the collection
                    for sub_item in collection.iter() {
                        if let FhirPathValue::Resource(json, type_info, _) = sub_item {
                            // Use same enhanced navigation for collection items
                            let navigation_result = self.model_provider
                                .navigate_with_data(&type_info.type_name, identifier, json)
                                .await
                                .map_err(|e| FhirPathError::evaluation_error(
                                    crate::core::error_code::FP0054,
                                    format!("Model provider navigation error: {}", e)
                                ))?;

                            if navigation_result.success {
                                // Property found by ModelProvider - extract the value directly from JSON
                                if let Some(property_value) = json.get(identifier) {
                                    let property_type_info = navigation_result.result_type.unwrap_or_else(|| {
                                        crate::core::model_provider::TypeInfo {
                                            type_name: "Unknown".to_string(),
                                            singleton: true,
                                            namespace: Some("FHIR".to_string()),
                                            name: Some(identifier.to_string()),
                                            is_empty: Some(false),
                                            is_union_type: Some(false),
                                            union_choices: None,
                                        }
                                    });

                                    let flattened_values = self.navigate_property_with_flattening(
                                        property_value,
                                        &property_type_info,
                                    ).await?;
                                    result_values.extend(flattened_values);
                                }
                            } else {
                                    // Apply same fallback logic as above
                                    if let Ok(true) = self.model_provider
                                        .is_polymorphic_property(&type_info.type_name, identifier)
                                        .await
                                    {
                                        let choice_results = self.navigate_choice_property(
                                            json,
                                            identifier,
                                            &type_info.type_name
                                        ).await?;
                                        result_values.extend(choice_results);
                                    } else if identifier.starts_with("extension") {
                                        let extension_results = self.navigate_extension_property(
                                            json,
                                            identifier
                                        ).await?;
                                        result_values.extend(extension_results);
                                    } else if let Some(property_value) = json.get(identifier) {
                                        let fallback_type_info = crate::core::model_provider::TypeInfo {
                                            type_name: "Unknown".to_string(),
                                            singleton: true,
                                            namespace: Some("FHIR".to_string()),
                                            name: Some(identifier.to_string()),
                                            is_empty: Some(false),
                                            is_union_type: Some(false),
                                            union_choices: None,
                                        };
                                        let flattened_values = self.navigate_property_with_flattening(
                                            property_value,
                                            &fallback_type_info,
                                        ).await?;
                                        result_values.extend(flattened_values);
                                    } else {
                                        // Check if this property is valid for the current type
                                        match self.model_provider.get_element_type(type_info, identifier).await {
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
                                                    )
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

    /// Navigate choice type properties (valueX patterns)
    async fn navigate_choice_property(
        &self,
        json: &serde_json::Value,
        base_property: &str,
        parent_type: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        // Get collection element types for this choice property
        if let Ok(choice_types) = self.model_provider
            .get_collection_element_types(parent_type, base_property)
            .await
        {
            // Look for valueX variants (e.g., valueString, valueInteger)
            for choice_type in choice_types {
                let choice_property_name = format!("{}{}", base_property, choice_type.type_name);
                if let Some(property_value) = json.get(&choice_property_name) {
                    // Convert with the specific choice type information
                    let fhir_value = self.convert_json_to_fhirpath_with_type(
                        property_value.clone(),
                        &choice_type,
                    ).await?;
                    results.push(fhir_value);
                }
            }
        }

        // Fallback: look for common valueX patterns
        if results.is_empty() && base_property == "value" {
            let common_types = vec![
                "String", "Integer", "Decimal", "Boolean", "Date", "DateTime",
                "Time", "Code", "CodeableConcept", "Coding", "Quantity", "Reference"
            ];

            for type_name in common_types {
                let property_name = format!("value{}", type_name);
                if let Some(property_value) = json.get(&property_name) {
                    // Create basic type info for the choice type
                    let type_info = crate::core::model_provider::TypeInfo {
                        type_name: type_name.to_string(),
                        singleton: true,
                        namespace: Some("System".to_string()),
                        name: Some(type_name.to_string()),
                        is_empty: Some(false),
                        is_union_type: Some(false),
                        union_choices: None,
                    };

                    let fhir_value = self.convert_json_to_fhirpath_with_type(
                        property_value.clone(),
                        &type_info,
                    ).await?;
                    results.push(fhir_value);
                }
            }
        }

        Ok(results)
    }

    /// Navigate extension properties
    async fn navigate_extension_property(
        &self,
        json: &serde_json::Value,
        property_name: &str,
    ) -> Result<Vec<FhirPathValue>> {
        let mut results = Vec::new();

        // Handle different extension access patterns
        if property_name == "extension" {
            // Access all extensions
            if let Some(extensions) = json.get("extension").and_then(|e| e.as_array()) {
                for ext in extensions {
                    let type_info = crate::core::model_provider::TypeInfo {
                        type_name: "Extension".to_string(),
                        singleton: true,
                        namespace: Some("FHIR".to_string()),
                        name: Some("Extension".to_string()),
                        is_empty: Some(false),
                        is_union_type: Some(false),
                        union_choices: None,
                    };

                    let fhir_value = FhirPathValue::wrap_value(
                        crate::core::value::utils::json_to_fhirpath_value(ext.clone()),
                        type_info,
                        None
                    );
                    results.push(fhir_value);
                }
            }
        } else if property_name.starts_with("extension(") && property_name.ends_with(')') {
            // Access extension by URL: extension('http://example.com/ext')
            let url = &property_name[10..property_name.len()-1].trim_matches('\'').trim_matches('"');
            if let Some(extensions) = json.get("extension").and_then(|e| e.as_array()) {
                for ext in extensions {
                    if let Some(ext_url) = ext.get("url").and_then(|u| u.as_str()) {
                        if ext_url == *url {
                            let type_info = crate::core::model_provider::TypeInfo {
                                type_name: "Extension".to_string(),
                                singleton: true,
                                namespace: Some("FHIR".to_string()),
                                name: Some("Extension".to_string()),
                                is_empty: Some(false),
                                is_union_type: Some(false),
                                union_choices: None,
                            };

                            let fhir_value = FhirPathValue::wrap_value(
                                crate::core::value::utils::json_to_fhirpath_value(ext.clone()),
                                type_info,
                                None
                            );
                            results.push(fhir_value);
                        }
                    }
                }
            }
        }

        Ok(results)
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
                // Return $total variable
                if let Some(total_value) = context.get_system_total() {
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
                        format!("Unknown variable: ${}", variable_name),
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
                // Flatten arrays - each element becomes a separate result
                let element_type_info = crate::core::model_provider::TypeInfo {
                    type_name: type_info.type_name.clone(),
                    singleton: true, // Each element is singular
                    namespace: type_info.namespace.clone(),
                    name: type_info.name.clone(),
                    is_empty: Some(false),
                    is_union_type: type_info.is_union_type,
                    union_choices: type_info.union_choices.clone(),
                };

                for element in arr {
                    // For resources, re-box them properly
                    if element.is_object() && element.get("resourceType").is_some() {
                        let base_value = crate::core::value::utils::json_to_fhirpath_value(element.clone());
                        results.push(FhirPathValue::wrap_value(base_value, element_type_info.clone(), None));
                    } else {
                        // For primitives and other values
                        let base_value = crate::core::value::utils::json_to_fhirpath_value(element.clone());
                        results.push(FhirPathValue::wrap_value(base_value, element_type_info.clone(), None));
                    }
                }
            }
            _ => {
                // Single value - convert normally
                if property_value.is_object() && property_value.get("resourceType").is_some() {
                    let base_value = crate::core::value::utils::json_to_fhirpath_value(property_value.clone());
                    results.push(FhirPathValue::wrap_value(base_value, type_info.clone(), None));
                } else {
                    let base_value = crate::core::value::utils::json_to_fhirpath_value(property_value.clone());
                    results.push(FhirPathValue::wrap_value(base_value, type_info.clone(), None));
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
        Ok(FhirPathValue::wrap_value(base_value, type_info.clone(), None))
    }

    /// Convert JSON value to FhirPathValue using ModelProvider for type information
    async fn convert_json_with_type_info(
        &self,
        json: serde_json::Value,
        property_name: &str,
        parent_type_info: &crate::core::model_provider::TypeInfo,
    ) -> Result<FhirPathValue> {
        // Use ModelProvider to get property type information
        let property_type_info = self.model_provider
            .get_element_type(
                parent_type_info,
                property_name,
            ).await
            .unwrap_or(None)
            .unwrap_or_else(|| {
                // Default type info if not found
                crate::core::model_provider::TypeInfo {
                    type_name: "Unknown".to_string(),
                    singleton: true,
                    namespace: None,
                    name: Some(property_name.to_string()),
                    is_empty: Some(false),
                    is_union_type: Some(false),
                    union_choices: None,
                }
            });

        // Convert JSON to FhirPathValue using type information
        let value = crate::core::value::utils::json_to_fhirpath_value(json);

        // Wrap with proper type information
        Ok(FhirPathValue::wrap_value(value, property_type_info, None))
    }

    /// Evaluate a binary operation using the operator registry
    async fn evaluate_binary_operation(
        &self,
        operator: &crate::ast::BinaryOperator,
        left: Collection,
        right: Collection,
        context: &EvaluationContext,
    ) -> Result<EvaluationResult> {
        // Get the operation evaluator from the registry
        if let Some(evaluator) = self.operator_registry.get_binary_operator(operator) {
            let input = Collection::empty(); // Binary operations don't use input collection
            evaluator.evaluate(input.into_vec(), context, left.into_vec(), right.into_vec()).await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Unsupported binary operator: {:?}", operator),
            ))
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
            evaluator.evaluate(input.into_vec(), context, operand.into_vec(), empty.into_vec()).await
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
            // Create async node evaluator closure
            let async_evaluator = AsyncNodeEvaluator::new(self);

            evaluator.evaluate(
                context.input_collection().values().to_vec(),
                context,
                arguments.to_vec(),
                async_evaluator,
            ).await
        } else {
            Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0054,
                format!("Unknown function: {}", function_name),
            ))
        }
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