//! Children function implementation - sync version

use crate::signature::{FunctionSignature, ValueType};
use crate::traits::{EvaluationContext, SyncOperation, validation};
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::{Collection, FhirPathValue};

/// Children function - returns a collection with all immediate child nodes
#[derive(Debug, Clone)]
pub struct ChildrenFunction;

impl ChildrenFunction {
    pub fn new() -> Self {
        Self
    }

    fn get_children_from_value(&self, value: &FhirPathValue) -> Vec<FhirPathValue> {
        match value {
            FhirPathValue::JsonValue(json_val) => {
                if json_val.is_object() {
                    // Get all property values as children
                    let mut children = Vec::new();
                    // Use sonic-rs object iteration - no conversions!
                    if let Some(iter) = json_val.object_iter() {
                        for (_property_name, property_value) in iter {
                            // Each property value becomes a child node
                            // Arrays ARE unrolled - each element becomes a separate child
                            if property_value.is_array() {
                                if let Some(array_iter) = property_value.array_iter() {
                                    for element in array_iter {
                                        children.push(FhirPathValue::JsonValue(element));
                                    }
                                }
                            } else {
                                children.push(FhirPathValue::JsonValue(property_value));
                            }
                        }
                    }
                    children
                } else if json_val.is_array() {
                    // If the input itself is an array, each element is a child
                    if let Some(iter) = json_val.array_iter() {
                        iter.map(FhirPathValue::JsonValue).collect()
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
                self.get_children_from_value(&FhirPathValue::JsonValue(json_value))
            }
            FhirPathValue::Collection(items) => {
                // Get children of all items in collection
                let mut all_children = Vec::new();
                for item in items.iter() {
                    all_children.extend(self.get_children_from_value(item));
                }
                all_children
            }
            // Primitive values don't have children
            _ => Vec::new(),
        }
    }
}

impl SyncOperation for ChildrenFunction {
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
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        validation::validate_no_args(args, "children")?;

        // For children(), we operate on the focus (current context)
        let focus = &context.input;
        let children = self.get_children_from_value(focus);

        Ok(FhirPathValue::Collection(Collection::from(children)))
    }
}

impl Default for ChildrenFunction {
    fn default() -> Self {
        Self::new()
    }
}
