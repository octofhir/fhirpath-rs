//! Environment variables implementation for FHIRPath 3.0.0
//!
//! This module implements the environment variables as specified in FHIR specification
//! section 2.1.9.1.7 Environment variables.
//!
//! The following environmental values are set for all contexts:
//! - %sct        // (string) url for snomed ct
//! - %loinc      // (string) url for loinc
//! - %vs-[name]  // (string) full url for the provided HL7 value set with id [name]
//! - %ext-[name] // (string) full url for the provided HL7 extension with id [name]
//! - %resource   // The original resource current context is part of
//!
//! Implementation Guides are allowed to define their own externals, and implementers
//! should provide some appropriate configuration framework to allow these constants
//! to be provided to the evaluation engine at run-time.

use crate::core::FhirPathValue;
use std::collections::HashMap;

/// Configuration for FHIRPath environment variables
///
/// This structure holds the configuration for all environment variables that should
/// be available during FHIRPath evaluation, as specified in the FHIR specification.
#[derive(Debug, Clone)]
pub struct EnvironmentVariables {
    /// URL for SNOMED CT terminology system (%sct)
    pub sct_url: Option<String>,
    /// URL for LOINC terminology system (%loinc)
    pub loinc_url: Option<String>,
    /// URLs for HL7 value sets (%vs-[name])
    pub value_sets: HashMap<String, String>,
    /// URLs for HL7 extensions (%ext-[name])
    pub extensions: HashMap<String, String>,
    /// Additional custom environment variables
    pub custom_variables: HashMap<String, FhirPathValue>,
}

impl EnvironmentVariables {
    /// Create a new environment variables configuration with defaults
    pub fn new() -> Self {
        let mut custom_variables = HashMap::new();
        // Add UCUM as a custom variable since it doesn't fit the standard pattern
        custom_variables.insert(
            "%ucum".to_string(),
            FhirPathValue::string("http://unitsofmeasure.org".to_string()),
        );

        let mut value_sets = HashMap::new();
        // Add commonly used HL7 value sets
        value_sets.insert(
            "administrative-gender".to_string(),
            "http://hl7.org/fhir/ValueSet/administrative-gender".to_string(),
        );

        let mut extensions = HashMap::new();
        // Add commonly used HL7 extensions
        extensions.insert(
            "patient-birthTime".to_string(),
            "http://hl7.org/fhir/StructureDefinition/patient-birthTime".to_string(),
        );

        Self {
            sct_url: Some("http://snomed.info/sct".to_string()),
            loinc_url: Some("http://loinc.org".to_string()),
            value_sets,
            extensions,
            custom_variables,
        }
    }

    /// Create environment variables with custom configuration
    pub fn with_config(
        sct_url: Option<String>,
        loinc_url: Option<String>,
        value_sets: HashMap<String, String>,
        extensions: HashMap<String, String>,
        custom_variables: HashMap<String, FhirPathValue>,
    ) -> Self {
        Self {
            sct_url,
            loinc_url,
            value_sets,
            extensions,
            custom_variables,
        }
    }

    /// Set the SNOMED CT URL
    pub fn with_sct_url<S: Into<String>>(mut self, url: S) -> Self {
        self.sct_url = Some(url.into());
        self
    }

    /// Set the LOINC URL
    pub fn with_loinc_url<S: Into<String>>(mut self, url: S) -> Self {
        self.loinc_url = Some(url.into());
        self
    }

    /// Add an HL7 value set URL
    pub fn with_value_set<S: Into<String>>(mut self, name: S, url: S) -> Self {
        self.value_sets.insert(name.into(), url.into());
        self
    }

    /// Add an HL7 extension URL
    pub fn with_extension<S: Into<String>>(mut self, name: S, url: S) -> Self {
        self.extensions.insert(name.into(), url.into());
        self
    }

    /// Add a custom environment variable
    pub fn with_custom_variable<S: Into<String>>(mut self, name: S, value: FhirPathValue) -> Self {
        self.custom_variables.insert(name.into(), value);
        self
    }

