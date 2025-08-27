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

//! Profile resolution and constraint handling system

use super::cache::Cache;
use super::provider::*;
use octofhir_fhirschema::{Element as FhirSchemaElement, FhirSchema};
use std::collections::HashMap;
use url::Url;

/// Profile resolver for FHIR profiles and constraints
pub struct ProfileResolver {
    /// Cache for resolved profiles
    profile_cache: Cache<String, ResolvedProfile>,
    /// Cache for constraint information
    constraint_cache: Cache<String, Vec<ConstraintInfo>>,
}

/// A resolved profile with merged constraints and elements
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    /// Profile URL
    pub profile_url: String,
    /// Base type URL
    pub base_type_url: String,
    /// Merged elements from base and profile
    pub elements: HashMap<String, ResolvedElement>,
    /// All constraints from inheritance chain
    pub constraints: Vec<ConstraintInfo>,
    /// Slicing information
    pub slicing_info: HashMap<String, SlicingInfo>,
}

/// Resolved element with profile constraints applied
#[derive(Debug, Clone)]
pub struct ResolvedElement {
    /// Element path
    pub path: String,
    /// Type information
    pub type_info: TypeReflectionInfo,
    /// Minimum cardinality constraint (how many times this element must appear)
    pub min_cardinality: u32,
    /// Maximum cardinality constraint (how many times this element can appear, None for unlimited)
    pub max_cardinality: Option<u32>,
    /// Profile-specific constraints
    pub constraints: Vec<ConstraintInfo>,
    /// Fixed value that this element must have
    pub fixed_value: Option<sonic_rs::Value>,
    /// Pattern value that this element should match
    pub pattern_value: Option<sonic_rs::Value>,
    /// Binding information
    pub binding: Option<BindingInfo>,
    /// Whether this element is modified by the profile
    pub is_profiled: bool,
}

/// Slicing information for arrays and choice types
#[derive(Debug, Clone)]
pub struct SlicingInfo {
    /// Discriminator paths
    pub discriminators: Vec<String>,
    /// Slicing rules (open, closed, openAtEnd)
    pub rules: String,
    /// Ordered slicing
    pub ordered: bool,
    /// Slice definitions
    pub slices: HashMap<String, SliceInfo>,
}

/// Information about a specific slice
#[derive(Debug, Clone)]
pub struct SliceInfo {
    /// Slice name
    pub name: String,
    /// Slice discriminator values
    pub discriminator_values: HashMap<String, String>,
    /// Elements specific to this slice
    pub elements: HashMap<String, ResolvedElement>,
}

/// Binding information for coded elements
#[derive(Debug, Clone)]
pub struct BindingInfo {
    /// Binding strength (required, extensible, preferred, example)
    pub strength: String,
    /// Value set URL
    pub value_set: Option<String>,
    /// Description
    pub description: Option<String>,
}

impl ProfileResolver {
    /// Convert serde_json::Value to sonic_rs::Value using optimized string conversion
    fn convert_json_value(value: &serde_json::Value) -> sonic_rs::Value {
        // Convert between JSON libraries - external dependency (octofhir-fhirschema) uses serde_json
        // We standardize on sonic_rs internally for better performance
        match serde_json::to_string(value) {
            Ok(json_str) => sonic_rs::from_str(&json_str).unwrap_or(sonic_rs::Value::new_null()),
            Err(_) => sonic_rs::Value::new_null(),
        }
    }
    /// Create a new profile resolver
    pub fn new() -> Self {
        Self {
            profile_cache: Cache::new(1000),    // Default capacity for profiles
            constraint_cache: Cache::new(2000), // Default capacity for constraints
        }
    }

    /// Create with custom cache configuration
    pub fn with_cache_config(cache_config: super::cache::CacheConfig) -> Self {
        let profile_capacity = cache_config.capacity.max(100);
        let constraint_capacity = (cache_config.capacity * 2).max(200);

        Self {
            profile_cache: Cache::new(profile_capacity),
            constraint_cache: Cache::new(constraint_capacity),
        }
    }

