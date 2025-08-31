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

//! Integration test helpers and utilities for comprehensive FHIRPath testing

use octofhir_fhirpath::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Comprehensive test context for integration testing
pub struct IntegrationTestContext {
    pub fhirpath: FhirPath,
    pub schema_manager: Arc<FhirSchemaPackageManager>,
    pub test_bundles: TestBundles,
}

/// Collection of test bundles for various scenarios
pub struct TestBundles {
    pub simple_bundle: JsonValue,
    pub comprehensive_bundle: JsonValue,
    pub vitals_bundle: JsonValue,
    pub medication_bundle: JsonValue,
    pub care_team_bundle: JsonValue,
    pub diagnostic_bundle: JsonValue,
    pub large_bundle: JsonValue,
}

impl IntegrationTestContext {
    /// Create a new integration test context with full configuration
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let fhirpath = FhirPathConfigBuilder::new()
            .with_fhir_version(FhirVersion::R4)
            .with_analyzer(true)
            .with_performance_tracking(true)
            .build()
            .await?;
        
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let schema_manager = Arc::new(FhirSchemaPackageManager::new(fcm_config, config).await?);
        
        let test_bundles = TestBundles::new();
        
        Ok(Self {
            fhirpath,
            schema_manager,
            test_bundles,
        })
    }
    
    /// Create a lightweight context for performance tests
    pub async fn lightweight() -> Result<Self, Box<dyn std::error::Error>> {
        let fhirpath = FhirPath::new().await?;
        
        let fcm_config = octofhir_canonical_manager::FcmConfig::default();
        let config = PackageManagerConfig::default();
        let schema_manager = Arc::new(FhirSchemaPackageManager::new(fcm_config, config).await?);
        
        let test_bundles = TestBundles::minimal();
        
        Ok(Self {
            fhirpath,
            schema_manager,
            test_bundles,
        })
    }
}

impl TestBundles {
    /// Create comprehensive test bundles
    pub fn new() -> Self {
        Self {
            simple_bundle: create_simple_test_bundle(),
            comprehensive_bundle: create_comprehensive_test_bundle(),
            vitals_bundle: create_vitals_monitoring_bundle(),
            medication_bundle: create_medication_scenario_bundle(),
            care_team_bundle: create_care_team_bundle(),
            diagnostic_bundle: create_diagnostic_results_bundle(),
            large_bundle: create_large_test_bundle(1000),
        }
    }
    
    /// Create minimal test bundles for performance tests
    pub fn minimal() -> Self {
        Self {
            simple_bundle: create_simple_test_bundle(),
            comprehensive_bundle: create_simple_test_bundle(),
            vitals_bundle: create_simple_test_bundle(),
            medication_bundle: create_simple_test_bundle(),
            care_team_bundle: create_simple_test_bundle(),
            diagnostic_bundle: create_simple_test_bundle(),
            large_bundle: create_large_test_bundle(100),
        }
    }
}

/// Create a simple test bundle for basic scenarios
pub fn create_simple_test_bundle() -> JsonValue {
    json!({
        "resourceType": "Bundle",
        "id": "simple-test-bundle",
        "type": "collection",
        "entry": [
            {
                "resource": create_test_patient("patient-1", "John", "Doe")
            },
            {
                "resource": create_test_observation("obs-1", "patient-1", "vital-signs", 120.0)
            },
            {
                "resource": create_test_practitioner("pract-1", "Dr. Jane", "Smith")
            }
        ]
    })
}

/// Create a comprehensive test bundle with multiple resource types
pub fn create_comprehensive_test_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add multiple patients
    entries.push(json!({"resource": create_test_patient("patient-1", "John", "Doe")}));
    entries.push(json!({"resource": create_test_patient("patient-2", "Jane", "Smith")}));
    
    // Add multiple observations
    entries.push(json!({"resource": create_test_observation("obs-1", "patient-1", "vital-signs", 120.0)}));
    entries.push(json!({"resource": create_test_observation("obs-2", "patient-1", "laboratory", 98.0)}));
    entries.push(json!({"resource": create_test_observation("obs-3", "patient-2", "vital-signs", 110.0)}));
    
    // Add practitioners
    entries.push(json!({"resource": create_test_practitioner("pract-1", "Dr. Jane", "Smith")}));
    entries.push(json!({"resource": create_test_practitioner("pract-2", "Dr. Bob", "Johnson")}));
    
    // Add organizations
    entries.push(json!({"resource": create_test_organization("org-1", "General Hospital")}));
    
    // Add encounters
    entries.push(json!({"resource": create_test_encounter("enc-1", "patient-1", "pract-1")}));
    
    json!({
        "resourceType": "Bundle",
        "id": "comprehensive-test-bundle",
        "type": "collection",
        "total": entries.len(),
        "entry": entries
    })
}

