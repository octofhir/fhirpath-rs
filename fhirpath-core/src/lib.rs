// FHIRPath Core Implementation
//
// This crate provides the core functionality for parsing and evaluating FHIRPath expressions.

pub mod errors;
pub mod evaluator;
pub mod fhir_schema;
pub mod lexer;
pub mod model;
// pub mod optimized_model;
// pub mod optimized_parser;
pub mod parser;


/// Version of the FHIRPath specification implemented
pub const FHIRPATH_SPEC_VERSION: &str = "v2.0.0";

// Re-export visitor types for public use
pub use evaluator::{AstVisitor, LoggingVisitor, NoopVisitor};

// Re-export schema validator for public use
pub use fhir_schema::FhirSchemaValidator;

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
    convert_fhirpath_value_to_json(result)
}

/// Evaluates a FHIRPath expression against a FHIR resource with context validation
///
/// This function validates that the expression is appropriate for the given resource type
/// before evaluation, providing better error messages for invalid expressions.
pub fn evaluate_with_context_validation(
    expression: &str,
    resource: serde_json::Value,
) -> Result<serde_json::Value, errors::FhirPathError> {
    evaluate_with_context_validation_and_visitor(expression, resource, &NoopVisitor::new())
}

/// Evaluates a FHIRPath expression against a FHIR resource with context validation and a custom visitor
///
/// This function validates that the expression is appropriate for the given resource type
/// before evaluation and allows providing a custom visitor for debugging.
pub fn evaluate_with_context_validation_and_visitor(
    expression: &str,
    resource: serde_json::Value,
    visitor: &dyn AstVisitor,
) -> Result<serde_json::Value, errors::FhirPathError> {
    // Extract resource type from the resource
    let resource_type = resource
        .get("resourceType")
        .and_then(|rt| rt.as_str())
        .ok_or_else(|| errors::FhirPathError::ResourceTypeError {
            resource_type: "Unknown".to_string(),
            reason: "Resource must have a 'resourceType' field".to_string(),
        })?;

    // Create schema validator
    let schema_validator = FhirSchemaValidator::new();

    // Validate the expression against the resource schema
    validate_expression_context(expression, resource_type, &schema_validator)?;

    // If validation passes, evaluate the expression
    let result = evaluator::evaluate_expression_with_visitor(expression, resource, visitor)?;

    // Convert the FhirPathValue to a serde_json::Value
    convert_fhirpath_value_to_json(result)
}

/// Helper function to convert FhirPathValue to JSON
fn convert_fhirpath_value_to_json(result: model::FhirPathValue) -> Result<serde_json::Value, errors::FhirPathError> {
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

/// Validates a FHIRPath expression against a resource schema
fn validate_expression_context(
    expression: &str,
    resource_type: &str,
    schema_validator: &FhirSchemaValidator,
) -> Result<(), errors::FhirPathError> {
    // Parse the expression to extract property paths
    let tokens = lexer::tokenize(expression)?;
    let ast = parser::parse(&tokens)?;

    // Extract property paths from the AST and validate them
    validate_ast_paths(&ast, resource_type, schema_validator)
}

/// Recursively validates property paths in an AST node
fn validate_ast_paths(
    node: &parser::AstNode,
    resource_type: &str,
    schema_validator: &FhirSchemaValidator,
) -> Result<(), errors::FhirPathError> {
    use parser::AstNode;

    match node {
        AstNode::Identifier(name) => {
            // Validate simple property access
            schema_validator.validate_property_path(resource_type, name)?;
        }
        AstNode::Path(object, member) => {
            // Build the full path and validate it
            let path = build_path_from_ast(node);
            schema_validator.validate_property_path(resource_type, &path)?;
        }
        AstNode::FunctionCall { name: _, arguments } => {
            // Validate arguments recursively
            for arg in arguments {
                validate_ast_paths(arg, resource_type, schema_validator)?;
            }
        }
        AstNode::BinaryOp { left, right, .. } => {
            validate_ast_paths(left, resource_type, schema_validator)?;
            validate_ast_paths(right, resource_type, schema_validator)?;
        }
        AstNode::UnaryOp { operand, .. } => {
            validate_ast_paths(operand, resource_type, schema_validator)?;
        }
        AstNode::Indexer { collection, index } => {
            validate_ast_paths(collection, resource_type, schema_validator)?;
            validate_ast_paths(index, resource_type, schema_validator)?;
        }
        // For other node types, we don't need to validate paths
        _ => {}
    }

    Ok(())
}

/// Build a property path from AST nodes
fn build_path_from_ast(node: &parser::AstNode) -> String {
    use parser::AstNode;

    match node {
        AstNode::Identifier(name) => name.clone(),
        AstNode::Path(object, member) => {
            let object_path = build_path_from_ast(object);
            let member_path = build_path_from_ast(member);
            if object_path.is_empty() {
                member_path
            } else {
                format!("{}.{}", object_path, member_path)
            }
        }
        _ => String::new(), // Fallback for complex expressions
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
mod ucum_test;

#[cfg(test)]
mod context_validation_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_validation() {
        println!("Testing FHIRPath Context Validation");

        // Create a Patient resource
        let patient = json!({
            "resourceType": "Patient",
            "id": "example",
            "name": [{
                "use": "official",
                "family": "Smith",
                "given": ["John", "Jacob"]
            }],
            "gender": "male",
            "birthDate": "1974-12-25"
        });

        // Test cases
        let test_cases = vec![
            // Valid expressions for Patient
            ("name.given", true, "Valid Patient property"),
            ("gender", true, "Valid Patient property"),
            ("birthDate", true, "Valid Patient property"),
            ("id", true, "Valid base resource property"),

            // Invalid expressions - properties from other resource types
            ("status", false, "Encounter property on Patient"),
            ("code", false, "Observation property on Patient"),
            ("subject", false, "Reference property not in Patient"),
            ("performer", false, "Procedure property on Patient"),
        ];

        let mut passed = 0;
        let mut failed = 0;

        for (expression, should_succeed, description) in test_cases {
            println!("Testing: {} - {} ... ", expression, description);

            match evaluate_with_context_validation(expression, patient.clone()) {
                Ok(_) => {
                    if should_succeed {
                        println!("✓ PASS (validation allowed as expected)");
                        passed += 1;
                    } else {
                        println!("✗ FAIL (validation should have rejected this)");
                        failed += 1;
                    }
                }
                Err(errors::FhirPathError::InvalidContextPath { path, resource_type, .. }) => {
                    if !should_succeed {
                        println!("✓ PASS (validation correctly rejected: {} for {})", path, resource_type);
                        passed += 1;
                    } else {
                        println!("✗ FAIL (validation incorrectly rejected: {} for {})", path, resource_type);
                        failed += 1;
                    }
                }
                Err(e) => {
                    println!("✗ FAIL (unexpected error: {:?})", e);
                    failed += 1;
                }
            }
        }

        println!("\nContext Validation Test Results:");
        println!("Passed: {}", passed);
        println!("Failed: {}", failed);
        println!("Total: {}", passed + failed);

        if failed == 0 {
            println!("🎉 All context validation tests passed!");
        } else {
            println!("❌ Some context validation tests failed.");
        }

        // Assert that all tests passed
        assert_eq!(failed, 0, "Some context validation tests failed");
    }
}
