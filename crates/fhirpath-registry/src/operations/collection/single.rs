use crate::operations::EvaluationContext;
use crate::{
    FhirPathOperation,
    metadata::{MetadataBuilder, OperationMetadata, OperationType, TypeConstraint},
};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

pub struct SingleFunction {
    metadata: OperationMetadata,
}

impl Default for SingleFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl SingleFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("single", OperationType::Function)
            .description("Returns the single item in the collection if there is exactly one item. Returns empty collection if input is empty. Signals error if multiple items exist")
            .returns(TypeConstraint::Any)
            .example("Patient.name.single()")
            .example("Bundle.entry.single().resource")
            .build()
    }
}

#[async_trait]
impl FhirPathOperation for SingleFunction {
    fn identifier(&self) -> &str {
        "single"
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
        // Validate no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArguments {
                message: "single() takes no arguments".to_string(),
            });
        }

        let collection = context.input.clone().to_collection();

        match collection.len() {
            0 => Ok(FhirPathValue::Empty),
            1 => Ok(collection.get(0).unwrap().clone()),
            n => Err(FhirPathError::function_error(
                "single",
                format!("single() requires exactly one item, found {n}"),
            )),
        }
    }

    fn try_evaluate_sync(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Option<Result<FhirPathValue>> {
        // Can be synchronous - no complex evaluation needed
        Some(futures::executor::block_on(self.evaluate(args, context)))
    }

    fn supports_sync(&self) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
