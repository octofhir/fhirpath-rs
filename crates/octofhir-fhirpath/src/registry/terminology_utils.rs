//! Utilities and lightweight types for terminology-oriented functions

use rust_decimal::Decimal;
use serde_json::{Map, Value as JsonValue};
use std::sync::Arc;

use crate::core::error_code::FP0051;
use crate::core::temporal::PrecisionDateTime;
use crate::core::{FhirPathError, FhirPathValue, Result};

/// Minimal Coding representation used by terminology functions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coding {
    pub system: String,
    pub code: String,
    pub version: Option<String>,
    pub display: Option<String>,
}

impl Coding {
    pub fn new(system: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            system: system.into(),
            code: code.into(),
            version: None,
            display: None,
        }
    }

    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Represents a concept translation result from a ConceptMap
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptTranslation {
    pub equivalence: String,
    pub concept: Coding,
    pub comment: Option<String>,
}

/// Represents a concept designation (display name in specific language/use)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptDesignation {
    pub language: Option<String>,
    pub use_coding: Option<Coding>,
    pub value: String,
}

/// Represents a concept property value
#[derive(Debug, Clone, PartialEq)]
pub struct ConceptProperty {
    pub code: String,
    pub value: PropertyValue,
    pub description: Option<String>,
}

/// Different types of property values supported by FHIR terminology
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Decimal(Decimal),
    Coding(Coding),
    DateTime(PrecisionDateTime),
}

pub struct TerminologyUtils;

