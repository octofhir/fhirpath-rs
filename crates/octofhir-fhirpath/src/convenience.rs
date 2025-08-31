//! Convenience functions for common FHIRPath operations

use crate::{
    FhirPath, FhirPathConfigBuilder, FhirPathError, FhirPathValue,
    config::{FhirPathEvaluationResult, FhirVersion},
};
use octofhir_fhirpath_analyzer::BridgeValidationResult;

/// Convenience function for quick FHIRPath evaluation with default configuration
///
/// This is the simplest way to evaluate a FHIRPath expression.
/// Uses R4 FHIR version with basic configuration.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::evaluate_fhirpath;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "name": [{"given": ["John"], "family": "Doe"}]
///     });
///
///     let names = evaluate_fhirpath("Patient.name.given", &patient).await?;
///     println!("Names: {:?}", names);
///     
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Vec<FhirPathValue>, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.evaluate(expression, context).await
}

/// Convenience function for FHIRPath evaluation with specific FHIR version
///
/// Use this when you need to evaluate against a specific FHIR version.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
/// * `fhir_version` - The FHIR version to use (R4, R4B, or R5)
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::{evaluate_fhirpath_with_version, FhirVersion};
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "name": [{"given": ["Alice"], "family": "Smith"}]
///     });
///
///     let names = evaluate_fhirpath_with_version(
///         "Patient.name.family",
///         &patient,
///         FhirVersion::R5
///     ).await?;
///     
///     println!("Family names: {:?}", names);
///     
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath_with_version(
    expression: &str,
    context: &serde_json::Value,
    fhir_version: FhirVersion,
) -> Result<Vec<FhirPathValue>, FhirPathError> {
    let fhirpath = FhirPathConfigBuilder::new()
        .with_fhir_version(fhir_version)
        .create_fhirpath()
        .await?;

    fhirpath.evaluate(expression, context).await
}

/// Convenience function for FHIRPath evaluation with analysis
///
/// Provides comprehensive analysis including validation and performance metrics.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::evaluate_fhirpath_with_analysis;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "telecom": [{"system": "email", "value": "john@example.com"}]
///     });
///
///     let result = evaluate_fhirpath_with_analysis(
///         "Patient.telecom.where(system = 'email').value",
///         &patient
///     ).await?;
///     
///     println!("Values: {:?}", result.values);
///     println!("Execution time: {:?}", result.execution_time);
///     
///     if !result.warnings.is_empty() {
///         println!("Warnings: {:?}", result.warnings);
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath_with_analysis(
    expression: &str,
    context: &serde_json::Value,
) -> Result<FhirPathEvaluationResult, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.evaluate_with_analysis(expression, context).await
}

/// Convenience function for FHIRPath validation
///
/// Validates both syntax and semantics of a FHIRPath expression.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to validate
/// * `resource_type` - Optional resource type for context (defaults to "Resource")
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::validate_fhirpath;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Valid expression
///     match validate_fhirpath("Patient.name.given", Some("Patient")).await? {
///         Some(result) if result.is_valid => println!("Expression is valid"),
///         Some(result) => println!("Invalid: {:?}", result.messages),
///         None => println!("Validation not available"),
///     }
///     
///     // Invalid expression
///     match validate_fhirpath("Patient.invalidField", Some("Patient")).await? {
///         Some(result) if !result.is_valid => {
///             println!("Validation errors: {:?}", result.messages);
///             if !result.suggestions.is_empty() {
///                 println!("Suggestions: {:?}", result.suggestions);
///             }
///         },
///         _ => println!("Unexpected result"),
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn validate_fhirpath(
    expression: &str,
    _resource_type: Option<&str>,
) -> Result<Option<BridgeValidationResult>, FhirPathError> {
    let fhirpath = FhirPath::new().await?;

    // First check syntax by parsing
    if let Err(e) = fhirpath.parse_expression(expression) {
        return Ok(Some(BridgeValidationResult {
            is_valid: false,
            field_path: expression.to_string(),
            resource_type: "Unknown".to_string(),
            suggestions: vec![format!("Syntax error: {}", e)],
            context_info: Some("Expression syntax validation".to_string()),
            optimization_hints: vec![],
            property_info: None,
        }));
    }

    // Then check semantics if analyzer is available
    fhirpath.validate_expression(expression).await
}

