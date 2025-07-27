//! FHIR-specific type system functions

use crate::function::{FhirPathFunction, FunctionError, FunctionResult, EvaluationContext};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo, Quantity};


/// is() function - checks FHIR type inheritance
pub struct IsFunction;

impl FhirPathFunction for IsFunction {
    fn name(&self) -> &str { "is" }
    fn human_friendly_name(&self) -> &str { "Is Type" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "is",
                vec![ParameterInfo::required("type", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let target_type = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "String".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };

        let result = match &context.input {
            FhirPathValue::String(_) => {
                // String type hierarchy: string
                matches!(target_type.as_str(), "string")
            },
            FhirPathValue::Integer(_) => {
                // Integer type hierarchy: integer
                matches!(target_type.as_str(), "integer")
            },
            FhirPathValue::Decimal(_) => {
                // Decimal type hierarchy: decimal
                matches!(target_type.as_str(), "decimal")
            },
            FhirPathValue::Boolean(_) => {
                // Boolean type hierarchy: boolean
                matches!(target_type.as_str(), "boolean")
            },
            FhirPathValue::Date(_) => {
                // Date type hierarchy: date
                matches!(target_type.as_str(), "date")
            },
            FhirPathValue::DateTime(_) => {
                // DateTime type hierarchy: dateTime
                matches!(target_type.as_str(), "dateTime")
            },
            FhirPathValue::Time(_) => {
                // Time type hierarchy: time
                matches!(target_type.as_str(), "time")
            },
            FhirPathValue::Quantity(_) => {
                // Quantity type hierarchy: Quantity
                matches!(target_type.as_str(), "Quantity")
            },
            FhirPathValue::Resource(resource) => {
                // FHIR resource type hierarchy
                check_fhir_resource_type(resource, target_type)
            },
            FhirPathValue::Collection(_) => {
                // Collections don't have a specific type
                false
            },
            FhirPathValue::Empty => {
                // Empty has no type
                false
            },
        };

        Ok(FhirPathValue::Boolean(result))
    }
}

/// comparable() function - checks if two quantities have compatible units
pub struct ComparableFunction;

impl FhirPathFunction for ComparableFunction {
    fn name(&self) -> &str { "comparable" }
    fn human_friendly_name(&self) -> &str { "Comparable" }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "comparable",
                vec![ParameterInfo::required("other", TypeInfo::Quantity)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn evaluate(&self, args: &[FhirPathValue], context: &EvaluationContext) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let this_quantity = match &context.input {
            FhirPathValue::Quantity(q) => q,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Quantity".to_string(),
                actual: context.input.type_name().to_string(),
            }),
        };

        let other_quantity = match &args[0] {
            FhirPathValue::Quantity(q) => q,
            _ => return Err(FunctionError::InvalidArgumentType {
                name: self.name().to_string(),
                index: 0,
                expected: "Quantity".to_string(),
                actual: format!("{:?}", args[0]),
            }),
        };

        // Check if quantities have compatible dimensions using existing method
        let result = this_quantity.has_compatible_dimensions(other_quantity);
        Ok(FhirPathValue::Boolean(result))
    }
}

// Helper functions

fn check_fhir_resource_type(resource: &fhirpath_model::FhirResource, target_type: &str) -> bool {
    // Get the resource type from the resource
    if let Some(resource_type) = resource.resource_type() {
        // Check direct match first
        if resource_type == target_type {
            return true;
        }

        // Check FHIR inheritance hierarchy
        match (resource_type, target_type) {
            // Patient inherits from DomainResource
            ("Patient", "DomainResource") => true,
            ("Patient", "Resource") => true,

            // Observation inherits from DomainResource
            ("Observation", "DomainResource") => true,
            ("Observation", "Resource") => true,

            // DomainResource inherits from Resource
            ("DomainResource", "Resource") => true,

            // Add more inheritance relationships as needed
            _ => false,
        }
    } else {
        false
    }
}
