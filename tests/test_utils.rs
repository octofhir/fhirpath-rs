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

//! Shared test utilities for Bridge Support Architecture
//!
//! This module provides common test infrastructure, data creators, and utilities
//! for testing the FHIRPath implementation with bridge support across all crates.

use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

/// Central test context that provides shared resources for all bridge tests
pub struct TestContext {
    pub schema_manager: Arc<FhirSchemaPackageManager>,
    pub test_patient: JsonValue,
    pub test_observation: JsonValue,
    pub test_bundle: JsonValue,
    pub test_organization: JsonValue,
}

impl TestContext {
    /// Create a new test context with initialized schema manager and test data
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let schema_manager = Self::create_schema_manager().await?;
        
        Ok(Self {
            schema_manager,
            test_patient: create_test_patient(),
            test_observation: create_test_observation(),
            test_bundle: create_test_bundle(),
            test_organization: create_test_organization(),
        })
    }
    
    /// Create a minimal test context for performance tests
    pub async fn minimal() -> Result<Self, Box<dyn std::error::Error>> {
        let schema_manager = Self::create_schema_manager().await?;
        
        Ok(Self {
            schema_manager,
            test_patient: create_minimal_patient(),
            test_observation: create_minimal_observation(),
            test_bundle: create_minimal_bundle(),
            test_organization: create_minimal_organization(),
        })
    }
    
    async fn create_schema_manager() -> Result<Arc<FhirSchemaPackageManager>, Box<dyn std::error::Error>> {
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let manager = FhirSchemaPackageManager::new(fcm_config, config).await?;
        Ok(Arc::new(manager))
    }
}

/// Create a comprehensive test Patient resource
pub fn create_test_patient() -> JsonValue {
    json!({
        "resourceType": "Patient",
        "id": "test-patient-1",
        "meta": {
            "versionId": "1",
            "lastUpdated": "2024-01-01T00:00:00Z",
            "profile": ["http://hl7.org/fhir/StructureDefinition/Patient"]
        },
        "text": {
            "status": "generated",
            "div": "<div>Test Patient</div>"
        },
        "identifier": [
            {
                "use": "usual",
                "type": {
                    "coding": [
                        {
                            "system": "http://terminology.hl7.org/CodeSystem/v2-0203",
                            "code": "MR",
                            "display": "Medical Record Number"
                        }
                    ]
                },
                "system": "http://hospital.smarthealthit.org",
                "value": "123456789"
            }
        ],
        "active": true,
        "name": [
            {
                "use": "official",
                "family": "Doe",
                "given": ["John", "David"],
                "prefix": ["Mr."]
            },
            {
                "use": "nickname",
                "given": ["Johnny"]
            }
        ],
        "telecom": [
            {
                "system": "phone",
                "value": "555-123-4567",
                "use": "home",
                "rank": 1
            },
            {
                "system": "email",
                "value": "john.doe@example.com",
                "use": "work"
            }
        ],
        "gender": "male",
        "birthDate": "1990-01-01",
        "address": [
            {
                "use": "home",
                "line": ["123 Main St", "Apt 4B"],
                "city": "Boston",
                "state": "MA",
                "postalCode": "02101",
                "country": "USA"
            }
        ],
        "contact": [
            {
                "relationship": [
                    {
                        "coding": [
                            {
                                "system": "http://terminology.hl7.org/CodeSystem/v2-0131",
                                "code": "C",
                                "display": "Emergency Contact"
                            }
                        ]
                    }
                ],
                "name": {
                    "family": "Doe",
                    "given": ["Jane"]
                },
                "telecom": [
                    {
                        "system": "phone",
                        "value": "555-987-6543"
                    }
                ]
            }
        ]
    })
}

/// Create a comprehensive test Observation resource
pub fn create_test_observation() -> JsonValue {
    json!({
        "resourceType": "Observation",
        "id": "test-observation-1",
        "meta": {
            "versionId": "1",
            "lastUpdated": "2024-01-01T00:00:00Z"
        },
        "status": "final",
        "category": [
            {
                "coding": [
                    {
                        "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                        "code": "vital-signs",
                        "display": "Vital Signs"
                    }
                ]
            }
        ],
        "code": {
            "coding": [
                {
                    "system": "http://loinc.org",
                    "code": "8480-6",
                    "display": "Systolic blood pressure"
                }
            ]
        },
        "subject": {
            "reference": "Patient/test-patient-1"
        },
        "effectiveDateTime": "2024-01-01T10:00:00Z",
        "valueQuantity": {
            "value": 120,
            "unit": "mmHg",
            "system": "http://unitsofmeasure.org",
            "code": "mm[Hg]"
        },
        "valueString": "Normal blood pressure",
        "component": [
            {
                "code": {
                    "coding": [
                        {
                            "system": "http://loinc.org",
                            "code": "8462-4",
                            "display": "Diastolic blood pressure"
                        }
                    ]
                },
                "valueQuantity": {
                    "value": 80,
                    "unit": "mmHg",
                    "system": "http://unitsofmeasure.org",
                    "code": "mm[Hg]"
                }
            }
        ]
    })
}

