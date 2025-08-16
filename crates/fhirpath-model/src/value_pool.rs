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

//! Memory pool for FHIRPath values with async-aware design
//!
//! This module provides memory pooling for FhirPathValue instances to reduce allocation
//! overhead during evaluation. The pool uses async-aware synchronization primitives
//! to prevent task interference in async environments.

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde_json::Value as JsonValue;
use std::collections::VecDeque;

use super::value::{Collection, FhirPathValue};

/// Configuration for value pools
#[derive(Debug, Clone)]
pub struct ValuePoolConfig {
    /// Maximum number of values to keep in each pool
    pub max_pool_size: usize,
    /// Initial capacity for collections
    pub initial_collection_capacity: usize,
    /// Enable pool statistics tracking
    pub enable_stats: bool,
}

impl Default for ValuePoolConfig {
    fn default() -> Self {
        Self {
            max_pool_size: 1000,
            initial_collection_capacity: 16,
            enable_stats: false,
        }
    }
}

/// Statistics for a value pool
#[derive(Debug, Default, Clone)]
pub struct ValuePoolStats {
    /// Total number of allocations requested
    pub allocations_requested: u64,
    /// Number of allocations served from pool
    pub allocations_from_pool: u64,
    /// Current pool size
    pub current_pool_size: usize,
    /// Maximum pool size reached
    pub max_pool_size_reached: usize,
    /// Number of values returned to pool
    pub values_returned: u64,
    /// Number of values that couldn't be pooled (pool full)
    pub values_dropped: u64,
}

impl ValuePoolStats {
    /// Get the cache hit ratio
    pub fn hit_ratio(&self) -> f64 {
        if self.allocations_requested == 0 {
            0.0
        } else {
            self.allocations_from_pool as f64 / self.allocations_requested as f64
        }
    }
}

/// A pool for reusing FhirPathValue instances
pub struct ValuePool<T> {
    pool: VecDeque<T>,
    config: ValuePoolConfig,
    stats: ValuePoolStats,
}

impl<T> ValuePool<T> {
    /// Create a new value pool with the given configuration
    pub fn new(config: ValuePoolConfig) -> Self {
        Self {
            pool: VecDeque::with_capacity(config.max_pool_size.min(64)),
            config,
            stats: ValuePoolStats::default(),
        }
    }

    /// Try to get a value from the pool
    pub fn get(&mut self) -> Option<T> {
        if self.config.enable_stats {
            self.stats.allocations_requested += 1;
        }

        if let Some(value) = self.pool.pop_front() {
            if self.config.enable_stats {
                self.stats.allocations_from_pool += 1;
            }
            Some(value)
        } else {
            None
        }
    }