/// Create a vitals monitoring bundle
pub fn create_vitals_monitoring_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add patient
    entries.push(json!({"resource": create_test_patient("vitals-patient", "Alice", "Johnson")}));
    
    // Add vital signs observations
    entries.push(json!({
        "resource": {
            "resourceType": "Observation",
            "id": "bp-reading",
            "status": "final",
            "category": [{
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                    "code": "vital-signs",
                    "display": "Vital Signs"
                }]
            }],
            "code": {
                "coding": [{
                    "system": "http://loinc.org",
                    "code": "85354-9",
                    "display": "Blood pressure panel"
                }]
            },
            "subject": {"reference": "Patient/vitals-patient"},
            "effectiveDateTime": "2024-01-01T10:30:00Z",
            "component": [{
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8480-6",
                        "display": "Systolic blood pressure"
                    }]
                },
                "valueQuantity": {
                    "value": 120,
                    "unit": "mmHg",
                    "system": "http://unitsofmeasure.org",
                    "code": "mm[Hg]"
                }
            }, {
                "code": {
                    "coding": [{
                        "system": "http://loinc.org",
                        "code": "8462-4",
                        "display": "Diastolic blood pressure"
                    }]
                },
                "valueQuantity": {
                    "value": 80,
                    "unit": "mmHg",
                    "system": "http://unitsofmeasure.org",
                    "code": "mm[Hg]"
                }
            }]
        }
    }));
    
    // Add temperature reading
    entries.push(json!({
        "resource": {
            "resourceType": "Observation",
            "id": "temp-reading",
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
                    "code": "8310-5",
                    "display": "Body temperature"
                }]
            },
            "subject": {"reference": "Patient/vitals-patient"},
            "effectiveDateTime": "2024-01-01T10:35:00Z",
            "valueQuantity": {
                "value": 98.6,
                "unit": "degF",
                "system": "http://unitsofmeasure.org",
                "code": "[degF]"
            }
        }
    }));
    
    json!({
        "resourceType": "Bundle",
        "id": "vitals-monitoring-bundle",
        "type": "collection",
        "entry": entries
    })
}

/// Create a medication scenario bundle
pub fn create_medication_scenario_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add patient
    entries.push(json!({"resource": create_test_patient("med-patient", "Bob", "Wilson")}));
    
    // Add medication
    entries.push(json!({
        "resource": {
            "resourceType": "Medication",
            "id": "med-aspirin",
            "code": {
                "coding": [{
                    "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                    "code": "1191",
                    "display": "Aspirin"
                }]
            },
            "form": {
                "coding": [{
                    "system": "http://snomed.info/sct",
                    "code": "385055001",
                    "display": "Tablet"
                }]
            }
        }
    }));
    
    // Add medication administration
    entries.push(json!({
        "resource": {
            "resourceType": "MedicationAdministration",
            "id": "med-admin-1",
            "status": "completed",
            "medicationReference": {"reference": "Medication/med-aspirin"},
            "subject": {"reference": "Patient/med-patient"},
            "effectiveDateTime": "2024-01-01T08:00:00Z",
            "dosage": {
                "dose": {
                    "value": 81,
                    "unit": "mg"
                },
                "route": {
                    "coding": [{
                        "system": "http://snomed.info/sct",
                        "code": "26643006",
                        "display": "Oral"
                    }]
                }
            }
        }
    }));
    
    json!({
        "resourceType": "Bundle",
        "id": "medication-scenario-bundle",
        "type": "collection",
        "entry": entries
    })
}

/// Create a care team coordination bundle
pub fn create_care_team_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add patient
    entries.push(json!({"resource": create_test_patient("care-patient", "Mary", "Davis")}));
    
    // Add practitioners
    entries.push(json!({"resource": create_test_practitioner("pcp", "Dr. Sarah", "Johnson")}));
    entries.push(json!({"resource": create_test_practitioner("specialist", "Dr. Mike", "Brown")}));
    
    // Add care team
    entries.push(json!({
        "resource": {
            "resourceType": "CareTeam",
            "id": "care-team-1",
            "status": "active",
            "subject": {"reference": "Patient/care-patient"},
            "participant": [{
                "role": [{
                    "coding": [{
                        "system": "http://snomed.info/sct",
                        "code": "446050000",
                        "display": "Primary care physician"
                    }]
                }],
                "member": {"reference": "Practitioner/pcp"}
            }, {
                "role": [{
                    "coding": [{
                        "system": "http://snomed.info/sct", 
                        "code": "309343006",
                        "display": "Physician"
                    }]
                }],
                "member": {"reference": "Practitioner/specialist"}
            }]
        }
    }));
    
    // Add appointment
    entries.push(json!({
        "resource": {
            "resourceType": "Appointment",
            "id": "appointment-1",
            "status": "booked",
            "start": "2024-01-15T14:00:00Z",
            "end": "2024-01-15T14:30:00Z",
            "participant": [{
                "actor": {"reference": "Patient/care-patient"},
                "status": "accepted"
            }, {
                "actor": {"reference": "Practitioner/pcp"},
                "status": "accepted"
            }]
        }
    }));
    
    json!({
        "resourceType": "Bundle",
        "id": "care-team-bundle",
        "type": "collection",
        "entry": entries
    })
}

