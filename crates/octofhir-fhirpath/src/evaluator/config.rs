//! Engine Configuration for FHIRPath Evaluation
//!
//! This module provides configuration types and defaults for the FhirPathEngine,
//! including performance settings, timeout configurations, and service endpoints.

/// Engine configuration for FhirPathEngine
///
/// Provides comprehensive configuration options for controlling FHIRPath evaluation behavior,
/// performance characteristics, and integration settings.
///
/// # Examples
///
/// ```rust
/// use octofhir_fhirpath::evaluator::EngineConfig;
///
/// // Use default configuration
/// let config = EngineConfig::default();
///
/// // Create custom configuration
/// let config = EngineConfig {
///     max_recursion_depth: 200,
///     operation_timeout_ms: 60000, // 60 seconds
///     enable_ast_cache: true,
///     max_cache_size: 5000,
///     default_terminology_server: "https://custom.tx.server/r4/".to_string(),
///     default_fhir_version: "r4".to_string(),
/// };
///
/// // Or use builder pattern methods
/// let config = EngineConfig::default()
///     .with_max_recursion_depth(150)
///     .with_operation_timeout_ms(45000)
///     .with_cache_size(2000);
/// ```
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum recursion depth for expression evaluation
    ///
    /// This limits how deeply nested expressions can be evaluated to prevent
    /// stack overflow and infinite recursion scenarios. Default: 100
    pub max_recursion_depth: usize,

    /// Timeout for individual operations (milliseconds)
    ///
    /// Sets the maximum time allowed for individual evaluation operations
    /// including model provider calls and service requests. Default: 30000ms (30 seconds)
    pub operation_timeout_ms: u64,

    /// Enable AST caching for performance
    ///
    /// When enabled, frequently used expressions are cached as parsed AST to avoid
    /// repeated parsing overhead. Recommended for production use. Default: true
    pub enable_ast_cache: bool,

    /// Maximum size of AST cache
    ///
    /// Limits the number of cached AST entries to prevent unbounded memory growth.
    /// Uses simple LRU-like eviction when limit is reached. Default: 1000
    pub max_cache_size: usize,

    /// Default terminology server URL
    ///
    /// Base URL for terminology service operations when not explicitly configured
    /// in the evaluation context. Default: "https://tx.fhir.org/r4/"
    pub default_terminology_server: String,

    /// Default FHIR version for operations
    ///
    /// FHIR version string used for terminology server URLs and type validation
    /// when not explicitly specified. Default: "r4"
    pub default_fhir_version: String,
}

