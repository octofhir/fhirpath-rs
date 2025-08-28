//! Shared FhirPathEngine registry for all FHIR versions
//!
//! This module implements the critical requirement from the task specification:
//! - Single Registry: Create ONE shared FhirPathEngine registry and reuse for all endpoints
//! - Engine Reuse: Pre-initialize engines for each FHIR version and reuse them across HTTP calls

use crate::FhirPathEngineWithAnalyzer;
use crate::cli::server::{error::ServerResult, version::ServerFhirVersion};
use octofhir_fhirpath_evaluator::FhirPathEngine;
use octofhir_fhirpath_model::{
    fhirschema_provider::FhirSchemaModelProvider, provider::ModelProvider,
};
use octofhir_fhirpath_registry::create_standard_registry;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Shared registry containing pre-initialized FhirPathEngines for all FHIR versions
#[derive(Clone)]
pub struct ServerRegistry {
    /// Engines for evaluation (without analyzer)
    evaluation_engines: HashMap<ServerFhirVersion, Arc<FhirPathEngine>>,
    /// Engines with analyzer for analysis operations
    analysis_engines: HashMap<ServerFhirVersion, Arc<FhirPathEngineWithAnalyzer>>,
}

impl ServerRegistry {
    /// Create a new server registry with engines for all FHIR versions
    pub async fn new() -> ServerResult<Self> {
        let mut evaluation_engines = HashMap::new();
        let mut analysis_engines = HashMap::new();

        // Create shared function registry once
        let function_registry = Arc::new(create_standard_registry().await);
        info!("✅ Created shared function registry");

        // Initialize engines for all supported FHIR versions
        for &version in ServerFhirVersion::all() {
            info!("🔧 Initializing engines for FHIR {}", version);

            // Create model provider for this version
            let model_provider: FhirSchemaModelProvider =
                create_model_provider_for_version(version).await?;
            let model_provider_arc = Arc::new(model_provider);

            // Create evaluation engine
            let eval_engine =
                FhirPathEngine::new(function_registry.clone(), model_provider_arc.clone());
            evaluation_engines.insert(version, Arc::new(eval_engine));

            // Create analysis engine
            match FhirPathEngineWithAnalyzer::with_full_analysis(
                Box::new((*model_provider_arc).clone()),
                function_registry.clone(),
            )
            .await
            {
                Ok(analyzer_engine) => {
                    analysis_engines.insert(version, Arc::new(analyzer_engine));
                    info!("✅ Engines initialized for FHIR {}", version);
                }
                Err(e) => {
                    error!(
                        "❌ Failed to create analyzer engine for FHIR {}: {}",
                        version, e
                    );
                    warn!(
                        "🔄 Analysis operations will be unavailable for FHIR {}",
                        version
                    );
                }
            }
        }

        info!(
            "🚀 Server registry initialized with {} evaluation engines and {} analysis engines",
            evaluation_engines.len(),
            analysis_engines.len()
        );

        Ok(Self {
            evaluation_engines,
            analysis_engines,
        })
    }

    /// Get the evaluation engine for a specific FHIR version
    pub fn get_evaluation_engine(&self, version: ServerFhirVersion) -> Option<Arc<FhirPathEngine>> {
        self.evaluation_engines.get(&version).cloned()
    }

    /// Get the analysis engine for a specific FHIR version
    pub fn get_analysis_engine(
        &self,
        version: ServerFhirVersion,
    ) -> Option<Arc<FhirPathEngineWithAnalyzer>> {
        self.analysis_engines.get(&version).cloned()
    }

    /// Get the number of FHIR versions supported
    pub fn version_count(&self) -> usize {
        self.evaluation_engines.len()
    }

    /// Get all supported FHIR versions
    pub fn supported_versions(&self) -> Vec<ServerFhirVersion> {
        self.evaluation_engines.keys().copied().collect()
    }

    /// Check if a FHIR version is supported
    pub fn supports_version(&self, version: ServerFhirVersion) -> bool {
        self.evaluation_engines.contains_key(&version)
    }

    /// Check if analysis is available for a FHIR version
    pub fn supports_analysis(&self, version: ServerFhirVersion) -> bool {
        self.analysis_engines.contains_key(&version)
    }
}

/// Create a model provider for a specific FHIR version
async fn create_model_provider_for_version(
    version: ServerFhirVersion,
) -> ServerResult<FhirSchemaModelProvider> {
    let _model_version = version.to_model_version();

    match version {
        ServerFhirVersion::R4 => FhirSchemaModelProvider::r4().await.map_err(|e| {
            error!("Failed to create R4 model provider: {}", e);
            e.into()
        }),
        ServerFhirVersion::R4B => FhirSchemaModelProvider::r4b().await.map_err(|e| {
            error!("Failed to create R4B model provider: {}", e);
            e.into()
        }),
        ServerFhirVersion::R5 => FhirSchemaModelProvider::r5().await.map_err(|e| {
            error!("Failed to create R5 model provider: {}", e);
            e.into()
        }),
        ServerFhirVersion::R6 => {
            // R6 uses R5 schema for now since R6 is still in development
            warn!("FHIR R6 is using R5 schema as R6 is still in development");
            FhirSchemaModelProvider::r5().await.map_err(|e| {
                error!("Failed to create R6 (R5 schema) model provider: {}", e);
                e.into()
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = ServerRegistry::new().await;
        assert!(registry.is_ok());

        let registry = registry.unwrap();
        assert!(registry.version_count() > 0);

        // Test that we have engines for major FHIR versions
        assert!(registry.supports_version(ServerFhirVersion::R4));
        assert!(registry.supports_version(ServerFhirVersion::R5));
    }

    #[tokio::test]
    async fn test_engine_retrieval() {
        let registry = ServerRegistry::new().await.unwrap();

        // Test evaluation engine retrieval
        let r4_engine = registry.get_evaluation_engine(ServerFhirVersion::R4);
        assert!(r4_engine.is_some());

        // Test analysis engine retrieval
        let r4_analyzer = registry.get_analysis_engine(ServerFhirVersion::R4);
        assert!(r4_analyzer.is_some());
    }
}
