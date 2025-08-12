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

//! join() function - joins collection of strings

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use fhirpath_model::{FhirPathValue, types::TypeInfo};
/// join() function - joins collection of strings
pub struct JoinFunction;

#[async_trait]
impl AsyncFhirPathFunction for JoinFunction {
    fn name(&self) -> &str {
        "join"
    }
    fn human_friendly_name(&self) -> &str {
        "Join"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "join",
                vec![ParameterInfo::optional("separator", TypeInfo::String)],
                TypeInfo::String,
            )
        });
        &SIG
    }
    fn is_pure(&self) -> bool {
        true // join() is a pure string function
    }
    async fn evaluate(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> FunctionResult<FhirPathValue> {
        self.validate_args(args)?;
        let separator = match args.first() {
            Some(FhirPathValue::String(s)) => s.as_ref(),
            Some(_) => {
                return Err(FunctionError::InvalidArgumentType {
                    name: self.name().to_string(),
                    index: 0,
                    expected: "String".to_string(),
                    actual: format!("{:?}", args[0]),
                });
            }
            None => "",
        };

        let items = context.input.clone().to_collection();
        let strings: Vec<String> = items
            .into_iter()
            .map(|item| match item {
                FhirPathValue::String(s) => s.as_ref().to_string(),
                FhirPathValue::Integer(i) => i.to_string(),
                FhirPathValue::Decimal(d) => d.to_string(),
                FhirPathValue::Boolean(b) => b.to_string(),
                FhirPathValue::Date(d) => d.to_string(),
                FhirPathValue::DateTime(dt) => dt.to_string(),
                FhirPathValue::Time(t) => t.to_string(),
                FhirPathValue::Quantity(q) => q.to_string(),
                FhirPathValue::Resource(r) => {
                    // Try to extract string value from FhirResource
                    match r.as_json() {
                        serde_json::Value::String(s) => s.clone(),
                        _ => format!("{r:?}"),
                    }
                }
                FhirPathValue::Empty => String::new(),
                FhirPathValue::Collection(_) => String::new(), // Empty collections become empty strings
                FhirPathValue::TypeInfoObject { namespace, name } => {
                    format!("{namespace}::{name}")
                }
                FhirPathValue::JsonValue(json) => {
                    // Convert JsonValue to string representation
                    json.as_json().to_string()
                }
            })
            .collect();

        Ok(FhirPathValue::String(
            strings.join(separator.as_ref()).into(),
        ))
    }
}
