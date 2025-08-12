// Copyright 2024 OctoFHIR Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Choice type resolution system for FHIRPath

use super::provider::TypeReflectionInfo;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Choice type resolver for handling FHIR choice elements
pub struct ChoiceTypeResolver {
    /// Known choice type patterns
    choice_patterns: HashMap<String, ChoicePattern>,
}

/// Information about a choice type pattern
#[derive(Debug, Clone)]
pub struct ChoicePattern {
    /// Base element path (e.g., "Observation.value")
    pub base_path: String,
    /// Available choice type suffixes and their target types
    pub choices: HashMap<String, String>,
}

/// Result of choice type resolution
#[derive(Debug, Clone)]
pub struct ChoiceResolution {
    /// The resolved concrete type
    pub concrete_type: String,
    /// The original choice element path
    pub base_path: String,
    /// The suffix that was matched
    pub suffix: String,
    /// Whether this was a valid choice type resolution
    pub is_valid: bool,
}

static CHOICE_TYPE_SUFFIX_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([a-z][a-zA-Z0-9]*[a-z0-9])([A-Z][a-zA-Z0-9]*)$").unwrap());

impl ChoiceTypeResolver {
    /// Create a new choice type resolver
    pub fn new() -> Self {
        let mut resolver = Self {
            choice_patterns: HashMap::new(),
        };

        resolver.initialize_standard_patterns();
        resolver
    }

    /// Initialize with standard FHIR choice type patterns
    fn initialize_standard_patterns(&mut self) {
        // Observation.value[x] choices
        let mut observation_value_choices = HashMap::new();
        observation_value_choices.insert("Quantity".to_string(), "Quantity".to_string());
        observation_value_choices
            .insert("CodeableConcept".to_string(), "CodeableConcept".to_string());
        observation_value_choices.insert("String".to_string(), "string".to_string());
        observation_value_choices.insert("Boolean".to_string(), "boolean".to_string());
        observation_value_choices.insert("Integer".to_string(), "integer".to_string());
        observation_value_choices.insert("Range".to_string(), "Range".to_string());
        observation_value_choices.insert("Ratio".to_string(), "Ratio".to_string());
        observation_value_choices.insert("SampledData".to_string(), "SampledData".to_string());
        observation_value_choices.insert("Time".to_string(), "time".to_string());
        observation_value_choices.insert("DateTime".to_string(), "dateTime".to_string());
        observation_value_choices.insert("Period".to_string(), "Period".to_string());

        self.choice_patterns.insert(
            "Observation.value".to_string(),
            ChoicePattern {
                base_path: "Observation.value".to_string(),
                choices: observation_value_choices,
            },
        );

        // Condition.onset[x] choices
        let mut condition_onset_choices = HashMap::new();
        condition_onset_choices.insert("DateTime".to_string(), "dateTime".to_string());
        condition_onset_choices.insert("Age".to_string(), "Age".to_string());
        condition_onset_choices.insert("Period".to_string(), "Period".to_string());
        condition_onset_choices.insert("Range".to_string(), "Range".to_string());
        condition_onset_choices.insert("String".to_string(), "string".to_string());

        self.choice_patterns.insert(
            "Condition.onset".to_string(),
            ChoicePattern {
                base_path: "Condition.onset".to_string(),
                choices: condition_onset_choices,
            },
        );

        // Condition.abatement[x] choices
        let mut condition_abatement_choices = HashMap::new();
        condition_abatement_choices.insert("DateTime".to_string(), "dateTime".to_string());
        condition_abatement_choices.insert("Age".to_string(), "Age".to_string());
        condition_abatement_choices.insert("Period".to_string(), "Period".to_string());
        condition_abatement_choices.insert("Range".to_string(), "Range".to_string());
        condition_abatement_choices.insert("String".to_string(), "string".to_string());
        condition_abatement_choices.insert("Boolean".to_string(), "boolean".to_string());

        self.choice_patterns.insert(
            "Condition.abatement".to_string(),
            ChoicePattern {
                base_path: "Condition.abatement".to_string(),
                choices: condition_abatement_choices,
            },
        );

        // Extension.value[x] choices (common extension pattern)
        let mut extension_value_choices = HashMap::new();
        extension_value_choices.insert("String".to_string(), "string".to_string());
        extension_value_choices.insert("Boolean".to_string(), "boolean".to_string());
        extension_value_choices.insert("Integer".to_string(), "integer".to_string());
        extension_value_choices.insert("Decimal".to_string(), "decimal".to_string());
        extension_value_choices.insert("DateTime".to_string(), "dateTime".to_string());
        extension_value_choices.insert("Date".to_string(), "date".to_string());
        extension_value_choices.insert("Time".to_string(), "time".to_string());
        extension_value_choices.insert("Code".to_string(), "code".to_string());
        extension_value_choices.insert("Uri".to_string(), "uri".to_string());
        extension_value_choices.insert("Canonical".to_string(), "canonical".to_string());
        extension_value_choices.insert("Quantity".to_string(), "Quantity".to_string());
        extension_value_choices
            .insert("CodeableConcept".to_string(), "CodeableConcept".to_string());
        extension_value_choices.insert("Reference".to_string(), "Reference".to_string());

        self.choice_patterns.insert(
            "Extension.value".to_string(),
            ChoicePattern {
                base_path: "Extension.value".to_string(),
                choices: extension_value_choices,
            },
        );
    }

