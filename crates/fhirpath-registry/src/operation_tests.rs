//! Test for signature-based validation functionality

#[cfg(test)]
mod signature_validation_tests {
    use super::*;
    use crate::operation::{FhirPathOperation};
    use crate::metadata::{MetadataBuilder, OperationType, TypeConstraint, FhirPathType, PerformanceComplexity};
    use octofhir_fhirpath_core::{Result, FhirPathError};
    use octofhir_fhirpath_model::FhirPathValue;
    use crate::operations::EvaluationContext;
    use async_trait::async_trait;

    struct MockFunction;

    impl MockFunction {
        fn create_metadata() -> crate::metadata::OperationMetadata {
            MetadataBuilder::new("mock", OperationType::Function)
                .description("Mock function for testing signature validation")
                .parameter("input", TypeConstraint::Specific(FhirPathType::String), false)
                .returns(TypeConstraint::Specific(FhirPathType::Integer))
                .performance(PerformanceComplexity::Constant, true)
                .build()
        }
    }

    #[async_trait]
    impl FhirPathOperation for MockFunction {
        fn identifier(&self) -> &str {
            "mock"
        }

        fn operation_type(&self) -> OperationType {
            OperationType::Function
        }

        fn metadata(&self) -> &crate::metadata::OperationMetadata {
            static METADATA: std::sync::LazyLock<crate::metadata::OperationMetadata> = 
                std::sync::LazyLock::new(|| MockFunction::create_metadata());
            &METADATA
        }

        async fn evaluate(
            &self,
            _args: &[FhirPathValue],
            _context: &EvaluationContext,
        ) -> Result<FhirPathValue> {
            Ok(FhirPathValue::Integer(42))
        }
    }

    #[test]
    fn test_signature_based_validation() {
        let mock_fn = MockFunction;
        
        // Test 1: Valid arguments (should pass)
        let valid_args = vec![FhirPathValue::String("hello".into())];
        let validation_result = mock_fn.validate_args(&valid_args);
        assert!(validation_result.is_ok(), "Valid args should pass validation");

        // Test 2: No arguments (should fail - requires 1 argument)
        let no_args: Vec<FhirPathValue> = vec![];
        let validation_result = mock_fn.validate_args(&no_args);
        assert!(validation_result.is_err(), "No args should fail validation");
        
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = validation_result {
            assert_eq!(expected, 1);
            assert_eq!(actual, 0);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }

        // Test 3: Too many arguments (should fail - max 1 argument)
        let too_many_args = vec![
            FhirPathValue::String("hello".into()),
            FhirPathValue::String("world".into()),
        ];
        let validation_result = mock_fn.validate_args(&too_many_args);
        assert!(validation_result.is_err(), "Too many args should fail validation");
        
        if let Err(FhirPathError::InvalidArgumentCount { expected, actual, .. }) = validation_result {
            assert_eq!(expected, 1);
            assert_eq!(actual, 2);
        } else {
            panic!("Expected InvalidArgumentCount error");
        }
    }

    #[test]
    fn test_signature_generation_from_metadata() {
        let mock_fn = MockFunction;
        let signature = mock_fn.signature();
        
        // Test that signature was properly generated from metadata
        match signature {
            crate::operation::OperationSignature::Function(func_sig) => {
                assert_eq!(func_sig.name, "mock");
                assert_eq!(func_sig.min_arity, 1);
                assert_eq!(func_sig.max_arity, Some(1));
                assert_eq!(func_sig.parameters.len(), 1);
                assert_eq!(func_sig.parameters[0].name, "input");
            }
            _ => panic!("Expected Function signature"),
        }
    }
}