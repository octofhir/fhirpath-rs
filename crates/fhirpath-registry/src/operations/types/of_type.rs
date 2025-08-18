use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{FhirPathType, MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

pub struct OfTypeFunction {
    metadata: OperationMetadata,
}

impl Default for OfTypeFunction {
    fn default() -> Self {
        Self::new()
    }
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
                message: "ofType() requires exactly one argument (type specifier)".to_string(),
            });
        }

        // Extract type from argument
        let type_arg = &args[0];
        let target_type = match self.extract_type_info(type_arg, context) {
            Ok(name) => name,
            Err(_) => {
                // If type cannot be extracted, return empty collection
                use octofhir_fhirpath_model::Collection;
                return Ok(FhirPathValue::Collection(Collection::from_vec(vec![])));
            }
        };

        let collection = context.input.clone().to_collection();
        let mut result = Vec::new();

        for item in collection.iter() {
            if self.is_of_type(item, &target_type, context).await? {
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
    fn extract_type_info(
        &self,
        type_arg: &FhirPathValue,
        context: &EvaluationContext,
    ) -> Result<String> {
        // Debug: Log what we're getting as type argument
        eprintln!("ofType() received type argument: {type_arg:?}");

        // Check for empty collection first - this might indicate a parsing issue
        if let FhirPathValue::Collection(c) = type_arg {
            if c.is_empty() {
                eprintln!(
                    "ofType() received empty collection as type argument - this suggests a parsing issue"
                );
                return Err(FhirPathError::InvalidArguments {
                    message: "ofType() received empty collection as type argument".to_string(),
                });
            }
        }

        // Use the shared type extraction method from model provider
        context
            .model_provider
            .extract_type_name(type_arg)
            .map_err(|e| FhirPathError::InvalidArguments {
                message: format!("ofType() {e}"),
            })
    }

    async fn is_of_type(
        &self,
        item: &FhirPathValue,
        target_type: &str,
        context: &EvaluationContext,
    ) -> Result<bool> {
        match item {
            FhirPathValue::String(_) => Ok(target_type == "string" || target_type == "String"),
            FhirPathValue::Integer(_) => Ok(target_type == "integer" || target_type == "Integer"),
            FhirPathValue::Decimal(_) => Ok(target_type == "decimal" || target_type == "Decimal"),
            FhirPathValue::Boolean(_) => Ok(target_type == "boolean" || target_type == "Boolean"),
            FhirPathValue::Date(_) => Ok(target_type == "date" || target_type == "Date"),
            FhirPathValue::DateTime(_) => {
                Ok(target_type == "dateTime" || target_type == "DateTime")
            }
            FhirPathValue::Time(_) => Ok(target_type == "time" || target_type == "Time"),
            FhirPathValue::Quantity(_) => {
                Ok(target_type == "Quantity" || target_type == "quantity")
            }
            FhirPathValue::JsonValue(json_val) => {
                // For FHIR resources, check the resourceType property
                if json_val.is_object() {
                    if let Some(resource_type_val) = json_val.get_property("resourceType") {
                        if let Some(resource_type) = resource_type_val.as_str() {
                            let is_direct_match = resource_type == target_type;
                            let is_subtype = context
                                .model_provider
                                .is_subtype_of(resource_type, target_type)
                                .await;
                            Ok(is_direct_match || is_subtype)
                        } else {
                            // For FHIR primitive types, check the actual value type
                            self.check_fhir_primitive_type_sonic(json_val, target_type)
                        }
                    } else {
                        // For FHIR primitive types, check the actual value type
                        self.check_fhir_primitive_type_sonic(json_val, target_type)
                    }
                } else {
                    // For primitive JSON values, check their type
                    self.check_fhir_primitive_type_sonic(json_val, target_type)
                }
            }
            FhirPathValue::Collection(_) => {
                Ok(target_type == "collection" || target_type == "Collection")
            }
            FhirPathValue::Resource(resource) => {
                // Check resource type from the resource itself
                if let Some(resource_type) = resource.resource_type() {
                    let is_direct_match = resource_type == target_type;
                    let is_subtype = context
                        .model_provider
                        .is_subtype_of(resource_type, target_type)
                        .await;
                    Ok(is_direct_match || is_subtype)
                } else {
                    Ok(target_type == "Resource" || target_type == "resource")
                }
            }
            FhirPathValue::TypeInfoObject { .. } => {
                Ok(target_type == "TypeInfo" || target_type == "typeinfo")
            }
            FhirPathValue::Empty => Ok(false), // Empty values don't have a type
        }
    }

    fn check_fhir_primitive_type(
        &self,
        json_val: &sonic_rs::Value,
        target_type: &str,
    ) -> Result<bool> {
        use sonic_rs::JsonValueTrait;

        if json_val.as_str().is_some() {
            Ok(target_type == "string" || target_type == "String")
        } else if let Some(n) = json_val.as_f64() {
            if n.fract() == 0.0 {
                Ok(target_type == "integer" || target_type == "Integer")
            } else {
                Ok(target_type == "decimal" || target_type == "Decimal")
            }
        } else if json_val.as_bool().is_some() {
            Ok(target_type == "boolean" || target_type == "Boolean")
        } else {
            Ok(false)
        }
    }

    fn check_fhir_primitive_type_sonic(
        &self,
        json_val: &octofhir_fhirpath_model::JsonValue,
        target_type: &str,
    ) -> Result<bool> {
        if json_val.as_str().is_some() {
            Ok(target_type == "string" || target_type == "String")
        } else if json_val.is_number() {
            if json_val.as_i64().is_some() {
                Ok(target_type == "integer" || target_type == "Integer")
            } else {
                Ok(target_type == "decimal" || target_type == "Decimal")
            }
        } else if json_val.as_bool().is_some() {
            Ok(target_type == "boolean" || target_type == "Boolean")
        } else {
            Ok(false)
        }
    }
}
