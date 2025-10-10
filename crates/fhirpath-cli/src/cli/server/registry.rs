//! Shared FhirPathEngine registry for all FHIR versions
//!
//! This module implements the critical requirement from the task specification:
//! - Single Registry: Create ONE shared FhirPathEngine registry and reuse for all endpoints
//! - Engine Reuse: Pre-initialize engines for each FHIR version and reuse them across HTTP calls

use crate::EmbeddedModelProvider;
use crate::cli::server::{
    error::{ServerError, ServerResult},
    version::ServerFhirVersion,
};
use octofhir_fhir_model::{HttpTerminologyProvider, TerminologyProvider, provider::ModelProvider};
use octofhir_fhirpath::evaluator::FhirPathEngine;
use octofhir_fhirpath::{FunctionRegistry, create_function_registry};
use octofhir_fhirschema::create_validation_provider_from_embedded;
use papaya::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Shared registry containing pre-initialized FhirPathEngines for all FHIR versions
#[derive(Clone)]
pub struct ServerRegistry {
    /// Engines for evaluation (without analyzer)
    evaluation_engines: HashMap<ServerFhirVersion, Arc<Mutex<FhirPathEngine>>>,
    /// Shared function registry for all engines
    function_registry: Arc<FunctionRegistry>,
    /// Shared model providers for per-request engine creation
    model_providers: HashMap<ServerFhirVersion, Arc<EmbeddedModelProvider>>,
}

impl ServerRegistry {
    /// Create a new server registry with engines for all FHIR versions
    pub async fn new() -> ServerResult<Self> {
        let evaluation_engines = HashMap::new();
        let model_providers = HashMap::new();

        // Create shared function registry once
        let function_registry: Arc<FunctionRegistry> = Arc::new(create_function_registry());
        info!("✅ Created shared function registry");

        // Initialize engines for all supported FHIR versions
        for &version in ServerFhirVersion::all() {
            let start_time = std::time::Instant::now();
            info!("🔧 Initializing engines for FHIR {}", version);

            // Create model provider for this version
            let model_provider_start = std::time::Instant::now();
            let model_provider: EmbeddedModelProvider =
                create_model_provider_for_version(version).await?;
            let model_provider_arc = Arc::new(model_provider);
            let model_provider_time = model_provider_start.elapsed();
            info!(
                "📊 Model provider for {} created in {:?}",
                version, model_provider_time
            );

            // Store shared model provider for per-request engine creation
            model_providers
                .pin()
                .insert(version, model_provider_arc.clone());

            // Create evaluation engine
            let engine_start = std::time::Instant::now();
            let mut eval_engine =
                FhirPathEngine::new(function_registry.clone(), model_provider_arc.clone()).await?;

            if let Ok(validation_provider) = create_validation_provider_from_embedded(
                model_provider_arc.clone() as Arc<dyn ModelProvider + Send + Sync>,
            )
            .await
            {
                eval_engine = eval_engine.with_validation_provider(validation_provider);
            }

            if let Some(tx_provider) = create_default_terminology_provider(version) {
                eval_engine = eval_engine.with_terminology_provider(tx_provider);
            }

            let engine_time = engine_start.elapsed();
            info!("📊 Engine for {} created in {:?}", version, engine_time);

            evaluation_engines
                .pin()
                .insert(version, Arc::new(Mutex::new(eval_engine)));

            let total_time = start_time.elapsed();
            info!(
                "✅ Engine initialized for FHIR {} (total: {:?})",
                version, total_time
            );
        }

        info!(
            "🚀 Server registry initialized with {} evaluation engines",
            evaluation_engines.len()
        );

        Ok(Self {
            evaluation_engines,
            function_registry,
            model_providers,
        })
    }

    /// Get the evaluation engine for a specific FHIR version
    pub fn get_evaluation_engine(
        &self,
        version: ServerFhirVersion,
    ) -> Option<Arc<Mutex<FhirPathEngine>>> {
        self.evaluation_engines.pin().get(&version).cloned()
    }
    /// Get the number of FHIR versions supported
    pub fn version_count(&self) -> usize {
        self.evaluation_engines.pin().len()
    }

