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

//! trace() function - debugging function that logs and returns input

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::function::{AsyncFhirPathFunction, EvaluationContext, FunctionResult};
use crate::registry::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;

/// trace() function - debugging function that logs and returns input
pub struct TraceFunction;

#[async_trait]
impl AsyncFhirPathFunction for TraceFunction {
    fn name(&self) -> &str {
        "trace"
    }
    fn human_friendly_name(&self) -> &str {
        "Trace"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "trace",
                vec![
                    ParameterInfo::required("name", TypeInfo::String),
                    ParameterInfo::optional("selector", TypeInfo::Any),
                ],
                TypeInfo::Any,
            )
        });
        &SIG
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;

        let _name = match &args[0] {
            FhirPathValue::String(s) => s.as_ref().to_string(),
            _ => "trace".to_string(),
        };

        // Check if there's a second argument (selector)
        let _value_to_trace = if args.len() > 1 {
            &args[1]
        } else {
            &context.input
        };

        // In a real implementation, this would log to appropriate output
        // For debugging: eprintln!("{}: {:?}", name, value_to_trace);

        // trace() function always returns the original input (context), not the traced value
        Ok(context.input.clone())
    }
}
