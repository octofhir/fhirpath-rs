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

//! Generate comprehensive test metadata for easy lookup and discovery

use fhirpath_dev_tools::test_support::TestSuite;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseMetadata {
    pub name: String,
    pub expression: String,
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub expect_error: Option<bool>,
    pub invalid_kind: Option<String>,
    pub file_path: String,
    pub suite_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteMetadata {
    pub name: String,
    pub file_path: String,
    pub category: Option<String>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub test_count: usize,
    pub test_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetadata {
    pub generated_at: String,
    pub total_suites: usize,
    pub total_tests: usize,
    pub suites: HashMap<String, TestSuiteMetadata>,
    pub test_cases: HashMap<String, TestCaseMetadata>,
    pub categories: HashMap<String, Vec<String>>, // category -> list of suite names
    pub file_index: HashMap<String, String>,      // filename -> full path
    pub name_index: HashMap<String, String>,      // test name -> suite name
}

fn scan_test_files(test_dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut test_files = Vec::new();

    // Only scan the groups directory for test suites, skip input directory
    let groups_dir = test_dir.join("groups");
    if !groups_dir.exists() {
        return Err("test-cases/groups directory not found".into());
    }

    fn scan_dir(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                scan_dir(&path, files)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                files.push(path);
            }
        }
        Ok(())
    }

    scan_dir(&groups_dir, &mut test_files)?;
    test_files.sort();
    Ok(test_files)
}

fn load_test_suite(file_path: &Path) -> Result<TestSuite, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let suite: TestSuite = serde_json::from_str(&content)?;
    Ok(suite)
}

fn generate_metadata(test_dir: &Path) -> Result<TestMetadata, Box<dyn std::error::Error>> {
    let test_files = scan_test_files(test_dir)?;

    let mut suites = HashMap::new();
    let mut test_cases = HashMap::new();
    let mut categories: HashMap<String, Vec<String>> = HashMap::new();
    let mut file_index = HashMap::new();
    let mut name_index = HashMap::new();
    let mut total_tests = 0;

    println!("🔍 Scanning {} test files...", test_files.len());

    for file_path in &test_files {
        let relative_path = file_path
            .strip_prefix(test_dir)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();

        println!("📄 Processing: {relative_path}");

        let suite = match load_test_suite(file_path) {
            Ok(suite) => suite,
            Err(e) => {
                eprintln!("⚠️  Failed to load {relative_path}: {e}");
                continue;
            }
        };

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Add to file index (both with and without extension)
        file_index.insert(file_name.clone(), relative_path.clone());
        if let Some(name_without_ext) = file_name.strip_suffix(".json") {
            file_index.insert(name_without_ext.to_string(), relative_path.clone());
        }

        // Process test cases
        let mut test_names = Vec::new();
        for test_case in &suite.tests {
            test_names.push(test_case.name.clone());
            name_index.insert(test_case.name.clone(), suite.name.clone());

            let test_metadata = TestCaseMetadata {
                name: test_case.name.clone(),
                expression: test_case.expression.clone(),
                category: test_case
                    .category
                    .clone()
                    .or_else(|| suite.category.clone()),
                subcategory: test_case.subcategory.clone(),
                tags: test_case.tags.clone(),
                description: test_case.description.clone(),
                expect_error: test_case.expect_error,
                invalid_kind: test_case.invalid_kind.clone(),
                file_path: relative_path.clone(),
                suite_name: suite.name.clone(),
            };

            test_cases.insert(test_case.name.clone(), test_metadata);
        }

        total_tests += suite.tests.len();

        // Create suite metadata
        let suite_metadata = TestSuiteMetadata {
            name: suite.name.clone(),
            file_path: relative_path.clone(),
            category: suite.category.clone(),
            description: suite.description.clone(),
            source: suite.source.clone(),
            test_count: suite.tests.len(),
            test_names,
        };

        suites.insert(suite.name.clone(), suite_metadata);

        // Add to category index
        if let Some(category) = &suite.category {
            categories
                .entry(category.clone())
                .or_default()
                .push(suite.name.clone());
        }
    }

    // Sort category lists
    for suite_list in categories.values_mut() {
        suite_list.sort();
    }

    Ok(TestMetadata {
        generated_at: chrono::Utc::now().to_rfc3339(),
        total_suites: suites.len(),
        total_tests,
        suites,
        test_cases,
        categories,
        file_index,
        name_index,
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 FHIRPath Test Metadata Generator");
    println!("===================================");

    let test_dir = Path::new("test-cases");
    if !test_dir.exists() {
        return Err("test-cases directory not found".into());
    }

    let metadata = generate_metadata(test_dir)?;

    println!("\n📊 Generated metadata:");
    println!("  • Test suites: {}", metadata.total_suites);
    println!("  • Test cases: {}", metadata.total_tests);
    println!("  • Categories: {}", metadata.categories.len());
    println!("  • File mappings: {}", metadata.file_index.len());

    println!("\n📂 Categories found:");
    for (category, suites) in &metadata.categories {
        println!("  • {}: {} suites", category, suites.len());
    }

    // Write metadata file
    let metadata_path = "test-cases/metadata.json";
    let metadata_json = serde_json::to_string_pretty(&metadata)?;
    fs::write(metadata_path, metadata_json)?;

    println!("\n✅ Metadata written to: {metadata_path}");
    println!("\n🔍 Usage examples:");
    println!("  cargo run --bin test-runner analyzer.json");
    println!("  cargo run --bin test-runner analyzer");
    println!("  cargo run --bin test-runner testBooleanLogicAnd1");
    println!("  cargo run --bin test-runner boolean");

    Ok(())
}
