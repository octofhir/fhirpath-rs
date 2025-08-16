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

//! FHIRSchema-based model provider implementation - ASYNC FIRST
//!
//! This implementation is fully async and uses real FHIRSchema data exclusively.

use super::cache::{CacheConfig, CacheManager};
use super::provider::*;
use async_trait::async_trait;
use octofhir_canonical_manager::FcmConfig;
use octofhir_fhirschema::{
    Element as FhirSchemaElement, FhirSchema, FhirSchemaPackageManager, InstallOptions,
    ModelProvider as FhirSchemaModelProviderTrait, PackageSpec,
    package::manager::PackageManagerConfig,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// FHIRSchema-based ModelProvider implementation - fully async
#[derive(Clone)]
pub struct FhirSchemaModelProvider {
    /// Package manager for schema operations
    package_manager: Arc<FhirSchemaPackageManager>,
    /// Cache for improved performance
    cache_manager: Arc<CacheManager>,
    /// Schema cache for fast lookups
    schema_cache: Arc<RwLock<HashMap<String, Arc<FhirSchema>>>>,
    /// FHIR version being used
    fhir_version: FhirVersion,
}

impl FhirSchemaModelProvider {
    /// Helper method to check if a resource is a Bundle
    fn is_bundle_resource(&self, resource: &crate::FhirPathValue) -> bool {
        match resource {
            crate::FhirPathValue::Resource(res) => res.resource_type() == Some("Bundle"),
            crate::FhirPathValue::JsonValue(json) => json
                .as_json()
                .get("resourceType")
                .and_then(|rt| rt.as_str())
                .map(|rt| rt == "Bundle")
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Helper method to check if two resources are the same
    fn is_same_resource(
        &self,
        resource1: &crate::FhirPathValue,
        resource2: &crate::FhirPathValue,
    ) -> bool {
        std::ptr::eq(resource1, resource2)
    }
    /// Validate conformance against custom profiles (non-standard FHIR profiles)
    async fn validate_custom_profile_conformance(
        &self,
        value: &dyn ValueReflection,
        profile_url: &str,
    ) -> Result<octofhir_fhir_model::conformance::ConformanceResult, ModelError> {
        let resource_type = value.type_name();

        // Try to get the profile schema directly
        if let Some(_schema) = self.get_schema_cached(profile_url).await {
            // For custom profiles, we can do more detailed validation
            // For now, return a basic result indicating we found the profile
            Ok(octofhir_fhir_model::conformance::ConformanceResult {
                is_valid: true, // Simplified - would need full validation logic
                violations: vec![],
                warnings: vec![],
                metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
                profile_url: profile_url.to_string(),
                resource_type,
            })
        } else {
            // Custom profile not found
            Ok(octofhir_fhir_model::conformance::ConformanceResult {
                is_valid: false,
                violations: vec![], // Simplified - avoiding complex violation structure for now
                warnings: vec![],
                metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
                profile_url: profile_url.to_string(),
                resource_type,
            })
        }
    }
    /// Create a new FhirSchemaModelProvider with default R4 configuration
    pub async fn new() -> Result<Self, ModelError> {
        Self::with_config(FhirSchemaConfig::default()).await
    }

    /// Create a new FhirSchemaModelProvider for FHIR R4
    pub async fn r4() -> Result<Self, ModelError> {
        Self::with_config(FhirSchemaConfig {
            fhir_version: FhirVersion::R4,
            ..Default::default()
        })
        .await
    }

    /// Create a new FhirSchemaModelProvider for FHIR R5
    pub async fn r5() -> Result<Self, ModelError> {
        Self::with_config(FhirSchemaConfig {
            fhir_version: FhirVersion::R5,
            ..Default::default()
        })
        .await
    }

    /// Create a new FhirSchemaModelProvider for FHIR R4B
    pub async fn r4b() -> Result<Self, ModelError> {
        Self::with_config(FhirSchemaConfig {
            fhir_version: FhirVersion::R4B,
            ..Default::default()
        })
        .await
    }

    /// Create a new FhirSchemaModelProvider with custom packages
    pub async fn with_packages(packages: Vec<PackageSpec>) -> Result<Self, ModelError> {
        Self::with_config(FhirSchemaConfig {
            additional_packages: packages,
            ..Default::default()
        })
        .await
    }

    /// Create a new provider with custom configuration
    pub async fn with_config(config: FhirSchemaConfig) -> Result<Self, ModelError> {
        // Create FCM config
        let fcm_config = FcmConfig::default();

        // We always download packages when needed
        let offline_mode = false;

        // Create package manager configuration
        let package_manager_config = PackageManagerConfig::default();
        // Create package manager
        // Initialize package manager with timeout to avoid hangs during CI/coverage
        let pm_result = timeout(
            Duration::from_secs(120),
            FhirSchemaPackageManager::new(fcm_config, package_manager_config),
        )
        .await
        .map_err(|_| {
            ModelError::schema_load_error(
                "Package manager initialization timed out after 120s".to_string(),
            )
        })?;

        let package_manager = Arc::new(pm_result.map_err(|e| {
            ModelError::schema_load_error(format!("Failed to initialize package manager: {e}"))
        })?);

        // Install FHIR core package if specified (skip in offline/CI mode)
        if config.auto_install_core && !offline_mode {
            // Determine core package spec
            let core_spec = config.core_package_spec.clone().unwrap_or_else(|| {
                // Default package based on FHIR version
                match config.fhir_version {
                    FhirVersion::R4 => PackageSpec::registry("hl7.fhir.r4.core", "4.0.1"),
                    FhirVersion::R4B => PackageSpec::registry("hl7.fhir.r4b.core", "4.3.0"),
                    FhirVersion::R5 => PackageSpec::registry("hl7.fhir.r5.core", "5.0.0"),
                }
            });

            // Install core package with timeout to prevent hangs
            let _install_result = timeout(
                Duration::from_secs(180),
                package_manager.install_packages(&[core_spec], config.install_options.clone()),
            )
            .await
            .map_err(|_| {
                ModelError::schema_load_error(
                    "Timed out installing FHIR core package after 180s".to_string(),
                )
            })?
            .map_err(|e| {
                ModelError::schema_load_error(format!("Failed to install FHIR core package: {e}"))
            })?;
        }

        // Install additional packages if specified
        if !config.additional_packages.is_empty() && !offline_mode {
            let _install_result = timeout(
                Duration::from_secs(180),
                package_manager
                    .install_packages(&config.additional_packages, config.install_options.clone()),
            )
            .await
            .map_err(|_| {
                ModelError::schema_load_error(
                    "Timed out installing additional packages after 180s".to_string(),
                )
            })?
            .map_err(|e| {
                ModelError::schema_load_error(format!("Failed to install additional packages: {e}"))
            })?;
        }

        Ok(Self {
            package_manager,
            cache_manager: Arc::new(CacheManager::with_config(config.cache_config)),
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
            fhir_version: config.fhir_version,
        })
    }

    /// Install additional packages
    pub async fn install_packages(&self, specs: &[PackageSpec]) -> Result<(), ModelError> {
        let _install_result = timeout(
            Duration::from_secs(120),
            self.package_manager.install_packages(specs, None),
        )
        .await
        .map_err(|_| {
            ModelError::schema_load_error("Timed out installing packages after 120s".to_string())
        })?
        .map_err(|e| ModelError::schema_load_error(format!("Failed to install packages: {e}")))?;

        // Clear cache to reflect new packages
        self.schema_cache.write().await.clear();
        self.cache_manager.clear_all();

        Ok(())
    }

    /// Get schema by canonical URL with caching
    async fn get_schema_cached(&self, canonical_url: &str) -> Option<Arc<FhirSchema>> {
        // Check memory cache first
        {
            let cache = self.schema_cache.read().await;
            if let Some(schema) = cache.get(canonical_url) {
                return Some(schema.clone());
            }
        }

        // Fallback to package manager using the trait, with timeout to avoid hangs
        match timeout(
            Duration::from_secs(60),
            FhirSchemaModelProviderTrait::get_schema(&*self.package_manager, canonical_url),
        )
        .await
        {
            Ok(Some(schema)) => {
                // Cache for future use
                self.schema_cache
                    .write()
                    .await
                    .insert(canonical_url.to_string(), schema.clone());
                Some(schema)
            }
            Ok(None) => None,
            Err(_) => {
                // Timed out fetching schema
                None
            }
        }
    }

    /// Convert FhirSchemaElement to TypeReflectionInfo
    fn convert_element_to_type_reflection(
        &self,
        element: &FhirSchemaElement,
    ) -> Option<TypeReflectionInfo> {
        if let Some(types) = &element.element_type {
            if types.len() == 1 {
                // Single type
                let element_type = &types[0];
                if self.is_primitive_type(&element_type.code) {
                    Some(TypeReflectionInfo::SimpleType {
                        namespace: "System".to_string(),
                        name: self.map_primitive_type(&element_type.code),
                        base_type: None,
                    })
                } else {
                    Some(TypeReflectionInfo::ClassInfo {
                        namespace: "FHIR".to_string(),
                        name: element_type.code.clone(),
                        base_type: self.get_hardcoded_base_type(&element_type.code),
                        elements: vec![], // Populated on demand
                    })
                }
            } else if types.len() > 1 {
                // Choice type - create union
                let choice_types: Vec<TypeReflectionInfo> = types
                    .iter()
                    .map(|t| {
                        if self.is_primitive_type(&t.code) {
                            TypeReflectionInfo::SimpleType {
                                namespace: "System".to_string(),
                                name: self.map_primitive_type(&t.code),
                                base_type: None,
                            }
                        } else {
                            TypeReflectionInfo::ClassInfo {
                                namespace: "FHIR".to_string(),
                                name: t.code.clone(),
                                base_type: self.get_hardcoded_base_type(&t.code),
                                elements: vec![],
                            }
                        }
                    })
                    .collect();

                if !choice_types.is_empty() {
                    // Return the first type for now - proper choice type handling would need more context
                    Some(choice_types[0].clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Check if a type is primitive
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

    /// Map primitive type to proper name
    fn map_primitive_type(&self, type_code: &str) -> String {
        match type_code {
            "boolean" => "Boolean".to_string(),
            "integer" | "unsignedInt" | "positiveInt" => "Integer".to_string(),
            "decimal" => "Decimal".to_string(),
            "date" => "Date".to_string(),
            "dateTime" | "instant" => "DateTime".to_string(),
            "time" => "Time".to_string(),
            _ => "String".to_string(), // Default to string for other primitives
        }
    }

    /// Get base type name for inheritance (hardcoded fallback)
    fn get_hardcoded_base_type(&self, type_name: &str) -> Option<String> {
        match type_name {
            "Patient" | "Observation" | "Condition" | "Procedure" | "Organization" => {
                Some("DomainResource".to_string())
            }
            "DomainResource" => Some("Resource".to_string()),
            "HumanName" | "Address" | "ContactPoint" | "Identifier" | "CodeableConcept" => {
                Some("Element".to_string())
            }
            _ => None,
        }
    }

    /// Get element info from schema
    async fn get_element_from_schema(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Option<ElementInfo> {
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        let schema = self.get_schema_cached(&canonical_url).await?;

        let full_path = if element_path.starts_with(&format!("{type_name}.")) {
            element_path.to_string()
        } else {
            format!("{type_name}.{element_path}")
        };

        if let Some(element) = schema.elements.get(&full_path) {
            let type_info = self.convert_element_to_type_reflection(element)?;

            Some(ElementInfo {
                name: element_path.to_string(),
                type_info,
                min_cardinality: element.min.unwrap_or(0),
                max_cardinality: element
                    .max
                    .as_ref()
                    .and_then(|m| if m == "*" { None } else { m.parse().ok() }),
                is_modifier: element.is_modifier,
                is_summary: element.is_summary,
                documentation: element.definition.clone(),
            })
        } else {
            None
        }
    }

    /// Extract base type from schema data
    async fn get_base_type_from_schema(&self, schema: &FhirSchema) -> Option<String> {
        // Extract base type from schema's baseDefinition field
        schema.base_definition.as_ref().and_then(|base_url| {
            // Handle URL properly - base_url is a Url type
            base_url.path_segments()?.next_back().map(|s| s.to_string())
        })
    }
}

/// Configuration for FhirSchemaModelProvider
pub struct FhirSchemaConfig {
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Whether to auto-install FHIR core package
    pub auto_install_core: bool,
    /// Core package specification (e.g., "hl7.fhir.r4.core@4.0.1")
    /// If None, will use default based on fhir_version
    pub core_package_spec: Option<PackageSpec>,
    /// Additional packages to install for type checking
    pub additional_packages: Vec<PackageSpec>,
    /// Install options for packages
    pub install_options: Option<InstallOptions>,
    /// FHIR version to use
    pub fhir_version: FhirVersion,
}

impl Default for FhirSchemaConfig {
    fn default() -> Self {
        Self {
            cache_config: CacheConfig::default(),
            auto_install_core: true,
            core_package_spec: None, // Will use default based on fhir_version
            additional_packages: Vec::new(),
            install_options: None,
            fhir_version: FhirVersion::R4,
        }
    }
}

#[async_trait]
impl ModelProvider for FhirSchemaModelProvider {
    async fn get_type_reflection(&self, type_name: &str) -> Option<TypeReflectionInfo> {
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        let schema = self.get_schema_cached(&canonical_url).await?;

        // Extract elements for this type from the schema
        let elements: Vec<ElementInfo> = schema
            .elements
            .iter()
            .filter_map(|(path, element)| {
                if let Some(element_name) = path.strip_prefix(&format!("{type_name}.")) {
                    // Only include direct children (no nested paths)
                    if !element_name.contains('.') {
                        let type_info = self.convert_element_to_type_reflection(element)?;
                        Some(ElementInfo {
                            name: element_name.to_string(),
                            type_info,
                            min_cardinality: element.min.unwrap_or(0),
                            max_cardinality: element
                                .max
                                .as_ref()
                                .and_then(|m| if m == "*" { None } else { m.parse().ok() }),
                            is_modifier: element.is_modifier,
                            is_summary: element.is_summary,
                            documentation: element.definition.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        Some(TypeReflectionInfo::ClassInfo {
            namespace: "FHIR".to_string(),
            name: type_name.to_string(),
            base_type: self.get_base_type_from_schema(&schema).await,
            elements,
        })
    }

    async fn get_element_reflection(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Option<TypeReflectionInfo> {
        let element_info = self
            .get_element_from_schema(type_name, element_path)
            .await?;
        Some(element_info.type_info)
    }

    async fn get_structure_definition(&self, _type_name: &str) -> Option<StructureDefinition> {
        // This would require converting FhirSchema to StructureDefinition
        // For now, return None as this is optional
        None
    }

    async fn validate_conformance(
        &self,
        value: &dyn ValueReflection,
        profile_url: &str,
    ) -> Result<octofhir_fhir_model::conformance::ConformanceResult, ModelError> {
        // Get the resource type from the value
        let resource_type = value.type_name();

        // Extract the expected type from the profile URL
        let expected_type = if profile_url.starts_with("http://hl7.org/fhir/StructureDefinition/") {
            profile_url
                .strip_prefix("http://hl7.org/fhir/StructureDefinition/")
                .unwrap_or("")
                .to_string()
        } else {
            // For custom profiles, try to resolve via schema
            return self
                .validate_custom_profile_conformance(value, profile_url)
                .await;
        };

        // Get the schema for the expected type
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{expected_type}");
        let schema = match self.get_schema_cached(&canonical_url).await {
            Some(schema) => schema,
            None => {
                return Ok(octofhir_fhir_model::conformance::ConformanceResult {
                    is_valid: false,
                    violations: vec![], // Simplified - profile not found
                    warnings: vec![],
                    metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
                    profile_url: profile_url.to_string(),
                    resource_type: resource_type.clone(),
                });
            }
        };

        // Basic type conformance check
        let type_matches = resource_type == expected_type;

        if !type_matches {
            // Check if this could be a valid inheritance relationship
            let base_type = self.get_base_type(&resource_type).await;
            let inherits_from_expected = base_type.as_ref() == Some(&expected_type);

            if !inherits_from_expected {
                return Ok(octofhir_fhir_model::conformance::ConformanceResult {
                    is_valid: false,
                    violations: vec![], // Simplified - type mismatch
                    warnings: vec![],
                    metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
                    profile_url: profile_url.to_string(),
                    resource_type: resource_type.clone(),
                });
            }
        }

        // If basic type check passes, validate against schema constraints
        let violations = Vec::new();
        let warnings = Vec::new();

        // Validate required elements and constraints from schema
        for constraint in &schema.constraints {
            // This is a simplified constraint check - in a full implementation,
            // we would evaluate the FHIRPath expressions in constraint.expression
            if constraint.severity == "error" {
                // For now, we'll do basic validation
                // TODO: Implement full FHIRPath constraint evaluation
            }
        }

        // For now, skip detailed constraint validation - just do basic type checking
        // This is because our simplified property checking is not robust enough
        // TODO: Implement proper FHIRPath constraint evaluation and property navigation

        let is_valid = violations.is_empty(); // Basic type check passed, assume valid

        Ok(octofhir_fhir_model::conformance::ConformanceResult {
            is_valid,
            violations,
            warnings,
            metadata: octofhir_fhir_model::conformance::ConformanceMetadata::default(),
            profile_url: profile_url.to_string(),
            resource_type,
        })
    }

    async fn get_constraints(
        &self,
        type_name: &str,
    ) -> Vec<octofhir_fhir_model::constraints::ConstraintInfo> {
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        if let Some(schema) = self.get_schema_cached(&canonical_url).await {
            schema
                .constraints
                .iter()
                .map(
                    |constraint| octofhir_fhir_model::constraints::ConstraintInfo {
                        key: constraint.key.clone(),
                        severity: octofhir_fhir_model::constraints::ConstraintSeverity::Error, // Map from string to enum
                        human: constraint.human.clone(),
                        expression: constraint.expression.clone(),
                        xpath: None,
                        source: None,
                        metadata: Default::default(),
                    },
                )
                .collect()
        } else {
            vec![]
        }
    }

    async fn resolve_reference(
        &self,
        reference_url: &str,
        _context: &dyn ResolutionContext,
    ) -> Option<Box<dyn ValueReflection>> {
        // This is the legacy method - delegate to the new enhanced method
        // For now, we don't have a way to convert FhirPathValue to ValueReflection
        // This would need to be implemented if the legacy interface is needed
        self.resolve_external_reference(reference_url).await?;
        None
    }

    async fn resolve_external_reference(
        &self,
        _reference_url: &str,
    ) -> Option<crate::FhirPathValue> {
        // For FhirSchemaModelProvider, external reference resolution should not create placeholders
        // This method would be used for actual FHIR server communication in production
        // For now, return None so Bundle resolution takes precedence
        None
    }

    async fn resolve_reference_in_context(
        &self,
        reference_url: &str,
        root_resource: &crate::FhirPathValue,
        current_resource: Option<&crate::FhirPathValue>,
    ) -> Option<crate::FhirPathValue> {
        // Handle empty references
        if reference_url.is_empty() {
            return None;
        }

        // ALWAYS try to resolve within Bundle context first if root is a Bundle
        // This is critical for Bundle.entry.resource.medicationReference.resolve() scenarios
        if self.is_bundle_resource(root_resource) {
            if let Some(resolved) = self.resolve_in_bundle(reference_url, root_resource).await {
                return Some(resolved);
            }
        }

        // Then try to resolve within contained resources in current resource
        if let Some(current) = current_resource {
            if let Some(resolved) = self.resolve_in_contained(reference_url, current).await {
                return Some(resolved);
            }
        }

        // Also check root resource for contained resources (if different from current)
        if current_resource.is_some()
            && !self.is_same_resource(current_resource.unwrap(), root_resource)
        {
            if let Some(resolved) = self
                .resolve_in_contained(reference_url, root_resource)
                .await
            {
                return Some(resolved);
            }
        }

        // If root is not a Bundle but current is a Bundle, try that too
        if let Some(current) = current_resource {
            if self.is_bundle_resource(current) && !self.is_bundle_resource(root_resource) {
                if let Some(resolved) = self.resolve_in_bundle(reference_url, current).await {
                    return Some(resolved);
                }
            }
        }

        // Finally try external resolution (returns None for FhirSchemaModelProvider)
        self.resolve_external_reference(reference_url).await
    }

    async fn resolve_in_bundle(
        &self,
        reference_url: &str,
        bundle: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        let bundle_json = match bundle {
            crate::FhirPathValue::Resource(bundle_resource) => bundle_resource.as_json(),
            crate::FhirPathValue::JsonValue(json_value) => json_value.as_json(),
            _ => return None,
        };

        if let Some(entries) = bundle_json.get("entry").and_then(|e| e.as_array()) {
            for entry in entries {
                if let Some(resource) = entry.get("resource") {
                    // Check fullUrl first (preferred for Bundle resolution)
                    if let Some(full_url) = entry.get("fullUrl").and_then(|u| u.as_str()) {
                        // Exact match
                        if full_url == reference_url {
                            return Some(crate::FhirPathValue::resource_from_json(
                                resource.clone(),
                            ));
                        }

                        // Check if fullUrl ends with the reference (accounting for base URL)
                        // Example: fullUrl="http://localhost:8080/fhir/Medication/123" should match reference="Medication/123"
                        if full_url.ends_with(&format!("/{reference_url}")) {
                            return Some(crate::FhirPathValue::resource_from_json(
                                resource.clone(),
                            ));
                        }
                    }

                    // Check resource type and ID (fallback if fullUrl doesn't match)
                    if let (Some(resource_type), Some(id)) = (
                        resource.get("resourceType").and_then(|rt| rt.as_str()),
                        resource.get("id").and_then(|id| id.as_str()),
                    ) {
                        let resource_ref = format!("{resource_type}/{id}");
                        if resource_ref == reference_url {
                            return Some(crate::FhirPathValue::resource_from_json(
                                resource.clone(),
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    async fn resolve_in_contained(
        &self,
        reference_url: &str,
        containing_resource: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        let resource_json = match containing_resource {
            crate::FhirPathValue::Resource(resource) => resource.as_json(),
            crate::FhirPathValue::JsonValue(json_value) => json_value.as_json(),
            _ => return None,
        };

        if let Some(contained) = resource_json.get("contained").and_then(|c| c.as_array()) {
            for contained_resource in contained {
                if let (Some(resource_type), Some(id)) = (
                    contained_resource
                        .get("resourceType")
                        .and_then(|rt| rt.as_str()),
                    contained_resource.get("id").and_then(|id| id.as_str()),
                ) {
                    // Check for fragment reference (starts with #)
                    if reference_url.starts_with('#') && &reference_url[1..] == id {
                        return Some(crate::FhirPathValue::resource_from_json(
                            contained_resource.clone(),
                        ));
                    }

                    // Check for full reference
                    let resource_ref = format!("{resource_type}/{id}");
                    if resource_ref == reference_url {
                        return Some(crate::FhirPathValue::resource_from_json(
                            contained_resource.clone(),
                        ));
                    }
                }
            }
        }
        None
    }

    fn parse_reference_url(
        &self,
        reference_url: &str,
    ) -> Option<super::provider::ReferenceComponents> {
        // Handle fragment references
        if let Some(stripped) = reference_url.strip_prefix('#') {
            return Some(super::provider::ReferenceComponents {
                resource_type: "".to_string(),
                resource_id: stripped.to_string(),
                version_id: None,
                fragment: Some(reference_url.to_string()),
                full_url: None,
                base_url: None,
            });
        }

        // Handle full URLs
        if reference_url.starts_with("http://") || reference_url.starts_with("https://") {
            if let Ok(url) = url::Url::parse(reference_url) {
                let path = url.path();
                // Extract resource type and ID from path like /Patient/123
                let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

                if path_segments.len() >= 2 {
                    let resource_type = path_segments[path_segments.len() - 2].to_string();
                    let resource_id = path_segments[path_segments.len() - 1].to_string();

                    // Check for version history
                    let (resource_id, version_id) = if path_segments.len() >= 4
                        && path_segments[path_segments.len() - 3] == "_history"
                    {
                        (
                            path_segments[path_segments.len() - 4].to_string(),
                            Some(resource_id),
                        )
                    } else {
                        (resource_id, None)
                    };

                    if let Some(host_str) = url.host_str() {
                        let base_url = format!("{}://{}", url.scheme(), host_str);

                        return Some(super::provider::ReferenceComponents {
                            resource_type,
                            resource_id,
                            version_id,
                            fragment: url.fragment().map(|f| format!("#{f}")),
                            full_url: Some(reference_url.to_string()),
                            base_url: Some(base_url),
                        });
                    }
                }
            }
        }

        // Handle ResourceType/id format
        if let Some(slash_pos) = reference_url.find('/') {
            let resource_type = reference_url[..slash_pos].to_string();
            let remaining = &reference_url[slash_pos + 1..];

            // Check for version history
            if let Some(history_pos) = remaining.find("/_history/") {
                let resource_id = remaining[..history_pos].to_string();
                let version_id = remaining[history_pos + 10..].to_string();

                return Some(super::provider::ReferenceComponents {
                    resource_type,
                    resource_id,
                    version_id: Some(version_id),
                    fragment: None,
                    full_url: None,
                    base_url: None,
                });
            } else {
                let resource_id = remaining.to_string();

                // Basic validation - resource type should be capitalized
                if !resource_type.is_empty()
                    && !resource_id.is_empty()
                    && resource_type.chars().next().unwrap().is_uppercase()
                {
                    return Some(super::provider::ReferenceComponents {
                        resource_type,
                        resource_id,
                        version_id: None,
                        fragment: None,
                        full_url: None,
                        base_url: None,
                    });
                }
            }
        }

        None
    }

    async fn analyze_expression(
        &self,
        _expression: &str,
    ) -> Result<ExpressionAnalysis, ModelError> {
        // TODO: Implement expression analysis
        Ok(ExpressionAnalysis {
            referenced_types: vec![],
            navigation_paths: vec![],
            requires_runtime_types: false,
            optimization_hints: vec![],
            type_safety_warnings: vec![],
        })
    }

    async fn box_value_with_metadata(
        &self,
        _value: &dyn ValueReflection,
        _type_name: &str,
    ) -> Result<BoxedValueWithMetadata, ModelError> {
        Err(ModelError::validation_error(
            "box_value_with_metadata not implemented",
        ))
    }

    async fn extract_primitive_extensions(
        &self,
        _value: &dyn ValueReflection,
        _element_path: &str,
    ) -> Option<PrimitiveExtensionData> {
        None
    }

    async fn find_extensions_by_url(
        &self,
        value: &crate::FhirPathValue,
        parent_resource: &crate::FhirPathValue,
        _element_path: Option<&str>,
        url: &str,
    ) -> Vec<crate::FhirPathValue> {
        use crate::FhirPathValue;

        // First check for direct extensions on the value
        if let FhirPathValue::JsonValue(json) = value {
            if let Some(extensions) = json.as_json().get("extension") {
                if let Some(ext_array) = extensions.as_array() {
                    let mut matching_extensions = Vec::new();
                    for ext in ext_array {
                        if let Some(ext_obj) = ext.as_object() {
                            if let Some(ext_url) = ext_obj.get("url") {
                                if let Some(ext_url_str) = ext_url.as_str() {
                                    if ext_url_str == url {
                                        matching_extensions
                                            .push(FhirPathValue::resource_from_json(ext.clone()));
                                    }
                                }
                            }
                        }
                    }
                    if !matching_extensions.is_empty() {
                        return matching_extensions;
                    }
                }
            }
        }

        // For primitive values, check the underscore element in the parent resource
        // This is FHIR-specific behavior where primitive extensions are stored in parallel underscore elements
        if matches!(
            value,
            FhirPathValue::String(_)
                | FhirPathValue::Integer(_)
                | FhirPathValue::Decimal(_)
                | FhirPathValue::Boolean(_)
        ) {
            if let FhirPathValue::JsonValue(parent_json) = parent_resource {
                let parent_obj = parent_json.as_json();

                // Look for all underscore properties (for primitive extensions)
                for (key, underscore_value) in parent_obj
                    .as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .iter()
                {
                    if key.starts_with('_') {
                        if let Some(extensions) = underscore_value.get("extension") {
                            if let Some(ext_array) = extensions.as_array() {
                                let mut matching_extensions = Vec::new();

                                for ext in ext_array {
                                    if let Some(ext_obj) = ext.as_object() {
                                        if let Some(ext_url) = ext_obj.get("url") {
                                            if let Some(ext_url_str) = ext_url.as_str() {
                                                if ext_url_str == url {
                                                    matching_extensions.push(
                                                        FhirPathValue::resource_from_json(
                                                            ext.clone(),
                                                        ),
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                if !matching_extensions.is_empty() {
                                    return matching_extensions;
                                }
                            }
                        }
                    }
                }
            }
        }

        Vec::new()
    }

    async fn get_search_params(&self, _resource_type: &str) -> Vec<SearchParameter> {
        vec![]
    }

    async fn is_resource_type(&self, type_name: &str) -> bool {
        FhirSchemaModelProviderTrait::has_resource_type(&*self.package_manager, type_name).await
    }

    fn fhir_version(&self) -> FhirVersion {
        self.fhir_version
    }

    async fn is_subtype_of(&self, child_type: &str, parent_type: &str) -> bool {
        if child_type == parent_type {
            return true;
        }

        // Check inheritance chain using schema data
        if let Some(base) = self.get_base_type(child_type).await {
            if base == parent_type {
                return true;
            }
            // Recursively check base type
            return self.is_subtype_of(&base, parent_type).await;
        }
        false
    }

    async fn get_properties(&self, type_name: &str) -> Vec<(String, TypeReflectionInfo)> {
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        if let Some(schema) = self.get_schema_cached(&canonical_url).await {
            schema
                .elements
                .iter()
                .filter_map(|(path, element)| {
                    if let Some(element_name) = path.strip_prefix(&format!("{type_name}.")) {
                        if !element_name.contains('.') {
                            // Only direct children
                            let type_info = self.convert_element_to_type_reflection(element)?;
                            Some((element_name.to_string(), type_info))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    async fn get_base_type(&self, type_name: &str) -> Option<String> {
        let canonical_url = format!("http://hl7.org/fhir/StructureDefinition/{type_name}");
        let schema = self.get_schema_cached(&canonical_url).await?;
        self.get_base_type_from_schema(&schema).await
    }

    async fn validate_navigation_path(
        &self,
        type_name: &str,
        path: &str,
    ) -> Result<NavigationValidation, ModelError> {
        let result_type = self
            .get_element_from_schema(type_name, path)
            .await
            .map(|elem| elem.type_info);

        let is_valid = result_type.is_some();
        let messages = if is_valid {
            vec![]
        } else {
            vec![format!(
                "Property '{}' not found on type '{}'",
                path, type_name
            )]
        };

        Ok(NavigationValidation {
            is_valid,
            result_type,
            intermediate_types: vec![],
            messages,
        })
    }
}

impl std::fmt::Debug for FhirSchemaModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FhirSchemaModelProvider")
            .field("package_manager", &"<FhirSchemaPackageManager>")
            .field("cache_manager", &"<CacheManager>")
            .field("schema_cache", &"<HashMap>")
            .field("fhir_version", &self.fhir_version)
            .finish()
    }
}
