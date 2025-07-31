//! String operators for FHIRPath expressions

use crate::operator::{FhirPathOperator, OperatorRegistry, OperatorResult, Associativity};
use crate::signature::OperatorSignature;
use fhirpath_model::{FhirPathValue, TypeInfo};

/// String concatenation operator (&)
pub struct ConcatenateOperator;

impl FhirPathOperator for ConcatenateOperator {
    fn symbol(&self) -> &str {
        "&"
    }
    fn human_friendly_name(&self) -> &str {
        "Concatenate"
    }
    fn precedence(&self) -> u8 {
        5
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "&",
                TypeInfo::String,
                TypeInfo::String,
                TypeInfo::String,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let left_str = left.to_string_value().unwrap_or_default();
        let right_str = right.to_string_value().unwrap_or_default();
        Ok(FhirPathValue::collection(vec![FhirPathValue::String(
            left_str + &right_str,
        )]))
    }
}

/// Register all string operators
pub fn register_string_operators(registry: &mut OperatorRegistry) {
    registry.register(ConcatenateOperator);
}