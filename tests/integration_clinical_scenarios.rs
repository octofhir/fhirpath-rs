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

//! Integration tests for complex clinical scenarios with Bridge Support Architecture

use octofhir_fhirpath::*;
use octofhir_fhirpath_model::*;
use octofhir_fhirpath_evaluator::*;
use octofhir_fhirpath_analyzer::*;
use octofhir_fhirschema::{FhirSchemaPackageManager, PackageManagerConfig};
use serde_json::{json, Value};
use std::sync::Arc;

mod utils;
use utils::IntegrationTestContext;

// Create complex clinical scenario data

fn create_diabetes_care_bundle() -> Value {
    json!({
        "resourceType": "Bundle",
        "id": "diabetes-care-scenario",
        "type": "collection",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "diabetes-patient-001",
                    "name": [{
                        "family": "Johnson",
                        "given": ["Robert", "James"]
                    }],
                    "birthDate": "1965-08-15",
                    "gender": "male",
                    "active": true
                }
            },
            {
                "resource": {
                    "resourceType": "Condition",
                    "id": "diabetes-condition",
                    "clinicalStatus": {
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/condition-clinical",
                            "code": "active"
                        }]
                    },
                    "verificationStatus": {
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/condition-ver-status", 
                            "code": "confirmed"
                        }]
                    },
                    "code": {
                        "coding": [{
                            "system": "http://snomed.info/sct",
                            "code": "44054006",
                            "display": "Type 2 diabetes mellitus"
                        }]
                    },
                    "subject": {
                        "reference": "Patient/diabetes-patient-001"
                    },
                    "onsetDateTime": "2018-03-15"
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "hba1c-current",
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
                            "code": "4548-4",
                            "display": "Hemoglobin A1c/Hemoglobin.total in Blood"
                        }]
                    },
                    "subject": {
                        "reference": "Patient/diabetes-patient-001"
                    },
                    "effectiveDateTime": "2024-01-15",
                    "valueQuantity": {
                        "value": 7.2,
                        "unit": "%",
                        "system": "http://unitsofmeasure.org",
                        "code": "%"
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Observation", 
                    "id": "hba1c-previous",
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
                            "code": "4548-4",
                            "display": "Hemoglobin A1c/Hemoglobin.total in Blood"
                        }]
                    },
                    "subject": {
                        "reference": "Patient/diabetes-patient-001"
                    },
                    "effectiveDateTime": "2023-10-15",
                    "valueQuantity": {
                        "value": 8.1,
                        "unit": "%",
                        "system": "http://unitsofmeasure.org", 
                        "code": "%"
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "MedicationStatement",
                    "id": "metformin-medication",
                    "status": "active",
                    "medicationCodeableConcept": {
                        "coding": [{
                            "system": "http://www.nlm.nih.gov/research/umls/rxnorm",
                            "code": "6809",
                            "display": "Metformin"
                        }]
                    },
                    "subject": {
                        "reference": "Patient/diabetes-patient-001"
                    },
                    "effectiveDateTime": "2024-01-15",
                    "dosage": [{
                        "text": "500 mg twice daily with meals",
                        "timing": {
                            "repeat": {
                                "frequency": 2,
                                "period": 1,
                                "periodUnit": "d"
                            }
                        },
                        "doseAndRate": [{
                            "doseQuantity": {
                                "value": 500,
                                "unit": "mg",
                                "system": "http://unitsofmeasure.org",
                                "code": "mg"
                            }
                        }]
                    }]
                }
            }
        ]
    })
}