    /// Return a value to the pool
    pub fn put(&mut self, value: T) {
        if self.pool.len() < self.config.max_pool_size {
            self.pool.push_back(value);
            if self.config.enable_stats {
                self.stats.values_returned += 1;
                self.stats.current_pool_size = self.pool.len();
                self.stats.max_pool_size_reached =
                    self.stats.max_pool_size_reached.max(self.pool.len());
            }
        } else if self.config.enable_stats {
            self.stats.values_dropped += 1;
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> ValuePoolStats {
        self.stats.clone()
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.pool.clear();
        if self.config.enable_stats {
            self.stats.current_pool_size = 0;
        }
    }

    /// Get current pool size
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

impl<T: Default> ValuePool<T> {
    /// Get a value from the pool or create a new default one
    pub fn get_or_default(&mut self) -> T {
        self.get().unwrap_or_default()
    }
}

/// Specialized pools for different value types
pub struct ValuePools {
    /// Pool for `Vec<FhirPathValue>` used in collections
    collection_vecs: ValuePool<Vec<FhirPathValue>>,
    /// Pool for String values
    strings: ValuePool<String>,
    /// Pool for JSON Values
    json_values: ValuePool<JsonValue>,
    /// Pool configuration
    config: ValuePoolConfig,
}

impl ValuePools {
    /// Create new value pools with the given configuration
    pub fn new(config: ValuePoolConfig) -> Self {
        Self {
            collection_vecs: ValuePool::new(config.clone()),
            strings: ValuePool::new(config.clone()),
            json_values: ValuePool::new(config.clone()),
            config,
        }
    }

    /// Get a pre-sized vector for collections
    pub fn get_collection_vec(&mut self) -> Vec<FhirPathValue> {
        let mut vec = self.collection_vecs.get_or_default();
        vec.clear();
        if vec.capacity() < self.config.initial_collection_capacity {
            vec.reserve(self.config.initial_collection_capacity - vec.capacity());
        }
        vec
    }

    /// Return a collection vector to the pool
    pub fn return_collection_vec(&mut self, mut vec: Vec<FhirPathValue>) {
        // Only pool vectors that aren't too large to avoid memory bloat
        if vec.capacity() <= self.config.max_pool_size * 2 {
            vec.clear();
            self.collection_vecs.put(vec);
        }
    }

    /// Get a string from the pool
    pub fn get_string(&mut self) -> String {
        self.strings.get_or_default()
    }

    /// Return a string to the pool
    pub fn return_string(&mut self, mut string: String) {
        // Only pool strings that aren't too large
        if string.capacity() <= 1024 {
            string.clear();
            self.strings.put(string);
        }
    }

    /// Get a JSON value from the pool
    pub fn get_json_value(&mut self) -> JsonValue {
        self.json_values.get_or_default()
    }

    /// Return a JSON value to the pool
    pub fn return_json_value(&mut self, value: JsonValue) {
        // Only pool simple JSON values to avoid memory bloat
        match &value {
            JsonValue::Object(obj) if obj.len() <= 10 => {
                self.json_values.put(value);
            }
            JsonValue::Array(arr) if arr.len() <= 10 => {
                self.json_values.put(value);
            }
            JsonValue::String(_) | JsonValue::Number(_) | JsonValue::Bool(_) | JsonValue::Null => {
                self.json_values.put(value);
            }
            _ => {
                // Don't pool large or complex values
            }
        }
    }

    /// Get combined statistics from all pools
    pub fn stats(&self) -> CombinedValuePoolStats {
        CombinedValuePoolStats {
            collection_vecs: self.collection_vecs.stats(),
            strings: self.strings.stats(),
            json_values: self.json_values.stats(),
        }
    }

    /// Clear all pools
    pub fn clear(&mut self) {
        self.collection_vecs.clear();
        self.strings.clear();
        self.json_values.clear();
    }
}

/// Combined statistics from all value pools
#[derive(Debug, Clone)]
pub struct CombinedValuePoolStats {
    /// Statistics for collection vector pools
    pub collection_vecs: ValuePoolStats,
    /// Statistics for string pools
    pub strings: ValuePoolStats,
    /// Statistics for JSON value pools
    pub json_values: ValuePoolStats,
}

impl CombinedValuePoolStats {
    /// Get overall hit ratio across all pools
    pub fn overall_hit_ratio(&self) -> f64 {
        let total_requested = self.collection_vecs.allocations_requested
            + self.strings.allocations_requested
            + self.json_values.allocations_requested;

        let total_from_pool = self.collection_vecs.allocations_from_pool
            + self.strings.allocations_from_pool
            + self.json_values.allocations_from_pool;

        if total_requested == 0 {
            0.0
        } else {
            total_from_pool as f64 / total_requested as f64
        }
    }
}

/// Thread-safe global value pools using parking_lot for async-friendly locking
static GLOBAL_VALUE_POOLS: Lazy<RwLock<ValuePools>> = Lazy::new(|| {
    RwLock::new(ValuePools::new(ValuePoolConfig {
        max_pool_size: 1000,
        initial_collection_capacity: 16,
        enable_stats: true,
    }))
});

/// Get a pre-sized vector for collection construction
pub fn get_pooled_collection_vec() -> Vec<FhirPathValue> {
    GLOBAL_VALUE_POOLS.write().get_collection_vec()
}

/// Return a vector to the global pool
pub fn return_pooled_collection_vec(vec: Vec<FhirPathValue>) {
    GLOBAL_VALUE_POOLS.write().return_collection_vec(vec);
}

/// Create a collection using pooled vectors when possible
pub fn create_pooled_collection(items: Vec<FhirPathValue>) -> Collection {
    // For small collections, use the provided vector directly
    if items.len() <= 8 {
        return Collection::from_vec(items);
    }

    // For larger collections, we might want to optimize further in the future
    Collection::from_vec(items)
}

/// Get a pooled string
pub fn get_pooled_string() -> String {
    GLOBAL_VALUE_POOLS.write().get_string()
}

/// Return a string to the global pool  
pub fn return_pooled_string(string: String) {
    GLOBAL_VALUE_POOLS.write().return_string(string);
}

/// Get a pooled JSON value
pub fn get_pooled_json_value() -> JsonValue {
    GLOBAL_VALUE_POOLS.write().get_json_value()
}

/// Return a JSON value to the global pool
pub fn return_pooled_json_value(value: JsonValue) {
    GLOBAL_VALUE_POOLS.write().return_json_value(value);
}

/// Configure the global value pools
pub fn configure_global_pools(config: ValuePoolConfig) {
    *GLOBAL_VALUE_POOLS.write() = ValuePools::new(config);
}

/// Get statistics from the global pools
pub fn global_pool_stats() -> CombinedValuePoolStats {
    GLOBAL_VALUE_POOLS.read().stats()
}

/// Clear all global pools (useful for testing)
pub fn clear_global_pools() {
    GLOBAL_VALUE_POOLS.write().clear();
}

/// RAII guard for automatic pool management
pub struct PooledValue<T> {
    value: Option<T>,
    return_fn: Option<Box<dyn FnOnce(T) + Send>>,
}

impl<T> PooledValue<T> {
    /// Create a new pooled value with a custom return function
    pub fn new<F>(value: T, return_fn: F) -> Self
    where
        F: FnOnce(T) + Send + 'static,
    {
        Self {
            value: Some(value),
            return_fn: Some(Box::new(return_fn)),
        }
    }

    /// Take the value out of the pool guard
    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }

    /// Get a reference to the pooled value
    pub fn as_ref(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Get a mutable reference to the pooled value
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
}

impl<T> Drop for PooledValue<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            if let Some(return_fn) = self.return_fn.take() {
                return_fn(value);
            }
        }
    }
}

