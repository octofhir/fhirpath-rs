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

//! Arithmetic operations for FHIRPath

use crate::fhirpath_registry::FhirPathRegistry;
use octofhir_fhirpath_core::Result;

pub mod addition;
pub mod subtraction;
pub mod multiplication;
pub mod division;
pub mod div;
pub mod mod_op;
pub mod concatenation;

pub use addition::AdditionOperation;
pub use subtraction::SubtractionOperation;
pub use multiplication::MultiplicationOperation;
pub use division::DivisionOperation;
pub use div::DivOperation;
pub use mod_op::ModOperation;
pub use concatenation::ConcatenationOperation;

/// Utility struct for registering all arithmetic operations
pub struct ArithmeticOperations;

impl ArithmeticOperations {
    /// Register all arithmetic operations in the registry
    pub async fn register_all(registry: &FhirPathRegistry) -> Result<()> {
        registry.register(AdditionOperation::new()).await?;
        registry.register(SubtractionOperation::new()).await?;
        registry.register(MultiplicationOperation::new()).await?;
        registry.register(DivisionOperation::new()).await?;
        registry.register(DivOperation::new()).await?;
        registry.register(ModOperation::new()).await?;
        registry.register(ConcatenationOperation::new()).await?;
        Ok(())
    }
}