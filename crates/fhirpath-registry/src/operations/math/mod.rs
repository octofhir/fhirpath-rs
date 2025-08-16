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

//! Math functions module

pub mod abs;
pub mod ceiling;
pub mod exp;
pub mod floor;
pub mod ln;
pub mod log;
pub mod power;
pub mod precision;
pub mod round;
pub mod sqrt;
pub mod truncate;

pub use abs::AbsFunction;
pub use ceiling::CeilingFunction;
pub use exp::ExpFunction;
pub use floor::FloorFunction;
pub use ln::LnFunction;
pub use log::LogFunction;
pub use power::PowerFunction;
pub use precision::PrecisionFunction;
pub use round::RoundFunction;
pub use sqrt::SqrtFunction;
pub use truncate::TruncateFunction;

/// Registry helper for math operations
pub struct MathOperations;

impl MathOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Enhanced existing functions
        registry.register(AbsFunction::new()).await?;
        registry.register(SqrtFunction::new()).await?;
        registry.register(CeilingFunction::new()).await?;
        registry.register(FloorFunction::new()).await?;
        registry.register(RoundFunction::new()).await?;

        // New functions
        registry.register(ExpFunction::new()).await?;
        registry.register(LnFunction::new()).await?;
        registry.register(LogFunction::new()).await?;
        registry.register(PowerFunction::new()).await?;
        registry.register(TruncateFunction::new()).await?;
        registry.register(PrecisionFunction::new()).await?;

        Ok(())
    }
}
