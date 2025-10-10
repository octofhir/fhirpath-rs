//! ofType function implementation
//!
//! The ofType function returns a collection that contains all items in the input
//! collection that are of the given type or a subclass thereof.
//! Per FHIRPath spec: "ofType(type : type specifier) : collection"

use async_trait::async_trait;
use std::sync::Arc;

use crate::ast::ExpressionNode;
use crate::core::{Collection, FhirPathError, FhirPathValue, Result};
use crate::evaluator::function_registry::{
    ArgumentEvaluationStrategy, EmptyPropagation, FunctionCategory, FunctionMetadata,
    FunctionParameter, FunctionSignature, LazyFunctionEvaluator, NullPropagationStrategy,
};
use crate::evaluator::{AsyncNodeEvaluator, EvaluationContext, EvaluationResult};

/// ofType function evaluator for filtering collections by type
pub struct OfTypeFunctionEvaluator {
    metadata: FunctionMetadata,
}

impl Default for OfTypeFunctionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl OfTypeFunctionEvaluator {
    /// Create a new ofType function evaluator
    pub fn new() -> Self {
        Self {
            metadata: create_metadata(),
        }
    }

    /// Create an Arc-wrapped instance for registry registration
    pub fn create() -> Arc<dyn LazyFunctionEvaluator> {
        Arc::new(Self::new())
    }

    /// Check if a type name is valid using ModelProvider
    async fn is_valid_type_name(&self, type_name: &str, context: &EvaluationContext) -> bool {
        let model_provider = context.model_provider();

        // Strip namespace prefixes for validation (FHIR.Patient -> Patient, System.String -> String)
        let base_type_name = if let Some(dot_pos) = type_name.rfind('.') {
            &type_name[dot_pos + 1..]
        } else {
            type_name
        };

        // Check if it's a primitive type
        if let Ok(primitive_types) = model_provider.get_primitive_types().await
            && (primitive_types.contains(&base_type_name.to_string())
                || primitive_types.contains(&type_name.to_string()))
        {
            return true;
        }

        // Check if it's a complex type
        if let Ok(complex_types) = model_provider.get_complex_types().await
            && (complex_types.contains(&base_type_name.to_string())
                || complex_types.contains(&type_name.to_string()))
        {
            return true;
        }

        // Check if it's a FHIR resource type - try to get resource types from model provider
        if let Ok(resource_types) = model_provider.get_resource_types().await
            && (resource_types.contains(&base_type_name.to_string())
                || resource_types.contains(&type_name.to_string()))
        {
            return true;
        }

        // Allow common FHIR resource types even if not in model provider
        let common_fhir_resources = [
            "Patient",
            "Observation",
            "Encounter",
            "Practitioner",
            "Organization",
            "Medication",
            "MedicationRequest",
            "Bundle",
            "Parameters",
            "ValueSet",
            "ConceptMap",
            "StructureDefinition",
            "CodeSystem",
            "OperationOutcome",
        ];

        if common_fhir_resources.contains(&base_type_name) {
            return true;
        }

        // Allow System and FHIR namespace types for common primitives
        matches!(
            type_name,
            "System.String"
                | "System.Integer"
                | "System.Decimal"
                | "System.Boolean"
                | "System.DateTime"
                | "System.Date"
                | "System.Time"
                | "FHIR.string"
                | "FHIR.integer"
                | "FHIR.decimal"
                | "FHIR.boolean"
                | "FHIR.dateTime"
                | "FHIR.date"
                | "FHIR.time"
                | "FHIR.uri"
                | "FHIR.url"
                | "FHIR.canonical"
                | "FHIR.code"
                | "FHIR.id"
        )
    }
}

#[async_trait]
impl LazyFunctionEvaluator for OfTypeFunctionEvaluator {
    async fn evaluate(
        &self,
        input: Vec<FhirPathValue>,
        context: &EvaluationContext,
        args: Vec<ExpressionNode>,
        _evaluator: AsyncNodeEvaluator<'_>,
    ) -> Result<EvaluationResult> {
        // Spec: ofType(type) filters the input collection to only items of the specified type

        // Check argument count
        if args.len() != 1 {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                "ofType function requires exactly one argument",
            ));
        }

        // Extract type identifier from AST node directly instead of evaluating
        let type_name = match &args[0] {
            ExpressionNode::Identifier(identifier_node) => identifier_node.name.clone(),
            ExpressionNode::PropertyAccess(property_access_node) => {
                // Handle property access like FHIR.boolean, System.Boolean
                if let ExpressionNode::Identifier(base_identifier) =
                    property_access_node.object.as_ref()
                {
                    format!("{}.{}", base_identifier.name, property_access_node.property)
                } else {
                    return Err(FhirPathError::evaluation_error(
                        crate::core::error_code::FP0062,
                        format!(
                            "Type argument must be an identifier or property access, got {:?}",
                            args[0]
                        ),
                    ));
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    crate::core::error_code::FP0062,
                    format!(
                        "Type argument must be an identifier or property access, got {:?}",
                        args[0]
                    ),
                ));
            }
        };

        // Validate the type name - reject invalid types like "string1"
        if !self.is_valid_type_name(&type_name, context).await {
            return Err(FhirPathError::evaluation_error(
                crate::core::error_code::FP0062,
                format!("Invalid type name: {type_name}"),
            ));
        }

        // Filter the input collection based on type matching
        // ofType() should only include items that ARE of the target type, not convert them
        let mut filtered_items = Vec::new();

        for item in input {
            // Check if the item is of the target type (including inheritance)
            let is_of_type = self.is_item_of_type(&item, &type_name, context);
            if is_of_type {
                filtered_items.push(item);
            }
        }

        Ok(EvaluationResult {
            value: Collection::from_values(filtered_items),
        })
    }

    fn metadata(&self) -> &FunctionMetadata {
        &self.metadata
    }
}

