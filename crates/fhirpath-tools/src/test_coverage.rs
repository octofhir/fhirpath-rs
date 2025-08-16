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

//! Test coverage generation and reporting

use anyhow::Result;
// use serde_json::Value; // Currently unused
use std::collections::HashMap;

/// Test coverage generator
pub struct TestCoverageGenerator {
    test_results: HashMap<String, bool>,
    total_tests: usize,
    passed_tests: usize,
}

impl TestCoverageGenerator {
    /// Create a new test coverage generator
    pub fn new() -> Self {
        Self {
            test_results: HashMap::new(),
            total_tests: 0,
            passed_tests: 0,
        }
    }

    /// Add test result
    pub fn add_test_result(&mut self, test_name: String, passed: bool) {
        self.test_results.insert(test_name, passed);
        self.total_tests += 1;
        if passed {
            self.passed_tests += 1;
        }
    }

    /// Generate coverage report
    pub fn generate_report(&self) -> Result<String> {
        let pass_rate = if self.total_tests > 0 {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        } else {
            0.0
        };

        let mut report = String::new();
        report.push_str("# FHIRPath Test Coverage Report\n\n");
        report.push_str(&format!("**Total Tests**: {}\n", self.total_tests));
        report.push_str(&format!("**Passed Tests**: {}\n", self.passed_tests));
        report.push_str(&format!(
            "**Failed Tests**: {}\n",
            self.total_tests - self.passed_tests
        ));
        report.push_str(&format!("**Pass Rate**: {pass_rate:.1}%\n\n"));

        // Add details of failed tests
        let failed_tests: Vec<_> = self
            .test_results
            .iter()
            .filter(|&(_, &passed)| !passed)
            .map(|(name, _)| name)
            .collect();

        if !failed_tests.is_empty() {
            report.push_str("## Failed Tests\n\n");
            for test_name in failed_tests {
                report.push_str(&format!("- {test_name}\n"));
            }
        }

        Ok(report)
    }

    /// Get pass rate
    pub fn pass_rate(&self) -> f64 {
        if self.total_tests > 0 {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        } else {
            0.0
        }
    }
}

impl Default for TestCoverageGenerator {
    fn default() -> Self {
        Self::new()
    }
}