/// Create a diagnostic results bundle
pub fn create_diagnostic_results_bundle() -> JsonValue {
    let mut entries = Vec::new();
    
    // Add patient
    entries.push(json!({"resource": create_test_patient("diag-patient", "Tom", "Anderson")}));
    
    // Add lab results
    entries.push(json!({
        "resource": {
            "resourceType": "Observation",
            "id": "glucose-result",
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
                    "code": "33747-0",
                    "display": "Glucose"
                }]
            },
            "subject": {"reference": "Patient/diag-patient"},
            "valueQuantity": {
                "value": 95,
                "unit": "mg/dL",
                "system": "http://unitsofmeasure.org"
            },
            "interpretation": [{
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
                    "code": "N",
                    "display": "Normal"
                }]
            }],
            "referenceRange": [{
                "low": {"value": 70, "unit": "mg/dL"},
                "high": {"value": 100, "unit": "mg/dL"}
            }]
        }
    }));
    
    // Add abnormal result
    entries.push(json!({
        "resource": {
            "resourceType": "Observation",
            "id": "cholesterol-result",
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
                    "code": "14647-2",
                    "display": "Cholesterol"
                }]
            },
            "subject": {"reference": "Patient/diag-patient"},
            "valueQuantity": {
                "value": 250,
                "unit": "mg/dL",
                "system": "http://unitsofmeasure.org"
            },
            "interpretation": [{
                "coding": [{
                    "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
                    "code": "H",
                    "display": "High"
                }]
            }],
            "referenceRange": [{
                "high": {"value": 200, "unit": "mg/dL"}
            }]
        }
    }));
    
    json!({
        "resourceType": "Bundle",
        "id": "diagnostic-results-bundle",
        "type": "collection",
        "entry": entries
    })
}

/// Create a large test bundle for performance testing
pub fn create_large_test_bundle(size: usize) -> JsonValue {
    let mut entries = Vec::new();
    
    for i in 0..size {
        match i % 3 {
            0 => entries.push(json!({"resource": create_test_patient(&format!("patient-{}", i), &format!("FirstName{}", i), &format!("LastName{}", i))})),
            1 => entries.push(json!({"resource": create_test_observation(&format!("obs-{}", i), &format!("patient-{}", i / 3), "vital-signs", 100.0 + (i as f64))})),
            2 => entries.push(json!({"resource": create_test_practitioner(&format!("pract-{}", i), &format!("Dr. First{}", i), &format!("Last{}", i))})),
            _ => unreachable!(),
        }
    }
    
    json!({
        "resourceType": "Bundle",
        "id": "large-test-bundle",
        "type": "collection",
        "total": entries.len(),
        "entry": entries
    })
}

/// Helper function to create test patients
pub fn create_test_patient(id: &str, given: &str, family: &str) -> JsonValue {
    json!({
        "resourceType": "Patient",
        "id": id,
        "active": true,
        "name": [{
            "use": "official",
            "given": [given],
            "family": family
        }],
        "gender": "unknown",
        "birthDate": "1990-01-01"
    })
}

/// Helper function to create test observations
pub fn create_test_observation(id: &str, patient_ref: &str, category: &str, value: f64) -> JsonValue {
    json!({
        "resourceType": "Observation",
        "id": id,
        "status": "final",
        "category": [{
            "coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/observation-category",
                "code": category
            }]
        }],
        "code": {
            "coding": [{
                "system": "http://loinc.org",
                "code": "33747-0",
                "display": "Test observation"
            }]
        },
        "subject": {
            "reference": format!("Patient/{}", patient_ref)
        },
        "valueQuantity": {
            "value": value,
            "unit": "unit"
        }
    })
}

/// Helper function to create test practitioners
pub fn create_test_practitioner(id: &str, given: &str, family: &str) -> JsonValue {
    json!({
        "resourceType": "Practitioner",
        "id": id,
        "active": true,
        "name": [{
            "use": "official",
            "given": [given],
            "family": family
        }]
    })
}

