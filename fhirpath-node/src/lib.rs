// FHIRPath Node.js Bindings
//
// This crate provides Node.js bindings for the FHIRPath engine.

#[macro_use]
extern crate napi_derive;

use napi::{Error, Result};

#[napi]
#[derive(Default)]
pub struct FhirPathEngine {
    // Internally state if needed
}

#[napi]
impl FhirPathEngine {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates an FHIRPath expression against a FHIR resource (synchronous)
    #[napi]
    pub fn evaluate(&self, expression: String, resource: String) -> Result<String> {
        // Parse the resource as JSON
        let resource_json = match serde_json::from_str::<serde_json::Value>(&resource) {
            Ok(json) => json,
            Err(err) => {
                return Err(Error::from_reason(format!(
                    "Failed to parse resource as JSON: {}",
                    err
                )));
            }
        };

        // Evaluate the expression using the core FHIRPath engine
        let result = match fhirpath_core::evaluate(&expression, resource_json) {
            Ok(value) => serde_json::to_string(&value).map_err(|err| {
                Error::from_reason(format!("Failed to serialize result: {}", err))
            })?,
            Err(err) => {
                return Err(Error::from_reason(format!(
                    "FHIRPath evaluation error: {}",
                    err
                )));
            }
        };

        Ok(result)
    }

    /// Evaluates an FHIRPath expression against a FHIR resource (asynchronous)
    /// Uses a thread pool for CPU-bound operations to avoid blocking the event loop
    #[napi]
    pub async fn evaluate_async(&self, expression: String, resource: String) -> Result<String> {
        // Use tokio::task::spawn_blocking to run CPU-bound work in a thread pool
        let result = tokio::task::spawn_blocking(move || {
            // Parse the resource as JSON
            let resource_json =
                serde_json::from_str::<serde_json::Value>(&resource).map_err(|err| {
                    Error::from_reason(format!("Failed to parse resource as JSON: {}", err))
                })?;

            // Evaluate the expression using the core FHIRPath engine
            let result = fhirpath_core::evaluate(&expression, resource_json)
                .map_err(|err| Error::from_reason(format!("FHIRPath evaluation error: {}", err)))?;

            serde_json::to_string(&result)
                .map_err(|err| Error::from_reason(format!("Failed to serialize result: {}", err)))
        })
        .await
        .map_err(|err| Error::from_reason(format!("Task execution error: {}", err)))??;

        Ok(result)
    }

    /// Validates a FHIRPath expression syntax
    #[napi]
    pub fn validate(&self, expression: String) -> Result<bool> {
        // Tokenize the expression
        let tokens = match fhirpath_core::lexer::tokenize(&expression) {
            Ok(tokens) => tokens,
            Err(_) => {
                // Return false for syntax errors in tokenization
                return Ok(false);
            }
        };

        // Parse the tokens
        match fhirpath_core::parser::parse(&tokens) {
            Ok(_) => Ok(true),   // Parsing succeeded, expression is valid
            Err(_) => Ok(false), // Parsing failed, expression is invalid
        }
    }

    /// Returns the version of the FHIRPath engine
    #[napi]
    pub fn version(&self) -> String {
        format!(
            "FHIRPath Engine v{} (spec: {})",
            env!("CARGO_PKG_VERSION"),
            fhirpath_core::FHIRPATH_SPEC_VERSION
        )
    }
}

#[napi]
pub fn get_engine_info() -> String {
    format!(
        "FHIRPath Rust Engine v{} (Node.js bindings)",
        env!("CARGO_PKG_VERSION")
    )
}

/// Convenience function to check if an FHIRPath expression returns any results
#[napi]
pub fn exists(expression: String, resource: String) -> Result<bool> {
    // Parse the resource as JSON
    let resource_json = serde_json::from_str::<serde_json::Value>(&resource)
        .map_err(|err| Error::from_reason(format!("Failed to parse resource as JSON: {}", err)))?;

    // Evaluate the expression using the core FHIRPath engine
    let result = fhirpath_core::evaluate(&expression, resource_json)
        .map_err(|err| Error::from_reason(format!("FHIRPath evaluation error: {}", err)))?;

    // Check if result is non-empty
    match result {
        serde_json::Value::Array(arr) => Ok(!arr.is_empty()),
        serde_json::Value::Null => Ok(false),
        _ => Ok(true),
    }
}
