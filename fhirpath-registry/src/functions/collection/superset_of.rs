//! supersetOf() function implementation

use crate::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use fhirpath_model::{FhirPathValue, TypeInfo};

/// supersetOf() function - returns true if the input collection is a superset of the argument collection
pub struct SupersetOfFunction;

impl FhirPathFunction for SupersetOfFunction {
    fn name(&self) -> &str {
        "supersetOf"
    }
    fn human_friendly_name(&self) -> &str {
        "Superset Of"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "supersetOf",
                vec![ParameterInfo::required("subset", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }
    
    fn is_pure(&self) -> bool {
        true // supersetOf() is a pure collection function
    }
    
    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let subset_arg = &args[0];
        let superset = context.input.clone().to_collection();
        let subset = subset_arg.clone().to_collection();

        // Any set is superset of empty set
        if subset.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if every element in subset exists in superset
        let is_superset = subset
            .iter()
            .all(|item| superset.iter().any(|super_item| super_item == item));

        Ok(FhirPathValue::Boolean(is_superset))
    }
}