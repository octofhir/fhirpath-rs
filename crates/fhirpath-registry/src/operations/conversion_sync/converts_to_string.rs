//! convertsToString() sync implementation
use octofhir_fhirpath_core::JsonValueExt;
use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::SyncOperation;
use octofhir_fhirpath_core::Result;
use octofhir_fhirpath_core::FhirPathValue;

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
        | FhirPathValue::Quantity { value: _, .. } => Ok(true),

        // JSON simple types convertible by to_string_value()
        FhirPathValue::JsonValue(json) => {
            let inner = json.as_inner();
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
