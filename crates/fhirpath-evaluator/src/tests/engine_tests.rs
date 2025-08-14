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

use super::super::engine::{FhirPathEngine, EvaluationConfig};
// Basic engine test placeholder - currently unused

#[tokio::test]
async fn test_engine_creation() {
    let _engine = FhirPathEngine::with_mock_provider();

    // Verify engine can be created
    // Basic functionality will be tested in Task 2
}

#[tokio::test]
async fn test_engine_with_custom_config() {
    let config = EvaluationConfig {
        max_recursion_depth: 500,
        timeout_ms: 15000,
        enable_lambda_optimization: false,
        memory_limit_mb: Some(100),
    };

    // Create engine with mock provider first, then apply config
    use octofhir_fhirpath_registry::create_standard_registries;
    use octofhir_fhirpath_model::MockModelProvider;
    use std::sync::Arc;

    let (functions, operators) = create_standard_registries();
    let model_provider = Arc::new(MockModelProvider::empty());

    let engine = FhirPathEngine::new_with_config(
        Arc::new(functions),
        Arc::new(operators),
        model_provider,
        config.clone(),
    );

    assert_eq!(engine.config().max_recursion_depth, 500);
    assert_eq!(engine.config().timeout_ms, 15000);
    assert_eq!(engine.config().enable_lambda_optimization, false);
    assert_eq!(engine.config().memory_limit_mb, Some(100));
}

#[tokio::test]
async fn test_thread_safety() {
    let engine = FhirPathEngine::with_mock_provider();

    // Test that engine can be shared between threads
    let engine_clone = engine.clone();
    let handle = tokio::spawn(async move {
        // This should compile and run without issues
        let _ = engine_clone;
    });

    handle.await.unwrap();
}

// TODO: Add functional tests once core evaluation is implemented in Task 2
