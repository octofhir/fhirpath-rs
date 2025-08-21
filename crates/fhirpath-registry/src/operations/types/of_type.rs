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
        _args: &[FhirPathValue],
        _context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // ofType() requires model provider access which is async
        // Cannot be evaluated synchronously
        None
    }

    fn supports_sync(&self) -> bool {
        false
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
        // Check for empty collection first - this might indicate a parsing issue
        if let FhirPathValue::Collection(c) = type_arg {
            if c.is_empty() {
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
        // Use ModelProvider's comprehensive type checking which handles:
        // - Primitive types
        // - FHIR resources with inheritance
        // - Collections
        // - Type normalization
        let is_of_type = context
            .model_provider
            .is_value_of_type(item, target_type)
            .await;
        Ok(is_of_type)
    }

    // check_fhir_primitive_type method removed - ModelProvider.is_value_of_type handles all type checking
}
