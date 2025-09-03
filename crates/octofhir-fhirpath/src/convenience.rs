//! Convenience functions for common FHIRPath operations

use crate::{
    FhirPath, FhirPathConfigBuilder, FhirPathError, FhirPathValue,
    config::{FhirPathEvaluationResult, FhirVersion},
};

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
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Vec<FhirPathValue>, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    let result = fhirpath.evaluate(expression, context.clone()).await?;
    Ok(result.values)
}

/// Evaluate FHIRPath with specific FHIR version
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
/// * `context` - The JSON resource to evaluate against
/// * `version` - FHIR version to use
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
///         "name": [{"given": ["John"], "family": "Doe"}]
///     });
///
///     let names = evaluate_fhirpath_with_version(
///         "Patient.name.given",
///         &patient,
///         FhirVersion::R5
///     ).await?;
///     println!("Names: {:?}", names);
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath_with_version(
    expression: &str,
    context: &serde_json::Value,
    version: FhirVersion,
) -> Result<Vec<FhirPathValue>, FhirPathError> {
    let config = FhirPathConfigBuilder::new()
        .with_fhir_version(version)
        .build();
    let fhirpath = FhirPath::with_config(config).await?;
    let result = fhirpath.evaluate(expression, context.clone()).await?;
    Ok(result.values)
}

/// Evaluate FHIRPath expression with detailed analysis
///
/// Returns comprehensive evaluation results including performance metrics,
/// warnings, and other diagnostic information.
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
///         "name": [{"given": ["John"], "family": "Doe"}]
///     });
///
///     let result = evaluate_fhirpath_with_analysis("Patient.name.given", &patient).await?;
///     println!("Values: {:?}", result.values);
///     println!("Execution time: {:?}", result.execution_time);
///     if !result.warnings.is_empty() {
///         println!("Warnings: {:?}", result.warnings);
///     }
///     Ok(())
/// }
/// ```
pub async fn evaluate_fhirpath_with_analysis(
    expression: &str,
    context: &serde_json::Value,
) -> Result<FhirPathEvaluationResult, FhirPathError> {
    let fhirpath = FhirPath::new().await?;
    fhirpath.evaluate(expression, context.clone()).await
}

/// Parse a FHIRPath expression and return the AST
///
/// This function only parses the expression without evaluating it,
/// useful for syntax validation and analysis.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to parse
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::parse_expression;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let ast = parse_expression("Patient.name.given")?;
///     println!("AST: {:?}", ast);
///     Ok(())
/// }
/// ```
pub fn parse_expression(expression: &str) -> Result<crate::ast::ExpressionNode, FhirPathError> {
    crate::parse(expression)
}

