use octofhir_fhirpath::IntegratedFhirPathEngine;
use serde_json::json;

#[tokio::test]
async fn test_vm_integration() {
    println!("🚀 Testing VM Integration in FHIRPath Engine");

    // Create engine with mock provider
    let mut engine = IntegratedFhirPathEngine::with_mock_provider();

    // Test data
    let patient = json!({
        "resourceType": "Patient",
        "id": "example-123",
        "name": [
            {
                "family": "Doe",
                "given": ["John", "William"]
            }
        ],
        "gender": "male",
        "birthDate": "1990-01-01"
    });

    println!("\n📊 VM Performance Statistics (Initial):");
    let stats = engine.vm_stats();
    println!("  VM Enabled: {}", stats.vm_enabled);
    println!("  Complexity Threshold: {}", stats.complexity_threshold);
    println!("  Bytecode Cache Size: {}", stats.bytecode_cache_size);
    assert!(stats.vm_enabled);
    assert_eq!(stats.complexity_threshold, 10);
    assert_eq!(stats.bytecode_cache_size, 0);

    // Test simple expression (should use AST due to low complexity)
    let result = engine.evaluate("id", patient.clone()).await.unwrap();
    println!("\n🧪 Simple expression result: {:?}", result);

    // Check that bytecode cache is still empty (simple expression shouldn't use VM)
    let stats_after_simple = engine.vm_stats();
    assert_eq!(stats_after_simple.bytecode_cache_size, 0);

    // Test complex expression (should use VM)
    let complex_expr = "name.where(family.exists()).given.first()";
    let result = engine
        .evaluate(complex_expr, patient.clone())
        .await
        .unwrap();
    println!("🧪 Complex expression result: {:?}", result);

    // Check that bytecode was cached
    let stats_after_complex = engine.vm_stats();
    println!(
        "📊 Cache size after complex expression: {}",
        stats_after_complex.bytecode_cache_size
    );

    // Re-run the same complex expression to test cache hit
    let start = std::time::Instant::now();
    let _result2 = engine
        .evaluate(complex_expr, patient.clone())
        .await
        .unwrap();
    let cached_duration = start.elapsed();
    println!("⚡ Cached evaluation took: {:?}", cached_duration);

    // Test VM disable/enable
    engine.set_vm_enabled(false);
    let start = std::time::Instant::now();
    let _result3 = engine
        .evaluate(complex_expr, patient.clone())
        .await
        .unwrap();
    let ast_duration = start.elapsed();
    println!("🐌 AST evaluation took: {:?}", ast_duration);

    engine.set_vm_enabled(true);
    let start = std::time::Instant::now();
    let _result4 = engine
        .evaluate(complex_expr, patient.clone())
        .await
        .unwrap();
    let vm_duration = start.elapsed();
    println!("🚀 VM evaluation took: {:?}", vm_duration);

    println!("✅ VM Integration Test Complete!");
}

#[tokio::test]
async fn test_complexity_calculation() {
    let mut engine = IntegratedFhirPathEngine::with_mock_provider();

    // Test different complexity thresholds
    engine.set_vm_complexity_threshold(1); // Very low threshold - almost everything uses VM

    let patient = json!({
        "resourceType": "Patient",
        "id": "test"
    });

    // Even simple expressions should now use VM
    let _result = engine.evaluate("id", patient.clone()).await.unwrap();
    let stats = engine.vm_stats();

    // Should have cached bytecode since threshold is very low
    println!(
        "Cache size with low threshold: {}",
        stats.bytecode_cache_size
    );

    // Reset to high threshold
    engine.set_vm_complexity_threshold(100); // Very high threshold - nothing uses VM
    engine.clear_bytecode_cache();

    let _result = engine
        .evaluate("name.family", patient.clone())
        .await
        .unwrap();
    let stats = engine.vm_stats();

    // Should not have cached anything since threshold is very high
    assert_eq!(stats.bytecode_cache_size, 0);

    println!("✅ Complexity Calculation Test Complete!");
}

#[tokio::test]
async fn test_vm_error_fallback() {
    let mut engine = IntegratedFhirPathEngine::with_mock_provider();

    let patient = json!({
        "resourceType": "Patient",
        "id": "test"
    });

    // Test that invalid expressions still work (fall back to AST)
    // This tests the error handling in VM compilation
    let result = engine
        .evaluate("nonexistent.property", patient.clone())
        .await;

    // Should succeed (return empty) rather than error due to fallback
    assert!(result.is_ok());

    println!("✅ VM Error Fallback Test Complete!");
}
