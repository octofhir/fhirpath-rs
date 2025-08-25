//! Utility functions and type checking helpers
//!
//! This module contains various utility functions used throughout the FHIRPath
//! evaluation engine, including type checking, value comparison, and validation.

use octofhir_fhirpath_ast::ExpressionNode;
use octofhir_fhirpath_core::{EvaluationError, EvaluationResult};
use octofhir_fhirpath_model::FhirPathValue;

/// Utility functions for the FHIRPath evaluation engine
impl crate::FhirPathEngine {
    /// Ensure the result is wrapped in a collection if it's not already
    /// According to FHIRPath specification, all evaluation results should be collections
    pub fn ensure_collection_result(value: FhirPathValue) -> FhirPathValue {
        match value {
            // Already a collection - return as is
            FhirPathValue::Collection(_) => value,
            // Empty becomes empty collection
            FhirPathValue::Empty => FhirPathValue::Collection(Default::default()),
            // All other values must be wrapped in single-item collections
            other => {
                let mut collection = octofhir_fhirpath_model::Collection::new();
                collection.push(other);
                FhirPathValue::Collection(collection)
            }
        }
    }

    /// Check if a FHIRPath value is truthy according to FHIRPath semantics
    /// - Boolean true is truthy
    /// - Non-empty collections are truthy (single items delegate to their value)
    /// - Empty collections and Empty values are falsy
    /// - All other non-empty values are truthy
    pub fn is_truthy(&self, value: &FhirPathValue) -> bool {
        match value {
            FhirPathValue::Boolean(b) => *b,
            FhirPathValue::Collection(items) => {
                if items.is_empty() {
                    false
                } else if items.len() == 1 {
                    // Single item collection: check the item's truthiness
                    self.is_truthy(items.first().unwrap())
                } else {
                    // Multiple items: collection is truthy
                    true
                }
            }
            FhirPathValue::Empty => false,
            _ => true, // Non-empty values are generally truthy
        }
    }