/// Simple validation of FHIRPath expression syntax
///
/// Returns true if the expression can be parsed successfully, false otherwise.
/// This only checks syntax, not semantic validity.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to validate
/// * `_resource_type` - Optional resource type for context (currently unused)
///
/// # Example
/// ```no_run
/// use octofhir_fhirpath::validate_fhirpath;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Valid expression
///     if validate_fhirpath("Patient.name.given", Some("Patient")).await? {
///         println!("Expression is valid");
///     }
///
///     // Invalid expression
///     if !validate_fhirpath("Patient.invalidSyntax(", Some("Patient")).await? {
///         println!("Invalid syntax");
///     }
///     Ok(())
/// }
/// ```
pub async fn validate_fhirpath(
    expression: &str,
    _resource_type: Option<&str>,
) -> Result<bool, FhirPathError> {
    // Check syntax by parsing
    match parse_expression(expression) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
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
///         "name": [{"given": ["John"], "family": "Doe"}]
///     });
///
///     if path_exists("Patient.name", &patient).await? {
///         println!("Patient has name");
///     }
///
///     if !path_exists("Patient.photo", &patient).await? {
///         println!("Patient has no photo");
///     }
///     Ok(())
/// }
/// ```
pub async fn path_exists(path: &str, context: &serde_json::Value) -> Result<bool, FhirPathError> {
    let values = evaluate_fhirpath(path, context).await?;
    Ok(!values.is_empty())
}

/// Get the first string value from a FHIRPath evaluation
///
/// Returns the first string result, or None if no string values found.
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
///     if let Some(family) = get_string_value("Patient.name.family", &patient).await? {
///         println!("Family name: {}", family);
///     }
///     Ok(())
/// }
/// ```
pub async fn get_string_value(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Option<String>, FhirPathError> {
    let values = evaluate_fhirpath(expression, context).await?;
    for value in values {
        if let FhirPathValue::String(s) = value {
            return Ok(Some(s));
        }
    }
    Ok(None)
}

/// Get all string values from a FHIRPath evaluation
///
/// Returns all string results as a vector.
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
///         "name": [{"given": ["John", "James"]}]
///     });
///
///     let given_names = get_all_string_values("Patient.name.given", &patient).await?;
///     println!("Given names: {:?}", given_names);
///     Ok(())
/// }
/// ```
pub async fn get_all_string_values(
    expression: &str,
    context: &serde_json::Value,
) -> Result<Vec<String>, FhirPathError> {
    let values = evaluate_fhirpath(expression, context).await?;
    let mut strings = Vec::new();
    for value in values {
        if let FhirPathValue::String(s) = value {
            strings.push(s);
        }
    }
    Ok(strings)
}

/// Evaluate a FHIRPath expression and return only boolean result
///
/// Returns true if the expression evaluates to true, false otherwise.
/// Useful for conditional expressions.
///
/// # Arguments
/// * `expression` - The FHIRPath expression to evaluate
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
///         "active": true
///     });
///
///     if evaluate_boolean("Patient.active", &patient).await? {
///         println!("Patient is active");
///     }
///     Ok(())
/// }
/// ```
pub async fn evaluate_boolean(
    expression: &str,
    context: &serde_json::Value,
) -> Result<bool, FhirPathError> {
    let values = evaluate_fhirpath(expression, context).await?;

    // Return true if we have any non-empty, non-false values
    for value in values {
        match value {
            FhirPathValue::Boolean(b) => return Ok(b),
            FhirPathValue::String(s) if !s.is_empty() => return Ok(true),
            FhirPathValue::Integer(i) if i != 0 => return Ok(true),
            FhirPathValue::Decimal(d) if d != rust_decimal::Decimal::ZERO => return Ok(true),
            FhirPathValue::JsonValue(_) => return Ok(true),
            _ => continue,
        }
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_evaluate_fhirpath_basic() {
        let patient = json!({
            "resourceType": "Patient",
            "name": [{"given": ["John"], "family": "Doe"}]
        });

        let result = evaluate_fhirpath("Patient.name.given", &patient)
            .await
            .unwrap();
        assert!(!result.is_empty());
    }

    #[tokio::test]
    async fn test_validate_fhirpath() {
        // Valid syntax
        assert!(validate_fhirpath("Patient.name", None).await.unwrap());

        // Invalid syntax
        assert!(!validate_fhirpath("Patient.name(", None).await.unwrap());
    }

    #[tokio::test]
    async fn test_path_exists() {
        let patient = json!({
            "resourceType": "Patient",
            "name": [{"given": ["John"]}]
        });

        assert!(path_exists("Patient.name", &patient).await.unwrap());
        assert!(!path_exists("Patient.photo", &patient).await.unwrap());
    }

    #[tokio::test]
    async fn test_get_string_value() {
        let patient = json!({
            "resourceType": "Patient",
            "name": [{"family": "Doe"}]
        });

        let family = get_string_value("Patient.name.family", &patient)
            .await
            .unwrap();
        assert_eq!(family, Some("Doe".to_string()));

        let missing = get_string_value("Patient.name.suffix", &patient)
            .await
            .unwrap();
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_evaluate_boolean() {
        let patient = json!({
            "resourceType": "Patient",
            "active": true,
            "inactive": false
        });

        assert!(evaluate_boolean("Patient.active", &patient).await.unwrap());
        assert!(
            !evaluate_boolean("Patient.inactive", &patient)
                .await
                .unwrap()
        );
        assert!(!evaluate_boolean("Patient.missing", &patient).await.unwrap());
    }
}