/// Convenience function to check if a path exists in a resource
///
/// Returns true if the path exists and has a value, false otherwise.
///
/// # Arguments
/// * `path` - The FHIRPath expression to check
/// * `context` - The JSON resource to check against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::path_exists;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "name": [{"given": ["John"]}],
///         "telecom": []
///     });
///
///     println!("Has name: {}", path_exists("Patient.name", &patient).await?);
///     println!("Has telecom: {}", path_exists("Patient.telecom", &patient).await?);
///     
///     Ok(())
/// }
/// ```
pub async fn path_exists(path: &str, context: &serde_json::Value) -> Result<bool, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.path_exists(path, context).await
}

/// Convenience function to get a single string value from a FHIRPath expression
///
/// Returns the first string value found, or None if no string values exist.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::get_string_value;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "name": [{"family": "Doe"}]
///     });
///
///     if let Some(family_name) = get_string_value("Patient.name.family.first()", &patient).await? {
///         println!("Family name: {}", family_name);
///     } else {
///         println!("No family name found");
///     }
///     
///     Ok(())
/// }
/// ```
pub async fn get_string_value(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Option<String>, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    let results = fhirpath.evaluate_to_string(expression, context).await?;
    Ok(results.into_iter().next())
}

/// Convenience function to get all string values from a FHIRPath expression
///
/// Returns all string values found in the evaluation results.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::get_all_string_values;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "name": [
///             {"given": ["John", "James"]},
///             {"given": ["Johnny"]}
///         ]
///     });
///
///     let given_names = get_all_string_values("Patient.name.given", &patient).await?;
///     println!("All given names: {:?}", given_names);
///     
///     Ok(())
/// }
/// ```
pub async fn get_all_string_values(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Vec<String>, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.evaluate_to_string(expression, context).await
}

/// Convenience function to evaluate a boolean expression
///
/// Useful for where clauses and conditions. Returns false if evaluation fails.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate (should return boolean)
/// * `context` - The JSON resource to evaluate against
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::evaluate_boolean;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let patient = json!({
///         "resourceType": "Patient",
///         "active": true,
///         "name": [{"family": "Doe"}]
///     });
///
///     let is_active = evaluate_boolean("Patient.active", &patient).await?;
///     let has_name = evaluate_boolean("Patient.name.exists()", &patient).await?;
///     
///     println!("Patient active: {}", is_active);
///     println!("Patient has name: {}", has_name);
///     
///     Ok(())
/// }
/// ```
pub async fn evaluate_boolean(
    expression: &str,
    context: &serde_json::Value,
) -> Result<bool, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.evaluate_to_boolean(expression, context).await
}

/// Convenience function to parse and validate expression syntax
///
/// Checks only syntax, not semantic validity. Returns the AST on success.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to parse
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::parse_expression;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Valid expression
///     match parse_expression("Patient.name.given.first()") {
///         Ok(ast) => println!("Valid syntax: {:?}", ast),
///         Err(e) => println!("Syntax error: {}", e),
///     }
///     
///     // Invalid expression  
///     match parse_expression("Patient.name.") {
///         Ok(_) => println!("Unexpectedly valid"),
///         Err(e) => println!("Expected syntax error: {}", e),
///     }
///     
///     Ok(())
/// }
/// ```
pub fn parse_expression(expression: &str) -> Result<crate::ExpressionNode, FhirPathError> {
    crate::parse(expression).map_err(|e| FhirPathError::Generic {
        message: format!("Parse error: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_expression() {
        let result = parse_expression("Patient.name.given");
        assert!(result.is_ok());

        let invalid_result = parse_expression("Patient.name.");
        assert!(invalid_result.is_err());
    }

    // Note: Other tests would require full integration setup
    // In a real implementation, these would use proper mocks

    #[tokio::test]
    async fn test_convenience_functions_compilation() {
        // These tests just ensure the functions compile correctly
        // Real functionality tests would require proper mock setup

        let patient = json!({
            "resourceType": "Patient",
            "name": [{"given": ["John"]}]
        });

        // This would fail at runtime without proper setup, but should compile
        let _ = evaluate_fhirpath("Patient.name", &patient).await;
    }
}
