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

//! Comparison functions module (operators moved to evaluator)

/// Registry helper for comparison functions
pub struct ComparisonOperations;

impl ComparisonOperations {
    pub async fn register_all(_registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // All comparison operators moved to dedicated evaluator
        Ok(())
    }
}
