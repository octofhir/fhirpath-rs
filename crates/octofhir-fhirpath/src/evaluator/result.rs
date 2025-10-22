//! Evaluation result types for FHIRPath evaluation
//!
//! This module defines the result types returned by FHIRPath evaluation.

use crate::core::model_provider::TypeInfo;
use crate::core::{Collection, FhirPathValue};
use octofhir_fhir_model::{EvaluationResult as ModelEvalResult, TypeInfoResult};
use std::collections::HashMap;

/// Evaluation result containing the resulting collection
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Result collection (always a Collection per FHIRPath spec)
    pub value: Collection,
}

impl EvaluationResult {
    /// Create new evaluation result
    pub fn new(value: Collection) -> Self {
        Self { value }
    }

    /// Create evaluation result from values
    pub fn from_values(values: Vec<FhirPathValue>) -> Self {
        Self {
            value: Collection::from(values),
        }
    }

    /// Convert to ModelEvaluationResult for external interface
    pub fn to_evaluation_result(&self) -> octofhir_fhir_model::EvaluationResult {
        // Handle empty collection
        if self.value.is_empty() {
            return ModelEvalResult::Empty;
        }

        // Handle singleton collection - unwrap to scalar per FHIRPath semantics
        if self.value.len() == 1
            && let Some(first) = self.value.first()
        {
            return convert_fhirpath_value_to_eval_result(first);
        }

        // Handle multi-value collection
        let items: Vec<ModelEvalResult> = self
            .value
            .iter()
            .map(convert_fhirpath_value_to_eval_result)
            .collect();

        ModelEvalResult::Collection {
            items,
            has_undefined_order: !self.value.is_ordered(),
            type_info: None, // Could extract common type if needed
        }
    }

    /// Check if result represents true (for boolean evaluation)
    pub fn to_boolean(&self) -> bool {
        // Follow FHIRPath boolean conversion rules
        if self.value.is_empty() {
            false
        } else if self.value.len() == 1 {
            match self.value.iter().next() {
                Some(FhirPathValue::Boolean(b, _, _)) => *b,
                Some(_) => true, // Non-empty single value is truthy
                None => false,
            }
        } else {
            true // Multiple values are truthy
        }
    }
}

/// Evaluation result with comprehensive metadata for CLI debugging
#[derive(Debug, Clone)]
pub struct EvaluationResultWithMetadata {
    /// Core evaluation result
    pub result: EvaluationResult,
    /// Metadata collected during evaluation
    pub metadata: crate::evaluator::metadata_collector::EvaluationSummary,
}

impl EvaluationResultWithMetadata {
    /// Create new result with metadata
    pub fn new(
        result: EvaluationResult,
        metadata: crate::evaluator::metadata_collector::EvaluationSummary,
    ) -> Self {
        Self { result, metadata }
    }

    /// Get the core result
    pub fn result(&self) -> &EvaluationResult {
        &self.result
    }

    /// Get the metadata
    pub fn metadata(&self) -> &crate::evaluator::metadata_collector::EvaluationSummary {
        &self.metadata
    }
}

/// Convert TypeInfo to Option<TypeInfoResult>
fn convert_type_info(type_info: &TypeInfo) -> Option<TypeInfoResult> {
    // Extract namespace and name from TypeInfo
    let namespace = type_info
        .namespace
        .as_ref()
        .unwrap_or(&"System".to_string())
        .clone();

    let name = type_info
        .name
        .as_ref()
        .unwrap_or(&type_info.type_name)
        .clone();

    Some(TypeInfoResult { namespace, name })
}

