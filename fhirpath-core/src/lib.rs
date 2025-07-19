// FHIRPath Core Implementation
//
// This crate provides the core functionality for parsing and evaluating FHIRPath expressions.

pub mod errors;
pub mod evaluator;
pub mod lexer;
pub mod model;
pub mod parser;

#[cfg(test)]
pub mod debug_tokens;

/// Version of the FHIRPath specification implemented
pub const FHIRPATH_SPEC_VERSION: &str = "N1";

// Re-export visitor types for public use
pub use evaluator::{AstVisitor, LoggingVisitor, NoopVisitor};

/// Evaluates a FHIRPath expression against a FHIR resource
///
/// This function evaluates a FHIRPath expression against a FHIR resource and returns the result.
pub fn evaluate(
    expression: &str,
    resource: serde_json::Value,
) -> Result<serde_json::Value, errors::FhirPathError> {
    evaluate_with_visitor(expression, resource, &NoopVisitor::new())
}

/// Evaluates a FHIRPath expression against a FHIR resource with a custom visitor
///
/// This function evaluates a FHIRPath expression against a FHIR resource and returns the result.
/// It allows providing a custom visitor for debugging or tracing the evaluation process.
pub fn evaluate_with_visitor(
    expression: &str,
    resource: serde_json::Value,
    visitor: &dyn AstVisitor,
) -> Result<serde_json::Value, errors::FhirPathError> {
    // Use the evaluator to evaluate the expression with the provided visitor
    let result = evaluator::evaluate_expression_with_visitor(expression, resource, visitor)?;

    // Convert the FhirPathValue to a serde_json::Value
    match result {
        model::FhirPathValue::Empty => Ok(serde_json::Value::Null),
        model::FhirPathValue::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        model::FhirPathValue::Integer(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i)))
        }
        model::FhirPathValue::Decimal(d) => {
            if let Some(n) = serde_json::Number::from_f64(d) {
                Ok(serde_json::Value::Number(n))
            } else {
                Err(errors::FhirPathError::TypeError(format!(
                    "Cannot convert {} to JSON number",
                    d
                )))
            }
        }
        model::FhirPathValue::String(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Date(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::DateTime(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Time(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Quantity { value, unit } => {
            let mut map = serde_json::Map::new();
            if let Some(n) = serde_json::Number::from_f64(value) {
                map.insert("value".to_string(), serde_json::Value::Number(n));
            } else {
                return Err(errors::FhirPathError::TypeError(format!(
                    "Cannot convert {} to JSON number",
                    value
                )));
            }
            map.insert("unit".to_string(), serde_json::Value::String(unit));
            Ok(serde_json::Value::Object(map))
        }
        model::FhirPathValue::Collection(items) => {
            let mut array = Vec::new();
            for item in items {
                let json_value = evaluate_internal_value(item)?;
                array.push(json_value);
            }
            Ok(serde_json::Value::Array(array))
        }
        model::FhirPathValue::Resource(resource) => Ok(resource.to_json()),
    }
}

/// Helper function to convert a FhirPathValue to a serde_json::Value
fn evaluate_internal_value(
    value: model::FhirPathValue,
) -> Result<serde_json::Value, errors::FhirPathError> {
    match value {
        model::FhirPathValue::Empty => Ok(serde_json::Value::Null),
        model::FhirPathValue::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        model::FhirPathValue::Integer(i) => {
            Ok(serde_json::Value::Number(serde_json::Number::from(i)))
        }
        model::FhirPathValue::Decimal(d) => {
            if let Some(n) = serde_json::Number::from_f64(d) {
                Ok(serde_json::Value::Number(n))
            } else {
                Err(errors::FhirPathError::TypeError(format!(
                    "Cannot convert {} to JSON number",
                    d
                )))
            }
        }
        model::FhirPathValue::String(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Date(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::DateTime(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Time(s) => Ok(serde_json::Value::String(s)),
        model::FhirPathValue::Quantity { value, unit } => {
            let mut map = serde_json::Map::new();
            if let Some(n) = serde_json::Number::from_f64(value) {
                map.insert("value".to_string(), serde_json::Value::Number(n));
            } else {
                return Err(errors::FhirPathError::TypeError(format!(
                    "Cannot convert {} to JSON number",
                    value
                )));
            }
            map.insert("unit".to_string(), serde_json::Value::String(unit));
            Ok(serde_json::Value::Object(map))
        }
        model::FhirPathValue::Collection(items) => {
            let mut array = Vec::new();
            for item in items {
                let json_value = evaluate_internal_value(item)?;
                array.push(json_value);
            }
            Ok(serde_json::Value::Array(array))
        }
        model::FhirPathValue::Resource(resource) => Ok(resource.to_json()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
