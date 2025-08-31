//! Configuration system for FHIRPath with Bridge Support

use crate::{FhirPathError, FhirSchemaPackageManager};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// FHIR version enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FhirVersion {
    R4,
    R4B,
    R5,
}

impl std::fmt::Display for FhirVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FhirVersion::R4 => write!(f, "R4"),
            FhirVersion::R4B => write!(f, "R4B"),
            FhirVersion::R5 => write!(f, "R5"),
        }
    }
}

impl std::str::FromStr for FhirVersion {
    type Err = FhirPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "r4" => Ok(FhirVersion::R4),
            "r4b" => Ok(FhirVersion::R4B),
            "r5" => Ok(FhirVersion::R5),
            _ => Err(FhirPathError::Generic {
                message: format!("Invalid FHIR version: {}", s),
            }),
        }
    }
}

impl Default for FhirVersion {
    fn default() -> Self {
        FhirVersion::R4
    }
}

/// Output format for CLI and evaluation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Raw,
    Pretty,
    Json,
    Table,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Raw
    }
}

/// Schema configuration for Bridge Support
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    /// FHIR version to use
    pub fhir_version: FhirVersion,
    /// FHIR packages to load (e.g., ["hl7.fhir.r4.core", "hl7.fhir.us.core"])
    pub packages: Vec<String>,
    /// Custom profiles to load
    pub custom_profiles: Vec<String>,
    /// Enable schema caching for performance
    pub cache_enabled: bool,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            fhir_version: FhirVersion::R4,
            packages: vec!["hl7.fhir.r4.core".to_string()],
            custom_profiles: vec![],
            cache_enabled: true,
        }
    }
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct FhirPathEngineConfig {
    /// Enable detailed analysis capabilities
    pub enable_analysis: bool,
    /// Enable performance tracking
    pub enable_performance_tracking: bool,
    /// Enable result caching
    pub enable_caching: bool,
    /// Output format preference
    pub output_format: OutputFormat,
    /// Enable strict mode (more validation)
    pub strict_mode: bool,
    /// Maximum evaluation depth to prevent infinite recursion
    pub max_evaluation_depth: usize,
    /// Evaluation timeout in milliseconds
    pub evaluation_timeout_ms: u64,
}

impl Default for FhirPathEngineConfig {
    fn default() -> Self {
        Self {
            enable_analysis: true,
            enable_performance_tracking: false,
            enable_caching: true,
            output_format: OutputFormat::Raw,
            strict_mode: false,
            max_evaluation_depth: 100,
            evaluation_timeout_ms: 5000,
        }
    }
}

/// Main FHIRPath configuration
#[derive(Debug, Clone)]
pub struct FhirPathConfig {
    /// Schema configuration
    pub schema_config: SchemaConfig,
    /// Engine configuration
    pub engine_config: FhirPathEngineConfig,
    /// Enable analyzer component
    pub analyzer_enabled: bool,
    /// Enable performance tracking globally
    pub performance_tracking: bool,
    /// Enable global caching
    pub caching_enabled: bool,
}

impl Default for FhirPathConfig {
    fn default() -> Self {
        Self {
            schema_config: SchemaConfig::default(),
            engine_config: FhirPathEngineConfig::default(),
            analyzer_enabled: true,
            performance_tracking: false,
            caching_enabled: true,
        }
    }
}

/// Builder for FHIRPath configuration with fluent API
pub struct FhirPathConfigBuilder {
    config: FhirPathConfig,
}