impl TerminologyUtils {
    /// Extract a Coding from a FhirPathValue. Supports Coding objects, and CodeableConcept (takes first coding).
    pub fn extract_coding(value: &FhirPathValue) -> Result<Coding> {
        match value {
            FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                Self::extract_coding_from_json(j)
            }
            _ => Err(FhirPathError::evaluation_error(
                FP0051,
                "Expected Coding or CodeableConcept value".to_string(),
            )),
        }
    }

    pub fn extract_coding_from_json(j: &JsonValue) -> Result<Coding> {
        if let Some(obj) = j.as_object() {
            // Direct Coding
            if let (Some(system), Some(code)) = (
                obj.get("system").and_then(|v| v.as_str()),
                obj.get("code").and_then(|v| v.as_str()),
            ) {
                return Ok(Coding {
                    system: system.to_string(),
                    code: code.to_string(),
                    version: obj
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    display: obj
                        .get("display")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            }

            // CodeableConcept (take first coding)
            if let Some(coding) = obj
                .get("coding")
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
            {
                return Self::extract_coding_from_json(coding);
            }
        }
        Err(FhirPathError::evaluation_error(
            FP0051,
            "Object is not a valid Coding or CodeableConcept".to_string(),
        ))
    }

    /// Convert Coding to a JSON Resource (FHIRPathValue::Resource)
    pub fn coding_to_value(c: &Coding) -> FhirPathValue {
        let mut m = Map::new();
        m.insert("system".to_string(), JsonValue::String(c.system.clone()));
        m.insert("code".to_string(), JsonValue::String(c.code.clone()));
        if let Some(v) = &c.version {
            m.insert("version".to_string(), JsonValue::String(v.clone()));
        }
        if let Some(d) = &c.display {
            m.insert("display".to_string(), JsonValue::String(d.clone()));
        }
        FhirPathValue::Resource(Arc::new(JsonValue::Object(m)))
    }

    /// Create a Coding from system and code strings
    pub fn create_coding(system: &str, code: &str) -> Coding {
        Coding::new(system, code)
    }

    /// Create a Coding with display
    pub fn create_coding_with_display(system: &str, code: &str, display: &str) -> Coding {
        Coding::new(system, code).with_display(display)
    }

    /// Check if two codings are equivalent
    pub fn codings_equal(coding1: &Coding, coding2: &Coding) -> bool {
        coding1.system == coding2.system
            && coding1.code == coding2.code
            && coding1.version == coding2.version
    }

    /// Validate that a coding has required fields
    pub fn validate_coding(coding: &Coding) -> Result<()> {
        if coding.system.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0051,
                "Coding system is required".to_string(),
            ));
        }

        if coding.code.is_empty() {
            return Err(FhirPathError::evaluation_error(
                FP0051,
                "Coding code is required".to_string(),
            ));
        }

        // Validate system URL format (basic check)
        if !coding.system.starts_with("http://")
            && !coding.system.starts_with("https://")
            && !coding.system.starts_with("urn:")
        {
            return Err(FhirPathError::evaluation_error(
                FP0051,
                "Coding system should be a valid URI".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert ConceptTranslation to FhirPathValue
    pub fn translation_to_fhir_value(translation: &ConceptTranslation) -> FhirPathValue {
        let mut translation_obj = Map::new();
        translation_obj.insert(
            "equivalence".to_string(),
            JsonValue::String(translation.equivalence.clone()),
        );

        if let FhirPathValue::Resource(concept_arc) = Self::coding_to_value(&translation.concept) {
            if let JsonValue::Object(concept_obj) = concept_arc.as_ref() {
                translation_obj.insert(
                    "concept".to_string(),
                    JsonValue::Object(concept_obj.clone()),
                );
            }
        }

        if let Some(ref comment) = translation.comment {
            translation_obj.insert("comment".to_string(), JsonValue::String(comment.clone()));
        }

        FhirPathValue::Resource(Arc::new(JsonValue::Object(translation_obj)))
    }

    /// Convert ConceptDesignation to FhirPathValue
    pub fn designation_to_fhir_value(designation: &ConceptDesignation) -> FhirPathValue {
        let mut designation_obj = Map::new();
        designation_obj.insert(
            "value".to_string(),
            JsonValue::String(designation.value.clone()),
        );

        if let Some(ref language) = designation.language {
            designation_obj.insert("language".to_string(), JsonValue::String(language.clone()));
        }

        if let Some(ref use_coding) = designation.use_coding {
            if let FhirPathValue::Resource(use_arc) = Self::coding_to_value(use_coding) {
                if let JsonValue::Object(use_obj) = use_arc.as_ref() {
                    designation_obj.insert("use".to_string(), JsonValue::Object(use_obj.clone()));
                }
            }
        }

        FhirPathValue::Resource(Arc::new(JsonValue::Object(designation_obj)))
    }

    /// Convert PropertyValue to FhirPathValue
    pub fn property_value_to_fhir_value(property_value: &PropertyValue) -> FhirPathValue {
        match property_value {
            PropertyValue::String(s) => FhirPathValue::String(s.clone()),
            PropertyValue::Boolean(b) => FhirPathValue::Boolean(*b),
            PropertyValue::Integer(i) => FhirPathValue::Integer(*i),
            PropertyValue::Decimal(d) => FhirPathValue::Decimal(*d),
            PropertyValue::Coding(c) => Self::coding_to_value(c),
            PropertyValue::DateTime(dt) => FhirPathValue::DateTime(dt.clone()),
        }
    }

    /// Extract all codings from a CodeableConcept
    pub fn extract_all_codings(value: &FhirPathValue) -> Result<Vec<Coding>> {
        let mut codings = Vec::new();

        match value {
            FhirPathValue::Resource(j) | FhirPathValue::JsonValue(j) => {
                if let Some(obj) = j.as_object() {
                    // Single coding
                    if obj.contains_key("system") && obj.contains_key("code") {
                        codings.push(Self::extract_coding(value)?);
                    }
                    // CodeableConcept with multiple codings
                    else if let Some(coding_array) = obj.get("coding").and_then(|v| v.as_array())
                    {
                        for coding_value in coding_array {
                            if let Ok(coding) = Self::extract_coding_from_json(coding_value) {
                                codings.push(coding);
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(FhirPathError::evaluation_error(
                    FP0051,
                    "Cannot extract codings from this value type".to_string(),
                ));
            }
        }

        Ok(codings)
    }
}