/// Convert a single FhirPathValue to ModelEvaluationResult
fn convert_fhirpath_value_to_eval_result(value: &FhirPathValue) -> ModelEvalResult {
    match value {
        FhirPathValue::Boolean(b, type_info, _) => {
            ModelEvalResult::Boolean(*b, convert_type_info(type_info))
        }
        FhirPathValue::Integer(i, type_info, _) => {
            ModelEvalResult::Integer(*i, convert_type_info(type_info))
        }
        FhirPathValue::Decimal(d, type_info, _) => {
            ModelEvalResult::Decimal(*d, convert_type_info(type_info))
        }
        FhirPathValue::String(s, type_info, _) => {
            ModelEvalResult::String(s.clone(), convert_type_info(type_info))
        }
        FhirPathValue::Date(date, type_info, _) => {
            ModelEvalResult::Date(date.to_string(), convert_type_info(type_info))
        }
        FhirPathValue::DateTime(dt, type_info, _) => {
            ModelEvalResult::DateTime(dt.to_string(), convert_type_info(type_info))
        }
        FhirPathValue::Time(time, type_info, _) => {
            ModelEvalResult::Time(time.to_string(), convert_type_info(type_info))
        }
        FhirPathValue::Quantity {
            value,
            unit,
            type_info,
            ..
        } => {
            let unit_str = unit.as_ref().cloned().unwrap_or_else(|| "1".to_string());
            ModelEvalResult::Quantity(*value, unit_str, convert_type_info(type_info))
        }
        FhirPathValue::Resource(json, type_info, _) => {
            // Convert JSON object to HashMap<String, EvaluationResult>
            let map = if let Some(obj) = json.as_object() {
                obj.iter()
                    .map(|(k, v)| {
                        let eval_result = json_value_to_eval_result(v);
                        (k.clone(), eval_result)
                    })
                    .collect()
            } else {
                HashMap::new()
            };

            ModelEvalResult::Object {
                map,
                type_info: convert_type_info(type_info),
            }
        }
        FhirPathValue::Collection(collection) => {
            // Recursively convert nested collection
            if collection.is_empty() {
                ModelEvalResult::Empty
            } else if collection.len() == 1 {
                if let Some(first) = collection.first() {
                    convert_fhirpath_value_to_eval_result(first)
                } else {
                    ModelEvalResult::Empty
                }
            } else {
                let items: Vec<ModelEvalResult> = collection
                    .iter()
                    .map(convert_fhirpath_value_to_eval_result)
                    .collect();

                ModelEvalResult::Collection {
                    items,
                    has_undefined_order: !collection.is_ordered(),
                    type_info: None,
                }
            }
        }
        FhirPathValue::Empty => ModelEvalResult::Empty,
    }
}

/// Convert a serde_json::Value to ModelEvaluationResult (for Resource objects)
fn json_value_to_eval_result(value: &serde_json::Value) -> ModelEvalResult {
    match value {
        serde_json::Value::Null => ModelEvalResult::Empty,
        serde_json::Value::Bool(b) => ModelEvalResult::boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ModelEvalResult::integer(i)
            } else if let Some(f) = n.as_f64() {
                if let Some(d) = rust_decimal::Decimal::from_f64_retain(f) {
                    ModelEvalResult::decimal(d)
                } else {
                    ModelEvalResult::Empty
                }
            } else {
                ModelEvalResult::Empty
            }
        }
        serde_json::Value::String(s) => ModelEvalResult::string(s.clone()),
        serde_json::Value::Array(arr) => {
            let items: Vec<ModelEvalResult> = arr.iter().map(json_value_to_eval_result).collect();
            ModelEvalResult::collection(items)
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, ModelEvalResult> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_eval_result(v)))
                .collect();
            ModelEvalResult::object(map)
        }
    }
}

