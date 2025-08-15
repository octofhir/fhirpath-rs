use async_trait::async_trait;
use crate::{FhirPathOperation, metadata::{OperationType, OperationMetadata, MetadataBuilder, TypeConstraint}};
use crate::operations::EvaluationContext;
use octofhir_fhirpath_core::{Result, FhirPathError};
use octofhir_fhirpath_model::FhirPathValue;

pub struct SingleFunction {
    metadata: OperationMetadata,
}

impl SingleFunction {
    pub fn new() -> Self {
        Self {
            metadata: Self::create_metadata(),
        }
    }

    fn create_metadata() -> OperationMetadata {
        MetadataBuilder::new("single", OperationType::Function)
            .description("Returns the single item in the collection. If the collection contains more than one item, or is empty, an error is returned")
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
                message: "single() takes no arguments".to_string()
            });
        }

        let collection = context.input.clone().to_collection();

        match collection.len() {
            0 => Err(FhirPathError::EvaluationError {
                message: "single() called on empty collection".to_string()
            }),
            1 => Ok(collection.get(0).unwrap().clone()),
            n => Err(FhirPathError::EvaluationError {
                message: format!("single() called on collection with {} items (expected exactly 1)", n)
            }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operations::create_test_context;

    #[tokio::test]
    async fn test_single_with_single_item() {
        let function = SingleFunction::new();
        
        let collection = vec![FhirPathValue::String("test".into())];
        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![];
        
        let result = function.evaluate(&args, &context).await.unwrap();
        
        match result {
            FhirPathValue::String(s) => {
                assert_eq!(s, "test");
            }
            _ => panic!("Expected string result"),
        }
    }

    #[tokio::test]
    async fn test_single_with_empty_collection() {
        let function = SingleFunction::new();
        
        let context = create_test_context(FhirPathValue::Collection(vec![].into()));
        let args = vec![];
        
        let result = function.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty collection"));
    }

    #[tokio::test]
    async fn test_single_with_multiple_items() {
        let function = SingleFunction::new();
        
        let collection = vec![
            FhirPathValue::String("first".into()),
            FhirPathValue::String("second".into()),
        ];
        let context = create_test_context(FhirPathValue::Collection(collection.into()));
        let args = vec![];
        
        let result = function.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("2 items"));
    }

    #[tokio::test]
    async fn test_single_with_non_collection_input() {
        let function = SingleFunction::new();
        
        // Non-collection input should be treated as single-item collection
        let context = create_test_context(FhirPathValue::String("test".into()));
        let args = vec![];
        
        let result = function.evaluate(&args, &context).await.unwrap();
        
        match result {
            FhirPathValue::String(s) => {
                assert_eq!(s, "test");
            }
            _ => panic!("Expected string result"),
        }
    }

    #[tokio::test]
    async fn test_single_with_arguments() {
        let function = SingleFunction::new();
        let context = create_test_context(FhirPathValue::String("test".into()));
        let args = vec![FhirPathValue::String("invalid".into())];
        
        let result = function.evaluate(&args, &context).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no arguments"));
    }

    #[tokio::test]
    async fn test_single_with_various_types() {
        let function = SingleFunction::new();
        
        // Test with integer
        let context = create_test_context(FhirPathValue::Collection(vec![
            FhirPathValue::Integer(42)
        ].into()));
        let args = vec![];
        
        let result = function.evaluate(&args, &context).await.unwrap();
        assert!(matches!(result, FhirPathValue::Integer(42)));

        // Test with boolean
        let context = create_test_context(FhirPathValue::Collection(vec![
            FhirPathValue::Boolean(true)
        ].into()));
        
        let result = function.evaluate(&args, &context).await.unwrap();
        assert!(matches!(result, FhirPathValue::Boolean(true)));
    }
}