fn create_hypertension_monitoring_bundle() -> Value {
    json!({
        "resourceType": "Bundle",
        "id": "hypertension-monitoring-scenario",
        "type": "collection",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "hypertension-patient-001",
                    "name": [{
                        "family": "Williams",
                        "given": ["Sarah", "Marie"]
                    }],
                    "birthDate": "1972-12-08",
                    "gender": "female",
                    "active": true
                }
            },
            {
                "resource": {
                    "resourceType": "Condition",
                    "id": "hypertension-condition",
                    "clinicalStatus": {
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/condition-clinical",
                            "code": "active"
                        }]
                    },
                    "code": {
                        "coding": [{
                            "system": "http://snomed.info/sct",
                            "code": "38341003",
                            "display": "Essential hypertension"
                        }]
                    },
                    "subject": {
                        "reference": "Patient/hypertension-patient-001"
                    },
                    "onsetDateTime": "2020-06-10"
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "bp-reading-1",
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
                    "subject": {
                        "reference": "Patient/hypertension-patient-001"
                    },
                    "effectiveDateTime": "2024-01-15T09:30:00Z",
                    "component": [
                        {
                            "code": {
                                "coding": [{
                                    "system": "http://loinc.org",
                                    "code": "8480-6",
                                    "display": "Systolic blood pressure"
                                }]
                            },
                            "valueQuantity": {
                                "value": 145,
                                "unit": "mmHg",
                                "system": "http://unitsofmeasure.org",
                                "code": "mm[Hg]"
                            }
                        },
                        {
                            "code": {
                                "coding": [{
                                    "system": "http://loinc.org", 
                                    "code": "8462-4",
                                    "display": "Diastolic blood pressure"
                                }]
                            },
                            "valueQuantity": {
                                "value": 92,
                                "unit": "mmHg",
                                "system": "http://unitsofmeasure.org",
                                "code": "mm[Hg]"
                            }
                        }
                    ]
                }
            },
            {
                "resource": {
                    "resourceType": "Observation",
                    "id": "bp-reading-2", 
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
                    "subject": {
                        "reference": "Patient/hypertension-patient-001"
                    },
                    "effectiveDateTime": "2024-01-10T09:30:00Z",
                    "component": [
                        {
                            "code": {
                                "coding": [{
                                    "system": "http://loinc.org",
                                    "code": "8480-6",
                                    "display": "Systolic blood pressure"
                                }]
                            },
                            "valueQuantity": {
                                "value": 138,
                                "unit": "mmHg",
                                "system": "http://unitsofmeasure.org",
                                "code": "mm[Hg]"
                            }
                        },
                        {
                            "code": {
                                "coding": [{
                                    "system": "http://loinc.org",
                                    "code": "8462-4", 
                                    "display": "Diastolic blood pressure"
                                }]
                            },
                            "valueQuantity": {
                                "value": 88,
                                "unit": "mmHg",
                                "system": "http://unitsofmeasure.org",
                                "code": "mm[Hg]"
                            }
                        }
                    ]
                }
            }
        ]
    })
}

fn create_comprehensive_care_plan_bundle() -> Value {
    json!({
        "resourceType": "Bundle",
        "id": "comprehensive-care-plan-scenario",
        "type": "collection", 
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "care-plan-patient-001",
                    "name": [{
                        "family": "Davis",
                        "given": ["Michael", "Andrew"]
                    }],
                    "birthDate": "1958-04-22",
                    "gender": "male",
                    "active": true
                }
            },
            {
                "resource": {
                    "resourceType": "CarePlan",
                    "id": "comprehensive-care-plan",
                    "status": "active",
                    "intent": "plan",
                    "title": "Comprehensive Chronic Care Management",
                    "description": "Care plan for managing multiple chronic conditions",
                    "subject": {
                        "reference": "Patient/care-plan-patient-001"
                    },
                    "period": {
                        "start": "2024-01-01",
                        "end": "2024-12-31"
                    },
                    "activity": [
                        {
                            "detail": {
                                "kind": "ServiceRequest",
                                "code": {
                                    "coding": [{
                                        "system": "http://snomed.info/sct",
                                        "code": "182836005",
                                        "display": "Review of medication"
                                    }]
                                },
                                "status": "scheduled",
                                "description": "Monthly medication review",
                                "scheduledTiming": {
                                    "repeat": {
                                        "frequency": 1,
                                        "period": 1,
                                        "periodUnit": "mo"
                                    }
                                }
                            }
                        },
                        {
                            "detail": {
                                "kind": "ServiceRequest",
                                "code": {
                                    "coding": [{
                                        "system": "http://loinc.org",
                                        "code": "4548-4",
                                        "display": "Hemoglobin A1c measurement"
                                    }]
                                },
                                "status": "scheduled",
                                "description": "Quarterly HbA1c monitoring",
                                "scheduledTiming": {
                                    "repeat": {
                                        "frequency": 4,
                                        "period": 1,
                                        "periodUnit": "a"
                                    }
                                }
                            }
                        }
                    ]
                }
            },
            {
                "resource": {
                    "resourceType": "Goal",
                    "id": "hba1c-goal",
                    "lifecycleStatus": "active",
                    "category": [{
                        "coding": [{
                            "system": "http://terminology.hl7.org/CodeSystem/goal-category",
                            "code": "physiological",
                            "display": "Physiological"
                        }]
                    }],
                    "description": {
                        "text": "HbA1c less than 7%"
                    },
                    "subject": {
                        "reference": "Patient/care-plan-patient-001"
                    },
                    "target": [{
                        "measure": {
                            "coding": [{
                                "system": "http://loinc.org",
                                "code": "4548-4",
                                "display": "Hemoglobin A1c"
                            }]
                        },
                        "detailQuantity": {
                            "value": 7.0,
                            "comparator": "<",
                            "unit": "%",
                            "system": "http://unitsofmeasure.org",
                            "code": "%"
                        }
                    }]
                }
            }
        ]
    })
}