/// Convert ModelEvaluationResult back to FhirPathValue (for variable binding)
pub fn eval_result_to_fhirpath_value(
    value: &ModelEvalResult,
    model_provider: Option<std::sync::Arc<dyn crate::core::ModelProvider + Send + Sync>>,
) -> FhirPathValue {
    match value {
        ModelEvalResult::Empty => FhirPathValue::Empty,
        ModelEvalResult::Boolean(b, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::Boolean(*b, ti, None)
        }
        ModelEvalResult::Integer(i, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::Integer(*i, ti, None)
        }
        ModelEvalResult::Integer64(i, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::Integer(*i, ti, None)
        }
        ModelEvalResult::Decimal(d, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::Decimal(*d, ti, None)
        }
        ModelEvalResult::String(s, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::String(s.clone(), ti, None)
        }
        ModelEvalResult::Date(date_str, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            // Parse date string back to PrecisionDate
            use crate::core::temporal::PrecisionDate;
            if let Some(date) = PrecisionDate::parse(date_str) {
                FhirPathValue::Date(date, ti, None)
            } else {
                FhirPathValue::String(date_str.clone(), ti, None)
            }
        }
        ModelEvalResult::DateTime(dt_str, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            // Parse datetime string back to PrecisionDateTime
            use crate::core::temporal::PrecisionDateTime;
            if let Some(dt) = PrecisionDateTime::parse(dt_str) {
                FhirPathValue::DateTime(dt, ti, None)
            } else {
                FhirPathValue::String(dt_str.clone(), ti, None)
            }
        }
        ModelEvalResult::Time(time_str, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            // Parse time string back to PrecisionTime
            use crate::core::temporal::PrecisionTime;
            if let Some(time) = PrecisionTime::parse(time_str) {
                FhirPathValue::Time(time, ti, None)
            } else {
                FhirPathValue::String(time_str.clone(), ti, None)
            }
        }
        ModelEvalResult::Quantity(value, unit, type_info) => {
            let ti = convert_type_info_result_to_type_info(type_info);
            FhirPathValue::Quantity {
                value: *value,
                unit: Some(unit.clone()),
                code: None,
                system: None,
                ucum_unit: None,
                calendar_unit: None,
                type_info: ti,
                primitive_element: None,
            }
        }
        ModelEvalResult::Collection {
            items,
            has_undefined_order,
            ..
        } => {
            let fhir_values: Vec<FhirPathValue> = items
                .iter()
                .map(|item| eval_result_to_fhirpath_value(item, model_provider.clone()))
                .collect();

            let collection =
                Collection::from_values_with_ordering(fhir_values, !has_undefined_order);
            FhirPathValue::Collection(collection)
        }
        ModelEvalResult::Object { map, type_info } => {
            let ti = convert_type_info_result_to_type_info(type_info);
            // Convert HashMap to JsonValue
            let json_map: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), eval_result_to_json_value(v)))
                .collect();
            let json_value = serde_json::Value::Object(json_map);
            FhirPathValue::Resource(std::sync::Arc::new(json_value), ti, None)
        }
    }
}

/// Convert TypeInfoResult to TypeInfo
fn convert_type_info_result_to_type_info(type_info_result: &Option<TypeInfoResult>) -> TypeInfo {
    if let Some(tir) = type_info_result {
        TypeInfo {
            type_name: tir.name.clone(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some(tir.namespace.clone()),
            name: Some(tir.name.clone()),
        }
    } else {
        TypeInfo {
            type_name: "Any".to_string(),
            singleton: Some(true),
            is_empty: Some(false),
            namespace: Some("System".to_string()),
            name: Some("Any".to_string()),
        }
    }
}

/// Convert ModelEvaluationResult to serde_json::Value (helper for Object conversion)
fn eval_result_to_json_value(value: &ModelEvalResult) -> serde_json::Value {
    match value {
        ModelEvalResult::Empty => serde_json::Value::Null,
        ModelEvalResult::Boolean(b, _) => serde_json::Value::Bool(*b),
        ModelEvalResult::Integer(i, _) => serde_json::json!(i),
        ModelEvalResult::Integer64(i, _) => serde_json::json!(i),
        ModelEvalResult::Decimal(d, _) => serde_json::json!(d.to_string()),
        ModelEvalResult::String(s, _) => serde_json::Value::String(s.clone()),
        ModelEvalResult::Date(d, _) => serde_json::Value::String(d.clone()),
        ModelEvalResult::DateTime(dt, _) => serde_json::Value::String(dt.clone()),
        ModelEvalResult::Time(t, _) => serde_json::Value::String(t.clone()),
        ModelEvalResult::Quantity(val, unit, _) => {
            serde_json::json!({
                "value": val,
                "unit": unit
            })
        }
        ModelEvalResult::Collection { items, .. } => {
            let arr: Vec<serde_json::Value> = items.iter().map(eval_result_to_json_value).collect();
            serde_json::Value::Array(arr)
        }
        ModelEvalResult::Object { map, .. } => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), eval_result_to_json_value(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}
