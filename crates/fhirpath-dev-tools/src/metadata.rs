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

//! Test metadata management and lookup utilities

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

#[derive(Debug)]
pub enum TestLookupResult {
    /// Single test file found
    TestFile(PathBuf),
    /// Single test case found (returns the file containing it)
    TestCase(PathBuf, String),
    /// Category found (multiple files)
    Category(Vec<PathBuf>),
    /// Multiple matches found
    MultipleMatches(Vec<String>),
    /// No matches found
    NotFound,
}

pub struct TestMetadataManager {
    metadata: TestMetadata,
    test_cases_base: PathBuf,
}

impl TestMetadataManager {
    /// Load metadata from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let metadata_path = Path::new("test-cases/metadata.json");

        if !metadata_path.exists() {
            return Err(
                "Metadata file not found. Run 'cargo run --bin generate-test-metadata' first."
                    .into(),
            );
        }

        let content = fs::read_to_string(metadata_path)?;
        let metadata: TestMetadata = serde_json::from_str(&content)?;

        Ok(Self {
            metadata,
            test_cases_base: PathBuf::from("test-cases"),
        })
    }

    /// Lookup test by name, filename, or category
    pub fn lookup(&self, query: &str) -> TestLookupResult {
        // Direct file name match (with or without extension)
        if let Some(relative_path) = self.metadata.file_index.get(query) {
            return TestLookupResult::TestFile(self.test_cases_base.join(relative_path));
        }

        // Direct test case name match
        if let Some(suite_name) = self.metadata.name_index.get(query)
            && let Some(suite) = self.metadata.suites.get(suite_name)
        {
            return TestLookupResult::TestCase(
                self.test_cases_base.join(&suite.file_path),
                query.to_string(),
            );
        }

        // Category match
        if let Some(suite_names) = self.metadata.categories.get(query) {
            let paths: Vec<PathBuf> = suite_names
                .iter()
                .filter_map(|suite_name| self.metadata.suites.get(suite_name))
                .map(|suite| self.test_cases_base.join(&suite.file_path))
                .collect();

            if !paths.is_empty() {
                return TestLookupResult::Category(paths);
            }
        }

        // Fuzzy matching - look for partial matches
        let mut matches = Vec::new();

        // Check file names
        for filename in self.metadata.file_index.keys() {
            if filename.contains(query) || query.contains(filename) {
                matches.push(format!("file:{}", filename));
            }
        }

        // Check test case names
        for test_name in self.metadata.name_index.keys() {
            if test_name.contains(query) || query.contains(test_name) {
                matches.push(format!("test:{}", test_name));
            }
        }

        // Check categories
        for category in self.metadata.categories.keys() {
            if category.contains(query) || query.contains(category) {
                matches.push(format!("category:{}", category));
            }
        }

        // Check suite names
        for suite_name in self.metadata.suites.keys() {
            if suite_name.contains(query) || query.contains(suite_name) {
                matches.push(format!("suite:{}", suite_name));
            }
        }

        if matches.is_empty() {
            TestLookupResult::NotFound
        } else if matches.len() == 1
            && let match_str = &matches[0]
        {
            // Single fuzzy match - try to resolve it
            if let Some(stripped) = match_str.strip_prefix("file:") {
                self.lookup(stripped)
            } else if let Some(stripped) = match_str.strip_prefix("test:") {
                self.lookup(stripped)
            } else if let Some(stripped) = match_str.strip_prefix("category:") {
                self.lookup(stripped)
            } else if let Some(stripped) = match_str.strip_prefix("suite:") {
                self.lookup(stripped)
            } else {
                TestLookupResult::NotFound
            }
        } else {
            TestLookupResult::MultipleMatches(matches)
        }
    }

    /// Get all available test files
    pub fn list_test_files(&self) -> Vec<(&str, &TestSuiteMetadata)> {
        let mut files: Vec<_> = self
            .metadata
            .suites
            .iter()
            .map(|(name, suite)| (name.as_str(), suite))
            .collect();
        files.sort_by(|a, b| a.1.file_path.cmp(&b.1.file_path));
        files
    }

    /// Get all available categories
    pub fn list_categories(&self) -> Vec<(&str, usize)> {
        let mut categories: Vec<_> = self
            .metadata
            .categories
            .iter()
            .map(|(name, suites)| (name.as_str(), suites.len()))
            .collect();
        categories.sort();
        categories
    }

    /// Get test cases in a suite
    pub fn get_test_cases_in_suite(&self, suite_name: &str) -> Option<Vec<&str>> {
        self.metadata
            .suites
            .get(suite_name)
            .map(|suite| suite.test_names.iter().map(|s| s.as_str()).collect())
    }

    /// Get statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (
            self.metadata.total_suites,
            self.metadata.total_tests,
            self.metadata.categories.len(),
        )
    }
}