    /// Resolve a profile by merging with its base type
    pub async fn resolve_profile(
        &self,
        profile_schema: &FhirSchema,
        base_schema: &FhirSchema,
    ) -> Result<ResolvedProfile, octofhir_fhir_model::error::ModelError> {
        // Check cache first
        let cache_key = format!(
            "{}#{}",
            profile_schema
                .url
                .as_ref()
                .map(|u| u.as_str())
                .unwrap_or("unknown"),
            base_schema
                .url
                .as_ref()
                .map(|u| u.as_str())
                .unwrap_or("unknown")
        );

        if let Some(cached) = self.profile_cache.get(&cache_key) {
            return Ok(cached);
        }

        // Merge base and profile elements
        let mut resolved_elements = HashMap::new();
        let mut all_constraints = Vec::new();
        let mut slicing_info = HashMap::new();

        // Start with base elements
        for (path, base_element) in &base_schema.elements {
            let resolved = self.resolve_base_element(path, base_element)?;
            resolved_elements.insert(path.clone(), resolved);
        }

        // Apply profile modifications
        for (path, profile_element) in &profile_schema.elements {
            if let Some(base_element) = resolved_elements.get_mut(path) {
                // Modify existing element
                self.apply_profile_constraints(base_element, profile_element)?;
                base_element.is_profiled = true;
            } else {
                // New element introduced by profile
                let mut resolved = self.resolve_profile_element(path, profile_element)?;
                resolved.is_profiled = true;
                resolved_elements.insert(path.clone(), resolved);
            }
        }

        // Collect constraints from both base and profile
        all_constraints.extend(
            base_schema
                .constraints
                .iter()
                .map(|c| self.convert_constraint(c)),
        );
        all_constraints.extend(
            profile_schema
                .constraints
                .iter()
                .map(|c| self.convert_constraint(c)),
        );

        // Process slicing information
        for (path, slice) in &profile_schema.slicing {
            slicing_info.insert(path.clone(), self.convert_slicing_info(slice));
        }

        let resolved = ResolvedProfile {
            profile_url: profile_schema
                .url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            base_type_url: base_schema
                .url
                .as_ref()
                .map(|u| u.to_string())
                .unwrap_or_default(),
            elements: resolved_elements,
            constraints: all_constraints,
            slicing_info,
        };

        // Cache the result
        self.profile_cache.insert(cache_key, resolved.clone());

        Ok(resolved)
    }

    /// Resolve constraints for a specific element path
    pub async fn resolve_element_constraints(
        &self,
        profile: &ResolvedProfile,
        element_path: &str,
    ) -> Vec<ConstraintInfo> {
        let cache_key = format!("{}#{}", profile.profile_url, element_path);

        if let Some(cached) = self.constraint_cache.get(&cache_key) {
            return cached;
        }

        let mut constraints = Vec::new();

        // Add profile-level constraints that apply to this element
        // TODO: Implement proper expression analysis to determine which constraints apply
        for constraint in &profile.constraints {
            // For now, add all constraints - this is a simplification
            constraints.push(constraint.clone());
        }

        // Add element-specific constraints
        if let Some(element) = profile.elements.get(element_path) {
            constraints.extend(element.constraints.clone());
        }

        // Cache the result
        self.constraint_cache.insert(cache_key, constraints.clone());

        constraints
    }

    /// Check if a profile constrains a specific element
    pub fn is_element_constrained(&self, profile: &ResolvedProfile, element_path: &str) -> bool {
        profile
            .elements
            .get(element_path)
            .map(|elem| elem.is_profiled || !elem.constraints.is_empty())
            .unwrap_or(false)
    }

    /// Get profile-specific type information for an element
    pub fn get_profiled_type(
        &self,
        profile: &ResolvedProfile,
        element_path: &str,
    ) -> Option<TypeReflectionInfo> {
        profile
            .elements
            .get(element_path)
            .map(|elem| elem.type_info.clone())
    }

    /// Validate that a value conforms to profile constraints
    pub fn validate_profile_conformance(
        &self,
        profile: &ResolvedProfile,
        element_path: &str,
        _value: &dyn ValueReflection,
    ) -> ConformanceResult {
        // Get constraints for this element
        let empty_constraints = Vec::new();
        let constraints = profile
            .elements
            .get(element_path)
            .map(|elem| &elem.constraints)
            .unwrap_or(&empty_constraints);

        let violations = Vec::new();
        let warnings = Vec::new();

        // Check each constraint
        for constraint in constraints {
            // TODO: Implement actual constraint evaluation
            // This would involve evaluating FHIRPath expressions against the value
            match constraint.severity {
                ConstraintSeverity::Error => {
                    // violations.push(...);
                }
                ConstraintSeverity::Warning => {
                    // warnings.push(...);
                }
                _ => {}
            }
        }

        ConformanceResult {
            is_valid: violations.is_empty(),
            violations,
            warnings,
            metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
            profile_url: profile.profile_url.clone(),
            resource_type: self.extract_resource_type(&profile.base_type_url),
        }
    }

