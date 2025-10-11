//! Configuration management

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// LSP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// FHIR version to use
    pub fhir_version: FhirVersion,
    /// Model provider settings
    pub model_provider: ModelProviderConfig,
    /// Terminology server settings
    pub terminology: TerminologyConfig,
    /// Feature toggles
    pub features: FeaturesConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// FHIR version
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FhirVersion {
    /// FHIR R4
    R4,
    /// FHIR R4B
    R4B,
    /// FHIR R5
    R5,
}

impl FhirVersion {
    /// Create embedded schema provider for this FHIR version
    pub fn create_embedded_provider(&self) -> octofhir_fhirschema::EmbeddedSchemaProvider {
        use octofhir_fhirschema::EmbeddedSchemaProvider;

        match self {
            FhirVersion::R4 => EmbeddedSchemaProvider::r4(),
            FhirVersion::R4B => EmbeddedSchemaProvider::r4(), // R4B uses R4 schemas
            FhirVersion::R5 => EmbeddedSchemaProvider::r5(),
        }
    }
}

/// Model provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelProviderConfig {
    /// Cache FHIR schemas in memory
    pub cache_schemas: bool,
    /// Maximum cache size in MB
    pub max_cache_size: usize,
}

/// Terminology server configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TerminologyConfig {
    /// Enable terminology validation
    pub enabled: bool,
    /// Terminology server URL
    pub server_url: Option<String>,
}

/// Feature toggles
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeaturesConfig {
    /// Enable diagnostics
    pub diagnostics: bool,
    /// Enable semantic tokens
    pub semantic_tokens: bool,
    /// Enable completion
    pub completion: bool,
    /// Enable hover
    pub hover: bool,
    /// Enable inlay hints
    pub inlay_hints: bool,
    /// Enable code actions
    pub code_actions: bool,
    /// Enable go to definition
    pub goto_definition: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Debounce delay for diagnostics in milliseconds
    pub diagnostic_debounce_ms: u64,
    /// Maximum document size to analyze in KB
    pub max_document_size_kb: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fhir_version: FhirVersion::R4, // Default to R4 as most widely used
            model_provider: ModelProviderConfig::default(),
            terminology: TerminologyConfig::default(),
            features: FeaturesConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for ModelProviderConfig {
    fn default() -> Self {
        Self {
            cache_schemas: true,
            max_cache_size: 100,
        }
    }
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        Self {
            diagnostics: true,
            semantic_tokens: true,
            completion: true,
            hover: true,
            inlay_hints: true,
            code_actions: true,
            goto_definition: true,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            diagnostic_debounce_ms: 300,
            max_document_size_kb: 1024,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        config.validate()?;

        Ok(config)
    }

    /// Find configuration file in workspace
    pub fn find_config_file(workspace_root: &Path) -> Option<PathBuf> {
        let config_path = workspace_root.join(".fhirpath-lsp.toml");
        if config_path.exists() {
            Some(config_path)
        } else {
            None
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate cache size
        if self.model_provider.max_cache_size == 0 {
            anyhow::bail!("model_provider.max_cache_size must be > 0");
        }

        // Validate diagnostic debounce
        if self.performance.diagnostic_debounce_ms == 0 {
            anyhow::bail!("performance.diagnostic_debounce_ms must be > 0");
        }

        // Validate terminology URL if enabled
        if self.terminology.enabled && self.terminology.server_url.is_none() {
            anyhow::bail!("terminology.server_url required when terminology.enabled = true");
        }

        Ok(())
    }

    /// Create a default config file template
    pub fn create_template(path: &Path) -> Result<()> {
        let template = r#"# FHIRPath LSP Server Configuration

# FHIR version to use for schema validation
fhir_version = "r5"  # Options: "r4", "r4b", "r5"

# Model provider settings
[model_provider]
# Cache FHIR schemas in memory
cache_schemas = true
# Maximum cache size (MB)
max_cache_size = 100

# Terminology server (for expansion validation)
[terminology]
enabled = false
# server_url = "https://tx.fhir.org/r5"

# LSP feature toggles
[features]
diagnostics = true
semantic_tokens = true
completion = true
hover = true
inlay_hints = true
code_actions = true
goto_definition = true

# Performance settings
[performance]
# Debounce delay for diagnostics (ms)
diagnostic_debounce_ms = 300
# Maximum document size to analyze (KB)
max_document_size_kb = 1024
"#;

        std::fs::write(path, template)
            .with_context(|| format!("Failed to write config template: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.fhir_version, FhirVersion::R4); // Default to R4
        assert!(config.features.diagnostics);
        assert!(config.model_provider.cache_schemas);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Invalid cache size
        config.model_provider.max_cache_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_parse() {
        let toml_str = r#"
            fhir_version = "r4"
            [model_provider]
            cache_schemas = false
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.fhir_version, FhirVersion::R4);
        assert!(!config.model_provider.cache_schemas);
    }
}
