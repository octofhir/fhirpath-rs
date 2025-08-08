//! ofType() function - filters collection to items of specified type

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{
    AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult,
};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// ofType() function - filters collection to items of specified type
pub struct OfTypeFunction;

#[async_trait]
impl AsyncFhirPathFunction for OfTypeFunction {
    fn name(&self) -> &str {
        "ofType"
    }
    fn human_friendly_name(&self) -> &str {
        "OfType"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "ofType",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )
        });
        &SIG
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let type_name = match &args[0] {
            FhirPathValue::String(t) => t,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Get the collection to filter
        let items = match &context.input {
            FhirPathValue::Collection(items) => items.iter().collect::<Vec<_>>(),
            FhirPathValue::Empty => return Ok(FhirPathValue::collection(vec![])),
            single => vec![single], // Single item treated as collection
        };

        let mut results = Vec::new();

        // Filter items by type
        for item in items {
            if self.matches_type(item, type_name) {
                results.push((*item).clone());
            }
        }

        Ok(FhirPathValue::collection(results))
    }
}

impl OfTypeFunction {
    /// Check if a value matches the specified type name
    fn matches_type(&self, value: &FhirPathValue, type_name: &str) -> bool {
        match value {
            FhirPathValue::Boolean(_) => {
                matches!(
                    type_name,
                    "Boolean" | "System.Boolean" | "boolean" | "FHIR.boolean"
                )
            }
            FhirPathValue::Integer(_) => {
                matches!(
                    type_name,
                    "Integer" | "System.Integer" | "integer" | "FHIR.integer"
                )
            }
            FhirPathValue::Decimal(_) => {
                matches!(
                    type_name,
                    "Decimal" | "System.Decimal" | "decimal" | "FHIR.decimal"
                )
            }
            FhirPathValue::String(_) => {
                matches!(
                    type_name,
                    "String"
                        | "System.String"
                        | "string"
                        | "FHIR.string"
                        | "uri"
                        | "FHIR.uri"
                        | "uuid"
                        | "FHIR.uuid"
                        | "code"
                        | "FHIR.code"
                        | "id"
                        | "FHIR.id"
                )
            }
            FhirPathValue::Date(_) => {
                matches!(type_name, "Date" | "System.Date" | "date" | "FHIR.date")
            }
            FhirPathValue::DateTime(_) => {
                matches!(
                    type_name,
                    "DateTime" | "System.DateTime" | "dateTime" | "FHIR.dateTime"
                )
            }
            FhirPathValue::Time(_) => {
                matches!(type_name, "Time" | "System.Time" | "time" | "FHIR.time")
            }
            FhirPathValue::Quantity { .. } => {
                matches!(type_name, "Quantity" | "System.Quantity" | "FHIR.Quantity")
            }
            FhirPathValue::Resource(resource) => {
                // Check FHIR resource type - support both with and without FHIR prefix
                if let Some(resource_type) = resource.resource_type() {
                    resource_type == type_name
                        || type_name == format!("FHIR.{resource_type}")
                        || type_name == format!("FHIR.`{resource_type}`")
                        // Handle case-insensitive matching for common FHIR resources
                        || resource_type.to_lowercase() == type_name.to_lowercase()
                } else {
                    false
                }
            }
            FhirPathValue::Collection(_) => {
                matches!(type_name, "Collection")
            }
            FhirPathValue::TypeInfoObject { .. } => {
                matches!(type_name, "TypeInfo" | "System.TypeInfo")
            }
            FhirPathValue::JsonValue(_) => {
                matches!(type_name, "JsonValue" | "Object" | "Any")
            }
            FhirPathValue::Empty => false,
        }
    }
}