/// Create a pooled collection vector with RAII cleanup
pub fn pooled_collection_vec() -> PooledValue<Vec<FhirPathValue>> {
    let vec = get_pooled_collection_vec();
    PooledValue::new(vec, return_pooled_collection_vec)
}

/// Create a pooled string with RAII cleanup
pub fn pooled_string() -> PooledValue<String> {
    let string = get_pooled_string();
    PooledValue::new(string, return_pooled_string)
}

/// Create a pooled JSON value with RAII cleanup  
pub fn pooled_json_value() -> PooledValue<JsonValue> {
    let value = get_pooled_json_value();
    PooledValue::new(value, return_pooled_json_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_pool_basic() {
        let config = ValuePoolConfig {
            max_pool_size: 5,
            initial_collection_capacity: 8,
            enable_stats: true,
        };
        let mut pool = ValuePool::<String>::new(config);

        // Pool should be empty initially
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
        assert!(pool.get().is_none());

        // Add some strings
        pool.put("test1".to_string());
        pool.put("test2".to_string());
        assert_eq!(pool.len(), 2);

        // Get strings back
        let s1 = pool.get().unwrap();
        let s2 = pool.get().unwrap();
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());

        // Strings can be in any order due to VecDeque
        assert!(s1 == "test1" || s1 == "test2");
        assert!(s2 == "test1" || s2 == "test2");
        assert_ne!(s1, s2);
    }

    #[test]
    fn test_value_pool_max_size() {
        let config = ValuePoolConfig {
            max_pool_size: 2,
            initial_collection_capacity: 8,
            enable_stats: true,
        };
        let mut pool = ValuePool::<String>::new(config);

        // Fill pool to max capacity
        pool.put("test1".to_string());
        pool.put("test2".to_string());
        assert_eq!(pool.len(), 2);

        // Adding more should not increase size
        pool.put("test3".to_string());
        assert_eq!(pool.len(), 2);

        // But stats should reflect the dropped value
        let stats = pool.stats();
        assert_eq!(stats.values_dropped, 1);
        assert_eq!(stats.values_returned, 2);
    }

    #[test]
    fn test_value_pool_stats() {
        let config = ValuePoolConfig {
            max_pool_size: 5,
            initial_collection_capacity: 8,
            enable_stats: true,
        };
        let mut pool = ValuePool::<String>::new(config);

        // Put some values
        pool.put("test1".to_string());
        pool.put("test2".to_string());

        // Get values (should come from pool)
        let _s1 = pool.get().unwrap();
        let _s2 = pool.get().unwrap();

        // Try to get another (should fail, creating cache miss)
        let s3 = pool.get();
        assert!(s3.is_none());

        let stats = pool.stats();
        assert_eq!(stats.allocations_requested, 3);
        assert_eq!(stats.allocations_from_pool, 2);
        assert_eq!(stats.hit_ratio(), 2.0 / 3.0);
    }

    #[test]
    fn test_value_pools_combined() {
        let config = ValuePoolConfig {
            max_pool_size: 5,
            initial_collection_capacity: 4,
            enable_stats: true,
        };
        let mut pools = ValuePools::new(config);

        // Test collection vec pooling
        let mut vec1 = pools.get_collection_vec();
        assert!(vec1.capacity() >= 4);
        vec1.push(FhirPathValue::Integer(42));
        pools.return_collection_vec(vec1);

        let vec2 = pools.get_collection_vec();
        assert!(vec2.is_empty());
        assert!(vec2.capacity() >= 4);

        // Test string pooling
        let mut s1 = pools.get_string();
        s1.push_str("test");
        pools.return_string(s1);

        let s2 = pools.get_string();
        assert!(s2.is_empty());

        let stats = pools.stats();
        assert!(stats.collection_vecs.allocations_requested > 0);
        assert!(stats.strings.allocations_requested > 0);
    }

    #[test]
    fn test_global_pools() {
        // Clear pools first
        clear_global_pools();

        // Test global collection vec
        let mut vec = get_pooled_collection_vec();
        vec.push(FhirPathValue::Boolean(true));
        return_pooled_collection_vec(vec);

        let vec2 = get_pooled_collection_vec();
        assert!(vec2.is_empty());

        // Test global string
        let mut s = get_pooled_string();
        s.push_str("global test");
        return_pooled_string(s);

        let s2 = get_pooled_string();
        assert!(s2.is_empty());

        let stats = global_pool_stats();
        assert!(stats.collection_vecs.allocations_requested > 0);
    }

    #[test]
    fn test_pooled_value_raii() {
        clear_global_pools();

        {
            let mut pooled = pooled_collection_vec();
            let vec = pooled.as_mut().unwrap();
            vec.push(FhirPathValue::String("test".into()));
            // pooled goes out of scope and should return vec to pool
        }

        // Get another vec - should be empty but from pool
        let vec2 = get_pooled_collection_vec();
        assert!(vec2.is_empty());

        let stats = global_pool_stats();
        assert!(stats.collection_vecs.values_returned > 0);
    }

    #[test]
    fn test_pooled_value_take() {
        clear_global_pools();

        let mut pooled = pooled_string();
        let mut string = pooled.take().unwrap();
        string.push_str("taken");

        // Manually return since we took it
        return_pooled_string(string);

        // pooled should be empty now
        assert!(pooled.as_ref().is_none());
    }

    #[test]
    fn test_create_pooled_collection() {
        let items = vec![
            FhirPathValue::Integer(1),
            FhirPathValue::Integer(2),
            FhirPathValue::Integer(3),
        ];

        let collection = create_pooled_collection(items);
        assert_eq!(collection.len(), 3);
        assert_eq!(collection.first(), Some(&FhirPathValue::Integer(1)));
    }

    #[test]
    fn test_json_value_pooling() {
        let config = ValuePoolConfig {
            max_pool_size: 5,
            initial_collection_capacity: 8,
            enable_stats: true,
        };
        let mut pools = ValuePools::new(config);

        // Test different JSON value types
        let json_obj = serde_json::json!({"key": "value"});
        let json_arr = serde_json::json!([1, 2, 3]);
        let json_str = serde_json::json!("test");
        let json_num = serde_json::json!(42);

        pools.return_json_value(json_obj);
        pools.return_json_value(json_arr);
        pools.return_json_value(json_str);
        pools.return_json_value(json_num);

        // Get values back
        let _val1 = pools.get_json_value();
        let _val2 = pools.get_json_value();
        let _val3 = pools.get_json_value();
        let _val4 = pools.get_json_value();

        // Should have gotten 4 values from pool
        let stats = pools.stats();
        assert_eq!(stats.json_values.allocations_from_pool, 4);
    }

    #[test]
    fn test_large_json_values_not_pooled() {
        let config = ValuePoolConfig {
            max_pool_size: 5,
            initial_collection_capacity: 8,
            enable_stats: true,
        };
        let mut pools = ValuePools::new(config);

        // Create a large JSON object that shouldn't be pooled
        let mut large_obj = serde_json::Map::new();
        for i in 0..20 {
            large_obj.insert(format!("key{i}"), serde_json::json!(format!("value{}", i)));
        }
        let large_json = JsonValue::Object(large_obj);

        pools.return_json_value(large_json);

        // Pool should still be empty
        let stats = pools.stats();
        assert_eq!(stats.json_values.values_returned, 0);
    }
}
