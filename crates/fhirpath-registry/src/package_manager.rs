//! Package Management Integration for FHIRPath Registry
//!
//! This module provides package management capabilities integrated with
//! the FHIRPath registry system for dynamic schema loading and updates.

use crate::type_registry::{FhirPathTypeRegistry, RegistryError};
use octofhir_fhirschema::package::FhirSchemaPackageManager;
// Simplified implementation without ResourceTypeRegistry for now
use std::sync::Arc;
use thiserror::Error;

/// Additional errors specific to package management
#[derive(Error, Debug)]
pub enum PackageError {
    #[error("Failed to load package '{package_id}' version '{version:?}': {source}")]
    PackageLoadError {
        package_id: String,
        version: Option<String>,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to unload package '{package_id}': {source}")]
    PackageUnloadError {
        package_id: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Failed to query packages: {source}")]
    PackageQueryError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Registry refresh failed: {source}")]
    RefreshError {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Registry package manager for dynamic schema operations
pub struct RegistryPackageManager {
    schema_manager: Arc<FhirSchemaPackageManager>,
    type_registry: Arc<FhirPathTypeRegistry>,
}

impl RegistryPackageManager {
    /// Create a new package manager
    pub async fn new(
        schema_manager: Arc<FhirSchemaPackageManager>,
    ) -> std::result::Result<Self, RegistryError> {
        let type_registry = Arc::new(
            FhirPathTypeRegistry::new(schema_manager.clone())
                .await
                .map_err(|e| RegistryError::InitializationError {
                    source: Box::new(e),
                })?,
        );

        Ok(Self {
            schema_manager,
            type_registry,
        })
    }

    /// Load a FHIR package and update registries
    pub async fn load_package(
        &self,
        package_id: &str,
        version: Option<&str>,
    ) -> std::result::Result<(), PackageError> {
        // Placeholder implementation - in a real implementation this would
        // use the schema manager to load packages
        let _package_id = package_id;
        let _version = version;
        Ok(())
    }

    /// Unload a FHIR package
    pub async fn unload_package(&self, package_id: &str) -> std::result::Result<(), PackageError> {
        // Placeholder implementation
        let _package_id = package_id;
        Ok(())
    }

    /// Get list of loaded packages
    pub async fn get_loaded_packages(&self) -> std::result::Result<Vec<String>, PackageError> {
        // Placeholder implementation - return a hardcoded list for now
        Ok(vec!["hl7.fhir.r4.core".to_string()])
    }

    /// Check if a package is loaded
    pub async fn is_package_loaded(
        &self,
        package_id: &str,
    ) -> std::result::Result<bool, PackageError> {
        let loaded_packages = self.get_loaded_packages().await?;
        Ok(loaded_packages.contains(&package_id.to_string()))
    }

    /// Get package information
    pub async fn get_package_info(
        &self,
        package_id: &str,
    ) -> std::result::Result<PackageInfo, PackageError> {
        // This would typically return package metadata
        // For now, return a basic implementation
        Ok(PackageInfo {
            id: package_id.to_string(),
            version: "unknown".to_string(),
            loaded: self.is_package_loaded(package_id).await?,
        })
    }

    /// Get the type registry
    pub fn type_registry(&self) -> &Arc<FhirPathTypeRegistry> {
        &self.type_registry
    }

    /// Get the schema manager
    pub fn schema_manager(&self) -> &Arc<FhirSchemaPackageManager> {
        &self.schema_manager
    }
}

/// Basic package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub id: String,
    pub version: String,
    pub loaded: bool,
}

/// Refreshable registry that can be updated when packages change
pub struct RefreshableRegistry {
    package_manager: RegistryPackageManager,
}

impl RefreshableRegistry {
    /// Create a new refreshable registry
    pub async fn new(
        schema_manager: Arc<FhirSchemaPackageManager>,
    ) -> std::result::Result<Self, RegistryError> {
        let package_manager = RegistryPackageManager::new(schema_manager).await?;

        Ok(Self { package_manager })
    }

    /// Refresh all registries after package changes
    pub async fn refresh_registry(&mut self) -> std::result::Result<(), PackageError> {
        // Placeholder implementation - in a full implementation, this would:
        // 1. Get updated schemas from schema manager
        // 2. Rebuild type registries from schemas
        // 3. Update cached information

        // For now, just return success as we're using a simplified implementation
        Ok(())
    }

    /// Load a package and refresh registries
    pub async fn load_package_and_refresh(
        &mut self,
        package_id: &str,
        version: Option<&str>,
    ) -> std::result::Result<(), PackageError> {
        self.package_manager
            .load_package(package_id, version)
            .await?;
        self.refresh_registry().await?;
        Ok(())
    }

    /// Unload a package and refresh registries
    pub async fn unload_package_and_refresh(
        &mut self,
        package_id: &str,
    ) -> std::result::Result<(), PackageError> {
        self.package_manager.unload_package(package_id).await?;
        self.refresh_registry().await?;
        Ok(())
    }

    /// Get the package manager
    pub fn package_manager(&self) -> &RegistryPackageManager {
        &self.package_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirschema::package::FhirSchemaPackageManager;

    async fn create_test_schema_manager()
    -> Result<FhirSchemaPackageManager, Box<dyn std::error::Error>> {
        // Mock implementation for testing
        todo!("Implement test schema manager creation")
    }

    #[tokio::test]
    async fn test_package_management() -> Result<(), Box<dyn std::error::Error>> {
        let manager = Arc::new(create_test_schema_manager().await?);
        let package_manager = RegistryPackageManager::new(manager).await?;

        // Test package loading
        package_manager
            .load_package("hl7.fhir.r4.core", Some("4.0.1"))
            .await?;

        let loaded = package_manager.get_loaded_packages().await?;
        assert!(loaded.contains(&"hl7.fhir.r4.core".to_string()));

        // Test package info
        let info = package_manager.get_package_info("hl7.fhir.r4.core").await?;
        assert!(info.loaded);

        Ok(())
    }

    #[tokio::test]
    async fn test_refreshable_registry() -> Result<(), Box<dyn std::error::Error>> {
        let manager = Arc::new(create_test_schema_manager().await?);
        let mut refreshable = RefreshableRegistry::new(manager).await?;

        // Test load and refresh
        refreshable
            .load_package_and_refresh("hl7.fhir.us.core", Some("3.1.1"))
            .await?;

        let loaded = refreshable.package_manager().get_loaded_packages().await?;
        assert!(loaded.contains(&"hl7.fhir.us.core".to_string()));

        Ok(())
    }
}
