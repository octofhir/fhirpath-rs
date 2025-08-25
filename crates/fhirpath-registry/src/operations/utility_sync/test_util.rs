//! Simple test file to verify utility sync operations work

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{SyncOperation, EvaluationContext};
    use octofhir_fhirpath_model::{FhirPathValue, MockModelProvider};
    use std::sync::Arc;

    fn create_test_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input, model_provider)
    }

    #[test]
    fn test_has_value_function() {
        let op = super::has_value::HasValueFunction::new();
        
        // Test with non-empty string
        let context = create_test_context(FhirPathValue::String("hello".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test with empty value
        let context = create_test_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_comparable_function() {
        let op = super::comparable::ComparableFunction::new();
        
        // Test comparable integers
        let context = create_test_context(FhirPathValue::Integer(42));
        let args = vec![FhirPathValue::Integer(24)];
        let result = op.execute(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
        
        // Test incomparable types
        let context = create_test_context(FhirPathValue::String("test".into()));
        let args = vec![FhirPathValue::Integer(24)];
        let result = op.execute(&args, &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(false));
    }

    #[test]
    fn test_trace_function() {
        let op = super::trace::TraceFunction::new();
        
        // Test trace returns input unchanged
        let input = FhirPathValue::String("test".into());
        let context = create_test_context(input.clone());
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, input);
    }
}