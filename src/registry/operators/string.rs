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

//! String operators for FHIRPath expressions

use super::super::operator::{Associativity, FhirPathOperator, OperatorRegistry, OperatorResult};
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::signature::OperatorSignature;

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
            (left_str + &right_str).into(),
        )]))
    }
}

/// Register all string operators
pub fn register_string_operators(registry: &mut OperatorRegistry) {
    registry.register(ConcatenateOperator);
}