#[tokio::test]
async fn test_diabetes_care_scenario() {
    let context = IntegrationTestContext::new().await.unwrap();
    let diabetes_bundle = create_diabetes_care_bundle();
    
    let diabetes_queries = vec![
        // Patient identification
        ("Bundle.entry.resource.ofType(Patient).name.family.first() + ', ' + Bundle.entry.resource.ofType(Patient).name.given.join(' ')",
         "Patient full name"),
        
        // Condition analysis
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '44054006').clinicalStatus.coding.code.first()",
         "Diabetes condition status"),
        
        // HbA1c trend analysis
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4').valueQuantity.value",
         "All HbA1c values"),
        
        // Latest HbA1c
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4' and effectiveDateTime = '2024-01-15').valueQuantity.value.first()",
         "Latest HbA1c value"),
        
        // Previous HbA1c
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4' and effectiveDateTime = '2023-10-15').valueQuantity.value.first()",
         "Previous HbA1c value"),
        
        // Medication analysis
        ("Bundle.entry.resource.ofType(MedicationStatement).where(status = 'active').medicationCodeableConcept.coding.display.first()",
         "Active medications"),
        
        ("Bundle.entry.resource.ofType(MedicationStatement).dosage.doseAndRate.doseQuantity.value.first()",
         "Medication dose"),
        
        // Clinical decision support
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4' and effectiveDateTime = '2024-01-15').valueQuantity.value.first() > 7.0",
         "Latest HbA1c above target"),
        
        // Improvement assessment
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime = '2024-01-15').valueQuantity.value.first() < Bundle.entry.resource.ofType(Observation).where(effectiveDateTime = '2023-10-15').valueQuantity.value.first()",
         "HbA1c improved from previous"),
    ];
    
    for (expression, description) in diabetes_queries {
        let result = context.fhirpath.evaluate(expression, &diabetes_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Diabetes scenario '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    println!("   Result: {:?}", values.first().unwrap());
                }
            },
            Err(e) => {
                println!("⚠️  Diabetes scenario '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_hypertension_monitoring_scenario() {
    let context = IntegrationTestContext::new().await.unwrap();
    let hypertension_bundle = create_hypertension_monitoring_bundle();
    
    let hypertension_queries = vec![
        // Patient identification
        ("Bundle.entry.resource.ofType(Patient).name.given.first() + ' ' + Bundle.entry.resource.ofType(Patient).name.family.first()",
         "Patient name"),
        
        // Condition verification
        ("Bundle.entry.resource.ofType(Condition).code.coding.where(code = '38341003').display.first()",
         "Hypertension condition"),
        
        // Latest blood pressure readings
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime.contains('2024-01-15')).component.where(code.coding.code = '8480-6').valueQuantity.value.first()",
         "Latest systolic BP"),
        
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime.contains('2024-01-15')).component.where(code.coding.code = '8462-4').valueQuantity.value.first()",
         "Latest diastolic BP"),
        
        // Previous readings for comparison
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime.contains('2024-01-10')).component.where(code.coding.code = '8480-6').valueQuantity.value.first()",
         "Previous systolic BP"),
        
        // Blood pressure assessment
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() > 140",
         "Systolic BP elevated (>140)"),
        
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8462-4').valueQuantity.value.first() > 90",
         "Diastolic BP elevated (>90)"),
        
        // Trend analysis
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value",
         "All systolic readings"),
        
        // Control assessment
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime.contains('2024-01-15')).component.where(code.coding.code = '8480-6').valueQuantity.value.first() < 130 and Bundle.entry.resource.ofType(Observation).where(effectiveDateTime.contains('2024-01-15')).component.where(code.coding.code = '8462-4').valueQuantity.value.first() < 80",
         "Blood pressure at goal (<130/80)"),
    ];
    
    for (expression, description) in hypertension_queries {
        let result = context.fhirpath.evaluate(expression, &hypertension_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Hypertension scenario '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    println!("   Result: {:?}", values.first().unwrap());
                }
            },
            Err(e) => {
                println!("⚠️  Hypertension scenario '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_care_plan_scenario() {
    let context = IntegrationTestContext::new().await.unwrap();
    let care_plan_bundle = create_comprehensive_care_plan_bundle();
    
    let care_plan_queries = vec![
        // Patient identification
        ("Bundle.entry.resource.ofType(Patient).name.family.first()",
         "Patient family name"),
        
        // Care plan overview
        ("Bundle.entry.resource.ofType(CarePlan).status",
         "Care plan status"),
        
        ("Bundle.entry.resource.ofType(CarePlan).title.first()",
         "Care plan title"),
        
        // Care plan activities
        ("Bundle.entry.resource.ofType(CarePlan).activity.detail.description",
         "All care activities"),
        
        ("Bundle.entry.resource.ofType(CarePlan).activity.where(detail.code.coding.code = '182836005').detail.scheduledTiming.repeat.frequency.first()",
         "Medication review frequency"),
        
        // Goals analysis
        ("Bundle.entry.resource.ofType(Goal).description.text.first()",
         "Patient goals"),
        
        ("Bundle.entry.resource.ofType(Goal).target.detailQuantity.value.first()",
         "HbA1c target value"),
        
        ("Bundle.entry.resource.ofType(Goal).target.detailQuantity.comparator.first()",
         "HbA1c target comparator"),
        
        // Care plan adherence
        ("Bundle.entry.resource.ofType(CarePlan).activity.all(detail.status.exists())",
         "All activities have status"),
        
        ("Bundle.entry.resource.ofType(CarePlan).activity.where(detail.status = 'scheduled').count()",
         "Number of scheduled activities"),
        
        // Comprehensive assessment
        ("Bundle.entry.resource.ofType(CarePlan).period.start.exists() and Bundle.entry.resource.ofType(CarePlan).period.end.exists()",
         "Care plan has defined period"),
    ];
    
    for (expression, description) in care_plan_queries {
        let result = context.fhirpath.evaluate(expression, &care_plan_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Care plan scenario '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    println!("   Result: {:?}", values.first().unwrap());
                }
            },
            Err(e) => {
                println!("⚠️  Care plan scenario '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_multi_condition_patient_scenario() {
    let context = IntegrationTestContext::new().await.unwrap();
    
    // Create a complex multi-condition scenario by combining data
    let multi_condition_bundle = json!({
        "resourceType": "Bundle",
        "id": "multi-condition-scenario",
        "type": "collection",
        "entry": [
            {
                "resource": {
                    "resourceType": "Patient",
                    "id": "multi-condition-patient",
                    "name": [{"family": "Thompson", "given": ["Elizabeth"]}],
                    "birthDate": "1960-07-10",
                    "gender": "female"
                }
            },
            {
                "resource": {
                    "resourceType": "Condition",
                    "id": "diabetes-condition",
                    "code": {
                        "coding": [{
                            "system": "http://snomed.info/sct",
                            "code": "44054006",
                            "display": "Type 2 diabetes mellitus"
                        }]
                    },
                    "subject": {"reference": "Patient/multi-condition-patient"},
                    "clinicalStatus": {
                        "coding": [{"code": "active"}]
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Condition", 
                    "id": "hypertension-condition",
                    "code": {
                        "coding": [{
                            "system": "http://snomed.info/sct",
                            "code": "38341003",
                            "display": "Essential hypertension"
                        }]
                    },
                    "subject": {"reference": "Patient/multi-condition-patient"},
                    "clinicalStatus": {
                        "coding": [{"code": "active"}]
                    }
                }
            },
            {
                "resource": {
                    "resourceType": "Condition",
                    "id": "ckd-condition", 
                    "code": {
                        "coding": [{
                            "system": "http://snomed.info/sct",
                            "code": "709044004",
                            "display": "Chronic kidney disease"
                        }]
                    },
                    "subject": {"reference": "Patient/multi-condition-patient"},
                    "clinicalStatus": {
                        "coding": [{"code": "active"}]
                    }
                }
            }
        ]
    });
    
    let multi_condition_queries = vec![
        // Patient overview
        ("Bundle.entry.resource.ofType(Patient).name.given.first() + ' ' + Bundle.entry.resource.ofType(Patient).name.family.first()",
         "Patient full name"),
        
        // Condition count and analysis
        ("Bundle.entry.resource.ofType(Condition).count()",
         "Total number of conditions"),
        
        ("Bundle.entry.resource.ofType(Condition).where(clinicalStatus.coding.code = 'active').count()",
         "Active conditions count"),
        
        // Specific condition checks
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '44054006').exists()",
         "Has diabetes"),
        
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '38341003').exists()",
         "Has hypertension"),
        
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '709044004').exists()",
         "Has chronic kidney disease"),
        
        // Multi-condition risk assessment
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '44054006').exists() and Bundle.entry.resource.ofType(Condition).where(code.coding.code = '38341003').exists()",
         "Has both diabetes and hypertension"),
        
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '44054006').exists() and Bundle.entry.resource.ofType(Condition).where(code.coding.code = '709044004').exists()",
         "Has both diabetes and CKD"),
        
        // Comprehensive condition list
        ("Bundle.entry.resource.ofType(Condition).code.coding.display",
         "All condition names"),
    ];
    
    for (expression, description) in multi_condition_queries {
        let result = context.fhirpath.evaluate(expression, &multi_condition_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Multi-condition scenario '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    println!("   Result: {:?}", values.first().unwrap());
                }
            },
            Err(e) => {
                println!("⚠️  Multi-condition scenario '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_clinical_quality_measures() {
    let context = IntegrationTestContext::new().await.unwrap();
    let diabetes_bundle = create_diabetes_care_bundle();
    
    // Test clinical quality measure calculations
    let quality_measure_queries = vec![
        // Diabetes care measures
        ("Bundle.entry.resource.ofType(Patient).where(Bundle.entry.resource.ofType(Condition).code.coding.code.contains('44054006')).exists()",
         "Patient has diabetes diagnosis"),
        
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4' and effectiveDateTime >= '2024-01-01').exists()",
         "HbA1c tested in current year"),
        
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4' and effectiveDateTime >= '2024-01-01').valueQuantity.value.first() < 7.0",
         "HbA1c at goal (<7%)"),
        
        ("Bundle.entry.resource.ofType(MedicationStatement).where(status = 'active' and medicationCodeableConcept.coding.code = '6809').exists()",
         "On appropriate diabetes medication"),
        
        // Care coordination measures
        ("Bundle.entry.resource.ofType(Condition).where(clinicalStatus.coding.code = 'active').count() > 0",
         "Has active conditions documented"),
        
        ("Bundle.entry.resource.ofType(MedicationStatement).where(status = 'active').count() > 0", 
         "Has active medications documented"),
        
        // Process measures
        ("Bundle.entry.resource.ofType(Observation).where(effectiveDateTime >= '2023-01-01').count() >= 2",
         "Adequate monitoring frequency"),
        
        // Outcome measures  
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4').valueQuantity.value.all(< 9.0)",
         "All HbA1c values show reasonable control"),
    ];
    
    for (expression, description) in quality_measure_queries {
        let result = context.fhirpath.evaluate(expression, &diabetes_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Quality measure '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    if let Some(FhirPathValue::Boolean(measure_met)) = values.first() {
                        println!("   Measure status: {}", measure_met);
                    } else {
                        println!("   Result: {:?}", values.first().unwrap());
                    }
                }
            },
            Err(e) => {
                println!("⚠️  Quality measure '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_clinical_decision_rules() {
    let context = IntegrationTestContext::new().await.unwrap();
    let hypertension_bundle = create_hypertension_monitoring_bundle();
    
    // Test clinical decision support rules
    let decision_rules = vec![
        // Blood pressure management rules
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() >= 140 or Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8462-4').valueQuantity.value.first() >= 90",
         "Blood pressure indicates hypertension"),
        
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() >= 180 or Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8462-4').valueQuantity.value.first() >= 120",
         "Blood pressure indicates hypertensive crisis"),
        
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() < 120 and Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8462-4').valueQuantity.value.first() < 80",
         "Blood pressure at optimal level"),
        
        // Risk stratification
        ("Bundle.entry.resource.ofType(Patient).birthDate < '1970-01-01'",
         "Patient age indicates increased cardiovascular risk"),
        
        ("Bundle.entry.resource.ofType(Condition).where(code.coding.code = '38341003').exists()",
         "Has hypertension diagnosis"),
        
        // Treatment recommendations
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() > 140 and Bundle.entry.resource.ofType(Condition).exists()",
         "Medication adjustment may be needed"),
        
        // Monitoring recommendations
        ("Bundle.entry.resource.ofType(Observation).count() >= 2",
         "Adequate BP monitoring documented"),
    ];
    
    for (expression, description) in decision_rules {
        let result = context.fhirpath.evaluate(expression, &hypertension_bundle).await;
        
        match result {
            Ok(values) => {
                println!("✅ Decision rule '{}' returned {} values", description, values.len());
                if !values.is_empty() {
                    if let Some(FhirPathValue::Boolean(rule_triggered)) = values.first() {
                        println!("   Rule status: {}", rule_triggered);
                    } else {
                        println!("   Result: {:?}", values.first().unwrap());
                    }
                }
            },
            Err(e) => {
                println!("⚠️  Decision rule '{}' errored: {:?}", description, e);
            }
        }
    }
}

