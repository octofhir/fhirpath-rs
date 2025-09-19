//! Integration tests for the %terminologies variable system
//!
//! These tests demonstrate the usage of the %terminologies system variable
//! for accessing terminology services in FHIRPath expressions.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::{Collection, FhirPathValue};
    use crate::evaluator::engine::create_engine_with_mock_provider;
    use crate::evaluator::{EvaluationContext, terminologies_variable::TerminologiesVariable};
    use octofhir_fhir_model::terminology::NoOpTerminologyProvider;

    #[tokio::test]
    async fn test_terminologies_variable_availability() {
        // Create a terminology provider
        let terminology_provider = Arc::new(NoOpTerminologyProvider::default());

        // Create an engine with terminology provider
        let engine = create_engine_with_mock_provider().await.unwrap();

        // Create evaluation context with terminology provider
        let input = Collection::empty();
        let context = EvaluationContext::new(
            input,
            engine.model_provider().clone(),
            Some(terminology_provider.clone()),
            None, // No validation provider
            None, // No trace provider
        )
        .await;

        // Test that %terminologies variable is available (stored as "terminologies")
        let terminologies_var = context.get_variable("terminologies");
        assert!(
            terminologies_var.is_some(),
            "%terminologies variable should be available when terminology provider is present"
        );

        // Verify it's recognized as a terminologies variable
        let terminologies_value = terminologies_var.unwrap();
        assert!(
            crate::evaluator::terminologies_variable::is_terminologies_variable(
                &terminologies_value
            )
        );
    }

    #[tokio::test]
    async fn test_terminologies_variable_unavailable_without_provider() {
        // Create an engine without terminology provider
        let engine = create_engine_with_mock_provider().await.unwrap();

        // Create evaluation context without terminology provider
        let input = Collection::empty();
        let context = EvaluationContext::new(
            input,
            engine.model_provider().clone(),
            None, // No terminology provider
            None, // No validation provider
            None, // No trace provider
        )
        .await;

        // Test that %terminologies variable is not available
        let terminologies_var = context.get_variable("%terminologies");
        assert!(
            terminologies_var.is_none(),
            "%terminologies variable should not be available when no terminology provider is present"
        );
    }

    #[tokio::test]
    async fn test_terminologies_variable_structure() {
        // Create a terminology provider
        let terminology_provider = Arc::new(NoOpTerminologyProvider::default());
        let terminologies_var = TerminologiesVariable::new(terminology_provider);
        let fhir_path_value = terminologies_var.to_fhir_path_value();

        // Verify the structure of the terminologies variable
        match &fhir_path_value {
            FhirPathValue::Resource(resource, _, _) => {
                // Check resource type
                assert_eq!(
                    resource.get("resourceType").and_then(|rt| rt.as_str()),
                    Some("TerminologiesVariable")
                );

                // Check supported operations
                let supported_ops = resource
                    .get("supportedOperations")
                    .and_then(|ops| ops.as_array())
                    .expect("supportedOperations should be an array");

                let operation_names: Vec<&str> =
                    supported_ops.iter().filter_map(|op| op.as_str()).collect();

                assert!(operation_names.contains(&"expand"));
                assert!(operation_names.contains(&"lookup"));
                assert!(operation_names.contains(&"validateVS"));
                assert!(operation_names.contains(&"validateCS"));
                assert!(operation_names.contains(&"subsumes"));
                assert!(operation_names.contains(&"translate"));
            }
            _ => panic!("terminologies variable should be a resource"),
        }
    }

    #[tokio::test]
    async fn test_system_variables_all_work() {
        // Create a terminology provider
        let terminology_provider = Arc::new(NoOpTerminologyProvider::default());

        // Create an engine with terminology provider
        let engine = create_engine_with_mock_provider().await.unwrap();

        // Create evaluation context
        let input = Collection::from(vec![FhirPathValue::string("test")]);
        let mut context = EvaluationContext::new(
            input,
            engine.model_provider().clone(),
            Some(terminology_provider.clone()),
            None, // No validation provider
            None, // No trace provider
        )
        .await;

        // Set system variables for testing
        context.set_variable("this".to_string(), FhirPathValue::string("this_value".to_string()));
        context.set_variable("index".to_string(), FhirPathValue::integer(5));
        context.set_variable("total".to_string(), FhirPathValue::integer(10));

        // Test all system variables - note: variables are stored without prefixes internally
        assert!(context.get_variable("this").is_some());
        assert!(context.get_variable("index").is_some());
        assert!(context.get_variable("total").is_some());
        assert!(context.get_variable("terminologies").is_some());

        // Verify values
        match context.get_variable("index").unwrap() {
            FhirPathValue::Integer(i, _, _) => assert_eq!(i, 5),
            _ => panic!("$index should be an integer"),
        }

        match context.get_variable("total").unwrap() {
            FhirPathValue::Integer(i, _, _) => assert_eq!(i, 10),
            _ => panic!("$total should be an integer"),
        }
    }

    #[test]
    fn test_terminologies_variable_detection() {
        // Test positive case
        let terminology_provider = Arc::new(NoOpTerminologyProvider::default());
        let terminologies_var = TerminologiesVariable::new(terminology_provider);
        let terminologies_value = terminologies_var.to_fhir_path_value();

        assert!(
            crate::evaluator::terminologies_variable::is_terminologies_variable(
                &terminologies_value
            )
        );

        // Test negative case - regular resource
        let regular_resource = FhirPathValue::resource(serde_json::json!({
            "resourceType": "Patient",
            "id": "example"
        }));
        assert!(
            !crate::evaluator::terminologies_variable::is_terminologies_variable(&regular_resource)
        );

        // Test negative case - non-resource
        let string_value = FhirPathValue::string("test");
        assert!(
            !crate::evaluator::terminologies_variable::is_terminologies_variable(&string_value)
        );
    }
}
