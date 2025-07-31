//! children() function implementation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::FunctionSignature;

/// children() function - returns direct children of nodes in the collection
pub struct ChildrenFunction;

impl FhirPathFunction for ChildrenFunction {
    fn name(&self) -> &str {
        "children"
    }
    fn human_friendly_name(&self) -> &str {
        "Children"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "children",
                vec![],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }

    fn documentation(&self) -> &str {
        "Returns a collection with all immediate child nodes of all items in the input collection. Child nodes are the direct properties of a resource or object, one level down from the current node."
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let items = context.input.clone().to_collection();
        let mut result = Vec::new();

        fn collect_children(value: &FhirPathValue, result: &mut Vec<FhirPathValue>) {
            match value {
                FhirPathValue::Resource(resource) => {
                    // Collect all direct child values from the resource
                    for (_key, field_value) in resource.properties() {
                        // Convert JSON Value to FhirPathValue
                        let fhir_path_value = value_to_fhir_path_value(field_value);

                        // Add only the direct child value (no recursion)
                        match &fhir_path_value {
                            FhirPathValue::Empty => {} // Skip empty values
                            _ => {
                                result.push(fhir_path_value);
                            }
                        }
                    }
                }
                FhirPathValue::Collection(items) => {
                    // For collections, the children are the items themselves
                    for item in items.iter() {
                        result.push(item.clone());
                    }
                }
                _ => {} // Primitives have no children
            }
        }

        fn value_to_fhir_path_value(value: &serde_json::Value) -> FhirPathValue {
            use crate::model::FhirResource;

            match value {
                serde_json::Value::Array(arr) => {
                    let mut collection = Vec::new();
                    for item in arr {
                        collection.push(value_to_fhir_path_value(item));
                    }
                    FhirPathValue::collection(collection)
                }
                serde_json::Value::Object(_) => {
                    let resource = FhirResource::from_json(value.clone());
                    FhirPathValue::Resource(resource)
                }
                serde_json::Value::String(s) => FhirPathValue::String(s.clone()),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        FhirPathValue::Integer(i)
                    } else if let Some(f) = n.as_f64() {
                        FhirPathValue::Decimal(
                            rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default(),
                        )
                    } else {
                        FhirPathValue::Empty
                    }
                }
                serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
                serde_json::Value::Null => FhirPathValue::Empty,
            }
        }

        for item in items.iter() {
            collect_children(item, &mut result);
        }

        Ok(FhirPathValue::collection(result))
    }
}
