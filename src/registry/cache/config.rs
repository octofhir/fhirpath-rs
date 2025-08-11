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

//! Cache configuration options

use std::time::Duration;

/// Configuration for function caching behavior
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in the resolution cache
    pub resolution_cache_size: usize,

    /// Maximum number of entries in the result cache
    pub result_cache_size: usize,

    /// Whether to enable result caching for pure functions
    pub enable_result_caching: bool,

    /// Optional TTL for cache entries
    pub cache_ttl: Option<Duration>,

    /// Whether to warm the cache with common functions on startup
    pub warm_cache_on_init: bool,
}

impl CacheConfig {
    /// Create a new cache configuration with custom settings
    pub fn new(
        resolution_cache_size: usize,
        result_cache_size: usize,
        enable_result_caching: bool,
        cache_ttl: Option<Duration>,
    ) -> Self {
        Self {
            resolution_cache_size,
            result_cache_size,
            enable_result_caching,
            cache_ttl,
            warm_cache_on_init: true,
        }
    }

    /// Create a configuration optimized for high performance
    pub fn high_performance() -> Self {
        Self {
            resolution_cache_size: 10_000,
            result_cache_size: 100_000,
            enable_result_caching: true,
            cache_ttl: None, // No expiration
            warm_cache_on_init: true,
        }
    }

    /// Create a configuration optimized for low memory usage
    pub fn low_memory() -> Self {
        Self {
            resolution_cache_size: 100,
            result_cache_size: 1_000,
            enable_result_caching: true,
            cache_ttl: Some(Duration::from_secs(300)), // 5 minute TTL
            warm_cache_on_init: false,
        }
    }

    /// Create a configuration with caching disabled
    pub fn disabled() -> Self {
        Self {
            resolution_cache_size: 0,
            result_cache_size: 0,
            enable_result_caching: false,
            cache_ttl: None,
            warm_cache_on_init: false,
        }
    }

    /// Create a configuration for testing
    pub fn testing() -> Self {
        Self {
            resolution_cache_size: 100,
            result_cache_size: 100,
            enable_result_caching: true,
            cache_ttl: Some(Duration::from_millis(100)), // Very short TTL for tests
            warm_cache_on_init: false,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            resolution_cache_size: 1_000,
            result_cache_size: 10_000,
            enable_result_caching: true,
            cache_ttl: Some(Duration::from_secs(900)), // 15 minute TTL
            warm_cache_on_init: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.resolution_cache_size, 1_000);
        assert_eq!(config.result_cache_size, 10_000);
        assert!(config.enable_result_caching);
        assert_eq!(config.cache_ttl, Some(Duration::from_secs(900)));
        assert!(config.warm_cache_on_init);
    }

    #[test]
    fn test_cache_config_high_performance() {
        let config = CacheConfig::high_performance();
        assert_eq!(config.resolution_cache_size, 10_000);
        assert_eq!(config.result_cache_size, 100_000);
        assert!(config.enable_result_caching);
        assert_eq!(config.cache_ttl, None);
    }

    #[test]
    fn test_cache_config_low_memory() {
        let config = CacheConfig::low_memory();
        assert_eq!(config.resolution_cache_size, 100);
        assert_eq!(config.result_cache_size, 1_000);
        assert!(config.enable_result_caching);
        assert_eq!(config.cache_ttl, Some(Duration::from_secs(300)));
        assert!(!config.warm_cache_on_init);
    }

    #[test]
    fn test_cache_config_disabled() {
        let config = CacheConfig::disabled();
        assert_eq!(config.resolution_cache_size, 0);
        assert_eq!(config.result_cache_size, 0);
        assert!(!config.enable_result_caching);
        assert!(!config.warm_cache_on_init);
    }

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig::new(500, 5000, true, Some(Duration::from_secs(600)));
        assert_eq!(config.resolution_cache_size, 500);
        assert_eq!(config.result_cache_size, 5000);
        assert!(config.enable_result_caching);
        assert_eq!(config.cache_ttl, Some(Duration::from_secs(600)));
    }
}
