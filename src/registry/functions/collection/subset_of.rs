//! subsetOf() function implementation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{EvaluationContext, FhirPathFunction, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};

/// subsetOf() function - returns true if the input collection is a subset of the argument collection
pub struct SubsetOfFunction;

impl FhirPathFunction for SubsetOfFunction {
    fn name(&self) -> &str {
        "subsetOf"
    }
    fn human_friendly_name(&self) -> &str {
        "Subset Of"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "subsetOf",
                vec![ParameterInfo::required("superset", TypeInfo::Any)],
                TypeInfo::Boolean,
            )
        });
        &SIG
    }

    fn is_pure(&self) -> bool {
        true // subsetOf() is a pure collection function
    }

    fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let superset_arg = &args[0];
        let subset = context.input.clone().to_collection();
        let superset = superset_arg.clone().to_collection();

        // Empty set is subset of any set
        if subset.is_empty() {
            return Ok(FhirPathValue::Boolean(true));
        }

        // Check if every element in subset exists in superset
        let is_subset = subset
            .iter()
            .all(|item| superset.iter().any(|super_item| super_item == item));

        Ok(FhirPathValue::Boolean(is_subset))
    }
}