    /// Convert FHIRPath value to boolean according to strict FHIRPath rules
    /// Used for functions like iif() that require strict boolean conditions
    /// Returns Some(bool) for valid boolean values, None for non-boolean values
    pub fn to_boolean_strict(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Empty => Some(false),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(false)
                } else if c.len() == 1 {
                    self.to_boolean_strict(c.first().unwrap())
                } else {
                    // Multiple items - not a valid boolean
                    None
                }
            }
            _ => None, // Non-boolean values are not valid
        }
    }

    /// Convert a value to boolean using FHIRPath standard boolean conversion rules
    /// Non-empty collections, non-zero numbers, non-empty strings are truthy
    pub fn to_boolean_fhirpath(&self, value: &FhirPathValue) -> Option<bool> {
        match value {
            FhirPathValue::Boolean(b) => Some(*b),
            FhirPathValue::Empty => Some(false),
            FhirPathValue::Collection(c) => {
                if c.is_empty() {
                    Some(false)
                } else if c.len() == 1 {
                    // Single item collection - use the item's boolean value if it's a boolean
                    match c.iter().next().unwrap() {
                        FhirPathValue::Boolean(b) => Some(*b),
                        _ => Some(true), // Non-boolean single items are truthy
                    }
                } else {
                    Some(true) // Multi-item collections are truthy in FHIRPath
                }
            }
            FhirPathValue::Integer(i) => Some(*i != 0),
            FhirPathValue::Decimal(d) => Some(!d.is_zero()),
            FhirPathValue::String(s) => Some(!s.is_empty()),
            _ => Some(true), // Other types (Date, DateTime, etc.) are considered truthy
        }
    }

    /// Validate that a collection size doesn't exceed configured limits
    pub fn validate_collection_size(&self, size: usize) -> EvaluationResult<()> {
        if size > self.config().max_collection_size {
            return Err(EvaluationError::InvalidOperation {
                message: format!(
                    "Collection size {} exceeds maximum allowed size of {}",
                    size,
                    self.config().max_collection_size
                ),
            });
        }
        Ok(())
    }

    /// Compare two FhirPathValue instances for sorting
    pub fn compare_fhir_values(&self, a: &FhirPathValue, b: &FhirPathValue) -> std::cmp::Ordering {
        use rust_decimal::Decimal;
        use std::cmp::Ordering;

        match (a, b) {
            (FhirPathValue::Integer(a), FhirPathValue::Integer(b)) => a.cmp(b),
            (FhirPathValue::String(a), FhirPathValue::String(b)) => a.cmp(b),
            (FhirPathValue::Decimal(a), FhirPathValue::Decimal(b)) => a.cmp(b),
            (FhirPathValue::Boolean(a), FhirPathValue::Boolean(b)) => a.cmp(b),
            (FhirPathValue::Date(a), FhirPathValue::Date(b)) => a.cmp(b),
            (FhirPathValue::DateTime(a), FhirPathValue::DateTime(b)) => a.cmp(b),
            (FhirPathValue::Time(a), FhirPathValue::Time(b)) => a.cmp(b),
            (FhirPathValue::Integer(a), FhirPathValue::Decimal(b)) => Decimal::from(*a).cmp(b),
            (FhirPathValue::Decimal(a), FhirPathValue::Integer(b)) => a.cmp(&Decimal::from(*b)),
            (FhirPathValue::Collection(a), FhirPathValue::Collection(b)) => {
                match (a.len(), b.len()) {
                    (1, 1) => self.compare_fhir_values(a.first().unwrap(), b.first().unwrap()),
                    (0, 0) => Ordering::Equal,
                    (0, _) => Ordering::Less,
                    (_, 0) => Ordering::Greater,
                    _ => Ordering::Equal,
                }
            }
            (FhirPathValue::Collection(a), other) if a.len() == 1 => {
                self.compare_fhir_values(a.first().unwrap(), other)
            }
            (other, FhirPathValue::Collection(b)) if b.len() == 1 => {
                self.compare_fhir_values(other, b.first().unwrap())
            }
            (FhirPathValue::Empty, FhirPathValue::Empty) => Ordering::Equal,
            (FhirPathValue::Empty, _) => Ordering::Less,
            (_, FhirPathValue::Empty) => Ordering::Greater,
            _ => self.type_precedence(a).cmp(&self.type_precedence(b)),
        }
    }

    /// Define type precedence for mixed-type sorting
    pub fn type_precedence(&self, value: &FhirPathValue) -> u8 {
        match value {
            FhirPathValue::Empty => 0,
            FhirPathValue::Boolean(_) => 1,
            FhirPathValue::Integer(_) => 2,
            FhirPathValue::Decimal(_) => 3,
            FhirPathValue::String(_) => 4,
            FhirPathValue::Date(_) => 5,
            FhirPathValue::DateTime(_) => 6,
            FhirPathValue::Time(_) => 7,
            FhirPathValue::Collection(_) => 8,
            _ => 9,
        }
    }

    /// Extract sort intent from expression AST - detect descending sort (unary minus)
    pub fn extract_sort_intent<'a>(
        &self,
        expression: &'a ExpressionNode,
    ) -> (&'a ExpressionNode, bool) {
        use octofhir_fhirpath_ast::UnaryOperator;

        match expression {
            ExpressionNode::UnaryOp {
                op: UnaryOperator::Minus,
                operand,
            } => (operand.as_ref(), true),
            _ => (expression, false),
        }
    }

    /// Generate a unique key for an item to detect duplicates in repeat()
    pub fn item_to_key(&self, item: &FhirPathValue) -> String {
        match item {
            FhirPathValue::String(s) => format!("string:{s}"),
            FhirPathValue::Integer(i) => format!("integer:{i}"),
            FhirPathValue::Decimal(d) => format!("decimal:{d}"),
            FhirPathValue::Boolean(b) => format!("boolean:{b}"),
            FhirPathValue::JsonValue(json_val) => {
                // For JSON objects, use id if available, otherwise use a hash-like approach
                if json_val.is_object() {
                    if let Some(id_val) = json_val.get_property("id") {
                        if let Some(id) = id_val.as_str() {
                            format!("object:id:{id}")
                        } else {
                            format!("object:hash:{}", json_val.to_string().unwrap_or_default())
                        }
                    } else {
                        format!("object:hash:{}", json_val.to_string().unwrap_or_default())
                    }
                } else {
                    format!("json:{json_val:?}")
                }
            }
            _ => format!("{item:?}"),
        }
    }

    /// Check if an identifier is a type identifier (starts with uppercase or known primitive type)
    pub fn is_type_identifier(&self, identifier: &str) -> bool {
        // Handle namespaced types
        if identifier.contains('.') {
            let parts: Vec<&str> = identifier.split('.').collect();
            if parts.len() == 2 {
                let (namespace, type_name) = (parts[0], parts[1]);
                match namespace {
                    "System" => matches!(
                        type_name,
                        "Boolean"
                            | "Integer"
                            | "Decimal"
                            | "String"
                            | "Date"
                            | "DateTime"
                            | "Time"
                            | "Quantity"
                            | "Collection"
                    ),
                    "FHIR" => {
                        // Common FHIR resource types and primitive types
                        matches!(
                            type_name,
                            "Patient"
                                | "Observation"
                                | "Practitioner"
                                | "Organization"
                                | "Encounter"
                                | "Condition"
                                | "Procedure"
                                | "DiagnosticReport"
                                | "Medication"
                                | "MedicationStatement"
                                | "AllergyIntolerance"
                                | "Bundle"
                                | "CapabilityStatement"
                                | "ValueSet"
                                | "CodeSystem"
                                | "StructureDefinition"
                                | "OperationDefinition"
                                | "SearchParameter"
                                | "Resource"
                                | "DomainResource"
                                | "MetadataResource"
                                | "boolean"
                                | "integer"
                                | "decimal"
                                | "string"
                                | "date"
                                | "dateTime"
                                | "time"
                                | "uri"
                                | "url"
                                | "canonical"
                                | "code"
                                | "id"
                                | "markdown"
                                | "base64Binary"
                                | "instant"
                                | "oid"
                                | "positiveInt"
                                | "unsignedInt"
                                | "uuid"
                                | "xhtml"
                        )
                    }
                    _ => false,
                }
            } else {
                false
            }
        } else {
            // Unqualified type names
            identifier.chars().next().is_some_and(|c| c.is_uppercase())
                || matches!(
                    identifier,
                    "boolean"
                        | "integer"
                        | "decimal"
                        | "string"
                        | "date"
                        | "dateTime"
                        | "time"
                        | "uri"
                        | "url"
                        | "canonical"
                        | "code"
                        | "id"
                        | "markdown"
                        | "base64Binary"
                        | "instant"
                        | "oid"
                        | "positiveInt"
                        | "unsignedInt"
                        | "uuid"
                        | "xhtml"
                        | "Boolean"
                        | "Integer"
                        | "Decimal"
                        | "String"
                        | "Date"
                        | "DateTime"
                        | "Time"
                        | "Quantity"
                        | "Collection"
                        | "System"
                        | "FHIR"
                )
        }
    }

    /// Check if an expression is a type identifier expression
    pub fn is_type_identifier_expression(expr: &ExpressionNode) -> bool {
        match expr {
            ExpressionNode::Identifier(name) => {
                // Check if this identifier looks like a type name
                // Type names typically start with uppercase letter
                name.chars().next().is_some_and(|c| c.is_uppercase()) ||
                // Or are known primitive type names
                matches!(name.as_str(), "boolean" | "integer" | "decimal" | "string" | "date" | "datetime" | "time" | "uri" | "url" | "canonical" | "code" | "id" | "markdown" | "base64Binary" | "instant" | "oid" | "positiveInt" | "unsignedInt" | "uuid" | "xhtml" | "collection" | "empty" | "quantity")
            }
            ExpressionNode::Path { base, path } => {
                // Handle qualified type names like FHIR.uuid, System.Boolean
                if let ExpressionNode::Identifier(namespace) = base.as_ref() {
                    // Check for known type namespaces and valid type names
                    matches!(namespace.as_str(), "FHIR" | "System")
                        && (path.chars().next().is_some_and(|c| c.is_uppercase())
                            || matches!(
                                path.as_str(),
                                "boolean"
                                    | "integer"
                                    | "decimal"
                                    | "string"
                                    | "date"
                                    | "datetime"
                                    | "time"
                                    | "uri"
                                    | "url"
                                    | "canonical"
                                    | "code"
                                    | "id"
                                    | "markdown"
                                    | "base64Binary"
                                    | "instant"
                                    | "oid"
                                    | "positiveInt"
                                    | "unsignedInt"
                                    | "uuid"
                                    | "xhtml"
                                    | "collection"
                                    | "empty"
                                    | "quantity"
                            ))
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
