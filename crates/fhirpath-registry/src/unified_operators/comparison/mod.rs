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

//! Unified comparison operator implementations

pub mod equals;
pub mod not_equals;
pub mod less_than;
pub mod greater_than;
pub mod greater_than_or_equal;
pub mod less_than_or_equal;
pub mod equivalent;
pub mod not_equivalent;

pub use equals::UnifiedEqualsOperator;
pub use not_equals::UnifiedNotEqualsOperator;
pub use less_than::UnifiedLessThanOperator;
pub use greater_than::UnifiedGreaterThanOperator;
pub use greater_than_or_equal::UnifiedGreaterThanOrEqualOperator;
pub use less_than_or_equal::UnifiedLessThanOrEqualOperator;
pub use equivalent::UnifiedEquivalentOperator;
pub use not_equivalent::UnifiedNotEquivalentOperator;