impl EngineConfig {
    /// Create new configuration with all defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum recursion depth
    ///
    /// # Arguments
    /// * `depth` - Maximum recursion depth (recommended: 50-200)
    pub fn with_max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Set operation timeout in milliseconds
    ///
    /// # Arguments
    /// * `timeout_ms` - Timeout in milliseconds (recommended: 10000-60000)
    pub fn with_operation_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.operation_timeout_ms = timeout_ms;
        self
    }

    /// Enable or disable AST caching
    ///
    /// # Arguments
    /// * `enabled` - Whether to enable AST caching
    pub fn with_ast_cache(mut self, enabled: bool) -> Self {
        self.enable_ast_cache = enabled;
        self
    }

    /// Set maximum cache size
    ///
    /// # Arguments
    /// * `size` - Maximum number of cached AST entries (recommended: 100-10000)
    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Set default terminology server URL
    ///
    /// # Arguments
    /// * `url` - Terminology server base URL
    pub fn with_terminology_server(mut self, url: String) -> Self {
        self.default_terminology_server = url;
        self
    }

    /// Set default FHIR version
    ///
    /// # Arguments
    /// * `version` - FHIR version string (r4, r4b, r5)
    pub fn with_fhir_version(mut self, version: String) -> Self {
        self.default_fhir_version = version.clone();
        // Update terminology server URL to match FHIR version
        if self.default_terminology_server.contains("tx.fhir.org") {
            self.default_terminology_server = format!("https://tx.fhir.org/{}/", version);
        }
        self
    }

    /// Create high-performance configuration
    ///
    /// Optimized for high-throughput scenarios with larger caches and timeouts.
    pub fn high_performance() -> Self {
        Self {
            max_recursion_depth: 150,
            operation_timeout_ms: 60000, // 60 seconds
            enable_ast_cache: true,
            max_cache_size: 10000,
            default_terminology_server: "https://tx.fhir.org/r4/".to_string(),
            default_fhir_version: "r4".to_string(),
        }
    }

    /// Create low-latency configuration
    ///
    /// Optimized for minimal latency with smaller timeouts and moderate caching.
    pub fn low_latency() -> Self {
        Self {
            max_recursion_depth: 50,
            operation_timeout_ms: 5000, // 5 seconds
            enable_ast_cache: true,
            max_cache_size: 500,
            default_terminology_server: "https://tx.fhir.org/r4/".to_string(),
            default_fhir_version: "r4".to_string(),
        }
    }

    /// Create memory-efficient configuration
    ///
    /// Optimized for minimal memory usage with disabled caching and lower limits.
    pub fn memory_efficient() -> Self {
        Self {
            max_recursion_depth: 50,
            operation_timeout_ms: 30000,
            enable_ast_cache: false,
            max_cache_size: 0,
            default_terminology_server: "https://tx.fhir.org/r4/".to_string(),
            default_fhir_version: "r4".to_string(),
        }
    }

    /// Create testing configuration
    ///
    /// Optimized for unit tests and development with shorter timeouts and smaller caches.
    pub fn for_testing() -> Self {
        Self {
            max_recursion_depth: 20,
            operation_timeout_ms: 1000, // 1 second
            enable_ast_cache: false,    // Disable for predictable test behavior
            max_cache_size: 10,
            default_terminology_server: "https://tx.fhir.org/r4/".to_string(),
            default_fhir_version: "r4".to_string(),
        }
    }

    /// Validate configuration values
    ///
    /// Checks that configuration values are within reasonable bounds and
    /// returns warnings for potentially problematic settings.
    ///
    /// # Returns
    /// * `Vec<String>` - List of validation warnings
    pub fn validate(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.max_recursion_depth > 1000 {
            warnings.push(
                "max_recursion_depth is very high (>1000) - may cause stack overflow".to_string(),
            );
        } else if self.max_recursion_depth < 10 {
            warnings.push(
                "max_recursion_depth is very low (<10) - may fail on complex expressions"
                    .to_string(),
            );
        }

        if self.operation_timeout_ms > 300000 {
            // 5 minutes
            warnings.push(
                "operation_timeout_ms is very high (>5min) - may cause poor user experience"
                    .to_string(),
            );
        } else if self.operation_timeout_ms < 1000 {
            warnings.push(
                "operation_timeout_ms is very low (<1s) - may fail on complex operations"
                    .to_string(),
            );
        }

        if self.enable_ast_cache && self.max_cache_size > 50000 {
            warnings.push(
                "max_cache_size is very high (>50000) - may consume excessive memory".to_string(),
            );
        }

        if self.enable_ast_cache && self.max_cache_size == 0 {
            warnings.push(
                "AST cache is enabled but max_cache_size is 0 - cache will not be effective"
                    .to_string(),
            );
        }

        if !self.default_terminology_server.starts_with("http") {
            warnings
                .push("default_terminology_server does not appear to be a valid URL".to_string());
        }

        if !["r4", "r4b", "r5"].contains(&self.default_fhir_version.as_str()) {
            warnings.push(format!(
                "default_fhir_version '{}' is not a recognized FHIR version",
                self.default_fhir_version
            ));
        }

        warnings
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_recursion_depth: 100,
            operation_timeout_ms: 30000, // 30 seconds
            enable_ast_cache: true,
            max_cache_size: 1000,
            default_terminology_server: "https://tx.fhir.org/r4/".to_string(),
            default_fhir_version: "r4".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.max_recursion_depth, 100);
        assert_eq!(config.operation_timeout_ms, 30000);
        assert!(config.enable_ast_cache);
        assert_eq!(config.max_cache_size, 1000);
        assert_eq!(config.default_fhir_version, "r4");
    }

    #[test]
    fn test_builder_pattern() {
        let config = EngineConfig::default()
            .with_max_recursion_depth(150)
            .with_operation_timeout_ms(45000)
            .with_cache_size(2000);

        assert_eq!(config.max_recursion_depth, 150);
        assert_eq!(config.operation_timeout_ms, 45000);
        assert_eq!(config.max_cache_size, 2000);
    }

    #[test]
    fn test_fhir_version_updates_terminology_server() {
        let config = EngineConfig::default().with_fhir_version("r5".to_string());

        assert_eq!(config.default_fhir_version, "r5");
        assert_eq!(config.default_terminology_server, "https://tx.fhir.org/r5/");
    }

    #[test]
    fn test_preset_configurations() {
        let high_perf = EngineConfig::high_performance();
        assert_eq!(high_perf.max_recursion_depth, 150);
        assert_eq!(high_perf.max_cache_size, 10000);

        let low_latency = EngineConfig::low_latency();
        assert_eq!(low_latency.operation_timeout_ms, 5000);
        assert_eq!(low_latency.max_recursion_depth, 50);

        let memory_eff = EngineConfig::memory_efficient();
        assert!(!memory_eff.enable_ast_cache);
        assert_eq!(memory_eff.max_cache_size, 0);

        let testing = EngineConfig::for_testing();
        assert_eq!(testing.operation_timeout_ms, 1000);
        assert!(!testing.enable_ast_cache);
    }

    #[test]
    fn test_config_validation() {
        // Valid configuration
        let config = EngineConfig::default();
        let warnings = config.validate();
        assert!(warnings.is_empty());

        // High recursion depth
        let config = EngineConfig::default().with_max_recursion_depth(2000);
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.contains("very high")));

        // Low timeout
        let config = EngineConfig::default().with_operation_timeout_ms(500);
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.contains("very low")));

        // Invalid FHIR version
        let config = EngineConfig::default().with_fhir_version("invalid".to_string());
        let warnings = config.validate();
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("not a recognized FHIR version"))
        );
    }

    #[test]
    fn test_cache_validation() {
        // Cache enabled but size 0
        let mut config = EngineConfig::default();
        config.enable_ast_cache = true;
        config.max_cache_size = 0;
        let warnings = config.validate();
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("cache will not be effective"))
        );

        // Very large cache size
        let config = EngineConfig::default().with_cache_size(100000);
        let warnings = config.validate();
        assert!(warnings.iter().any(|w| w.contains("excessive memory")));
    }
}