/// Create a comprehensive test Bundle resource
pub fn create_test_bundle() -> JsonValue {
    json!({
        "resourceType": "Bundle",
        "id": "test-bundle-1",
        "meta": {
            "lastUpdated": "2024-01-01T00:00:00Z"
        },
        "type": "collection",
        "timestamp": "2024-01-01T00:00:00Z",
        "total": 3,
        "entry": [
            {
                "fullUrl": "http://example.org/Patient/test-patient-1",
                "resource": create_test_patient()
            },
            {
                "fullUrl": "http://example.org/Observation/test-observation-1",
                "resource": create_test_observation()
            },
            {
                "fullUrl": "http://example.org/Organization/test-org-1",
                "resource": create_test_organization()
            }
        ]
    })
}

/// Create a test Organization resource
pub fn create_test_organization() -> JsonValue {
    json!({
        "resourceType": "Organization",
        "id": "test-org-1",
        "active": true,
        "type": [
            {
                "coding": [
                    {
                        "system": "http://terminology.hl7.org/CodeSystem/organization-type",
                        "code": "prov",
                        "display": "Healthcare Provider"
                    }
                ]
            }
        ],
        "name": "Test Healthcare Organization",
        "telecom": [
            {
                "system": "phone",
                "value": "555-HOSPITAL",
                "use": "work"
            }
        ]
    })
}

/// Create minimal test resources for performance tests
pub fn create_minimal_patient() -> JsonValue {
    json!({
        "resourceType": "Patient",
        "id": "minimal-patient",
        "name": [{"given": ["Test"], "family": "Patient"}],
        "active": true
    })
}

pub fn create_minimal_observation() -> JsonValue {
    json!({
        "resourceType": "Observation",
        "id": "minimal-observation",
        "status": "final",
        "code": {"text": "Test"},
        "valueString": "test-value"
    })
}

pub fn create_minimal_bundle() -> JsonValue {
    json!({
        "resourceType": "Bundle",
        "type": "collection",
        "entry": [
            {"resource": create_minimal_patient()},
            {"resource": create_minimal_observation()}
        ]
    })
}

pub fn create_minimal_organization() -> JsonValue {
    json!({
        "resourceType": "Organization",
        "id": "minimal-org",
        "name": "Test Org"
    })
}

/// Create a large Bundle for performance testing
pub fn create_large_test_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add multiple patients
    for i in 0..50 {
        let mut patient = create_test_patient();
        patient["id"] = json!(format!("patient-{}", i));
        entries.push(json!({"resource": patient}));
    }
    
    // Add multiple observations
    for i in 0..100 {
        let mut observation = create_test_observation();
        observation["id"] = json!(format!("observation-{}", i));
        observation["subject"]["reference"] = json!(format!("Patient/patient-{}", i % 50));
        entries.push(json!({"resource": observation}));
    }
    
    json!({
        "resourceType": "Bundle",
        "id": "large-test-bundle",
        "type": "collection",
        "total": entries.len(),
        "entry": entries
    })
}

/// Macro for creating async tests with TestContext
#[macro_export]
macro_rules! async_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let context = TestContext::new().await?;
            $test_body(context).await
        }
    };
}

/// Macro for creating minimal async tests for performance
#[macro_export]
macro_rules! minimal_async_test {
    ($test_name:ident, $test_body:expr) => {
        #[tokio::test]
        async fn $test_name() -> Result<(), Box<dyn std::error::Error>> {
            let context = TestContext::minimal().await?;
            $test_body(context).await
        }
    };
}

/// Helper for testing performance expectations
pub fn assert_performance<F>(operation: F, max_duration_ms: u64, operation_name: &str)
where
    F: FnOnce(),
{
    let start = std::time::Instant::now();
    operation();
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() <= max_duration_ms as u128,
        "{} took {}ms, expected <= {}ms",
        operation_name,
        duration.as_millis(),
        max_duration_ms
    );
}