impl OfTypeFunctionEvaluator {
    /// Check if an item is of the specified type (including inheritance)
    fn is_item_of_type(
        &self,
        item: &FhirPathValue,
        target_type: &str,
        context: &EvaluationContext,
    ) -> bool {
        // Get the actual type info from the item
        let type_info = item.type_info();
        let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
        // Strip namespace from target (e.g., FHIR.Patient -> Patient)
        let base_target = if let Some(dot_pos) = target_type.rfind('.') {
            &target_type[dot_pos + 1..]
        } else {
            target_type
        };

        // Direct type match (full or base target)
        if actual_type == target_type || actual_type == base_target {
            return true;
        }

        // Check for specific type matches with strict namespace handling
        match target_type {
            // FHIR string types
            "string" | "FHIR.string" => match item {
                FhirPathValue::String(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "string" || actual_type == "FHIR.string"
                }
                _ => false,
            },
            // System.String type (should be more generic)
            "String" | "System.String" => {
                match item {
                    FhirPathValue::String(_, type_info, _) => {
                        let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                        // System.String matches any string-like value
                        actual_type == "String"
                            || actual_type == "System.String"
                            || actual_type == "string"
                            || actual_type == "FHIR.string"
                            || actual_type == "code"
                            || actual_type == "id"
                            || actual_type == "uri"
                    }
                    _ => false,
                }
            }
            // FHIR specific string subtypes
            "code" | "FHIR.code" => match item {
                FhirPathValue::String(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "code" || actual_type == "FHIR.code"
                }
                _ => false,
            },
            "id" | "FHIR.id" => match item {
                FhirPathValue::String(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "id" || actual_type == "FHIR.id"
                }
                _ => false,
            },
            "uri" | "FHIR.uri" => match item {
                FhirPathValue::String(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "uri" || actual_type == "FHIR.uri"
                }
                _ => false,
            },
            // FHIR integer type
            "integer" | "FHIR.integer" => match item {
                FhirPathValue::Integer(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "integer" || actual_type == "FHIR.integer"
                }
                _ => false,
            },
            // System.Integer type (namespace-aware)
            "Integer" | "System.Integer" => {
                match item {
                    FhirPathValue::Integer(_, type_info, _) => {
                        // Only match non-FHIR namespace (System)
                        type_info.namespace.as_deref() != Some("FHIR")
                    }
                    _ => false,
                }
            }
            // FHIR decimal type
            "decimal" | "FHIR.decimal" => match item {
                FhirPathValue::Decimal(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "decimal" || actual_type == "FHIR.decimal"
                }
                _ => false,
            },
            // System.Decimal type (namespace-aware)
            "Decimal" | "System.Decimal" => {
                match item {
                    FhirPathValue::Decimal(_, type_info, _) => {
                        // Only match non-FHIR namespace (System)
                        type_info.namespace.as_deref() != Some("FHIR")
                    }
                    _ => false,
                }
            }
            // FHIR boolean type
            "boolean" | "FHIR.boolean" => match item {
                FhirPathValue::Boolean(_, type_info, _) => {
                    let actual_type = type_info.name.as_deref().unwrap_or(&type_info.type_name);
                    actual_type == "boolean" || actual_type == "FHIR.boolean"
                }
                _ => false,
            },
            // System.Boolean type (namespace-aware)
            "Boolean" | "System.Boolean" => {
                match item {
                    FhirPathValue::Boolean(_, type_info, _) => {
                        // Only match non-FHIR namespace (System)
                        type_info.namespace.as_deref() != Some("FHIR")
                    }
                    _ => false,
                }
            }
            _ => {
                // For complex types, use ModelProvider for inheritance checking
                let model_provider = context.model_provider();
                model_provider.is_type_derived_from(actual_type, base_target)
            }
        }
    }
}

/// Create metadata for the ofType function
fn create_metadata() -> FunctionMetadata {
    FunctionMetadata {
        name: "ofType".to_string(),
        description: "Filter collection to items of specified type or subclass thereof".to_string(),
        signature: FunctionSignature {
            input_type: "Collection".to_string(),
            parameters: vec![FunctionParameter {
                name: "type".to_string(),
                parameter_type: vec!["String".to_string()],
                optional: false,
                is_expression: false,
                description: "Type identifier or string literal to filter by".to_string(),
                default_value: None,
            }],
            return_type: "Collection".to_string(),
            polymorphic: true,
            min_params: 1,
            max_params: Some(1),
        },
        argument_evaluation: ArgumentEvaluationStrategy::Current,
        null_propagation: NullPropagationStrategy::Focus,
        empty_propagation: EmptyPropagation::Propagate,
        deterministic: true,
        category: FunctionCategory::FilteringProjection,
        requires_terminology: false,
        requires_model: true, // Requires ModelProvider for strict schema-based type checking
    }
}