    /// Get an environment variable value by name
    ///
    /// This method handles all the standard FHIR environment variables and custom variables.
    /// It supports the following patterns:
    /// - %sct: SNOMED CT URL
    /// - %loinc: LOINC URL
    /// - %vs-[name]: HL7 value set URL
    /// - %ext-[name]: HL7 extension URL
    /// - Custom variables defined in custom_variables
    pub fn get_variable(&self, name: &str) -> Option<FhirPathValue> {
        match name {
            "%sct" => self
                .sct_url
                .as_ref()
                .map(|url| FhirPathValue::string(url.clone())),
            "%loinc" => self
                .loinc_url
                .as_ref()
                .map(|url| FhirPathValue::string(url.clone())),
            var_name if var_name.starts_with("%vs-") => {
                let vs_name = &var_name[4..]; // Remove "%vs-" prefix
                self.value_sets
                    .get(vs_name)
                    .map(|url| FhirPathValue::string(url.clone()))
            }
            var_name if var_name.starts_with("%ext-") => {
                let ext_name = &var_name[5..]; // Remove "%ext-" prefix
                self.extensions
                    .get(ext_name)
                    .map(|url| FhirPathValue::string(url.clone()))
            }
            _ => self.custom_variables.get(name).cloned(),
        }
    }

    /// Check if a variable exists
    pub fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    /// List all available environment variable names
    pub fn list_variables(&self) -> Vec<String> {
        let mut vars = Vec::new();

        if self.sct_url.is_some() {
            vars.push("%sct".to_string());
        }
        if self.loinc_url.is_some() {
            vars.push("%loinc".to_string());
        }

        for vs_name in self.value_sets.keys() {
            vars.push(format!("%vs-{vs_name}"));
        }

        for ext_name in self.extensions.keys() {
            vars.push(format!("%ext-{ext_name}"));
        }

        for custom_name in self.custom_variables.keys() {
            vars.push(custom_name.clone());
        }

        vars.sort();
        vars
    }

    /// Add multiple value sets from a HashMap
    pub fn add_value_sets(&mut self, value_sets: HashMap<String, String>) {
        self.value_sets.extend(value_sets);
    }

    /// Add multiple extensions from a HashMap
    pub fn add_extensions(&mut self, extensions: HashMap<String, String>) {
        self.extensions.extend(extensions);
    }

    /// Add multiple custom variables
    pub fn add_custom_variables(&mut self, variables: HashMap<String, FhirPathValue>) {
        self.custom_variables.extend(variables);
    }
}

impl Default for EnvironmentVariables {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for EnvironmentVariables configuration
#[derive(Debug, Default)]
pub struct EnvironmentVariablesBuilder {
    sct_url: Option<String>,
    loinc_url: Option<String>,
    value_sets: HashMap<String, String>,
    extensions: HashMap<String, String>,
    custom_variables: HashMap<String, FhirPathValue>,
}

impl EnvironmentVariablesBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the SNOMED CT URL
    pub fn sct_url<S: Into<String>>(mut self, url: S) -> Self {
        self.sct_url = Some(url.into());
        self
    }

    /// Set the LOINC URL
    pub fn loinc_url<S: Into<String>>(mut self, url: S) -> Self {
        self.loinc_url = Some(url.into());
        self
    }

    /// Add an HL7 value set URL
    pub fn value_set<S: Into<String>>(mut self, name: S, url: S) -> Self {
        self.value_sets.insert(name.into(), url.into());
        self
    }

    /// Add an HL7 extension URL
    pub fn extension<S: Into<String>>(mut self, name: S, url: S) -> Self {
        self.extensions.insert(name.into(), url.into());
        self
    }

    /// Add a custom environment variable
    pub fn custom_variable<S: Into<String>>(mut self, name: S, value: FhirPathValue) -> Self {
        self.custom_variables.insert(name.into(), value);
        self
    }