    /// Resolve a choice type property access
    pub fn resolve_choice_type(&self, property_path: &str) -> Option<ChoiceResolution> {
        // Try to match against known patterns
        for (base_path, pattern) in &self.choice_patterns {
            if let Some(suffix) = property_path.strip_prefix(&format!("{base_path}[x]")) {
                // Handle explicit [x] syntax
                if suffix.is_empty() {
                    // Return the base choice type info
                    return Some(ChoiceResolution {
                        concrete_type: "Choice".to_string(),
                        base_path: base_path.clone(),
                        suffix: "[x]".to_string(),
                        is_valid: true,
                    });
                }
            } else if let Some(suffix) = property_path.strip_prefix(base_path) {
                // Handle concrete choice type (e.g., valueString, valueQuantity)
                if let Some(type_suffix) = self.extract_choice_suffix(suffix) {
                    if let Some(target_type) = pattern.choices.get(&type_suffix) {
                        return Some(ChoiceResolution {
                            concrete_type: target_type.clone(),
                            base_path: base_path.clone(),
                            suffix: type_suffix,
                            is_valid: true,
                        });
                    } else {
                        // Invalid choice type suffix
                        return Some(ChoiceResolution {
                            concrete_type: "Invalid".to_string(),
                            base_path: base_path.clone(),
                            suffix: type_suffix,
                            is_valid: false,
                        });
                    }
                }
            }
        }

        // Try generic pattern matching for unknown choice types
        self.attempt_generic_resolution(property_path)
    }

    /// Extract choice type suffix from a property name
    fn extract_choice_suffix(&self, suffix: &str) -> Option<String> {
        // Handle cases like "String" from "valueString"
        if suffix.is_empty() {
            return None;
        }

        // Check if it's a valid choice type suffix (starts with capital letter)
        if suffix.chars().next()?.is_uppercase() {
            Some(suffix.to_string())
        } else {
            None
        }
    }

    /// Attempt to resolve choice type using generic patterns
    fn attempt_generic_resolution(&self, property_path: &str) -> Option<ChoiceResolution> {
        // Look for pattern like "elementNameTypesSuffix"
        if let Some(captures) = CHOICE_TYPE_SUFFIX_REGEX.captures(property_path) {
            let base_element = captures.get(1)?.as_str();
            let type_suffix = captures.get(2)?.as_str();

            // Check if this looks like a choice type pattern
            if self.is_likely_choice_type(base_element, type_suffix) {
                return Some(ChoiceResolution {
                    concrete_type: self.map_suffix_to_type(type_suffix),
                    base_path: base_element.to_string(),
                    suffix: type_suffix.to_string(),
                    is_valid: true,
                });
            }
        }

        None
    }

