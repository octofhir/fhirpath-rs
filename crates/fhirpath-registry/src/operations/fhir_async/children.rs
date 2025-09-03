//! Children function implementation - async version
use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, AsyncOperation, validation};
use async_trait::async_trait;
use octofhir_fhirpath_core::{Collection, FhirPathValue, JsonValueExt, Result};

/// Children function - returns a collection with all immediate child nodes
#[derive(Debug, Clone)]
pub struct ChildrenFunction;

impl ChildrenFunction {
    pub fn new() -> Self {
        Self
    }

    async fn get_children_from_value(&self, value: &FhirPathValue, context: &EvaluationContext) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if json_val.is_object() {
                    // Try to get resource type for enhanced ModelProvider navigation
                    let resource_type = json_val.as_inner().get("resourceType")
                        .and_then(|rt| rt.as_str())
                        .unwrap_or("Unknown");
                    
                    let mut children = Vec::new();
                    
                    // Use ModelProvider for enhanced navigation if possible
                    if let Ok(supported_types) = context.model_provider.get_supported_resource_types().await {
                        if supported_types.contains(&resource_type.to_string()) {
                            // Try to get children using ModelProvider's navigation capabilities
                            if let Some(iter) = json_val.object_iter() {
                                for (property_name, property_value) in iter {
                                    // Skip structural properties that aren't actual children
                                    if matches!(property_name.as_str(), "id" | "meta" | "implicitRules" | "language" | "resourceType") {
                                        continue;
                                    }
                                    
                                    // Validate navigation path using ModelProvider
                                    if let Ok(validation) = context.model_provider.validate_navigation_safety(resource_type, &property_name).await {
                                        if validation.is_valid {
                                            // Arrays ARE unrolled - each element becomes a separate child
                                            if property_value.is_array() {
                                                if let Some(array_iter) = property_value.array_iter() {
                                                    for element in array_iter {
                                                        children.push(FhirPathValue::JsonValue(element.clone()));
                                                    }
                                                }
                                            } else {
                                                children.push(FhirPathValue::JsonValue(property_value.clone()));
                                            }
                                        }
                                    }
                                }
                            }
                            return children;
                        }
                    }
                    
                    // Fallback to original logic if ModelProvider doesn't support this type
                    if let Some(iter) = json_val.object_iter() {
                        for (_property_name, property_value) in iter {
                            // Each property value becomes a child node
                            // Arrays ARE unrolled - each element becomes a separate child
                            if property_value.is_array() {
                                if let Some(array_iter) = property_value.array_iter() {
                                    for element in array_iter {
                                        children.push(FhirPathValue::JsonValue(element.clone()));
                                    }
                                }
                            } else {
                                children.push(FhirPathValue::JsonValue(property_value.clone()));
                            }
                        }
                    }
                    children
                } else if json_val.is_array() {
                    // If the input itself is an array, each element is a child
                    if let Some(iter) = json_val.array_iter() {
                        iter.map(|v| FhirPathValue::JsonValue(v.clone())).collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    // Primitive values have no children
                    Vec::new()
                }
            }
            FhirPathValue::Resource(resource) => {
                // For FHIR resources, get children from their JSON representation
                // Use the resource's existing JSON value directly - no conversions!
                let json_value = resource.as_json_value().clone();
                Box::pin(self.get_children_from_value(&FhirPathValue::JsonValue(json_value), context)).await
            }
            FhirPathValue::Collection(items) => {
                // Get children of all items in collection
                let mut all_children = Vec::new();
                for item in items.iter() {
                    all_children.extend(Box::pin(self.get_children_from_value(item, context)).await);
                }
                all_children
            }
            // Primitive values don't have children
            _ => Vec::new(),
        }
    }
}

#[async_trait]
impl AsyncOperation for ChildrenFunction {
    fn name(&self) -> &'static str {
        "children"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "children",
                parameters: vec![],
                return_type: ValueType::Collection,
                variadic: false,
                category: FunctionCategory::Navigation,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "children")?;

        // For children(), we operate on the focus (current context)
        let focus = &context.input;
        let children = self.get_children_from_value(focus, context).await;

        Ok(FhirPathValue::Collection(children))
    }
}

impl Default for ChildrenFunction {
    fn default() -> Self {
        Self::new()
    }
}
