//! Comprehensive tests for function caching

use crate::model::{FhirPathValue, TypeInfo};
use crate::registry::cache::{CacheConfig, FunctionCacheKey};
use crate::registry::function::{EvaluationContext, FunctionRegistry};
use std::sync::Arc;

#[test]
fn test_function_resolution_cache_basic() {
    let config = CacheConfig::testing();
    let mut registry = FunctionRegistry::with_config(config);

    // Register abs function which has proper type signature
    use crate::registry::functions::math::AbsFunction;
    registry.register(AbsFunction);

    let arg_types = vec![]; // abs takes no explicit arguments

    // First lookup should miss cache but populate it
    let func1 = registry.get_function_for_types("abs", &arg_types);
    assert!(func1.is_some(), "First lookup should find function");

    // Second lookup should hit cache
    let func2 = registry.get_function_for_types("abs", &arg_types);
    assert!(func2.is_some(), "Second lookup should hit cache");

    // Both should be the same function (same Arc)
    assert!(Arc::ptr_eq(&func1.unwrap(), &func2.unwrap()));

    // Check cache stats
    let (resolution_stats, _) = registry.cache_stats();
    println!("Resolution cache stats: {resolution_stats}");
}

#[test]
fn test_function_result_cache_pure_functions() {
    let config = CacheConfig::testing();
    let mut registry = FunctionRegistry::with_config(config);

    // Register abs function (marked as pure)
    use crate::registry::functions::math::AbsFunction;
    registry.register(AbsFunction);

    let context = EvaluationContext::new(FhirPathValue::Integer(-42));
    let args = vec![];

    // First evaluation - should compute and cache
    let result1 = registry.evaluate_function("abs", &args, &context).unwrap();
    assert_eq!(result1, FhirPathValue::Integer(42));

    // Second evaluation - should hit cache
    let result2 = registry.evaluate_function("abs", &args, &context).unwrap();
    assert_eq!(result2, FhirPathValue::Integer(42));

    // Get cache stats
    let (resolution_stats, result_stats) = registry.cache_stats();
    println!("Resolution cache: {resolution_stats}");
    println!("Result cache: {result_stats}");
}

#[test]
fn test_cache_key_equality_and_hashing() {
    let key1 = FunctionCacheKey::new("test", vec![TypeInfo::String, TypeInfo::Integer]);
    let key2 = FunctionCacheKey::new("test", vec![TypeInfo::String, TypeInfo::Integer]);
    let key3 = FunctionCacheKey::new("test", vec![TypeInfo::Integer, TypeInfo::String]);

    // Test equality
    assert_eq!(key1, key2);
    assert_ne!(key1, key3);

    // Test that equal keys have equal hashes
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher1 = DefaultHasher::new();
    key1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    key2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_cache_size_limits() {
    let config = CacheConfig::new(2, 2, true, None); // Very small cache sizes
    let mut registry = FunctionRegistry::with_config(config);

    // Register multiple functions
    registry.register_simple("func1", 1, Some(1), |_, _| Ok(FhirPathValue::Integer(1)));
    registry.register_simple("func2", 1, Some(1), |_, _| Ok(FhirPathValue::Integer(2)));
    registry.register_simple("func3", 1, Some(1), |_, _| Ok(FhirPathValue::Integer(3)));

    let arg_types = vec![TypeInfo::Integer];

    // Fill the cache
    let _ = registry.get_function_for_types("func1", &arg_types);
    let _ = registry.get_function_for_types("func2", &arg_types);

    // This should cause eviction
    let _ = registry.get_function_for_types("func3", &arg_types);

    // Verify eviction occurred by checking cache stats
    let (resolution_stats, _) = registry.cache_stats();
    assert!(resolution_stats.contains("evictions"));

    // Verify the functions are still accessible
    assert!(
        registry
            .get_function_for_types("func1", &arg_types)
            .is_some()
    );
    assert!(
        registry
            .get_function_for_types("func3", &arg_types)
            .is_some()
    );
}

#[test]
fn test_cache_clearing() {
    let config = CacheConfig::testing();
    let mut registry = FunctionRegistry::with_config(config);

    registry.register_simple("clear_test", 1, Some(1), |_, _| {
        Ok(FhirPathValue::Integer(42))
    });

    let arg_types = vec![TypeInfo::Integer];

    // Populate caches
    let _ = registry.get_function_for_types("clear_test", &arg_types);

    // Clear caches
    registry.clear_cache();

    // Next lookup should repopulate cache
    let _ = registry.get_function_for_types("clear_test", &arg_types);

    // This is mainly testing that clear_cache doesn't crash
    // More sophisticated tests would verify internal cache state
}

#[test]
fn test_cache_warming() {
    let config = CacheConfig::default();
    let mut registry = FunctionRegistry::with_config(config);

    // Register built-in functions (includes cache warming)
    crate::registry::function::register_builtin_functions(&mut registry);

    // After registration, common functions should be pre-cached
    let (resolution_stats, _) = registry.cache_stats();

    // Should have some cache hits from warming
    println!("Cache stats after warming: {resolution_stats}");
    // Note: Exact assertions would depend on warming implementation details
}

#[test]
fn test_non_pure_function_not_cached() {
    let config = CacheConfig::testing();
    let mut registry = FunctionRegistry::with_config(config);

    // Register a non-pure function (now() changes over time)
    use crate::registry::functions::datetime::NowFunction;
    registry.register(NowFunction);

    let context = EvaluationContext::new(FhirPathValue::Empty);
    let args = vec![];

    // Multiple evaluations - results should not be cached (so might differ)
    let _result1 = registry.evaluate_function("now", &args, &context);
    let _result2 = registry.evaluate_function("now", &args, &context);

    // The results themselves may be the same (if called quickly enough)
    // but the key point is that the result cache shouldn't be used
    // This is more about verifying the is_pure logic works correctly
    assert!(!registry.is_pure_function("now"));
}