    /// Check if a base element and suffix combination is likely a choice type
    fn is_likely_choice_type(&self, base_element: &str, type_suffix: &str) -> bool {
        // Common choice type base elements
        let choice_bases = [
            "value",
            "onset",
            "abatement",
            "effective",
            "performed",
            "occurrence",
            "scheduled",
            "timing",
            "multipleBirth",
            "deceased",
            "born",
        ];

        if choice_bases
            .iter()
            .any(|&base| base_element.ends_with(base))
        {
            return true;
        }

        // Common choice type suffixes
        let choice_suffixes = [
            "String",
            "Boolean",
            "Integer",
            "Decimal",
            "DateTime",
            "Date",
            "Time",
            "Code",
            "Uri",
            "Canonical",
            "Quantity",
            "CodeableConcept",
            "Reference",
            "Period",
            "Range",
            "Ratio",
            "Age",
            "Count",
            "Distance",
            "Duration",
        ];

        choice_suffixes.contains(&type_suffix)
    }

    /// Map choice type suffix to actual FHIR type
    fn map_suffix_to_type(&self, suffix: &str) -> String {
        match suffix {
            "String" => "string".to_string(),
            "Boolean" => "boolean".to_string(),
            "Integer" => "integer".to_string(),
            "Decimal" => "decimal".to_string(),
            "DateTime" => "dateTime".to_string(),
            "Date" => "date".to_string(),
            "Time" => "time".to_string(),
            "Code" => "code".to_string(),
            "Uri" => "uri".to_string(),
            "Canonical" => "canonical".to_string(),
            // Complex types keep their names
            _ => suffix.to_string(),
        }
    }

    /// Get all possible choice types for a base path
    pub fn get_choice_options(&self, base_path: &str) -> Option<Vec<String>> {
        self.choice_patterns
            .get(base_path)
            .map(|pattern| pattern.choices.keys().cloned().collect())
    }

    /// Check if a property path represents a choice type
    pub fn is_choice_type_path(&self, property_path: &str) -> bool {
        self.resolve_choice_type(property_path)
            .map(|res| res.is_valid)
            .unwrap_or(false)
    }

    /// Convert choice resolution to TypeReflectionInfo
    pub fn choice_to_type_reflection(&self, resolution: &ChoiceResolution) -> TypeReflectionInfo {
        if resolution.concrete_type == "Choice" {
            // Return a union type for generic choice
            TypeReflectionInfo::TupleType {
                elements: vec![], // Would be populated with actual choice options
            }
        } else if self.is_primitive_type(&resolution.concrete_type) {
            TypeReflectionInfo::SimpleType {
                namespace: "System".to_string(),
                name: self.normalize_primitive_name(&resolution.concrete_type),
                base_type: None,
            }
        } else {
            TypeReflectionInfo::ClassInfo {
                namespace: "FHIR".to_string(),
                name: resolution.concrete_type.clone(),
                base_type: None,
                elements: vec![],
            }
        }
    }

    /// Check if a type is primitive
    fn is_primitive_type(&self, type_name: &str) -> bool {
        matches!(
            type_name,
            "boolean"
                | "integer"
                | "string"
                | "decimal"
                | "date"
                | "dateTime"
                | "time"
                | "code"
                | "uri"
                | "canonical"
        )
    }

