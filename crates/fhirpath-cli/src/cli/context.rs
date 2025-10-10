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

//! Shared CLI context and configuration utilities

use crate::cli::output::OutputFormat;
use std::sync::Arc;

/// Shared context for CLI commands containing common configuration
#[derive(Debug, Clone)]
pub struct CliContext {
    /// Output format for results
    pub output_format: OutputFormat,
    /// Disable colored output
    pub no_color: bool,
    /// Suppress informational messages
    pub quiet: bool,
    /// Verbose output with additional details
    pub verbose: bool,
    /// FHIR version to use
    pub fhir_version: String,
    /// Additional FHIR packages to load
    pub packages: Vec<String>,
    /// Enable performance profiling
    pub profile: bool,
    /// Template string for custom output formatting
    pub template: Option<String>,
}

impl CliContext {
    /// Create a new CLI context from global options
    pub fn new(
        output_format: OutputFormat,
        no_color: bool,
        quiet: bool,
        verbose: bool,
        fhir_version: String,
        packages: Vec<String>,
        profile: bool,
    ) -> Self {
        Self {
            output_format,
            no_color,
            quiet,
            verbose,
            fhir_version,
            packages,
            profile,
            template: None,
        }
    }

    /// Create context from CLI struct
    pub fn from_cli(cli: &crate::cli::Cli) -> Self {
        Self::new(
            cli.output_format.clone(),
            cli.no_color,
            cli.quiet,
            cli.verbose,
            cli.fhir_version.clone(),
            cli.packages.clone(),
            false, // profile flag is command-specific
        )
    }

    /// Merge subcommand options with global context
    pub fn with_subcommand_options(
        &self,
        output_format: Option<OutputFormat>,
        no_color: bool,
        quiet: bool,
        verbose: bool,
    ) -> Self {
        Self {
            output_format: output_format.unwrap_or_else(|| self.output_format.clone()),
            no_color: no_color || self.no_color,
            quiet: quiet || self.quiet,
            verbose: verbose || self.verbose,
            fhir_version: self.fhir_version.clone(),
            packages: self.packages.clone(),
            profile: self.profile,
            template: self.template.clone(),
        }
    }

    /// Set template string for output formatting
    pub fn with_template(mut self, template: Option<String>) -> Self {
        self.template = template;
        self
    }

    /// Enable performance profiling
    pub fn with_profile(mut self, profile: bool) -> Self {
        self.profile = profile;
        self
    }

    /// Check if colors should be enabled
    pub fn use_colors(&self) -> bool {
        !self.no_color
            && std::env::var("NO_COLOR").is_err()
            && std::env::var("FHIRPATH_NO_COLOR").is_err()
    }

    /// Create a formatter for this context
    pub fn create_formatter(&self) -> Box<dyn crate::cli::output::OutputFormatter> {
        use crate::cli::output::FormatterFactory;
        let factory = FormatterFactory::new(self.no_color);
        factory.create_formatter(self.output_format.clone(), self.template.clone())
    }
}

/// Builder for creating FhirPathEngine with common settings
pub struct EngineBuilder {
    model_provider: Option<Arc<crate::EmbeddedModelProvider>>,
    enable_validation: bool,
    enable_terminology: bool,
    enable_trace: bool,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            model_provider: None,
            enable_validation: true,
            enable_terminology: true,
            enable_trace: true,
        }
    }

    pub fn with_model_provider(mut self, provider: Arc<crate::EmbeddedModelProvider>) -> Self {
        self.model_provider = Some(provider);
        self
    }

    pub fn with_validation(mut self, enable: bool) -> Self {
        self.enable_validation = enable;
        self
    }

    pub fn with_terminology(mut self, enable: bool) -> Self {
        self.enable_terminology = enable;
        self
    }

    pub fn with_trace(mut self, enable: bool) -> Self {
        self.enable_trace = enable;
        self
    }

    pub async fn build(self) -> anyhow::Result<octofhir_fhirpath::evaluator::FhirPathEngine> {
        use octofhir_fhir_model::HttpTerminologyProvider;
        use octofhir_fhirpath::core::trace::create_cli_provider;
        use octofhir_fhirpath::create_function_registry;
        use octofhir_fhirschema::create_validation_provider_from_embedded;

        let provider = self
            .model_provider
            .ok_or_else(|| anyhow::anyhow!("Model provider is required"))?;

        let registry = Arc::new(create_function_registry());
        let mut engine =
            octofhir_fhirpath::evaluator::FhirPathEngine::new(registry, provider.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create FhirPath engine: {e}"))?;

        // Add trace provider
        if self.enable_trace {
            let trace_provider = create_cli_provider();
            engine = engine.with_trace_provider(trace_provider);
        }

        // Add validation provider
        if self.enable_validation
            && let Ok(validation_provider) = create_validation_provider_from_embedded(
                provider.clone() as Arc<dyn octofhir_fhir_model::provider::ModelProvider>,
            )
            .await
        {
            engine = engine.with_validation_provider(validation_provider);
        }

        // Add terminology provider
        if self.enable_terminology {
            let tx_path = match provider.get_fhir_version().await {
                Ok(octofhir_fhir_model::provider::FhirVersion::R4) => "r4",
                Ok(octofhir_fhir_model::provider::FhirVersion::R4B) => "r4b",
                Ok(octofhir_fhir_model::provider::FhirVersion::R5) => "r5",
                Ok(octofhir_fhir_model::provider::FhirVersion::R6) => "r6",
                _ => "r4",
            };
            let tx_url = format!("https://tx.fhir.org/{tx_path}");
            if let Ok(tx) = HttpTerminologyProvider::new(tx_url) {
                let tx_arc: Arc<dyn octofhir_fhir_model::terminology::TerminologyProvider> =
                    Arc::new(tx);
                engine = engine.with_terminology_provider(tx_arc);
            }
        }

        Ok(engine)
    }
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}
