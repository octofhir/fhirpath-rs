use async_trait::async_trait;
use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint, FhirPathType}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;

pub struct OfTypeFunction {
    metadata: OperationMetadata,
}

impl OfTypeFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("ofType", OperationType::Function)
            .description("Returns a collection that contains all items in the input collection that are of the given type or a subclass thereof")
            .returns(TypeConstraint::Specific(FhirPathType::Collection))
            .example("children.ofType(Patient)")
            .example("descendants().ofType(Observation)")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for OfTypeFunction {
    fn identifier(&self) -> &str {
        "ofType"
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
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArguments {
                message: "ofType() requires exactly one argument (type specifier)".to_string()
            });
        }

        // Extract type from argument
        let type_arg = &args[0];
        let target_type = self.extract_type_info(type_arg)?;

        let collection = context.input.clone().to_collection();
        let mut result = Vec::new();

        for item in collection.iter() {
            if self.is_of_type(item, &target_type)? {
                result.push(item.clone());
            }
        }

        Ok(FhirPathValue::Collection(result.into()))
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Can be synchronous for simple type checking
        Some(futures::executor::block_on(self.evaluate(args, context)))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl OfTypeFunction {
    fn extract_type_info(&self, type_arg: &FhirPathValue) -> Result<String> {
        match type_arg {
            FhirPathValue::String(type_name) => Ok(type_name.to_string()),
            FhirPathValue::Collection(type_name) if type_name.len() == 1 => {
                match type_name.iter().next().unwrap() {
                    FhirPathValue::String(s) => Ok(s.to_string()),
                    _ => Err(FhirPathError::InvalidArguments { message:
                    "trace() name argument must be a string".to_string()
                    }),
                }
            },
            _ => Err(FhirPathError::InvalidArguments {
                message: "ofType() type argument must be a string".to_string()
            }),
        }
    }

    fn is_of_type(&self, item: &FhirPathValue, target_type: &str) -> Result<bool> {
        match item {
            FhirPathValue::String(_) => Ok(target_type == "string" || target_type == "String"),
            FhirPathValue::Integer(_) => Ok(target_type == "integer" || target_type == "Integer"),
            FhirPathValue::Decimal(_) => Ok(target_type == "decimal" || target_type == "Decimal"),
            FhirPathValue::Boolean(_) => Ok(target_type == "boolean" || target_type == "Boolean"),
            FhirPathValue::Date(_) => Ok(target_type == "date" || target_type == "Date"),
            FhirPathValue::DateTime(_) => Ok(target_type == "dateTime" || target_type == "DateTime"),
            FhirPathValue::Time(_) => Ok(target_type == "time" || target_type == "Time"),
            FhirPathValue::Quantity(_) => Ok(target_type == "Quantity" || target_type == "quantity"),
            FhirPathValue::JsonValue(json_val) => {
                // For FHIR resources, check the resourceType property
                if let Some(obj) = json_val.as_object() {
                    if let Some(serde_json::Value::String(resource_type)) = obj.get("resourceType") {
                        Ok(resource_type == target_type || self.is_subtype_of(resource_type, target_type))
                    } else {
                        // For other objects, check if they match the generic "object" type
                        Ok(target_type == "object" || target_type == "Object")
                    }
                } else {
                    Ok(false)
                }
            }
            FhirPathValue::Collection(_) => Ok(target_type == "collection" || target_type == "Collection"),
            FhirPathValue::Resource(_) => Ok(target_type == "Resource" || target_type == "resource"),
            FhirPathValue::TypeInfoObject { .. } => Ok(target_type == "TypeInfo" || target_type == "typeinfo"),
            FhirPathValue::Empty => Ok(false), // Empty values don't have a type
        }
    }

    // Basic FHIR type inheritance checking
    fn is_subtype_of(&self, resource_type: &str, target_type: &str) -> bool {
        // Basic FHIR inheritance rules
        match target_type {
            "Resource" => {
                // All FHIR resources inherit from Resource
                matches!(resource_type,
                    "Patient" | "Observation" | "Practitioner" | "Organization" |
                    "Encounter" | "Condition" | "Procedure" | "DiagnosticReport" |
                    "Medication" | "MedicationStatement" | "AllergyIntolerance" |
                    "Bundle" | "CapabilityStatement" | "ValueSet" | "CodeSystem" |
                    "StructureDefinition" | "OperationDefinition" | "SearchParameter"
                    // Add more resource types as needed
                )
            }
            "DomainResource" => {
                // Most clinical resources inherit from DomainResource
                matches!(resource_type,
                    "Patient" | "Observation" | "Practitioner" | "Organization" |
                    "Encounter" | "Condition" | "Procedure" | "DiagnosticReport" |
                    "Medication" | "MedicationStatement" | "AllergyIntolerance"
                    // Add more domain resources as needed
                )
            }
            "MetadataResource" => {
                // Metadata resources
                matches!(resource_type,
                    "CapabilityStatement" | "ValueSet" | "CodeSystem" |
                    "StructureDefinition" | "OperationDefinition" | "SearchParameter"
                )
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::create_test_context;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_of_type_string() {
        let function = OfTypeFunction::new();

        let collection = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::String("another".into()),
            FhirPathValue::Boolean(true),
        ];

        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![FhirPathValue::String("string".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                for item in items.iter() {
                    assert!(matches!(item, FhirPathValue::String(_)));
                }
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_of_type_integer() {
        let function = OfTypeFunction::new();

        let collection = vec![
            FhirPathValue::String("test".into()),
            FhirPathValue::Integer(42),
            FhirPathValue::Integer(100),
            FhirPathValue::Boolean(true),
        ];

        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![FhirPathValue::String("integer".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2);
                for item in items.iter() {
                    assert!(matches!(item, FhirPathValue::Integer(_)));
                }
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_of_type_fhir_resource() {
        let function = OfTypeFunction::new();

        let mut patient = HashMap::new();
        patient.insert("resourceType".to_string(), FhirPathValue::String("Patient".into()));
        patient.insert("id".to_string(), FhirPathValue::String("123".into()));

        let mut observation = HashMap::new();
        observation.insert("resourceType".to_string(), FhirPathValue::String("Observation".into()));
        observation.insert("id".to_string(), FhirPathValue::String("456".into()));

        let collection = vec![
            FhirPathValue::JsonValue(serde_json::Value::Object(patient.into())),
            FhirPathValue::JsonValue(serde_json::Value::Object(observation.into())),
            FhirPathValue::String("not a resource".into()),
        ];

        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![FhirPathValue::String("Patient".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 1);
                if let FhirPathValue::JsonValue(serde_json::Value::Object(obj)) = &items.get(0).unwrap() {
                    assert_eq!(obj.get("resourceType"), Some(&FhirPathValue::String("Patient".into())));
                } else {
                    panic!("Expected object in result");
                }
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_of_type_empty_collection() {
        let function = OfTypeFunction::new();

        let context = create_test_context(FhirPathValue::Collection(vec![].into()));
        let args = vec![FhirPathValue::String("string".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 0);
            }
            _ => panic!("Expected collection result"),
        }
    }

    #[tokio::test]
    async fn test_of_type_no_arguments() {
        let function = OfTypeFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        let args = vec![];

        let result = function.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exactly one argument"));
    }

    #[tokio::test]
    async fn test_of_type_inheritance() {
        let function = OfTypeFunction::new();

        let mut patient = HashMap::new();
        patient.insert("resourceType".to_string(), FhirPathValue::String("Patient".into()));

        let mut observation = HashMap::new();
        observation.insert("resourceType".to_string(), FhirPathValue::String("Observation".into()));

        let collection = vec![
            FhirPathValue::JsonValue(serde_json::Value::Object(patient.into())),
            FhirPathValue::JsonValue(serde_json::Value::Object(observation.into())),
        ];

        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![FhirPathValue::String("DomainResource".into())];

        let result = function.evaluate(&args, &context).await.unwrap();

        match result {
            FhirPathValue::Collection(items) => {
                assert_eq!(items.len(), 2); // Both Patient and Observation inherit from DomainResource
            }
            _ => panic!("Expected collection result"),
        }
    }
}