/// Helper for testing async performance expectations
pub async fn assert_async_performance<F, Fut>(
    operation: F,
    max_duration_ms: u64,
    operation_name: &str,
) where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let start = std::time::Instant::now();
    operation().await;
    let duration = start.elapsed();
    
    assert!(
        duration.as_millis() <= max_duration_ms as u128,
        "{} took {}ms, expected <= {}ms",
        operation_name,
        duration.as_millis(),
        max_duration_ms
    );
}

/// Create test data for choice type testing
pub fn create_choice_type_test_data() -> JsonValue {
    json!({
        "observation_with_string": {
            "resourceType": "Observation",
            "id": "obs-string",
            "status": "final",
            "code": {"text": "Test"},
            "valueString": "test-string-value"
        },
        "observation_with_quantity": {
            "resourceType": "Observation",
            "id": "obs-quantity",
            "status": "final",
            "code": {"text": "Test"},
            "valueQuantity": {
                "value": 42.5,
                "unit": "mg",
                "system": "http://unitsofmeasure.org"
            }
        },
        "observation_with_codeable_concept": {
            "resourceType": "Observation",
            "id": "obs-codeable",
            "status": "final",
            "code": {"text": "Test"},
            "valueCodeableConcept": {
                "coding": [
                    {
                        "system": "http://example.org/codes",
                        "code": "TEST123",
                        "display": "Test Code"
                    }
                ]
            }
        }
    })
}

/// Create test data for complex navigation scenarios
pub fn create_navigation_test_bundle() -> JsonValue {
    json!({
        "resourceType": "Bundle",
        "id": "navigation-test-bundle",
        "type": "collection",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "patient-complex",
                    "name": [
                        {
                            "use": "official",
                            "family": "Smith",
                            "given": ["Alice", "Mary"]
                        },
                        {
                            "use": "maiden",
                            "family": "Johnson"
                        }
                    ],
                    "address": [
                        {
                            "use": "home",
                            "line": ["123 Oak St"],
                            "city": "Springfield",
                            "state": "IL"
                        },
                        {
                            "use": "work",
                            "line": ["456 Business Ave"],
                            "city": "Springfield",
                            "state": "IL"
                        }
                    ]
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "obs-vital-1",
                    "status": "final",
                    "category": [{
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                            "code": "vital-signs"
                        }]
                    }],
                    "code": {
                        "coding": [{
                            "system": "http://loinc.org",
                            "code": "85354-9",
                            "display": "Blood pressure panel"
                        }]
                    },
                    "subject": {"reference": "Patient/patient-complex"},
                    "valueQuantity": {
                        "value": 120,
                        "unit": "mmHg"
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "obs-lab-1",
                    "status": "final",
                    "category": [{
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                            "code": "laboratory"
                        }]
                    }],
                    "code": {
                        "coding": [{
                            "system": "http://loinc.org",
                            "code": "2951-2",
                            "display": "Sodium"
                        }]
                    },
                    "subject": {"reference": "Patient/patient-complex"},
                    "valueQuantity": {
                        "value": 140,
                        "unit": "mEq/L"
                    }
                }
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_context_creation() {
        let context = TestContext::new().await;
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.test_patient["resourceType"], "Patient");
        assert_eq!(ctx.test_observation["resourceType"], "Observation");
        assert_eq!(ctx.test_bundle["resourceType"], "Bundle");
    }
    
    #[tokio::test]
    async fn test_minimal_context_creation() {
        let context = TestContext::minimal().await;
        assert!(context.is_ok());
        
        let ctx = context.unwrap();
        assert_eq!(ctx.test_patient["resourceType"], "Patient");
        assert_eq!(ctx.test_observation["resourceType"], "Observation");
    }
    
    #[test]
    fn test_choice_type_data_structure() {
        let choice_data = create_choice_type_test_data();
        assert!(choice_data["observation_with_string"]["valueString"].is_string());
        assert!(choice_data["observation_with_quantity"]["valueQuantity"]["value"].is_number());
        assert!(choice_data["observation_with_codeable_concept"]["valueCodeableConcept"]["coding"].is_array());
    }
    
    #[test]
    fn test_performance_helpers() {
        assert_performance(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }, 50, "sleep test");
    }
    
    #[tokio::test]
    async fn test_async_performance_helpers() {
        assert_async_performance(|| async {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }, 50, "async sleep test").await;
    }
}