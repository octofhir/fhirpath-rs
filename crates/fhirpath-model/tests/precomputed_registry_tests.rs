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

//! Comprehensive tests for PrecomputedTypeRegistry

use octofhir_fhirpath_model::precomputed_registry::{PrecomputedTypeRegistry, PrimitiveTypeKind};
use std::time::Duration;

fn create_test_registry() -> PrecomputedTypeRegistry {
    // Since build_from_schemas is async and complex, we'll test the public interface
    PrecomputedTypeRegistry::new()
}

#[tokio::test]
async fn test_new_registry() {
    let registry = PrecomputedTypeRegistry::new();

    // A new registry should be empty
    assert!(registry.get_system_type("Boolean").is_none());
    assert!(registry.get_fhir_type("Patient").is_none());

    // Test statistics for empty registry
    let stats = registry.statistics();
    assert_eq!(stats.system_types_count, 0);
    assert_eq!(stats.fhir_types_count, 0);
    assert_eq!(stats.inheritance_relationships_count, 0);
    assert_eq!(stats.choice_mappings_count, 0);
    assert_eq!(stats.total_properties_count, 0);
}

#[tokio::test]
async fn test_namespace_lookup() {
    let registry = create_test_registry();

    // Test unknown types return None
    assert_eq!(registry.get_namespace("Boolean"), None);
    assert_eq!(registry.get_namespace("Patient"), None);
    assert_eq!(registry.get_namespace("UnknownType"), None);
}

#[tokio::test]
async fn test_inheritance_relationships() {
    let registry = create_test_registry();

    // Test identity - any type is subtype of itself
    assert!(registry.is_subtype_of("Patient", "Patient"));
    assert!(registry.is_subtype_of("Boolean", "Boolean"));

    // Test non-relationships (no inheritance data loaded)
    assert!(!registry.is_subtype_of("Patient", "Observation"));
    assert!(!registry.is_subtype_of("Boolean", "String"));
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let registry = create_test_registry();

    let start = std::time::Instant::now();
    for _ in 0..10000 {
        registry.get_system_type("Boolean");
    }
    let elapsed = start.elapsed();

    // Should be very fast - less than 10ms for 10k lookups
    assert!(
        elapsed < Duration::from_millis(10),
        "Lookup too slow: {elapsed:?}"
    );
}

#[tokio::test]
async fn test_choice_type_processing() {
    let registry = create_test_registry();

    // Test choice type mapping (empty for new registry)
    let choice_mapping = registry.get_choice_mapping("Observation", "value");
    assert!(choice_mapping.is_none());

    // Test property lookup (empty for new registry)
    let property = registry.get_property("Patient", "active");
    assert!(property.is_none());

    let properties = registry.get_properties("Patient");
    assert!(properties.is_none());
}

#[tokio::test]
async fn test_build_time_tracking() {
    // Test that build time is None for new registry
    let registry = PrecomputedTypeRegistry::new();
    assert!(registry.build_time().is_none());
}

#[test]
fn test_primitive_type_kind() {
    // Test that all primitive type kinds can be created and compared
    let boolean_kind = PrimitiveTypeKind::Boolean;
    let integer_kind = PrimitiveTypeKind::Integer;
    let string_kind = PrimitiveTypeKind::String;
    let decimal_kind = PrimitiveTypeKind::Decimal;
    let date_kind = PrimitiveTypeKind::Date;
    let datetime_kind = PrimitiveTypeKind::DateTime;
    let time_kind = PrimitiveTypeKind::Time;
    let quantity_kind = PrimitiveTypeKind::Quantity;

    // Test Debug trait
    format!("{boolean_kind:?}");
    format!("{integer_kind:?}");
    format!("{string_kind:?}");
    format!("{decimal_kind:?}");
    format!("{date_kind:?}");
    format!("{datetime_kind:?}");
    format!("{time_kind:?}");
    format!("{quantity_kind:?}");

    // Test Clone trait
    let _cloned_boolean = boolean_kind;
    let _cloned_integer = integer_kind;
}