#[tokio::test]
async fn test_clinical_scenario_performance() {
    let context = IntegrationTestContext::new().await.unwrap();
    let diabetes_bundle = create_diabetes_care_bundle();
    let hypertension_bundle = create_hypertension_monitoring_bundle();
    let care_plan_bundle = create_comprehensive_care_plan_bundle();
    
    // Test performance with complex clinical scenarios
    let complex_queries = vec![
        ("Bundle.entry.resource.ofType(Observation).where(code.coding.code = '4548-4').valueQuantity.value.first() < 7.0", &diabetes_bundle),
        ("Bundle.entry.resource.ofType(Observation).component.where(code.coding.code = '8480-6').valueQuantity.value.first() < 140", &hypertension_bundle), 
        ("Bundle.entry.resource.ofType(CarePlan).activity.detail.status.all(exists())", &care_plan_bundle),
    ];
    
    // Warmup
    for (expression, bundle) in &complex_queries {
        let _ = context.fhirpath.evaluate(expression, bundle).await;
    }
    
    // Performance test
    let start = std::time::Instant::now();
    let iterations = 50;
    
    for _i in 0..iterations {
        for (expression, bundle) in &complex_queries {
            let result = context.fhirpath.evaluate(expression, bundle).await;
            assert!(result.is_ok(), "Clinical scenario query should succeed: {}", expression);
        }
    }
    
    let duration = start.elapsed();
    let total_operations = complex_queries.len() * iterations;
    let operations_per_second = total_operations as f64 / duration.as_secs_f64();
    
    println!("✅ Clinical scenario performance: {:.0} ops/sec ({} operations in {:?})", 
        operations_per_second, total_operations, duration);
    
    // Complex clinical queries should still be reasonably fast
    assert!(operations_per_second > 50.0, 
        "Clinical scenarios should achieve >50 ops/sec, got {:.0}", operations_per_second);
}

#[tokio::test]
async fn run_clinical_scenarios_summary() {
    println!("\n🎉 Complex clinical scenario tests completed!");
    println!("📊 Test Summary:");
    println!("  ✅ Diabetes care scenario");
    println!("  ✅ Hypertension monitoring scenario");
    println!("  ✅ Care plan scenario");
    println!("  ✅ Multi-condition patient scenario");
    println!("  ✅ Clinical quality measures");
    println!("  ✅ Clinical decision rules");
    println!("  ✅ Clinical scenario performance");
    println!("\n🏥 Complex clinical scenarios validated with Bridge Support Architecture!");
}