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

//! Shared AST compilation cache for FHIRPath expressions
//!
//! This module provides a thread-safe cache for parsed AST nodes, eliminating
//! the need to re-parse identical expressions across multiple engine instances
//! or evaluation calls.

use dashmap::DashMap;
use octofhir_fhirpath_ast::ExpressionNode;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Shared AST that can be safely cloned across threads
pub type SharedAst = Arc<ExpressionNode>;

/// Statistics about the AST cache performance
#[derive(Debug, Clone)]
pub struct AstCacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of entries currently cached
    pub entries: usize,
    /// Number of entries evicted due to memory pressure
    pub evictions: u64,
}

impl AstCacheStats {
    /// Calculate cache hit rate as a percentage
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            (self.hits as f64) / ((self.hits + self.misses) as f64) * 100.0
        }
    }
}

/// Configuration for the AST cache
#[derive(Debug, Clone)]
pub struct AstCacheConfig {
    /// Maximum number of entries to cache
    pub max_entries: usize,
    /// Whether the cache is enabled
    pub enabled: bool,
    /// TTL for cache entries (None = never expire)
    pub entry_ttl: Option<Duration>,
    /// Minimum expression length to cache (avoid caching very simple expressions)
    pub min_expression_length: usize,
}

impl Default for AstCacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            enabled: true,
            entry_ttl: Some(Duration::from_secs(3600)), // 1 hour
            min_expression_length: 5,
        }
    }
}

/// Cache entry with metadata
#[derive(Debug)]
struct CacheEntry {
    ast: SharedAst,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
}

impl CacheEntry {
    fn new(ast: SharedAst) -> Self {
        let now = Instant::now();
        Self {
            ast,
            created_at: now,
            last_accessed: now,
            access_count: 1,
        }
    }

    fn access(&mut self) -> SharedAst {
        self.last_accessed = Instant::now();
        self.access_count += 1;
        Arc::clone(&self.ast)
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}

/// Thread-safe AST cache with LRU eviction and memory pressure monitoring
pub struct AstCache {
    /// The actual cache storage
    cache: DashMap<String, CacheEntry>,
    /// Cache configuration
    config: AstCacheConfig,
    /// Performance statistics
    hits: std::sync::atomic::AtomicU64,
    misses: std::sync::atomic::AtomicU64,
    evictions: std::sync::atomic::AtomicU64,
}

impl AstCache {
    /// Create a new AST cache with default configuration
    pub fn new() -> Self {
        Self::with_config(AstCacheConfig::default())
    }

