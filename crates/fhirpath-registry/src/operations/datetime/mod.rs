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

pub mod high_boundary;
pub mod low_boundary;
pub mod now;
pub mod time_of_day;
pub mod today;

// Component extraction functions
pub mod day_of;
pub mod hour_of;
pub mod millisecond_of;
pub mod minute_of;
pub mod month_of;
pub mod second_of;
pub mod timezone_offset_of;
pub mod year_of;

pub use high_boundary::HighBoundaryFunction;
pub use low_boundary::LowBoundaryFunction;
pub use now::NowFunction;
pub use time_of_day::TimeOfDayFunction;
pub use today::TodayFunction;

// Component extraction functions
pub use day_of::DayOfFunction;
pub use hour_of::HourOfFunction;
pub use millisecond_of::MillisecondOfFunction;
pub use minute_of::MinuteOfFunction;
pub use month_of::MonthOfFunction;
pub use second_of::SecondOfFunction;
pub use timezone_offset_of::TimezoneOffsetOfFunction;
pub use year_of::YearOfFunction;

/// Registry helper for datetime operations
pub struct DateTimeOperations;

impl DateTimeOperations {
    pub async fn register_all(registry: &crate::FhirPathRegistry) -> crate::Result<()> {
        // Existing datetime functions
        registry.register(NowFunction::new()).await?;
        registry.register(TodayFunction::new()).await?;
        registry.register(TimeOfDayFunction::new()).await?;
        registry.register(LowBoundaryFunction::new()).await?;
        registry.register(HighBoundaryFunction::new()).await?;

        // NEW: Component extraction functions
        registry.register(YearOfFunction::new()).await?;
        registry.register(MonthOfFunction::new()).await?;
        registry.register(DayOfFunction::new()).await?;
        registry.register(HourOfFunction::new()).await?;
        registry.register(MinuteOfFunction::new()).await?;
        registry.register(SecondOfFunction::new()).await?;
        registry.register(MillisecondOfFunction::new()).await?;
        registry.register(TimezoneOffsetOfFunction::new()).await?;

        Ok(())
    }
}
