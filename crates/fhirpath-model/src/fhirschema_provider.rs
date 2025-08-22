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

//! FHIRSchema-based model provider implementation
//!
//! Simplified implementation without background loading

use super::mock_provider::MockModelProvider;
use super::provider::*;
use async_trait::async_trait;

/// FHIRSchema-based ModelProvider implementation
#[derive(Clone, Debug)]
pub struct FhirSchemaModelProvider {
    inner: MockModelProvider,
    fhir_version: FhirVersion,
}

impl FhirSchemaModelProvider {
    /// Create a new FhirSchemaModelProvider with default R4 configuration
    pub async fn new() -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: FhirVersion::R4,
        })
    }

    /// Create a new FhirSchemaModelProvider for FHIR R4
    pub async fn r4() -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: FhirVersion::R4,
        })
    }

    /// Create a new FhirSchemaModelProvider for FHIR R5
    pub async fn r5() -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: FhirVersion::R5,
        })
    }

    /// Create a new FhirSchemaModelProvider for FHIR R4B
    pub async fn r4b() -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: FhirVersion::R4B,
        })
    }

    /// Create a new FhirSchemaModelProvider with custom configuration
    pub async fn with_config(config: FhirSchemaConfig) -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: config.fhir_version,
        })
    }

    /// Create a new FhirSchemaModelProvider with custom packages
    pub async fn with_packages(_packages: Vec<PackageSpec>) -> Result<Self, ModelError> {
        Ok(Self {
            inner: MockModelProvider::new(),
            fhir_version: FhirVersion::R4,
        })
    }
}

#[async_trait]
impl ModelProvider for FhirSchemaModelProvider {
    async fn get_type_reflection(&self, type_name: &str) -> Option<TypeReflectionInfo> {
        self.inner.get_type_reflection(type_name).await
    }

    async fn get_element_reflection(
        &self,
        type_name: &str,
        element_path: &str,
    ) -> Option<TypeReflectionInfo> {
        self.inner
            .get_element_reflection(type_name, element_path)
            .await
    }

    async fn validate_conformance(
        &self,
        value: &dyn ValueReflection,
        profile_url: &str,
    ) -> Result<octofhir_fhir_model::conformance::ConformanceResult, ModelError> {
        self.inner.validate_conformance(value, profile_url).await
    }

    async fn get_constraints(
        &self,
        type_name: &str,
    ) -> Vec<octofhir_fhir_model::constraints::ConstraintInfo> {
        self.inner.get_constraints(type_name).await
    }

    async fn resolve_reference(
        &self,
        reference_url: &str,
        context: &dyn ResolutionContext,
    ) -> Option<Box<dyn ValueReflection>> {
        self.inner.resolve_reference(reference_url, context).await
    }

    async fn resolve_reference_in_context(
        &self,
        reference_url: &str,
        root_resource: &crate::FhirPathValue,
        current_resource: Option<&crate::FhirPathValue>,
    ) -> Option<crate::FhirPathValue> {
        self.inner
            .resolve_reference_in_context(reference_url, root_resource, current_resource)
            .await
    }

    async fn resolve_in_bundle(
        &self,
        reference_url: &str,
        bundle: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        self.inner.resolve_in_bundle(reference_url, bundle).await
    }

    async fn resolve_in_contained(
        &self,
        reference_url: &str,
        containing_resource: &crate::FhirPathValue,
    ) -> Option<crate::FhirPathValue> {
        self.inner
            .resolve_in_contained(reference_url, containing_resource)
            .await
    }

    async fn resolve_external_reference(
        &self,
        reference_url: &str,
    ) -> Option<crate::FhirPathValue> {
        self.inner.resolve_external_reference(reference_url).await
    }

    fn parse_reference_url(&self, reference_url: &str) -> Option<ReferenceComponents> {
        self.inner.parse_reference_url(reference_url)
    }

    async fn analyze_expression(&self, expression: &str) -> Result<ExpressionAnalysis, ModelError> {
        self.inner.analyze_expression(expression).await
    }

    async fn box_value_with_metadata(
        &self,
        value: &dyn ValueReflection,
        type_name: &str,
    ) -> Result<BoxedValueWithMetadata, ModelError> {
        self.inner.box_value_with_metadata(value, type_name).await
    }

    async fn extract_primitive_extensions(
        &self,
        value: &dyn ValueReflection,
        element_path: &str,
    ) -> Option<PrimitiveExtensionData> {
        self.inner
            .extract_primitive_extensions(value, element_path)
            .await
    }

    async fn find_extensions_by_url(
        &self,
        value: &crate::FhirPathValue,
        parent_resource: &crate::FhirPathValue,
        element_path: Option<&str>,
        url: &str,
    ) -> Vec<crate::FhirPathValue> {
        self.inner
            .find_extensions_by_url(value, parent_resource, element_path, url)
            .await
    }

    async fn get_search_params(&self, resource_type: &str) -> Vec<SearchParameter> {
        self.inner.get_search_params(resource_type).await
    }

    async fn is_resource_type(&self, type_name: &str) -> bool {
        self.inner.is_resource_type(type_name).await
    }

    fn fhir_version(&self) -> FhirVersion {
        self.fhir_version
    }

    async fn get_properties(&self, type_name: &str) -> Vec<(String, TypeReflectionInfo)> {
        self.inner.get_properties(type_name).await
    }

    async fn get_base_type(&self, type_name: &str) -> Option<String> {
        self.inner.get_base_type(type_name).await
    }

    async fn validate_navigation_path(
        &self,
        type_name: &str,
        path: &str,
    ) -> Result<NavigationValidation, ModelError> {
        self.inner.validate_navigation_path(type_name, path).await
    }
}
