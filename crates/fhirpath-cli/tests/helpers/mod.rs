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

//! Test helpers for CLI integration tests

use std::path::{Path, PathBuf};

/// Get path to test fixture file
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Load fixture file content as string
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

/// Check if fixture file exists
pub fn fixture_exists(name: &str) -> bool {
    fixture_path(name).exists()
}

/// Get all fixture files in a directory
pub fn list_fixtures(dir: &str) -> Vec<String> {
    let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(dir);

    if !fixtures_dir.exists() {
        return vec![];
    }

    std::fs::read_dir(fixtures_dir)
        .expect("Failed to read fixtures directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_file() {
                path.file_name()?.to_str().map(String::from)
            } else {
                None
            }
        })
        .collect()
}

/// Create a temporary file with content and return its path
pub fn create_temp_file(content: &str) -> tempfile::NamedTempFile {
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().expect("Failed to create temp file");

    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");

    file
}

/// Create a temporary directory and return its path
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::TempDir::new().expect("Failed to create temp directory")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_path() {
        let path = fixture_path("patient.json");
        assert!(
            path.to_str()
                .unwrap()
                .ends_with("tests/fixtures/patient.json")
        );
    }

    #[test]
    fn test_fixture_exists() {
        assert!(fixture_exists("patient.json"));
        assert!(!fixture_exists("nonexistent.json"));
    }

    #[test]
    fn test_load_fixture() {
        let content = load_fixture("patient.json");
        assert!(content.contains("\"resourceType\": \"Patient\""));
    }

    #[test]
    fn test_create_temp_file() {
        let file = create_temp_file("{\"test\": true}");
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert_eq!(content, "{\"test\": true}");
    }

    #[test]
    fn test_create_temp_dir() {
        let dir = create_temp_dir();
        assert!(dir.path().exists());
        assert!(dir.path().is_dir());
    }
}