    /// Get slicing information for an element
    pub fn get_slicing_info<'a>(
        &self,
        profile: &'a ResolvedProfile,
        element_path: &str,
    ) -> Option<&'a SlicingInfo> {
        profile.slicing_info.get(element_path)
    }

    /// Resolve base element without profile modifications
    fn resolve_base_element(
        &self,
        path: &str,
        element: &FhirSchemaElement,
    ) -> Result<ResolvedElement, octofhir_fhir_model::error::ModelError> {
        Ok(ResolvedElement {
            path: path.to_string(),
            type_info: self.convert_element_to_type_reflection(element)?,
            min_cardinality: element.min.unwrap_or(0),
            max_cardinality: element
                .max
                .as_ref()
                .and_then(|m| if m == "*" { None } else { m.parse().ok() }),
            constraints: element
                .constraints
                .iter()
                .map(|c| self.convert_constraint(c))
                .collect(),
            fixed_value: element.fixed.as_ref().map(Self::convert_json_value),
            pattern_value: element.pattern.as_ref().map(Self::convert_json_value),
            binding: element.binding.as_ref().map(|b| self.convert_binding(b)),
            is_profiled: false,
        })
    }

    /// Resolve profile-specific element
    fn resolve_profile_element(
        &self,
        path: &str,
        element: &FhirSchemaElement,
    ) -> Result<ResolvedElement, octofhir_fhir_model::error::ModelError> {
        self.resolve_base_element(path, element)
    }

    /// Apply profile constraints to a base element
    fn apply_profile_constraints(
        &self,
        base_element: &mut ResolvedElement,
        profile_element: &FhirSchemaElement,
    ) -> Result<(), octofhir_fhir_model::error::ModelError> {
        // Tighten cardinality constraints
        if let Some(profile_min) = profile_element.min {
            if profile_min > base_element.min_cardinality {
                base_element.min_cardinality = profile_min;
            }
        }

        if let Some(profile_max) = &profile_element.max {
            let profile_max_num = if profile_max == "*" {
                None
            } else {
                profile_max.parse().ok()
            };

            match (base_element.max_cardinality, profile_max_num) {
                (Some(base_max), Some(profile_max)) => {
                    if profile_max < base_max {
                        base_element.max_cardinality = Some(profile_max);
                    }
                }
                (None, Some(profile_max)) => {
                    base_element.max_cardinality = Some(profile_max);
                }
                _ => {}
            }
        }

        // Add profile-specific constraints
        base_element.constraints.extend(
            profile_element
                .constraints
                .iter()
                .map(|c| self.convert_constraint(c)),
        );

        // Override fixed/pattern values
        if profile_element.fixed.is_some() {
            base_element.fixed_value = profile_element.fixed.as_ref().map(Self::convert_json_value);
        }
        if profile_element.pattern.is_some() {
            base_element.pattern_value = profile_element
                .pattern
                .as_ref()
                .map(Self::convert_json_value);
        }

        // Override binding if more restrictive
        if let Some(profile_binding) = &profile_element.binding {
            let profile_binding_info = self.convert_binding(profile_binding);

            // Profile binding is more restrictive if it's "required" or "extensible"
            if matches!(
                profile_binding_info.strength.as_str(),
                "required" | "extensible"
            ) {
                base_element.binding = Some(profile_binding_info);
            }
        }

        Ok(())
    }

    /// Convert FhirSchemaElement to TypeReflectionInfo
    fn convert_element_to_type_reflection(
        &self,
        element: &FhirSchemaElement,
    ) -> Result<TypeReflectionInfo, octofhir_fhir_model::error::ModelError> {
        if let Some(types) = &element.element_type {
            if types.len() == 1 {
                let element_type = &types[0];
                if self.is_primitive_type(&element_type.code) {
                    Ok(TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: self.normalize_primitive_name(&element_type.code),
                        base_type: None,
                    })
                } else {
                    Ok(TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: element_type.code.clone(),
                        base_type: self.get_base_type_name(&element_type.code),
                        elements: vec![],
                    })
                }
            } else if types.len() > 1 {
                // Choice type - return first type for now
                // In practice, this should be handled by choice type resolver
                let first_type = &types[0];
                if self.is_primitive_type(&first_type.code) {
                    Ok(TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: self.normalize_primitive_name(&first_type.code),
                        base_type: None,
                    })
                } else {
                    Ok(TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: first_type.code.clone(),
                        base_type: self.get_base_type_name(&first_type.code),
                        elements: vec![],
                    })
                }
            } else {
                Err(octofhir_fhir_model::error::ModelError::validation_error(
                    "Element has no type information",
                ))
            }
        } else {
            Err(octofhir_fhir_model::error::ModelError::validation_error(
                "Element has no type information",
            ))
        }
    }

    /// Convert constraint from FhirSchema format
    fn convert_constraint(&self, constraint: &octofhir_fhirschema::Constraint) -> ConstraintInfo {
        ConstraintInfo {
            key: constraint.key.clone(),
            severity: ConstraintSeverity::Error, // Map from string to enum
            human: constraint.human.clone(),
            expression: constraint.expression.clone(),
            xpath: None,
            source: None,
            metadata: Default::default(),
        }
    }

    /// Convert slicing information from FhirSchema format
    fn convert_slicing_info(&self, slice: &octofhir_fhirschema::Slicing) -> SlicingInfo {
        SlicingInfo {
            discriminators: slice.discriminator.iter().map(|d| d.path.clone()).collect(),
            rules: slice.rules.clone(),
            ordered: slice.ordered.unwrap_or(false),
            slices: HashMap::new(), // Would be populated from slice definitions
        }
    }

    /// Convert binding information from FhirSchema format
    fn convert_binding(&self, binding: &octofhir_fhirschema::Binding) -> BindingInfo {
        BindingInfo {
            strength: binding.strength.clone(),
            value_set: binding.value_set.as_ref().map(|vs| vs.to_string()),
            description: binding.description.clone(),
        }
    }

    /// Check if type is primitive
    fn is_primitive_type(&self, type_code: &str) -> bool {
        matches!(
            type_code,
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
        )
    }

    /// Normalize primitive type name
    fn normalize_primitive_name(&self, type_code: &str) -> String {
        match type_code {
            "boolean" => "Boolean".to_string(),
            "integer" | "unsignedInt" | "positiveInt" => "Integer".to_string(),
            "decimal" => "Decimal".to_string(),
            "date" => "Date".to_string(),
            "dateTime" | "instant" => "DateTime".to_string(),
            "time" => "Time".to_string(),
            _ => "String".to_string(),
        }
    }

    /// Get base type name for inheritance
    fn get_base_type_name(&self, type_name: &str) -> Option<String> {
        match type_name {
            "Patient" | "Observation" | "Condition" | "Procedure" => {
                Some("DomainResource".to_string())
            }
            "DomainResource" => Some("Resource".to_string()),
            "HumanName" | "Address" | "ContactPoint" => Some("Element".to_string()),
            _ => None,
        }
    }

    /// Extract resource type from URL
    fn extract_resource_type(&self, url: &str) -> String {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(mut segments) = parsed_url.path_segments() {
                if let Some(last_segment) = segments.next_back() {
                    return last_segment.to_string();
                }
            }
        }
        "Unknown".to_string()
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.profile_cache.clear();
        self.constraint_cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (super::cache::CacheStats, super::cache::CacheStats) {
        (self.profile_cache.stats(), self.constraint_cache.stats())
    }

    /// Get cache sizes
    pub fn cache_sizes(&self) -> (usize, usize) {
        (self.profile_cache.len(), self.constraint_cache.len())
    }
}

