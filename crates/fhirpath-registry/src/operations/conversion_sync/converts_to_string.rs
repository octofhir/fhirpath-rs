//! convertsToString() sync implementation

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_model::FhirPathValue;

/// convertsToString(): Returns true if the input can be converted to String
pub struct ConvertsToStringFunction;

impl SyncOperation for ConvertsToStringFunction {
    fn name(&self) -> &'static str {
        "convertsToString"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: FunctionSignature = FunctionSignature {
            name: "convertsToString",
            parameters: vec![],
            return_type: ValueType::Boolean,
            variadic: false,
            category: FunctionCategory::Scalar,
            cardinality_requirement: CardinalityRequirement::AcceptsBoth,
        };
        &SIGNATURE
    }

    fn execute(
        &self,
        _args: &[FhirPathValue],
        context: &crate::traits::EvaluationContext,
    ) -> Result<FhirPathValue> {
        let can_convert = can_convert_to_string(&context.input)?;
        Ok(FhirPathValue::Boolean(can_convert))
    }
}

fn can_convert_to_string(value: &FhirPathValue) -> Result<bool> {
    match value {
        // Already a string
        FhirPathValue::String(_) => Ok(true),

        // Primitives that have string representation
        FhirPathValue::Boolean(_)
        | FhirPathValue::Integer(_)
        | FhirPathValue::Decimal(_)
        | FhirPathValue::Date(_)
        | FhirPathValue::DateTime(_)
        | FhirPathValue::Time(_)
        | FhirPathValue::Quantity(_) => Ok(true),

        // JSON simple types convertible by to_string_value()
        FhirPathValue::JsonValue(json) => {
            let inner = json.as_inner();
            use sonic_rs::JsonValueTrait;
            Ok(inner.as_str().is_some()
                || inner.as_bool().is_some()
                || inner.as_f64().is_some()
                || inner.is_null())
        }

        // Empty yields true (per FHIRPath spec for convertsTo* operations)
        FhirPathValue::Empty => Ok(true),

        // Collection rules
        FhirPathValue::Collection(c) => {
            if c.is_empty() {
                Ok(true)
            } else if c.len() == 1 {
                can_convert_to_string(c.first().unwrap())
            } else {
                Ok(false) // Multiple items cannot convert
            }
        }

        // Other complex types cannot convert
        _ => Ok(false),
    }
}

#[cfg(not(test))]
mod tests {
    use super::*;
    use crate::traits::EvaluationContext;
    use octofhir_fhirpath_model::MockModelProvider;

    use std::sync::Arc;

    fn create_context(input: FhirPathValue) -> EvaluationContext {
        let model_provider = Arc::new(MockModelProvider::new());
        EvaluationContext::new(input.clone(), std::sync::Arc::new(input), model_provider)
    }

    #[test]
    fn test_converts_to_string() {
        let op = ConvertsToStringFunction;

        // Test string input
        let context = create_context(FhirPathValue::String("hello".into()));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test boolean input
        let context = create_context(FhirPathValue::Boolean(true));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test integer input
        let context = create_context(FhirPathValue::Integer(42));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test decimal input
        let context = create_context(FhirPathValue::Decimal(Decimal::new(425, 2))); // 4.25
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test date input
        let date = FhirDate::from_ymd(2023, 12, 25).unwrap();
        let context = create_context(FhirPathValue::Date(date));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test quantity input
        let quantity = Quantity::new(42.5, "mg");
        let context = create_context(FhirPathValue::Quantity(quantity));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test JSON string value
        let json_string = JsonValue::from(json!("test"));
        let context = create_context(FhirPathValue::JsonValue(json_string));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test JSON number value
        let json_number = JsonValue::from(json!(42));
        let context = create_context(FhirPathValue::JsonValue(json_number));
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));

        // Test empty input
        let context = create_context(FhirPathValue::Empty);
        let result = op.execute(&[], &context).unwrap();
        assert_eq!(result, FhirPathValue::Boolean(true));
    }
}
