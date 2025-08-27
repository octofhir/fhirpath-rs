//! Comparable function implementation - sync version

use crate::signature::{
    CardinalityRequirement, FunctionCategory, FunctionSignature, ParameterType, ValueType,
};
use crate::traits::{EvaluationContext, SyncOperation};
use octofhir_fhirpath_core::{FhirPathError, Result};
use octofhir_fhirpath_model::FhirPathValue;

/// Comparable function - checks if two values can be compared
#[derive(Debug, Clone)]
pub struct ComparableFunction;

impl ComparableFunction {
    pub fn new() -> Self {
        Self
    }

    fn can_compare_values(left: &FhirPathValue, right: &FhirPathValue) -> bool {
        use FhirPathValue::*;
        match (left, right) {
            (Integer(_), Integer(_)) => true,
            (Decimal(_), Decimal(_)) => true,
            (Integer(_), Decimal(_)) => true,
            (Decimal(_), Integer(_)) => true,
            (String(_), String(_)) => true,
            (Date(_), Date(_)) => true,
            (DateTime(_), DateTime(_)) => true,
            (Time(_), Time(_)) => true,
            (Boolean(_), Boolean(_)) => true,
            (Quantity(q1), Quantity(q2)) => Self::are_units_comparable(&q1.unit, &q2.unit),
            _ => false,
        }
    }

    /// Check if two units are comparable (same dimension/type of measurement)
    fn are_units_comparable(unit1: &Option<String>, unit2: &Option<String>) -> bool {
        match (unit1, unit2) {
            // Both units are None (dimensionless)
            (None, None) => true,
            // One has unit, other doesn't - not comparable
            (None, Some(_)) | (Some(_), None) => false,
            // Both have units - check compatibility
            (Some(u1), Some(u2)) => {
                if u1 == u2 {
                    // Exact match
                    true
                } else {
                    // Check unit compatibility by dimension
                    Self::get_unit_dimension(u1) == Self::get_unit_dimension(u2)
                }
            }
        }
    }

    /// Get the dimension type for a unit (length, mass, time, etc.)
    fn get_unit_dimension(unit: &str) -> String {
        match unit {
            // Length units
            "m" | "cm" | "mm" | "km" | "[in_i]" | "[ft_i]" | "[yd_i]" | "[mi_i]" => {
                "length".to_string()
            }
            // Mass units
            "kg" | "g" | "mg" | "[lb_av]" | "[oz_av]" => "mass".to_string(),
            // Time units
            "s" | "min" | "h" | "d" | "wk" | "mo" | "a" => "time".to_string(),
            // Temperature units
            "Cel" | "[degF]" | "K" => "temperature".to_string(),
            // Volume units
            "L" | "mL" | "[gal_us]" | "[qt_us]" | "[pt_us]" | "[cup_us]" => "volume".to_string(),
            // Energy units
            "J" | "cal" | "[Btu]" => "energy".to_string(),
            // Default: treat unknown units as unique dimensions
            _ => unit.to_string(),
        }
    }
}

impl SyncOperation for ComparableFunction {
    fn name(&self) -> &'static str {
        "comparable"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "comparable",
                parameters: vec![ParameterType::Any],
                return_type: ValueType::Boolean,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        if args.len() != 1 {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "comparable".to_string(),
                expected: 1,
                actual: args.len(),
            });
        }

        let can_compare = Self::can_compare_values(&context.input, &args[0]);
        Ok(FhirPathValue::Boolean(can_compare))
    }
}

impl Default for ComparableFunction {
    fn default() -> Self {
        Self::new()
    }
}
