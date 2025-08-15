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

//! Date/time functions module

pub mod now;
pub mod today;
pub mod time_of_day;
pub mod low_boundary;
pub mod high_boundary;

pub use now::NowFunction;
pub use today::TodayFunction;
pub use time_of_day::TimeOfDayFunction;
pub use low_boundary::LowBoundaryFunction;
pub use high_boundary::HighBoundaryFunction;

/// Registry helper for datetime operations
pub struct DateTimeOperations;

impl DateTimeOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        registry.register(NowFunction::new()).await?;
        registry.register(TodayFunction::new()).await?;
        registry.register(TimeOfDayFunction::new()).await?;
        registry.register(LowBoundaryFunction::new()).await?;
        registry.register(HighBoundaryFunction::new()).await?;
        Ok(())
    }
}
