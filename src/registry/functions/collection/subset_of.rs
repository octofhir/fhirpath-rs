// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! subsetOf() function implementation

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// subsetOf() function - returns true if the input collection is a subset of the argument collection
pub struct SubsetOfFunction;

#[async_trait]
impl AsyncFhirPathFunction for SubsetOfFunction {
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

    async fn evaluate(
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
