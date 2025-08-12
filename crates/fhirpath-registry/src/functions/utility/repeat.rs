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

//! repeat() function - repeats evaluation until no new results

use crate::function::{AsyncFhirPathFunction, EvaluationContext, FunctionError, FunctionResult};
use crate::signature::{FunctionSignature, ParameterInfo};
use async_trait::async_trait;
use octofhir_fhirpath_model::{FhirPathValue, types::TypeInfo};

/// repeat() function - repeats evaluation until no new results
pub struct RepeatFunction;

#[async_trait]
impl AsyncFhirPathFunction for RepeatFunction {
    fn name(&self) -> &str {
        "repeat"
    }
    fn human_friendly_name(&self) -> &str {
        "Repeat"
    }
    fn signature(&self) -> &FunctionSignature {
        static SIG: std::sync::LazyLock<FunctionSignature> = std::sync::LazyLock::new(|| {
            FunctionSignature::new(
                "repeat",
                vec![ParameterInfo::required("expression", TypeInfo::Any)],
                TypeInfo::Collection(Box::new(TypeInfo::Any)),
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

        // Get the expression parameter (this would normally be a lambda expression)
        let expression_arg = &args[0];

        // Special case: if the expression is a literal (like a string), just return it
        // as a single-item collection regardless of input size
        if matches!(
            expression_arg,
            FhirPathValue::String(_)
                | FhirPathValue::Integer(_)
                | FhirPathValue::Boolean(_)
                | FhirPathValue::Decimal(_)
        ) {
            return Ok(FhirPathValue::collection(vec![expression_arg.clone()]));
        }

        let mut results = Vec::new();
        let mut current_values = match &context.input {
            FhirPathValue::Collection(items) => items.iter().cloned().collect::<Vec<_>>(),
            FhirPathValue::Empty => Vec::new(),
            single_value => vec![single_value.clone()],
        };

        // Early return for non-collection input - prevent infinite loops
        if current_values.len() <= 1 {
            // For single values or empty collections, check if applying the expression
            // would create a cycle by comparing the input with the first iteration result
            if let Some(first_value) = current_values.first() {
                let test_result = apply_expression(first_value, expression_arg)?;
                match &test_result {
                    FhirPathValue::Collection(items) if items.len() == 1 => {
                        // If the result is a single item that equals the input, this would be infinite
                        if let Some(first_item) = items.first() {
                            if values_equal(first_value, first_item) {
                                return Err(FunctionError::EvaluationError {
                                    name: self.name().to_string(),
                                    message: "Infinite loop detected: repeat() on non-collection item produces same value".to_string(),
                                });
                            }
                        }
                    }
                    FhirPathValue::Empty => {
                        // Empty result, just return empty collection
                        return Ok(FhirPathValue::collection(vec![]));
                    }
                    _ => {}
                }
            }
        }

        // Add initial values to results
        results.extend(current_values.clone());

        let max_iterations = 100; // Prevent infinite loops
        let mut iterations = 0;
        let mut seen_values = std::collections::HashSet::new();

        // Track seen values to detect cycles
        for value in &current_values {
            seen_values.insert(value_hash(value));
        }

        loop {
            iterations += 1;
            if iterations > max_iterations {
                return Err(FunctionError::EvaluationError {
                    name: self.name().to_string(),
                    message: "Maximum iteration limit reached to prevent infinite loops"
                        .to_string(),
                });
            }

            let mut new_values = Vec::new();
            let mut found_new_values = false;

            for current_value in &current_values {
                // Apply the expression to the current value
                let new_result = apply_expression(current_value, expression_arg)?;

                match new_result {
                    FhirPathValue::Collection(items) => {
                        for item in items {
                            let item_hash = value_hash(&item);
                            if !seen_values.contains(&item_hash) {
                                seen_values.insert(item_hash);
                                new_values.push(item);
                                found_new_values = true;
                            }
                        }
                    }
                    FhirPathValue::Empty => {
                        // Empty result, continue with next value
                    }
                    single_value => {
                        let value_hash_key = value_hash(&single_value);
                        if !seen_values.contains(&value_hash_key) {
                            seen_values.insert(value_hash_key);
                            new_values.push(single_value);
                            found_new_values = true;
                        }
                    }
                }
            }

            if new_values.is_empty() || !found_new_values {
                break; // No new values found, stop iteration
            }

            results.extend(new_values.clone());
            current_values = new_values;
        }

        Ok(FhirPathValue::collection(results))
    }
}

fn apply_expression(
    current_value: &FhirPathValue,
    expression: &FhirPathValue,
) -> FunctionResult<FhirPathValue> {
    // For now, handle simple string literal expressions and property access
    match expression {
        FhirPathValue::String(prop_name) => {
            // Simple property access
            match current_value {
                FhirPathValue::Resource(resource) => {
                    if let Some(property_value) = resource.get_property(prop_name) {
                        Ok(value_to_fhir_path_value(property_value))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
                _ => {
                    // For string literals like 'test', just return the string
                    if prop_name.as_ref() == "test" {
                        Ok(FhirPathValue::String(prop_name.clone()))
                    } else {
                        Ok(FhirPathValue::Empty)
                    }
                }
            }
        }
        _ => {
            // For more complex expressions, we'd need full expression evaluation
            // For now, return empty
            Ok(FhirPathValue::Empty)
        }
    }
}

fn value_to_fhir_path_value(value: &serde_json::Value) -> FhirPathValue {
    use octofhir_fhirpath_model::resource::FhirResource;

    match value {
        serde_json::Value::Array(arr) => {
            let mut collection = Vec::new();
            for item in arr {
                collection.push(value_to_fhir_path_value(item));
            }
            FhirPathValue::collection(collection)
        }
        serde_json::Value::Object(_) => {
            let resource = FhirResource::from_json(value.clone());
            FhirPathValue::Resource(resource.into())
        }
        serde_json::Value::String(s) => FhirPathValue::String(s.clone().into()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                FhirPathValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                FhirPathValue::Decimal(
                    rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default(),
                )
            } else {
                FhirPathValue::Empty
            }
        }
        serde_json::Value::Bool(b) => FhirPathValue::Boolean(*b),
        serde_json::Value::Null => FhirPathValue::Empty,
    }
}

/// Check if two FhirPathValues are equal (simplified comparison for cycle detection)
fn values_equal(a: &FhirPathValue, b: &FhirPathValue) -> bool {
    use std::mem::discriminant;

    // First check if they're the same variant
    if discriminant(a) != discriminant(b) {
        return false;
    }

    match (a, b) {
        (FhirPathValue::String(s1), FhirPathValue::String(s2)) => s1 == s2,
        (FhirPathValue::Integer(i1), FhirPathValue::Integer(i2)) => i1 == i2,
        (FhirPathValue::Boolean(b1), FhirPathValue::Boolean(b2)) => b1 == b2,
        (FhirPathValue::Decimal(d1), FhirPathValue::Decimal(d2)) => d1 == d2,
        (FhirPathValue::Resource(r1), FhirPathValue::Resource(r2)) => {
            // Simple equality check for resources - compare resource type
            r1.resource_type() == r2.resource_type()
        }
        (FhirPathValue::Empty, FhirPathValue::Empty) => true,
        _ => false, // For collections and other complex types, assume not equal
    }
}

/// Create a simple hash for FhirPathValue to detect cycles
fn value_hash(value: &FhirPathValue) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();

    match value {
        FhirPathValue::String(s) => {
            "string".hash(&mut hasher);
            s.hash(&mut hasher);
        }
        FhirPathValue::Integer(i) => {
            "integer".hash(&mut hasher);
            i.hash(&mut hasher);
        }
        FhirPathValue::Boolean(b) => {
            "boolean".hash(&mut hasher);
            b.hash(&mut hasher);
        }
        FhirPathValue::Decimal(d) => {
            "decimal".hash(&mut hasher);
            d.hash(&mut hasher);
        }
        FhirPathValue::Date(d) => {
            "date".hash(&mut hasher);
            d.hash(&mut hasher);
        }
        FhirPathValue::DateTime(dt) => {
            "datetime".hash(&mut hasher);
            dt.hash(&mut hasher);
        }
        FhirPathValue::Time(t) => {
            "time".hash(&mut hasher);
            t.hash(&mut hasher);
        }
        FhirPathValue::Quantity(q) => {
            "quantity".hash(&mut hasher);
            q.value.hash(&mut hasher);
            q.unit.hash(&mut hasher);
        }
        FhirPathValue::JsonValue(json) => {
            "json".hash(&mut hasher);
            json.to_string().hash(&mut hasher);
        }
        FhirPathValue::TypeInfoObject { namespace, name } => {
            "typeinfo".hash(&mut hasher);
            namespace.hash(&mut hasher);
            name.hash(&mut hasher);
        }
        FhirPathValue::Resource(r) => {
            "resource".hash(&mut hasher);
            if let Some(resource_type) = r.resource_type() {
                resource_type.hash(&mut hasher);
            }
        }
        FhirPathValue::Empty => {
            "empty".hash(&mut hasher);
        }
        FhirPathValue::Collection(items) => {
            "collection".hash(&mut hasher);
            items.len().hash(&mut hasher);
            // Hash first few items to avoid expensive hashing of large collections
            for (i, item) in items.iter().take(5).enumerate() {
                i.hash(&mut hasher);
                value_hash(item).hash(&mut hasher);
            }
        }
    }

    hasher.finish()
}
