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

//! supersetOf() function implementation

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// supersetOf() function - returns true if the input collection is a superset of the argument collection
pub struct SupersetOfFunction;

#[async_trait]
impl AsyncFhirPathFunction for SupersetOfFunction {
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

    async fn evaluate(
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
