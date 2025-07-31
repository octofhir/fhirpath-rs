//! conformsTo() function - checks if resource conforms to profile

use crate::function::{
    EvaluationContext, FhirPathFunction, FunctionError, FunctionResult,
};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// conformsTo() function - checks if resource conforms to profile
pub struct ConformsToFunction;

impl FhirPathFunction for ConformsToFunction {
    fn name(&self) -> &str {
        "conformsTo"
    }
    fn human_friendly_name(&self) -> &str {
        "Conforms To"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "conformsTo",
                vec![ParameterInfo::required("profile", TypeInfo::String)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let profile_url = match &args[0] {
            FhirPathValue::String(s) => s,
            _ => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
        };

        // Extract resource type from context input
        let resource_type = match &context.input {
            FhirPathValue::Resource(resource) => {
                if let Some(resource_type) = resource.get_property("resourceType") {
                    match resource_type {
                        serde_json::Value::String(rt) => rt.clone(),
                        _ => return Ok(FhirPathValue::Empty),
                    }
                } else {
                    return Ok(FhirPathValue::Empty);
                }
            }
            _ => return Ok(FhirPathValue::Empty),
        };

        // Basic profile conformance check
        // Check if the profile URL is a valid FHIR StructureDefinition URL
        if !profile_url.starts_with("http://hl7.org/fhir/StructureDefinition/") {
            // Invalid or unknown profile URL returns empty collection
            return Ok(FhirPathValue::Empty);
        }

        // Extract the resource type from the profile URL
        let profile_resource_type = profile_url
            .strip_prefix("http://hl7.org/fhir/StructureDefinition/")
            .unwrap_or("");

        // Check if resource type matches the profile
        let conforms = resource_type == profile_resource_type;
        Ok(FhirPathValue::Boolean(conforms))
    }
}