    /// Create a new AST cache with custom configuration
    pub fn with_config(config: AstCacheConfig) -> Self {
        Self {
            cache: DashMap::new(),
            config,
            hits: std::sync::atomic::AtomicU64::new(0),
            misses: std::sync::atomic::AtomicU64::new(0),
            evictions: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Get a cached AST or None if not found
    pub fn get(&self, expression: &str) -> Option<SharedAst> {
        if !self.config.enabled {
            return None;
        }

        let key = self.normalize_expression(expression);

        if let Some(mut entry) = self.cache.get_mut(&key) {
            // Check if entry is expired
            if let Some(ttl) = self.config.entry_ttl {
                if entry.is_expired(ttl) {
                    drop(entry);
                    self.cache.remove(&key);
                    self.misses
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    return None;
                }
            }

            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(entry.access())
        } else {
            self.misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Cache an AST for future use
    pub fn put(&self, expression: &str, ast: ExpressionNode) {
        if !self.config.enabled || expression.len() < self.config.min_expression_length {
            return;
        }

        let key = self.normalize_expression(expression);
        let shared_ast = Arc::new(ast);
        let entry = CacheEntry::new(shared_ast);

        // Check if we need to evict entries due to size limits
        if self.cache.len() >= self.config.max_entries {
            self.evict_lru_entries();
        }

        self.cache.insert(key, entry);
    }

    /// Get cache statistics
    pub fn stats(&self) -> AstCacheStats {
        AstCacheStats {
            hits: self.hits.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.misses.load(std::sync::atomic::Ordering::Relaxed),
            entries: self.cache.len(),
            evictions: self.evictions.load(std::sync::atomic::Ordering::Relaxed),
        }
    }

    /// Clear all cached entries
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Perform cleanup of expired entries
    pub fn cleanup(&self) {
        if let Some(ttl) = self.config.entry_ttl {
            let expired_keys: Vec<String> = self
                .cache
                .iter()
                .filter_map(|entry| {
                    if entry.value().is_expired(ttl) {
                        Some(entry.key().clone())
                    } else {
                        None
                    }
                })
                .collect();

            for key in expired_keys {
                self.cache.remove(&key);
            }
        }
    }

    /// Normalize expression for consistent caching (remove extra whitespace, etc.)
    fn normalize_expression(&self, expression: &str) -> String {
        // More sophisticated normalization that handles operators and dots
        let mut result = String::new();
        let chars: Vec<char> = expression.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let ch = chars[i];

            if ch.is_whitespace() {
                // Skip consecutive whitespace, but preserve single spaces around operators
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                // Only add space if we're not at the beginning/end and not around dots
                if !result.is_empty() && i < chars.len() {
                    let last_char = result.chars().last().unwrap_or(' ');
                    let next_char = chars[i];

                    if last_char != '.' && next_char != '.' && last_char != '[' && next_char != ']'
                    {
                        result.push(' ');
                    }
                }
            } else {
                result.push(ch);
                i += 1;
            }
        }

        result.trim().to_string()
    }

    /// Evict least recently used entries when cache is full
    fn evict_lru_entries(&self) {
        // Collect entries with their last access times
        let mut entries: Vec<(String, Instant)> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_accessed))
            .collect();

        // Sort by last accessed time (oldest first)
        entries.sort_by_key(|(_, last_accessed)| *last_accessed);

        // Remove the oldest 20% of entries
        let remove_count = (self.config.max_entries as f64 * 0.2) as usize;
        for (key, _) in entries.into_iter().take(remove_count) {
            self.cache.remove(&key);
            self.evictions
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
}

impl Default for AstCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global shared AST cache instance
static GLOBAL_AST_CACHE: once_cell::sync::Lazy<AstCache> =
    once_cell::sync::Lazy::new(AstCache::new);

/// Get a reference to the global AST cache
pub fn global_ast_cache() -> &'static AstCache {
    &GLOBAL_AST_CACHE
}

/// Cache an AST in the global cache
pub fn cache_ast(expression: &str, ast: ExpressionNode) {
    global_ast_cache().put(expression, ast);
}

/// Get a cached AST from the global cache
pub fn get_cached_ast(expression: &str) -> Option<SharedAst> {
    global_ast_cache().get(expression)
}

/// Get statistics from the global AST cache
pub fn global_ast_cache_stats() -> AstCacheStats {
    global_ast_cache().stats()
}

#[cfg(test)]
mod tests {
    use super::*;
    use octofhir_fhirpath_ast::{ExpressionNode, LiteralValue};

    fn create_test_ast(value: &str) -> ExpressionNode {
        ExpressionNode::Literal(LiteralValue::String(value.to_string()))
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = AstCache::new();
        let expression = "Patient.name";
        let ast = create_test_ast("test");

        // Initially should be empty
        assert!(cache.get(expression).is_none());

        // Cache the AST
        cache.put(expression, ast);

        // Should now be cached
        let cached = cache.get(expression);
        assert!(cached.is_some());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn test_expression_normalization() {
        let cache = AstCache::new();
        let ast = create_test_ast("test");

        // Cache with extra spaces
        cache.put("  Patient . name  ", ast);

        // Should hit cache even with different spacing
        let cached = cache.get("Patient.name");
        assert!(cached.is_some());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_cache_eviction() {
        let config = AstCacheConfig {
            max_entries: 5,
            enabled: true,
            entry_ttl: None,
            min_expression_length: 1,
        };
        let cache = AstCache::with_config(config);

        // Fill cache beyond capacity
        for i in 0..10 {
            let expr = format!("Patient.name{i}");
            let ast = create_test_ast(&format!("test{i}"));
            cache.put(&expr, ast);
        }

        let stats = cache.stats();
        assert!(stats.entries <= 5);
        assert!(stats.evictions > 0);
    }

    #[test]
    fn test_cache_disabled() {
        let config = AstCacheConfig {
            enabled: false,
            ..Default::default()
        };
        let cache = AstCache::with_config(config);

        let ast = create_test_ast("test");
        cache.put("Patient.name", ast);

        // Should not cache when disabled
        assert!(cache.get("Patient.name").is_none());
        assert_eq!(cache.stats().entries, 0);
    }

    #[test]
    fn test_min_expression_length() {
        let config = AstCacheConfig {
            min_expression_length: 10,
            ..Default::default()
        };
        let cache = AstCache::with_config(config);

        let ast = create_test_ast("test");

        // Short expression should not be cached
        cache.put("id", ast.clone());
        assert!(cache.get("id").is_none());

        // Long expression should be cached
        cache.put("Patient.name.given", ast);
        assert!(cache.get("Patient.name.given").is_some());
    }

    #[test]
    fn test_hit_rate_calculation() {
        let cache = AstCache::new();
        let ast = create_test_ast("test");

        cache.put("Patient.name", ast);

        // 2 misses, 3 hits
        cache.get("nonexistent1");
        cache.get("nonexistent2");
        cache.get("Patient.name");
        cache.get("Patient.name");
        cache.get("Patient.name");

        let stats = cache.stats();
        assert_eq!(stats.hit_rate(), 60.0); // 3 hits out of 5 total
    }

    #[test]
    fn test_global_cache() {
        let ast = create_test_ast("global_test");

        cache_ast("global.expression", ast);
        let cached = get_cached_ast("global.expression");

        assert!(cached.is_some());

        let stats = global_ast_cache_stats();
        assert!(stats.entries > 0);
    }
}
