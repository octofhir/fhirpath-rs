//! Test Arc JSON optimization performance

use octofhir_fhirpath::model::{ArcJsonValue, FhirResource};
use serde_json::json;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Arc JSON Optimization");
    println!("=================================");

    // Create a moderately complex JSON structure
    let json = json!({
        "resourceType": "Bundle",
        "entry": (0..1000).map(|i| json!({
            "fullUrl": format!("http://example.com/Patient/{}", i),
            "resource": {
                "resourceType": "Patient",
                "id": format!("patient-{}", i),
                "name": [{
                    "given": [format!("John{}", i)],
                    "family": format!("Doe{}", i),
                    "use": "official"
                }],
                "telecom": [{
                    "system": "email",
                    "value": format!("john{}@example.com", i),
                    "use": "home"
                }]
            }
        })).collect::<Vec<_>>()
    });

    println!("📊 Testing with {} entries in Bundle", 1000);

    // Test 1: Traditional JSON cloning
    let start = Instant::now();
    let iterations = 100;

    for _ in 0..iterations {
        let _cloned = json.clone();
    }

    let traditional_time = start.elapsed();
    println!(
        "  Traditional clone: {:.2}ms total, {:.2}μs per clone",
        traditional_time.as_millis(),
        traditional_time.as_micros() as f64 / iterations as f64
    );

    // Test 2: Arc JSON sharing
    let arc_json = ArcJsonValue::new(json.clone());
    let start = Instant::now();

    for _ in 0..iterations {
        let _shared = arc_json.clone();
    }

    let arc_time = start.elapsed();
    println!(
        "  Arc sharing: {:.2}ms total, {:.2}μs per clone",
        arc_time.as_millis(),
        arc_time.as_micros() as f64 / iterations as f64
    );

    // Test 3: FhirResource optimization
    let resource = FhirResource::from_json(json.clone());
    let start = Instant::now();

    for _ in 0..iterations {
        let _shared_resource = resource.clone();
    }

    let resource_time = start.elapsed();
    println!(
        "  FhirResource Arc: {:.2}ms total, {:.2}μs per clone",
        resource_time.as_millis(),
        resource_time.as_micros() as f64 / iterations as f64
    );

    // Calculate improvements
    let traditional_us = traditional_time.as_micros() as f64 / iterations as f64;
    let arc_us = arc_time.as_micros() as f64 / iterations as f64;
    let resource_us = resource_time.as_micros() as f64 / iterations as f64;

    println!("\n📈 Performance Improvements:");
    println!(
        "  Arc sharing vs Traditional: {:.1}x faster",
        traditional_us / arc_us
    );
    println!(
        "  FhirResource Arc vs Traditional: {:.1}x faster",
        traditional_us / resource_us
    );

    // Test property access efficiency
    println!("\n🔍 Property Access Tests:");

    // Traditional property access
    let start = Instant::now();
    for _ in 0..1000 {
        if let Some(entries) = json.get("entry") {
            if let Some(array) = entries.as_array() {
                for (i, entry) in array.iter().enumerate().take(10) {
                    if let Some(resource) = entry.get("resource") {
                        let _name = resource.get("name");
                    }
                }
            }
        }
    }
    let traditional_access = start.elapsed();

    // Arc property access
    let start = Instant::now();
    for _ in 0..1000 {
        if let Some(entries) = arc_json.get_property("entry") {
            if let Some(iter) = entries.array_iter() {
                for (i, entry) in iter.enumerate().take(10) {
                    if let Some(resource) = entry.get_property("resource") {
                        let _name = resource.get_property("name");
                    }
                }
            }
        }
    }
    let arc_access = start.elapsed();

    println!(
        "  Traditional property access: {:.2}ms",
        traditional_access.as_millis()
    );
    println!("  Arc property access: {:.2}ms", arc_access.as_millis());
    println!(
        "  Property access improvement: {:.1}x",
        traditional_access.as_micros() as f64 / arc_access.as_micros() as f64
    );

    println!("\n✅ Arc JSON optimization test completed!");

    Ok(())
}