    /// Get all supported FHIR versions
    pub fn supported_versions(&self) -> Vec<ServerFhirVersion> {
        self.evaluation_engines.pin().keys().copied().collect()
    }

    /// Check if a FHIR version is supported
    pub fn supports_version(&self, version: ServerFhirVersion) -> bool {
        self.evaluation_engines.pin().contains_key(&version)
    }

    /// Check if analysis is supported for a FHIR version
    pub fn supports_analysis(&self, version: ServerFhirVersion) -> bool {
        self.evaluation_engines.pin().contains_key(&version)
    }

    /// Get model provider for a FHIR version
    pub fn get_model_provider(
        &self,
        version: ServerFhirVersion,
    ) -> Option<Arc<EmbeddedModelProvider>> {
        self.model_providers.pin().get(&version).cloned()
    }

    /// Create a new engine for the given FHIR version (per-request)
    /// Returns timing information for performance comparison
    pub async fn create_engine_for_version(
        &self,
        version: ServerFhirVersion,
    ) -> ServerResult<(FhirPathEngine, std::time::Duration)> {
        let start_time = std::time::Instant::now();

        let model_provider = self
            .model_providers
            .pin()
            .get(&version)
            .cloned()
            .ok_or_else(|| ServerError::BadRequest {
                message: format!("FHIR version {version} not supported"),
            })?;

        let mut engine =
            FhirPathEngine::new(self.function_registry.clone(), model_provider.clone()).await?;

        if let Ok(validation_provider) = create_validation_provider_from_embedded(
            model_provider.clone() as Arc<dyn ModelProvider + Send + Sync>,
        )
        .await
        {
            engine = engine.with_validation_provider(validation_provider);
        }

        if let Some(tx_provider) = create_default_terminology_provider(version) {
            engine = engine.with_terminology_provider(tx_provider);
        }

        let creation_time = start_time.elapsed();
        Ok((engine, creation_time))
    }

    /// Get shared function registry
    pub fn get_function_registry(&self) -> &Arc<FunctionRegistry> {
        &self.function_registry
    }

    /// Get shared model provider for a version
    pub fn get_model_provider(
        &self,
        version: ServerFhirVersion,
    ) -> Option<Arc<EmbeddedModelProvider>> {
        self.model_providers.pin().get(&version).cloned()
    }
}

/// Create a model provider for a specific FHIR version
async fn create_model_provider_for_version(
    version: ServerFhirVersion,
) -> ServerResult<EmbeddedModelProvider> {
    let _model_version = version.to_model_version();

    match version {
        ServerFhirVersion::R4 => Ok(crate::EmbeddedModelProvider::r4()),
        ServerFhirVersion::R4B => {
            // Use R4 for R4B as they are compatible
            Ok(crate::EmbeddedModelProvider::r4())
        }
        ServerFhirVersion::R5 => Ok(crate::EmbeddedModelProvider::r5()),
        ServerFhirVersion::R6 => {
            // R6 uses R5 schema for now since R6 is still in development
            warn!("FHIR R6 is using R5 schema as R6 is still in development");
            Ok(crate::EmbeddedModelProvider::r5())
        }
    }
}

fn create_default_terminology_provider(
    version: ServerFhirVersion,
) -> Option<Arc<dyn TerminologyProvider>> {
    let tx_path = match version {
        ServerFhirVersion::R4 => "r4",
        ServerFhirVersion::R4B => "r4b",
        ServerFhirVersion::R5 => "r5",
        ServerFhirVersion::R6 => "r6",
    };

    let tx_url = format!("https://tx.fhir.org/{tx_path}");
    match HttpTerminologyProvider::new(tx_url) {
        Ok(provider) => Some(Arc::new(provider) as Arc<dyn TerminologyProvider>),
        Err(error) => {
            warn!(
                "Failed to create default terminology provider for {}: {}",
                version, error
            );
            None
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
    }
}
