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

//! Collection operators for FHIRPath expressions

use super::super::operator::{Associativity, FhirPathOperator, OperatorRegistry, OperatorResult};
use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::signature::OperatorSignature;

/// Union operator (|)
pub struct UnionOperator;

impl FhirPathOperator for UnionOperator {
    fn symbol(&self) -> &str {
        "|"
    }
    fn human_friendly_name(&self) -> &str {
        "Union"
    }
    fn precedence(&self) -> u8 {
        7 // Per FHIRPath spec: | (union) has precedence #07
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "|",
                TypeInfo::Any,
                TypeInfo::Any,
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        let mut result = left.clone().to_collection();
        result.extend(right.clone().to_collection());
        Ok(FhirPathValue::Collection(result))
    }
}

/// In operator
pub struct InOperator;

impl FhirPathOperator for InOperator {
    fn symbol(&self) -> &str {
        "in"
    }
    fn human_friendly_name(&self) -> &str {
        "In"
    }
    fn precedence(&self) -> u8 {
        4
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "in",
                TypeInfo::Any,
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'in' operator:
        // - If left operand is empty, return empty
        // - If right operand is empty, return [false]
        // - If left operand is multi-item collection, special logic applies

        if left.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if right.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                false,
            )]));
        }

        // For multi-item left operand, each item is tested individually
        let left_collection = left.clone().to_collection();
        let right_collection = right.clone().to_collection();

        // If left has multiple items, return empty (based on test testIn5)
        if left_collection.len() > 1 {
            return Ok(FhirPathValue::Empty);
        }

        // Single item test
        if let Some(single_item) = left_collection.first() {
            Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                right_collection.contains(single_item),
            )]))
        } else {
            Ok(FhirPathValue::Empty)
        }
    }
}

/// Contains operator for collections
pub struct ContainsOperator;

impl FhirPathOperator for ContainsOperator {
    fn symbol(&self) -> &str {
        "contains"
    }
    fn human_friendly_name(&self) -> &str {
        "Contains"
    }
    fn precedence(&self) -> u8 {
        4
    }
    fn associativity(&self) -> Associativity {
        Associativity::Left
    }
    fn signatures(&self) -> &[OperatorSignature] {
        static SIGS: std::sync::LazyLock<Vec<OperatorSignature>> = std::sync::LazyLock::new(|| {
            vec![OperatorSignature::binary(
                "contains",
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
                TypeInfo::Any,
                TypeInfo::Boolean,
            )]
        });
        &SIGS
    }

    fn evaluate_binary(
        &self,
        left: &FhirPathValue,
        right: &FhirPathValue,
    ) -> OperatorResult<FhirPathValue> {
        // Per FHIRPath spec for 'contains' operator:
        // - If both operands are empty, return empty
        // - If left operand is empty (but right is not), return [false]
        // - If right operand is empty (but left is not), return empty

        if left.is_empty() && right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        if left.is_empty() {
            return Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
                false,
            )]));
        }

        if right.is_empty() {
            return Ok(FhirPathValue::Empty);
        }

        let left_collection = left.clone().to_collection();
        Ok(FhirPathValue::collection(vec![FhirPathValue::Boolean(
            left_collection.contains(right),
        )]))
    }
}

/// Register all collection operators
pub fn register_collection_operators(registry: &mut OperatorRegistry) {
    registry.register(UnionOperator);
    registry.register(InOperator);
    registry.register(ContainsOperator);
}
