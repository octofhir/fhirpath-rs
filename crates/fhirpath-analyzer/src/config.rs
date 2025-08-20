//! Configuration for the analyzer

use crate::types::AnalysisSettings;

/// Configuration for the analyzer
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Analysis settings
    pub settings: AnalysisSettings,
    /// Cache configuration
    pub cache_size: usize,
    /// Enable performance profiling
    pub enable_profiling: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            settings: AnalysisSettings::default(),
            cache_size: 10000,
            enable_profiling: false,
        }
    }
}

impl AnalyzerConfig {
    /// Create a new analyzer config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the cache size
    pub fn with_cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    /// Enable or disable profiling
    pub fn with_profiling(mut self, enable_profiling: bool) -> Self {
        self.enable_profiling = enable_profiling;
        self
    }

    /// Set analysis settings
    pub fn with_settings(mut self, settings: AnalysisSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Create a high-performance configuration
    pub fn high_performance() -> Self {
        Self {
            settings: AnalysisSettings {
                enable_type_inference: true,
                enable_function_validation: true,
                enable_union_analysis: true,
                max_analysis_depth: 200,
            },
            cache_size: 50000,
            enable_profiling: false,
        }
    }

    /// Create a minimal configuration for basic analysis
    pub fn minimal() -> Self {
        Self {
            settings: AnalysisSettings {
                enable_type_inference: true,
                enable_function_validation: false,
                enable_union_analysis: false,
                max_analysis_depth: 50,
            },
            cache_size: 1000,
            enable_profiling: false,
        }
    }
}