impl FhirPathConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: FhirPathConfig::default(),
        }
    }

    /// Set FHIR version
    pub fn with_fhir_version(mut self, version: FhirVersion) -> Self {
        self.config.schema_config.fhir_version = version;
        self
    }

    /// Set FHIR packages to load
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.config.schema_config.packages = packages;
        self
    }

    /// Add a single FHIR package
    pub fn add_package(mut self, package: impl Into<String>) -> Self {
        self.config.schema_config.packages.push(package.into());
        self
    }

    /// Set custom profiles
    pub fn with_custom_profiles(mut self, profiles: Vec<String>) -> Self {
        self.config.schema_config.custom_profiles = profiles;
        self
    }

    /// Enable or disable analyzer
    pub fn with_analyzer(mut self, enabled: bool) -> Self {
        self.config.analyzer_enabled = enabled;
        self
    }

    /// Enable or disable performance tracking
    pub fn with_performance_tracking(mut self, enabled: bool) -> Self {
        self.config.performance_tracking = enabled;
        self.config.engine_config.enable_performance_tracking = enabled;
        self
    }

    /// Enable or disable caching
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.config.caching_enabled = enabled;
        self.config.engine_config.enable_caching = enabled;
        self.config.schema_config.cache_enabled = enabled;
        self
    }

    /// Set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.config.engine_config.output_format = format;
        self
    }

    /// Enable or disable strict mode
    pub fn with_strict_mode(mut self, enabled: bool) -> Self {
        self.config.engine_config.strict_mode = enabled;
        self
    }

    /// Set evaluation timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.config.engine_config.evaluation_timeout_ms = timeout_ms;
        self
    }

    /// Set maximum evaluation depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.config.engine_config.max_evaluation_depth = depth;
        self
    }

    /// Build the final configuration
    pub fn build(self) -> FhirPathConfig {
        self.config
    }

    /// Build configuration and create FhirPath instance
    pub async fn create_fhirpath(self) -> Result<crate::FhirPath, FhirPathError> {
        crate::FhirPath::with_config(self.build()).await
    }
}

impl Default for FhirPathConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation result with metadata
#[derive(Debug)]
pub struct FhirPathEvaluationResult {
    /// The evaluated values
    pub values: Vec<crate::FhirPathValue>,
    /// Validation result if analyzer was enabled
    pub validation_result: Option<BridgeValidationResult>,
    /// Execution time
    pub execution_time: std::time::Duration,
    /// Warnings generated during evaluation
    pub warnings: Vec<String>,
    /// Performance metrics (if enabled)
    pub performance_metrics: Option<PerformanceMetrics>,
}

/// Performance metrics for evaluation
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Time spent parsing the expression
    pub parse_time: std::time::Duration,
    /// Time spent in evaluation
    pub evaluation_time: std::time::Duration,
    /// Time spent in analysis (if enabled)
    pub analysis_time: Option<std::time::Duration>,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses
    pub cache_misses: usize,
    /// Memory usage in bytes
    pub memory_usage: Option<usize>,
}

use octofhir_fhirpath_analyzer::BridgeValidationResult;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fhir_version_parsing() {
        assert_eq!("r4".parse::<FhirVersion>().unwrap(), FhirVersion::R4);
        assert_eq!("R4B".parse::<FhirVersion>().unwrap(), FhirVersion::R4B);
        assert_eq!("r5".parse::<FhirVersion>().unwrap(), FhirVersion::R5);
        assert!("invalid".parse::<FhirVersion>().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = FhirPathConfigBuilder::new()
            .with_fhir_version(FhirVersion::R5)
            .with_analyzer(false)
            .with_performance_tracking(true)
            .add_package("hl7.fhir.us.core")
            .build();

        assert_eq!(config.schema_config.fhir_version, FhirVersion::R5);
        assert!(!config.analyzer_enabled);
        assert!(config.performance_tracking);
        assert!(
            config
                .schema_config
                .packages
                .contains(&"hl7.fhir.us.core".to_string())
        );
    }

    #[test]
    fn test_default_config() {
        let config = FhirPathConfig::default();
        assert_eq!(config.schema_config.fhir_version, FhirVersion::R4);
        assert!(config.analyzer_enabled);
        assert!(!config.performance_tracking);
        assert!(config.caching_enabled);
    }
}
