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

//! Integration tests for the evaluate command

use assert_cmd::Command;
use predicates::prelude::*;

mod helpers;
use helpers::{fixture_path, load_fixture};

#[test]
fn test_evaluate_simple_expression() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.active", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_evaluate_extract_family_name() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name.family", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Doe"));
}

#[test]
fn test_evaluate_extract_given_names() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name.given", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("John"));
}

#[test]
fn test_evaluate_with_where_clause() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.name.where(use = 'official').family",
            "-i",
        ])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Doe"));
}

#[test]
fn test_evaluate_count_function() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name.count()", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_evaluate_first_function() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name.given.first()", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("John"));
}

#[test]
fn test_evaluate_exists_function() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name.exists()", "-i"])
        .arg(&patient_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_evaluate_invalid_syntax() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.(", "-i"])
        .arg(&patient_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("FP"));
}

#[test]
fn test_evaluate_invalid_field() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.invalidField", "-i"])
        .arg(&patient_path)
        .assert()
        .success(); // Should succeed but return empty collection
}

#[test]
fn test_evaluate_json_output_format() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.active",
            "-i",
            patient_path.to_str().unwrap(),
            "--output-format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"success\""))
        .stdout(predicate::str::contains("\"result\""));
}

#[test]
fn test_evaluate_raw_output_format() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.active",
            "-i",
            patient_path.to_str().unwrap(),
            "--output-format",
            "raw",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_evaluate_quiet_mode() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.active",
            "-i",
            patient_path.to_str().unwrap(),
            "--quiet",
        ])
        .assert()
        .success();
}

#[test]
fn test_evaluate_verbose_mode() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.active",
            "-i",
            patient_path.to_str().unwrap(),
            "--verbose",
        ])
        .assert()
        .success();
}

#[test]
fn test_evaluate_with_variable() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.name.where(use = %nameUse).family",
            "-i",
            patient_path.to_str().unwrap(),
            "--var",
            "nameUse=official",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Doe"));
}

#[test]
fn test_evaluate_from_stdin() {
    let patient_json = load_fixture("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.active"])
        .write_stdin(patient_json)
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_evaluate_complex_expression() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.name.where(use='official').given.first() + ' ' + Patient.name.where(use='official').family",
            "-i",
            patient_path.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("John Doe"));
}

#[test]
fn test_evaluate_observation_value() {
    let obs_path = fixture_path("observation.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Observation.valueQuantity.value", "-i"])
        .arg(&obs_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("75.5"));
}

#[test]
fn test_evaluate_with_analyze_flag() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.name",
            "-i",
            patient_path.to_str().unwrap(),
            "--analyze",
        ])
        .assert()
        .success();
}

#[test]
fn test_evaluate_with_profile_flag() {
    let patient_path = fixture_path("patient.json");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&[
            "evaluate",
            "Patient.name",
            "-i",
            patient_path.to_str().unwrap(),
            "--profile",
        ])
        .assert()
        .success();
}

#[test]
fn test_evaluate_nonexistent_file() {
    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name", "-i", "nonexistent.json"])
        .assert()
        .failure();
}

#[test]
fn test_evaluate_invalid_json() {
    use helpers::create_temp_file;

    let invalid_json = create_temp_file("{invalid json}");

    Command::cargo_bin("octofhir-fhirpath")
        .unwrap()
        .args(&["evaluate", "Patient.name", "-i"])
        .arg(invalid_json.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("JSON").or(predicate::str::contains("parse")));
}