impl Default for ProfileResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_resolver_creation() {
        let resolver = ProfileResolver::new();
        let (profile_size, constraint_size) = resolver.cache_sizes();
        assert_eq!(profile_size, 0);
        assert_eq!(constraint_size, 0);
    }

    #[test]
    fn test_primitive_type_detection() {
        let resolver = ProfileResolver::new();
        assert!(resolver.is_primitive_type("boolean"));
        assert!(resolver.is_primitive_type("string"));
        assert!(resolver.is_primitive_type("integer"));
        assert!(!resolver.is_primitive_type("Patient"));
        assert!(!resolver.is_primitive_type("HumanName"));
    }

    #[test]
    fn test_primitive_name_normalization() {
        let resolver = ProfileResolver::new();
        assert_eq!(resolver.normalize_primitive_name("boolean"), "Boolean");
        assert_eq!(resolver.normalize_primitive_name("integer"), "Integer");
        assert_eq!(resolver.normalize_primitive_name("dateTime"), "DateTime");
    }

    #[test]
    fn test_base_type_resolution() {
        let resolver = ProfileResolver::new();
        assert_eq!(
            resolver.get_base_type_name("Patient"),
            Some("DomainResource".to_string())
        );
        assert_eq!(
            resolver.get_base_type_name("DomainResource"),
            Some("Resource".to_string())
        );
        assert_eq!(
            resolver.get_base_type_name("HumanName"),
            Some("Element".to_string())
        );
        assert_eq!(resolver.get_base_type_name("UnknownType"), None);
    }

    #[test]
    fn test_resource_type_extraction() {
        let resolver = ProfileResolver::new();
        assert_eq!(
            resolver.extract_resource_type("http://hl7.org/fhir/StructureDefinition/Patient"),
            "Patient"
        );
        assert_eq!(
            resolver.extract_resource_type("http://example.com/profiles/CustomPatient"),
            "CustomPatient"
        );
    }

    #[test]
    fn test_cache_operations() {
        let resolver = ProfileResolver::new();

        // Test cache clearing
        resolver.clear_cache();
        let (profile_size, constraint_size) = resolver.cache_sizes();
        assert_eq!(profile_size, 0);
        assert_eq!(constraint_size, 0);
    }
}