/// Helper function to create test organizations
pub fn create_test_organization(id: &str, name: &str) -> JsonValue {
    json!({
        "resourceType": "Organization",
        "id": id,
        "active": true,
        "name": name,
        "type": [{
            "coding": [{
                "system": "http://terminology.hl7.org/CodeSystem/organization-type",
                "code": "prov",
                "display": "Healthcare Provider"
            }]
        }]
    })
}

/// Helper function to create test encounters
pub fn create_test_encounter(id: &str, patient_ref: &str, practitioner_ref: &str) -> JsonValue {
    json!({
        "resourceType": "Encounter",
        "id": id,
        "status": "finished",
        "class": {
            "system": "http://terminology.hl7.org/CodeSystem/v3-ActCode",
            "code": "AMB",
            "display": "ambulatory"
        },
        "subject": {
            "reference": format!("Patient/{}", patient_ref)
        },
        "participant": [{
            "individual": {
                "reference": format!("Practitioner/{}", practitioner_ref)
            }
        }]
    })
}

/// Performance testing utilities
pub struct PerformanceTracker {
    start_time: Instant,
    checkpoints: Vec<(String, Duration)>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            checkpoints: Vec::new(),
        }
    }
    
    pub fn checkpoint(&mut self, name: &str) {
        let elapsed = self.start_time.elapsed();
        self.checkpoints.push((name.to_string(), elapsed));
    }
    
    pub fn total_time(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn assert_under_threshold(&self, max_duration: Duration, operation_name: &str) {
        let total = self.total_time();
        assert!(
            total <= max_duration,
            "{} took {}ms, expected <= {}ms",
            operation_name,
            total.as_millis(),
            max_duration.as_millis()
        );
    }
    
    pub fn print_summary(&self, operation_name: &str) {
        println!("Performance Summary for {}", operation_name);
        println!("Total time: {}ms", self.total_time().as_millis());
        for (name, duration) in &self.checkpoints {
            println!("  {}: {}ms", name, duration.as_millis());
        }
    }
}

/// Memory usage tracking
pub fn get_current_memory_usage() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        use std::fs;
        if let Ok(contents) = fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        if let Ok(kb) = parts[1].parse::<u64>() {
                            return Some(kb * 1024); // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS implementation would go here
        // For now, return None to indicate unsupported
    }
    
    #[cfg(target_os = "windows")]
    {
        // Windows implementation would go here
        // For now, return None to indicate unsupported
    }
    
    None
}

/// Test validation helpers
pub fn assert_fhirpath_result_valid(result: &FhirPathEvaluationResult) {
    assert!(!result.values.is_empty(), "FHIRPath result should not be empty");
    assert!(result.execution_time > Duration::from_nanos(0), "Execution time should be recorded");
    assert!(result.errors.is_empty(), "Should not have errors: {:?}", result.errors);
}

pub fn assert_fhirpath_values_match<T>(
    result: &FhirPathEvaluationResult, 
    expected: &[T]
) where T: PartialEq + std::fmt::Debug + Clone,
    FhirPathValue: PartialEq<T>
{
    assert_eq!(
        result.values.len(), 
        expected.len(), 
        "Result count mismatch. Expected {:?}, got {:?}", 
        expected, 
        result.values
    );
    
    for (i, (actual, expected)) in result.values.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            actual, expected, 
            "Value mismatch at index {}: expected {:?}, got {:?}", 
            i, expected, actual
        );
    }
}

/// Async test utilities
pub async fn run_with_timeout<F, T>(
    future: F,
    timeout: Duration,
    operation_name: &str,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    match tokio::time::timeout(timeout, future).await {
        Ok(result) => result,
        Err(_) => Err(format!("Operation '{}' timed out after {:?}", operation_name, timeout).into()),
    }
}

/// Test data creation helpers for specific scenarios
pub fn create_test_patients(count: usize) -> Vec<JsonValue> {
    (0..count)
        .map(|i| create_test_patient(&format!("patient-{}", i), &format!("First{}", i), &format!("Last{}", i)))
        .collect()
}

pub async fn create_test_schema_manager() -> Result<Arc<FhirSchemaPackageManager>, Box<dyn std::error::Error>> {
    let fcm_config = octofhir_canonical_manager::FcmConfig::default();
    let config = PackageManagerConfig::default();
    let manager = FhirSchemaPackageManager::new(fcm_config, config).await?;
    Ok(Arc::new(manager))
}

/// Load test data from JSON files (if available)
pub fn load_clinical_test_data(filename: &str) -> Result<JsonValue, Box<dyn std::error::Error>> {
    let path = format!("tests/test_data/{}", filename);
    match std::fs::read_to_string(&path) {
        Ok(content) => Ok(serde_json::from_str(&content)?),
        Err(_) => {
            // Fallback to generated data if file doesn't exist
            println!("Warning: Test data file '{}' not found, using generated data", filename);
            Ok(create_comprehensive_test_bundle())
        }
    }
}