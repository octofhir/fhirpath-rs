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

//! Common utilities for development tools

use octofhir_fhirpath::ModelProvider;
use octofhir_fhirpath::MockModelProvider;
use octofhir_fhirschema::provider::FhirSchemaModelProvider;
use std::env;
use std::sync::Arc;

/// Create a model provider for development tools
/// This always uses FhirSchemaModelProvider for production-quality testing
/// Exits the process if FhirSchemaModelProvider cannot be initialized
pub async fn create_dev_model_provider() -> Arc<dyn ModelProvider> {
    // Allow opting into the lightweight MockModelProvider for offline/sandboxed testing
    let use_mock = env::var("FHIRPATH_USE_MOCK_PROVIDER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true") )
        .unwrap_or(false)
        || env::var("FHIRPATH_MODEL").map(|v| v.eq_ignore_ascii_case("mock")).unwrap_or(false);

    if use_mock {
        log::info!("Using MockModelProvider for development tools (env override)");
        return Arc::new(MockModelProvider::default());
    }

    let fhir_version = env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string());

    log::info!(
        "Using FhirSchemaModelProvider for development tools (FHIR version: {})",
        fhir_version
    );

    let provider = match fhir_version.to_lowercase().as_str() {
        "r4" => match FhirSchemaModelProvider::r4().await {
            Ok(provider) => provider,
            Err(e) => {
                eprintln!("❌ CRITICAL: Failed to initialize FHIR R4 schema provider: {e}");
                eprintln!("💡 This is required for proper FHIR operations.");
                eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                std::process::exit(1);
            }
        },
        "r4b" => match FhirSchemaModelProvider::r4b().await {
            Ok(provider) => provider,
            Err(e) => {
                eprintln!("❌ CRITICAL: Failed to initialize FHIR R4B schema provider: {e}");
                eprintln!("💡 This is required for proper FHIR operations.");
                eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                std::process::exit(1);
            }
        },
        "r5" => match FhirSchemaModelProvider::r5().await {
            Ok(provider) => provider,
            Err(e) => {
                eprintln!("❌ CRITICAL: Failed to initialize FHIR R5 schema provider: {e}");
                eprintln!("💡 This is required for proper FHIR operations.");
                eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                std::process::exit(1);
            }
        },
        _ => {
            log::warn!("Unknown FHIR version '{}', defaulting to R4", fhir_version);
            match FhirSchemaModelProvider::r4().await {
                Ok(provider) => provider,
                Err(e) => {
                    eprintln!(
                        "❌ CRITICAL: Failed to initialize FHIR R4 schema provider (fallback): {e}"
                    );
                    eprintln!("💡 This is required for proper FHIR operations.");
                    eprintln!("🔧 Please ensure FHIR schema data is available and try again.");
                    std::process::exit(1);
                }
            }
        }
    };

    Arc::new(provider)
}

/// Create a mock model provider specifically for unit tests
/// This should only be used in unit tests where speed is more important than accuracy
#[cfg(test)]
pub fn create_mock_provider_for_tests() -> Arc<dyn ModelProvider> {
    use octofhir_fhirpath::MockModelProvider;
    log::info!("Using MockModelProvider for unit tests only");
    Arc::new(MockModelProvider::new())
}

/// Common configuration for development tools
pub struct DevToolsConfig {
    pub fhir_version: String,
    pub timeout_seconds: u64,
    pub verbose: bool,
}

impl Default for DevToolsConfig {
    fn default() -> Self {
        Self {
            fhir_version: env::var("FHIRPATH_FHIR_VERSION").unwrap_or_else(|_| "r4".to_string()),
            timeout_seconds: env::var("FHIRPATH_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            verbose: env::var("FHIRPATH_VERBOSE").unwrap_or_default() == "1",
        }
    }
}

impl DevToolsConfig {
    /// Get the FHIR version for configuration
    pub fn fhir_version(&self) -> &str {
        &self.fhir_version
    }

    /// Get timeout in seconds
    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }

    /// Check if verbose mode is enabled
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}