    /// Build the EnvironmentVariables configuration
    pub fn build(self) -> EnvironmentVariables {
        EnvironmentVariables::with_config(
            self.sct_url,
            self.loinc_url,
            self.value_sets,
            self.extensions,
            self.custom_variables,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_variables_creation() {
        let env_vars = EnvironmentVariables::new();

        // Check default values
        assert!(env_vars.get_variable("%sct").is_some());
        assert!(env_vars.get_variable("%loinc").is_some());
        assert!(env_vars.get_variable("%ext-patient-birthTime").is_some());

        if let Some(FhirPathValue::String(sct_url, _, _)) = env_vars.get_variable("%sct") {
            assert_eq!(sct_url, "http://snomed.info/sct");
        } else {
            panic!("Expected string value for %sct");
        }

        if let Some(FhirPathValue::String(loinc_url, _, _)) = env_vars.get_variable("%loinc") {
            assert_eq!(loinc_url, "http://loinc.org");
        } else {
            panic!("Expected string value for %loinc");
        }

        if let Some(FhirPathValue::String(birthtime_url, _, _)) =
            env_vars.get_variable("%ext-patient-birthTime")
        {
            assert_eq!(
                birthtime_url,
                "http://hl7.org/fhir/StructureDefinition/patient-birthTime"
            );
        } else {
            panic!("Expected string value for %ext-patient-birthTime");
        }
    }

    #[test]
    fn test_value_set_variables() {
        let env_vars = EnvironmentVariables::new().with_value_set(
            "observation-vitalsignresult",
            "http://hl7.org/fhir/ValueSet/observation-vitalsignresult",
        );

        let vs_var = env_vars.get_variable("%vs-observation-vitalsignresult");
        assert!(vs_var.is_some());

        if let Some(FhirPathValue::String(url, _, _)) = vs_var {
            assert_eq!(
                url,
                "http://hl7.org/fhir/ValueSet/observation-vitalsignresult"
            );
        } else {
            panic!("Expected string value for value set variable");
        }
    }

    #[test]
    fn test_extension_variables() {
        let env_vars = EnvironmentVariables::new().with_extension(
            "patient-birthPlace",
            "http://hl7.org/fhir/StructureDefinition/patient-birthPlace",
        );

        let ext_var = env_vars.get_variable("%ext-patient-birthPlace");
        assert!(ext_var.is_some());

        if let Some(FhirPathValue::String(url, _, _)) = ext_var {
            assert_eq!(
                url,
                "http://hl7.org/fhir/StructureDefinition/patient-birthPlace"
            );
        } else {
            panic!("Expected string value for extension variable");
        }
    }

    #[test]
    fn test_custom_variables() {
        let env_vars = EnvironmentVariables::new().with_custom_variable(
            "%us-zip",
            FhirPathValue::string("[0-9]{5}(-[0-9]{4}){0,1}".to_string()),
        );

        let custom_var = env_vars.get_variable("%us-zip");
        assert!(custom_var.is_some());

        if let Some(FhirPathValue::String(pattern, _, _)) = custom_var {
            assert_eq!(pattern, "[0-9]{5}(-[0-9]{4}){0,1}");
        } else {
            panic!("Expected string value for custom variable");
        }
    }

    #[test]
    fn test_builder_pattern() {
        let env_vars = EnvironmentVariablesBuilder::new()
            .sct_url("http://custom.snomed.org")
            .loinc_url("http://custom.loinc.org")
            .value_set("test-vs", "http://example.org/ValueSet/test")
            .extension("test-ext", "http://example.org/StructureDefinition/test")
            .custom_variable("%test", FhirPathValue::integer(42))
            .build();

        assert_eq!(
            env_vars.get_variable("%sct"),
            Some(FhirPathValue::string(
                "http://custom.snomed.org".to_string()
            ))
        );
        assert_eq!(
            env_vars.get_variable("%loinc"),
            Some(FhirPathValue::string("http://custom.loinc.org".to_string()))
        );
        assert_eq!(
            env_vars.get_variable("%vs-test-vs"),
            Some(FhirPathValue::string(
                "http://example.org/ValueSet/test".to_string()
            ))
        );
        assert_eq!(
            env_vars.get_variable("%ext-test-ext"),
            Some(FhirPathValue::string(
                "http://example.org/StructureDefinition/test".to_string()
            ))
        );
        assert_eq!(
            env_vars.get_variable("%test"),
            Some(FhirPathValue::integer(42))
        );
    }

    #[test]
    fn test_list_variables() {
        let env_vars = EnvironmentVariables::new()
            .with_value_set("vs1", "http://example.org/vs1")
            .with_extension("ext1", "http://example.org/ext1")
            .with_custom_variable("%custom", FhirPathValue::string("test".to_string()));

        let vars = env_vars.list_variables();
        assert!(vars.contains(&"%sct".to_string()));
        assert!(vars.contains(&"%loinc".to_string()));
        assert!(vars.contains(&"%vs-vs1".to_string()));
        assert!(vars.contains(&"%ext-ext1".to_string()));
        assert!(vars.contains(&"%custom".to_string()));
    }

    #[test]
    fn test_nonexistent_variables() {
        let env_vars = EnvironmentVariables::new();

        assert!(env_vars.get_variable("%vs-nonexistent").is_none());
        assert!(env_vars.get_variable("%ext-nonexistent").is_none());
        assert!(env_vars.get_variable("%nonexistent").is_none());
        assert!(!env_vars.has_variable("%nonexistent"));
    }
}
