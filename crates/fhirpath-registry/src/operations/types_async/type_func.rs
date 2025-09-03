//! Type function - async implementation for FunctionRegistry

use crate::signature::{CardinalityRequirement, FunctionCategory, FunctionSignature, ValueType};
use crate::traits::{AsyncOperation, EvaluationContext};
use async_trait::async_trait;
use octofhir_fhirpath_core::{FhirPathError, FhirPathValue, Result};
use octofhir_fhir_model::ModelProvider;

/// Type function - returns type information for values
#[derive(Debug, Default, Clone)]
pub struct TypeFunction;

impl TypeFunction {
    pub fn new() -> Self {
        Self
    }

    /// Get type info for a FhirPathValue using ModelProvider exclusively
    async fn get_type_info(
        &self,
        value: &FhirPathValue,
        model_provider: &dyn ModelProvider,
    ) -> FhirPathValue {
        // Use basic type detection for now
        let raw_type_name = value.type_name();

        // Determine namespace and type name based on the context
        let (namespace, type_name) = match value {
            FhirPathValue::JsonValue(_) | FhirPathValue::Resource(_) => {
                // For FHIR data (from JSON/Resource), check if it's a FHIR primitive
                if self.is_fhir_primitive_type(&raw_type_name) {
                    // FHIR primitives keep their lowercase names and FHIR namespace
                    ("FHIR", raw_type_name.clone())
                } else if model_provider.resource_type_exists(&raw_type_name).unwrap_or(false) {
                    // FHIR resource types
                    ("FHIR", raw_type_name.clone())
                } else {
                    // Other types get normalized and System namespace
                    ("System", self.normalize_type_name(&raw_type_name))
                }
            }
            _ => {
                // For system types (created directly), use System namespace with normalized names
                ("System", self.normalize_type_name(&raw_type_name))
            }
        };

        FhirPathValue::TypeInfoObject {
            namespace: namespace.into(),
            name: type_name.into(),
        }
    }

    /// Normalize type names according to FHIRPath conventions
    fn normalize_type_name(&self, type_name: &str) -> String {
        match type_name {
            // System primitives should be title case
            "string" => "String".to_string(),
            "boolean" => "Boolean".to_string(),
            "integer" => "Integer".to_string(),
            "decimal" => "Decimal".to_string(),
            "dateTime" => "DateTime".to_string(),
            "date" => "Date".to_string(),
            "time" => "Time".to_string(),
            "quantity" => "Quantity".to_string(),
            // For other types, keep as is (FHIR types are usually already title case)
            _ => type_name.to_string(),
        }
    }

    /// Check if a type is a FHIR primitive type
    fn is_fhir_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "boolean"
                | "integer"
                | "string"
                | "decimal"
                | "uri"
                | "url"
                | "canonical"
                | "base64Binary"
                | "instant"
                | "date"
                | "dateTime"
                | "time"
                | "code"
                | "oid"
                | "id"
                | "markdown"
                | "unsignedInt"
                | "positiveInt"
                | "uuid"
                | "xhtml"
        )
    }
}

#[async_trait]
impl AsyncOperation for TypeFunction {
    fn name(&self) -> &'static str {
        "type"
    }

    fn signature(&self) -> &FunctionSignature {
        static SIGNATURE: std::sync::LazyLock<FunctionSignature> =
            std::sync::LazyLock::new(|| FunctionSignature {
                name: "type",
                parameters: vec![], // No parameters - works on current context
                return_type: ValueType::Collection,
                variadic: false,
                category: FunctionCategory::Universal,
                cardinality_requirement: CardinalityRequirement::AcceptsBoth,
            });
        &SIGNATURE
    }

    async fn execute(
        &self,
        args: &[FhirPathValue],
        context: &EvaluationContext,
    ) -> Result<FhirPathValue> {
        // type() takes no arguments
        if !args.is_empty() {
            return Err(FhirPathError::InvalidArgumentCount {
                function_name: "type".to_string(),
                expected: 0,
                actual: args.len(),
            });
        }

        match &context.input {
            FhirPathValue::Collection(col) => {
                let mut type_infos = Vec::new();

                for item in col.iter() {
                    let type_info = self
                        .get_type_info(item, context.model_provider.as_ref())
                        .await;
                    type_infos.push(type_info);
                }

                Ok(FhirPathValue::Collection(type_infos))
            }
            FhirPathValue::Empty => {
                // Empty input returns empty collection
                Ok(FhirPathValue::Collection(vec![]))
            }
            _ => {
                // Single item - return its type
                let type_info = self
                    .get_type_info(&context.input, context.model_provider.as_ref())
                    .await;
                Ok(FhirPathValue::Collection(vec![type_info]))
            }
        }
    }
}