    /// Normalize primitive type name for consistency
    fn normalize_primitive_name(&self, type_name: &str) -> String {
        match type_name {
            "boolean" => "Boolean".to_string(),
            "integer" => "Integer".to_string(),
            "decimal" => "Decimal".to_string(),
            "string" => "String".to_string(),
            "date" => "Date".to_string(),
            "dateTime" => "DateTime".to_string(),
            "time" => "Time".to_string(),
            _ => type_name.to_string(),
        }
    }

    /// Add a custom choice pattern
    pub fn add_choice_pattern(&mut self, base_path: String, pattern: ChoicePattern) {
        self.choice_patterns.insert(base_path, pattern);
    }
}

impl Default for ChoiceTypeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_value_choice_resolution() {
        let resolver = ChoiceTypeResolver::new();

        // Test valueString
        let result = resolver.resolve_choice_type("Observation.valueString");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.concrete_type, "string");
        assert_eq!(result.base_path, "Observation.value");
        assert_eq!(result.suffix, "String");
        assert!(result.is_valid);

        // Test valueQuantity
        let result = resolver.resolve_choice_type("Observation.valueQuantity");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.concrete_type, "Quantity");
        assert_eq!(result.base_path, "Observation.value");
        assert_eq!(result.suffix, "Quantity");
        assert!(result.is_valid);
    }

    #[test]
    fn test_condition_onset_choice_resolution() {
        let resolver = ChoiceTypeResolver::new();

        // Test onsetDateTime
        let result = resolver.resolve_choice_type("Condition.onsetDateTime");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.concrete_type, "dateTime");
        assert_eq!(result.base_path, "Condition.onset");
        assert!(result.is_valid);
    }

    #[test]
    fn test_invalid_choice_type() {
        let resolver = ChoiceTypeResolver::new();

        // Test invalid choice type
        let result = resolver.resolve_choice_type("Observation.valueInvalidType");
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_generic_choice_resolution() {
        let resolver = ChoiceTypeResolver::new();

        // Test generic pattern that's not in predefined patterns
        let result = resolver.resolve_choice_type("someElementString");
        if let Some(result) = result {
            assert_eq!(result.suffix, "String");
            assert_eq!(result.concrete_type, "string");
        }
    }

    #[test]
    fn test_choice_options() {
        let resolver = ChoiceTypeResolver::new();

        let options = resolver.get_choice_options("Observation.value");
        assert!(options.is_some());
        let options = options.unwrap();
        assert!(options.contains(&"String".to_string()));
        assert!(options.contains(&"Quantity".to_string()));
        assert!(options.contains(&"Boolean".to_string()));
    }

    #[test]
    fn test_choice_type_detection() {
        let resolver = ChoiceTypeResolver::new();

        assert!(resolver.is_choice_type_path("Observation.valueString"));
        assert!(resolver.is_choice_type_path("Condition.onsetDateTime"));
        assert!(!resolver.is_choice_type_path("Patient.name"));
    }

    #[test]
    fn test_choice_to_type_reflection() {
        let resolver = ChoiceTypeResolver::new();

        let resolution = ChoiceResolution {
            concrete_type: "string".to_string(),
            base_path: "Observation.value".to_string(),
            suffix: "String".to_string(),
            is_valid: true,
        };

        let type_reflection = resolver.choice_to_type_reflection(&resolution);
        match type_reflection {
            TypeReflectionInfo::SimpleType { name, .. } => {
                assert_eq!(name, "String");
            }
            _ => panic!("Expected SimpleType"),
        }
    }

    #[test]
    fn test_custom_choice_pattern() {
        let mut resolver = ChoiceTypeResolver::new();

        let mut custom_choices = HashMap::new();
        custom_choices.insert("Custom".to_string(), "CustomType".to_string());

        resolver.add_choice_pattern(
            "CustomElement.value".to_string(),
            ChoicePattern {
                base_path: "CustomElement.value".to_string(),
                choices: custom_choices,
            },
        );

        let result = resolver.resolve_choice_type("CustomElement.valueCustom");
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.concrete_type, "CustomType");
        assert!(result.is_valid);
    }
}
